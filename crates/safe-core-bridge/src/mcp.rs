#[cfg(feature = "mcp")]
pub mod mcp_impl {
    use crate::tools;
    use crate::state::BridgeState;
    use rmcp::{ServerHandler, ServiceExt, transport::stdio};
    use std::sync::Arc;

    #[derive(Clone)]
    pub struct SafeCoreMcpServer {
        state: Arc<BridgeState>,
    }

    impl SafeCoreMcpServer {
        pub fn new(state: Arc<BridgeState>) -> Self {
            Self { state }
        }

        pub async fn run_stdio(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.serve(transport::stdio()).await
        }
    }

    impl ServerHandler for SafeCoreMcpServer {}
}

#[cfg(not(feature = "mcp"))]
pub mod mcp_impl {
    pub struct SafeCoreMcpServer;

    impl SafeCoreMcpServer {
        pub fn new(_: std::sync::Arc<crate::state::BridgeState>) -> Self {
            Self
        }

        pub async fn run_stdio(self) -> Result<(), anyhow::Error> {
            Err(anyhow::anyhow!("MCP não disponível"))
        }
    }
}
