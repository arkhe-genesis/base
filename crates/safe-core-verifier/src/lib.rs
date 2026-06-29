
pub mod languages;
pub mod checks;
pub mod report;
pub mod context;
pub mod cgf;

use anyhow::{Result, Context as AnyhowContext};
use std::path::Path;
use std::collections::HashMap;
use tree_sitter::Parser;

use languages::Language;
use checks::{convention_x::ConventionXCheck, dependency::DependencyCheck, safety::SafetyCheck, AllChecks, Check};
use report::{FileReport, GlobalReport};
use context::FileContext;
use safe_core_utils::CgfEngine;

pub struct PolyglotVerifier {
    engines: HashMap<Language, Parser>,
    cgf: CgfEngine,
    check_runner: AllChecks,
}

impl PolyglotVerifier {
    pub fn new() -> Result<Self> {
        let mut engines = HashMap::new();

        for lang in Language::all() {
            let mut parser = Parser::new();
            parser.set_language(&lang.tree_sitter_language())?;
            engines.insert(lang, parser);
        }

        let checks: Vec<Box<dyn Check>> = vec![
            Box::new(ConventionXCheck),
            Box::new(DependencyCheck::new()),
            Box::new(SafetyCheck),
        ];

        Ok(Self {
            engines,
            cgf: CgfEngine::new(100),
            check_runner: AllChecks(checks),
        })
    }

    pub async fn verify_file(&mut self, path: &Path) -> Result<FileReport> {
        let code = std::fs::read_to_string(path)
            .with_context(|| format!("Falha ao ler {}", path.display()))?;

        let lang = Language::detect(path)?;

        let parser = self.engines.get_mut(&lang)
            .ok_or_else(|| anyhow::anyhow!("Nenhum parser registrado para linguagem {:?}", lang))?;

        let tree = parser.parse(&code, None)
            .ok_or_else(|| anyhow::anyhow!("Falha ao parsear {}", path.display()))?;

        // Análise de estrutura (Alpha proxy)
        let metrics = cgf::code_analyzer::CodeStructureAnalyzer::analyze(&tree, &code, lang);
        let alpha_hat = metrics.to_code_alpha();

        let ctx = FileContext {
            path: path.to_path_buf(),
            language: lang,
            code,
            tree,
            content_hash: 0, // Pode ser implementado hash real
        };

        let result = self.check_runner.execute(&ctx).await?;

        Ok(FileReport {
            path: path.to_string_lossy().into_owned(),
            language: lang.to_string(),
            alpha_hat,
            issues: result.issues,
            suggestions: result.suggestions,
            passed: result.passed,
        })
    }

    pub async fn verify_dir(&mut self, dir: &Path) -> Result<GlobalReport> {
        let mut file_reports = Vec::new();

        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();

            // Ignora arquivos que não suportamos para evitar erros no walk
            if Language::detect(path).is_ok() {
                match self.verify_file(path).await {
                    Ok(report) => file_reports.push(report),
                    Err(e) => tracing::warn!("Falha ao verificar {:?}: {}", path, e),
                }
            }
        }

        Ok(GlobalReport::from_file_reports(file_reports))
    }
}
