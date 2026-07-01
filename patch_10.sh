sed -i 's/Box<dyn std::error::Error>/Box<dyn std::error::Error + Send + Sync>/' crates/safe-core-bridge/src/mcp.rs
sed -i 's/.with_writer(tracing_subscriber::fmt::MakeWriter::new(std::io::stderr))/.with_writer(std::io::stderr)/' crates/safe-core-bridge/src/main.rs
