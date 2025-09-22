use crate::{Config, JupiterMcpError, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
};
use std::str::FromStr;

/// Get a configured Solana RPC client
pub fn get_connection(config: &Config) -> RpcClient {
    RpcClient::new_with_commitment(&config.rpc_url, config.commitment)
}

/// Load wallet from private key in config
pub fn load_wallet(config: &Config) -> Result<Keypair> {
    let decoded = bs58::decode(&config.private_key).into_vec()?;
    
    Keypair::from_bytes(&decoded).map_err(|e| {
        JupiterMcpError::SolanaSdk(format!("Invalid private key format: {}", e))
    })
}

/// Format lamports as SOL with proper decimal places
pub fn format_sol(lamports: u64) -> String {
    format!("{:.9}", lamports as f64 / 1_000_000_000.0)
}

/// Get explorer URL for a transaction signature
pub fn get_explorer_url(signature: &Signature, config: &Config) -> String {
    let base_url = "https://explorer.solana.com/tx";
    match config.network.cluster_param() {
        Some(cluster) => format!("{}?cluster={}", base_url, cluster),
        None => format!("{}/{}", base_url, signature),
    }
}

/// Parse a public key from string with better error handling
pub fn parse_pubkey(s: &str) -> Result<Pubkey> {
    Pubkey::from_str(s).map_err(|e| {
        JupiterMcpError::InvalidInput(format!("Invalid public key '{}': {}", s, e))
    })
}

/// Validate amount string and convert to u64
pub fn parse_amount(amount_str: &str) -> Result<u64> {
    amount_str.parse::<u64>().map_err(|e| {
        JupiterMcpError::InvalidInput(format!("Invalid amount '{}': {}", amount_str, e))
    })
}

/// Format token amount with proper decimals
pub fn format_token_amount(amount: u64, decimals: u8) -> String {
    let divisor = 10_u64.pow(decimals as u32) as f64;
    format!("{:.prec$}", amount as f64 / divisor, prec = decimals as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_sol() {
        assert_eq!(format_sol(1_000_000_000), "1.000000000");
        assert_eq!(format_sol(500_000_000), "0.500000000");
        assert_eq!(format_sol(1), "0.000000001");
    }
    
    #[test]
    fn test_parse_amount() {
        assert_eq!(parse_amount("1000").unwrap(), 1000);
        assert!(parse_amount("invalid").is_err());
        assert!(parse_amount("-100").is_err());
    }
    
    #[test]
    fn test_format_token_amount() {
        assert_eq!(format_token_amount(1_000_000, 6), "1.000000");
        assert_eq!(format_token_amount(500_000, 6), "0.500000");
    }
}