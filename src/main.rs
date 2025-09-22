use jup_mcp::{server::McpServer, Config};
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("jup_mpc=info".parse().unwrap()),
        )
        .init();

    // Load configuration
    let config = match Config::from_env() {
        Ok(config) => {
            info!("Configuration loaded successfully");
            info!("Network: {:?}", config.network);
            info!("RPC URL: {}", config.rpc_url);
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Create and run the MCP server
    let server = McpServer::new(config);

    info!("Starting Jupiter AG MCP Server...");

    if let Err(e) = server.run_stdio().await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}
