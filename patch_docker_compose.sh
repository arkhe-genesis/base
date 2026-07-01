cat << 'INNER_EOF' >> cathedral-arkhe/docker-compose.yml
  safe-core-bridge-mcp:
    build:
      context: .
      dockerfile: docker/Dockerfile.bridge
    container_name: safe-core-bridge-mcp
    ports:
      - "8081:8081"
    command: ["safe-core-bridge", "--mcp"]
    stdin_open: true
    tty: false
    environment:
      RUST_LOG: safe_core_bridge=warn
    restart: unless-stopped
    networks:
      - cathedral-net
INNER_EOF
