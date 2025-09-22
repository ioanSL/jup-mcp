use thiserror::Error;

#[derive(Error, Debug)]
pub enum JupiterMcpError {
    #[error("Solana client error: {0}")]
    SolanaClient(#[from] solana_client::client_error::ClientError),

    #[error("Solana SDK error: {0}")]
    SolanaSdk(String),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Base58 decode error: {0}")]
    Base58Decode(#[from] bs58::decode::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Environment error: {0}")]
    Environment(String),

    #[error("Jupiter API error: {0}")]
    JupiterApi(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("MCP protocol error: {0}")]
    McpProtocol(String),
}

pub type Result<T> = std::result::Result<T, JupiterMcpError>;
