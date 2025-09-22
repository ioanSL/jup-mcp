use crate::{Config, JupiterMcpError, Result};
use crate::mcp::{Tool, ToolInputSchema, ToolResponse};
use crate::utils::{get_connection, parse_pubkey, format_sol, format_token_amount};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, program_pack::Pack};
use spl_token::state::{Account as TokenAccount, Mint};

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceRequest {
    #[serde(rename = "walletAddress")]
    pub wallet_address: String,
    #[serde(rename = "tokenMint")]
    pub token_mint: Option<String>,
}

pub struct GetBalanceTool;

impl GetBalanceTool {
    pub fn definition() -> Tool {
        Tool {
            name: "get_token_balance".to_string(),
            description: "Get SOL or SPL token balance for a wallet".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: json!({
                    "walletAddress": {
                        "type": "string",
                        "description": "Wallet address to check balance for"
                    },
                    "tokenMint": {
                        "type": "string",
                        "description": "Token mint address (optional, omit for SOL balance)"
                    }
                }),
                required: Some(vec!["walletAddress".to_string()]),
            },
        }
    }
    
    pub async fn execute(config: &Config, args: Value) -> Result<ToolResponse> {
        let request: BalanceRequest = serde_json::from_value(args)
            .map_err(|e| JupiterMcpError::InvalidInput(format!("Invalid arguments: {}", e)))?;
        
        let connection = get_connection(config);
        let wallet_pubkey = parse_pubkey(&request.wallet_address)?;
        
        match request.token_mint {
            None => {
                // Get SOL balance
                let balance = connection.get_balance(&wallet_pubkey)?;
                let formatted_balance = format_sol(balance);
                Ok(ToolResponse::text(format!("SOL Balance: {} SOL", formatted_balance)))
            }
            Some(mint_address) => {
                // Get SPL token balance
                let mint_pubkey = parse_pubkey(&mint_address)?;
                let balance_result = get_token_balance(&connection, &wallet_pubkey, &mint_pubkey)?;
                
                match balance_result {
                    Some((balance, decimals)) => {
                        let formatted_balance = format_token_amount(balance, decimals);
                        Ok(ToolResponse::text(format!("Token Balance: {}", formatted_balance)))
                    }
                    None => {
                        Ok(ToolResponse::text("Token account not found - Balance: 0".to_string()))
                    }
                }
            }
        }
    }
}

fn get_token_balance(
    connection: &RpcClient,
    wallet_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
) -> Result<Option<(u64, u8)>> {
    // Get all token accounts for this wallet filtered by mint
    let token_accounts = connection.get_token_accounts_by_owner(
        wallet_pubkey,
        solana_client::rpc_request::TokenAccountsFilter::Mint(*mint_pubkey),
    )?;
    
    if token_accounts.is_empty() {
        return Ok(None);
    }
    
    // Get the first (and should be only) token account
    let token_account_pubkey = parse_pubkey(&token_accounts[0].pubkey)?;
    
    // Get account data
    let account_data = connection.get_account_data(&token_account_pubkey)?;
    
    // Parse token account
    let token_account = TokenAccount::unpack(&account_data)
        .map_err(|e| JupiterMcpError::SolanaSdk(format!("Failed to parse token account: {}", e)))?;
    
    // Get mint info for decimals
    let mint_data = connection.get_account_data(mint_pubkey)?;
    let mint = Mint::unpack(&mint_data)
        .map_err(|e| JupiterMcpError::SolanaSdk(format!("Failed to parse mint: {}", e)))?;
    
    Ok(Some((token_account.amount, mint.decimals)))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_balance_request_deserialization() {
        let json = json!({
            "walletAddress": "11111111111111111111111111111112"
        });
        
        let request: BalanceRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.wallet_address, "11111111111111111111111111111112");
        assert!(request.token_mint.is_none());
    }
    
    #[test]
    fn test_balance_request_with_token_mint() {
        let json = json!({
            "walletAddress": "11111111111111111111111111111112",
            "tokenMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        });
        
        let request: BalanceRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.wallet_address, "11111111111111111111111111111112");
        assert_eq!(request.token_mint.unwrap(), "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    }
}