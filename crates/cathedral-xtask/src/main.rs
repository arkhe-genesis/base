//! Cathedral ARKHE — xtask (versão expandida)
//! Selo: CATHEDRAL-ARKHE-XTASK-v2.0.0-2026-06-21

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::process::{Command, Stdio};
use std::time::Instant;
use which::which;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Cathedral ARKHE development tasks")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Verifica ferramentas instaladas
    CheckTools,
    /// Pre-commit rápido (fmt, check, clippy, deny, audit, unit tests)
    PreCommit,
    /// CI completo (pre-commit + integração, coverage, doc, bench)
    Ci,
    /// Auditoria completa para release
    FullAudit,
    /// Gera cobertura de código (HTML)
    Coverage,
    /// Gera documentação
    Doc,
    /// Executa benchmarks
    Bench,
    /// Verifica dependências (cargo-deny)
    Deny,
    /// Verifica vulnerabilidades (cargo-audit)
    Audit,
    /// Executa todos os testes (unitários + integração)
    Test,
    /// Gera SBOM (Software Bill of Materials)
    Sbom,
    /// Verifica links quebrados na documentação
    Deadlinks,
    /// Atualiza dependências (cargo update)
    Update,
    /// Limpa artefatos de build
    Clean,
    /// Executa todos os checks disponíveis
    All,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let start = Instant::now();

    match cli.command {
        Commands::CheckTools => check_tools()?,
        Commands::PreCommit => pre_commit()?,
        Commands::Ci => ci()?,
        Commands::FullAudit => full_audit()?,
        Commands::Coverage => coverage()?,
        Commands::Doc => doc()?,
        Commands::Bench => bench()?,
        Commands::Deny => deny()?,
        Commands::Audit => audit()?,
        Commands::Test => test()?,
        Commands::Sbom => sbom()?,
        Commands::Deadlinks => deadlinks()?,
        Commands::Update => update()?,
        Commands::Clean => clean()?,
        Commands::All => all_checks()?,
    }

    println!("\n{}", "✅ Todas as verificações concluídas com sucesso!".green());
    println!("⏱️  Tempo total: {:.2}s", start.elapsed().as_secs_f64());
    Ok(())
}

// ============================================================================
// COMANDOS INDIVIDUAIS
// ============================================================================

fn check_tools() -> Result<()> {
    step("🔧 Verificando ferramentas instaladas");
    let tools = [
        "cargo", "cargo-fmt", "cargo-clippy", "cargo-deny", "cargo-audit",
        "cargo-semver-checks", "cargo-llvm-cov", "cargo-insta", "cargo-deadlinks",
        "cargo-sbom", "cargo-ndk",
    ];
    let mut missing = Vec::new();
    for tool in &tools {
        if which(tool).is_ok() {
            println!("  ✅ {}", tool);
        } else {
            println!("  ❌ {} (não encontrado)", tool);
            missing.push(*tool);
        }
    }
    if !missing.is_empty() {
        println!("\n{}", "⚠️  Ferramentas faltando:".yellow());
        for tool in &missing {
            println!("     cargo install {}", tool);
        }
        return Err(anyhow!("Ferramentas faltando"));
    }
    Ok(())
}

fn pre_commit() -> Result<()> {
    step("🔍 Pre-commit");
    check_tools()?;
    run("cargo fmt --all -- --check", "Formatação")?;
    run("cargo check --workspace --all-targets --all-features", "MSRV e sintaxe")?;
    run("cargo clippy --workspace --all-features -- -D warnings", "Lints (clippy)")?;
    run("cargo deny check", "Dependências (deny)")?;
    run("cargo audit --deny-warnings", "Vulnerabilidades (audit)")?;
    run("cargo test --workspace --lib", "Testes unitários")?;
    Ok(())
}

fn ci() -> Result<()> {
    step("🔬 CI");
    pre_commit()?;
    run("cargo test --workspace --test '*'", "Testes de integração")?;
    run("cargo insta test --workspace", "Snapshot tests")?;
    run("cargo semver-checks --workspace --baseline-rev HEAD~1", "SemVer")?;
    run("cargo bench --workspace", "Benchmarks")?;
    run("cargo llvm-cov --workspace --html --output-dir target/coverage", "Cobertura")?;
    run("cargo doc --workspace --no-deps --document-private-items", "Documentação")?;
    run("cargo deadlinks --check-http", "Links quebrados")?;
    Ok(())
}

fn full_audit() -> Result<()> {
    step("🔒 Full Audit");
    ci()?;
    run("cargo publish --dry-run --no-verify", "Publicação (dry-run)")?;
    run("cargo sbom --output target/sbom.json", "SBOM")?;
    run("cargo audit --json > target/audit_report.json", "Relatório de vulnerabilidades")?;
    Ok(())
}

fn coverage() -> Result<()> {
    step("📊 Cobertura de código");
    run("cargo llvm-cov --workspace --html --output-dir target/coverage", "Gerando cobertura")?;
    println!("📁 Relatório: target/coverage/index.html");
    Ok(())
}

fn doc() -> Result<()> {
    step("📚 Documentação");
    run("cargo doc --workspace --no-deps --document-private-items", "Gerando documentação")?;
    println!("📁 Documentação: target/doc/cathedral_os/index.html");
    Ok(())
}

fn bench() -> Result<()> {
    step("⚡ Benchmarks");
    run("cargo bench --workspace", "Executando benchmarks")?;
    Ok(())
}

fn deny() -> Result<()> {
    step("🔍 Verificação de dependências");
    run("cargo deny check", "Verificando licenças e dependências")?;
    Ok(())
}

fn audit() -> Result<()> {
    step("🔐 Verificação de vulnerabilidades");
    run("cargo audit --deny-warnings", "Auditando crates")?;
    Ok(())
}

fn test() -> Result<()> {
    step("🧪 Testes");
    run("cargo test --workspace", "Executando todos os testes")?;
    Ok(())
}

fn sbom() -> Result<()> {
    step("📦 SBOM (Software Bill of Materials)");
    run("cargo sbom --output target/sbom.json", "Gerando SBOM")?;
    println!("📁 target/sbom.json");
    Ok(())
}

fn deadlinks() -> Result<()> {
    step("🔗 Verificação de links quebrados");
    run("cargo deadlinks --check-http", "Verificando links")?;
    Ok(())
}

fn update() -> Result<()> {
    step("🔄 Atualizando dependências");
    run("cargo update", "Atualizando Cargo.lock")?;
    Ok(())
}

fn clean() -> Result<()> {
    step("🧹 Limpando artefatos");
    run("cargo clean", "Limpando target/")?;
    Ok(())
}

fn all_checks() -> Result<()> {
    step("🚀 Executando todos os checks");
    pre_commit()?;
    ci()?;
    full_audit()?;
    Ok(())
}

// ============================================================================
// HELPERS
// ============================================================================

fn step(msg: &str) {
    println!("\n{}", msg.bold().cyan());
    println!("{}", "─".repeat(60));
}

fn run(cmd: &str, description: &str) -> Result<()> {
    print!("  ▶ {} ... ", description);
    let start = Instant::now();

    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Falha ao executar: {}", cmd))?;

    let elapsed = start.elapsed().as_secs_f64();
    if status.success() {
        println!("✅ ({:.2}s)", elapsed);
        Ok(())
    } else {
        println!("❌ ({:.2}s)", elapsed);
        Err(anyhow!("Comando falhou: {}", cmd))
    }
}