sed -i 's/RUN cargo build --release -p safe-core-bridge/RUN cargo build --release -p safe-core-bridge --features mcp/' cathedral-arkhe/docker/Dockerfile.bridge
