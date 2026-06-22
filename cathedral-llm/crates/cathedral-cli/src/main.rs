use clap::{Parser, Subcommand};
use reqwest::Client;
use serde_json::json;
use std::fs;

#[derive(Parser)]
#[command(name = "cathedral-cli")]
#[command(about = "CLI para Cathedral-LLM", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Infer {
        #[arg(short, long)]
        prompt: String,

        #[arg(short, long, default_value = "did:cathedral:agent:cathedral-llm-proto-001")]
        did: String,

        #[arg(short, long, default_value = "L0")]
        verification_level: String,

        #[arg(long, default_value_t = false)]
        show_thinking: bool,

        #[arg(long, default_value_t = false)]
        show_reputation: bool,

        #[arg(long, default_value_t = false)]
        show_tier: bool,

        #[arg(long)]
        signature_file: Option<String>,
    },

    Memory {
        #[arg(short, long)]
        did: String,

        #[arg(short, long, default_value_t = 10)]
        limit: usize,

        #[arg(long, default_value_t = false)]
        show_all: bool,
    },

    Status {
        #[arg(short, long)]
        did: String,
    },

    Search {
        #[arg(short, long)]
        did: String,

        #[arg(short, long)]
        query: String,

        #[arg(short, long, default_value_t = 5)]
        limit: usize,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = Client::new();
    let base_url = "http://localhost:8080";

    match cli.command {
        Commands::Infer {
            prompt,
            did,
            verification_level,
            show_thinking,
            show_reputation,
            show_tier,
            signature_file,
        } => {
            let signature = if let Some(path) = signature_file {
                hex::encode(fs::read(path).unwrap())
            } else {
                "00".repeat(64)
            };

            let resp = client
                .post(format!("{}/generate", base_url))
                .json(&json!({
                    "prompt": prompt,
                    "did": did,
                    "signature": signature,
                    "level": verification_level,
                }))
                .send()
                .await
                .unwrap()
                .json::<serde_json::Value>()
                .await
                .unwrap();

            let thinking = resp.get("thinking").and_then(|v| v.as_str()).unwrap_or("");
            let text = resp.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let reputation = resp.get("reputation").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let tier = resp.get("tier").and_then(|v| v.as_str()).unwrap_or("");

            if show_thinking && !thinking.is_empty() {
                println!("💭 Thinking:\n{}\n", thinking);
            }
            if show_reputation {
                println!("⭐ Reputation: {:.1}", reputation);
            }
            if show_tier {
                println!("🏷️  Model Tier: {}", tier);
            }
            println!("📝 Answer:\n{}", text);
            println!("\n🔏 Signature: {}", resp.get("signature").unwrap().as_str().unwrap_or(""));
            println!("📜 Receipt: {}", resp.get("receipt").unwrap().as_str().unwrap_or(""));
            println!("⏱️ Latency: {} ms", resp.get("latency_ms").unwrap_or(&json!(0)).as_u64().unwrap_or(0));
        }

        Commands::Memory { did, limit, show_all } => {
            let url = if show_all {
                format!("{}/memory/{}", base_url, did)
            } else {
                format!("{}/memory/{}?limit={}", base_url, did, limit)
            };
            let resp = client
                .get(&url)
                .send()
                .await
                .unwrap()
                .json::<serde_json::Value>()
                .await
                .unwrap();
            println!("{}", serde_json::to_string_pretty(&resp).unwrap());
        }

        Commands::Status { did } => {
            let resp = client
                .get(format!("{}/status/{}", base_url, did))
                .send()
                .await
                .unwrap()
                .json::<serde_json::Value>()
                .await
                .unwrap();
            println!("{}", serde_json::to_string_pretty(&resp).unwrap());
        }

        Commands::Search { did, query, limit } => {
            let resp = client
                .get(format!("{}/memory/{}/search?q={}&limit={}", base_url, did, query, limit))
                .send()
                .await
                .unwrap()
                .json::<serde_json::Value>()
                .await
                .unwrap();
            println!("{}", serde_json::to_string_pretty(&resp).unwrap());
        }
    }
}
