use crate::error::{JupiterMcpError, Result};
use solana_sdk::commitment_config::CommitmentConfig;

#[derive(Debug, Clone)]
pub enum SolanaNetwork {
    MainnetBeta,
    Testnet,
    Devnet,
}

impl SolanaNetwork {
    pub fn rpc_url(&self) -> &'static str {
        match self {
            SolanaNetwork::MainnetBeta => "https://api.mainnet-beta.solana.com",
            SolanaNetwork::Testnet => "https://api.testnet.solana.com",
            SolanaNetwork::Devnet => "https://api.devnet.solana.com",
        }
    }
    
    pub fn cluster_param(&self) -> Option<&'static str> {
        match self {
            SolanaNetwork::MainnetBeta => None,
            SolanaNetwork::Testnet => Some("testnet"),
            SolanaNetwork::Devnet => Some("devnet"),
        }
    }
}

impl std::str::FromStr for SolanaNetwork {
    type Err = JupiterMcpError;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "mainnet-beta" | "mainnet" => Ok(SolanaNetwork::MainnetBeta),
            "testnet" => Ok(SolanaNetwork::Testnet),
            "devnet" => Ok(SolanaNetwork::Devnet),
            _ => Err(JupiterMcpError::Environment(
                format!("Invalid network: {}. Use 'mainnet-beta', 'testnet', or 'devnet'", s)
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub network: SolanaNetwork,
    pub rpc_url: String,
    pub private_key: String,
    pub commitment: CommitmentConfig,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok(); // Load .env file if it exists
        
        let network: SolanaNetwork = std::env::var("SOLANA_NETWORK")
            .unwrap_or_else(|_| "devnet".to_string())
            .parse()?;
        
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| network.rpc_url().to_string());
        
        let private_key = std::env::var("SOLANA_PRIVATE_KEY")
            .map_err(|_| JupiterMcpError::Environment(
                "SOLANA_PRIVATE_KEY environment variable is required".to_string()
            ))?;
        
        Ok(Config {
            network,
            rpc_url,
            private_key,
            commitment: CommitmentConfig::confirmed(),
        })
    }
}