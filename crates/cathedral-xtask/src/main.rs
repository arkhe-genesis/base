use std::{
    process::{Command, Stdio},
    time::Instant,
};

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use colored::*;
use which::which;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Cathedral ARKHE development tasks")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    CheckTools,
    PreCommit,
    Ci,
    FullAudit,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let start = Instant::now();

    match cli.command {
        Commands::CheckTools => check_tools()?,
        Commands::PreCommit => pre_commit()?,
        Commands::Ci => ci()?,
        Commands::FullAudit => full_audit()?,
    }

    println!("\n{}", "✅ Todas as verificações concluídas com sucesso!".green());
    println!("⏱️  Tempo total: {:.2}s", start.elapsed().as_secs_f64());
    Ok(())
}

fn check_tools() -> Result<()> {
    step("🔧 Verificando ferramentas instaladas");
    let tools = [
        "cargo",
        "cargo-fmt",
        "cargo-clippy",
        "cargo-deny",
        "cargo-audit",
        "cargo-semver-checks",
        "cargo-llvm-cov",
        "cargo-insta",
        "cargo-deadlinks",
        "cargo-sbom",
        "cargo-ndk",
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
        .map_err(|e| anyhow!("Falha ao executar: {} {}", cmd, e))?;

    let elapsed = start.elapsed().as_secs_f64();
    if status.success() {
        println!("✅ ({:.2}s)", elapsed);
        Ok(())
    } else {
        println!("❌ ({:.2}s)", elapsed);
        Err(anyhow!("Comando falhou: {}", cmd))
    }
}
