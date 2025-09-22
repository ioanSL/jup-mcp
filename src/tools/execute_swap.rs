use crate::{Config, JupiterMcpError, Result};
use crate::mcp::{Tool, ToolInputSchema, ToolResponse};
use crate::utils::{get_connection, load_wallet, get_explorer_url};
use crate::tools::get_quote::QuoteResponse;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_sdk::{
    signature::Signer,
    transaction::VersionedTransaction,
};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapRequest {
    #[serde(rename = "quoteResponse")]
    pub quote_response: QuoteResponse,
    #[serde(rename = "userPublicKey")]
    pub user_public_key: Option<String>,
    #[serde(rename = "wrapAndUnwrapSol")]
    pub wrap_and_unwrap_sol: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapResponse {
    #[serde(rename = "swapTransaction")]
    pub swap_transaction: String,
}

pub struct ExecuteSwapTool;

impl ExecuteSwapTool {
    pub fn definition() -> Tool {
        Tool {
            name: "execute_swap".to_string(),
            description: "Execute a swap transaction using Jupiter AG".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: json!({
                    "quoteResponse": {
                        "type": "object",
                        "description": "Quote response from get_quote tool"
                    },
                    "userPublicKey": {
                        "type": "string",
                        "description": "User public key (optional, defaults to wallet)"
                    },
                    "wrapAndUnwrapSol": {
                        "type": "boolean",
                        "description": "Whether to wrap/unwrap SOL (default: true)"
                    }
                }),
                required: Some(vec!["quoteResponse".to_string()]),
            },
        }
    }
    
    pub async fn execute(config: &Config, args: Value) -> Result<ToolResponse> {
        let request: SwapRequest = serde_json::from_value(args)
            .map_err(|e| JupiterMcpError::InvalidInput(format!("Invalid arguments: {}", e)))?;
        
        let connection = get_connection(config);
        let wallet = load_wallet(config)?;
        
        let user_public_key = request.user_public_key
            .unwrap_or_else(|| wallet.pubkey().to_string());
        
        let wrap_and_unwrap_sol = request.wrap_and_unwrap_sol.unwrap_or(true);
        
        // Prepare swap request for Jupiter API
        let mut swap_request_body = HashMap::new();
        swap_request_body.insert("quoteResponse", serde_json::to_value(&request.quote_response)?);
        swap_request_body.insert("userPublicKey", json!(user_public_key));
        swap_request_body.insert("wrapAndUnwrapSol", json!(wrap_and_unwrap_sol));
        
        // Get swap transaction from Jupiter API
        let client = reqwest::Client::new();
        let response = client
            .post("https://quote-api.jup.ag/v6/swap")
            .header("Content-Type", "application/json")
            .json(&swap_request_body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(JupiterMcpError::JupiterApi(
                format!("Jupiter swap API error {}: {}", status, error_text)
            ));
        }
        
        let swap_response: SwapResponse = response.json().await?;
        
        // Deserialize the transaction from Jupiter
        use base64::{Engine as _, engine::general_purpose};
        let transaction_bytes = general_purpose::STANDARD.decode(&swap_response.swap_transaction)
            .map_err(|e| JupiterMcpError::SolanaSdk(format!("Failed to decode transaction: {}", e)))?;
        
        // Deserialize as VersionedTransaction
        let transaction: VersionedTransaction = bincode::deserialize(&transaction_bytes)
            .map_err(|e| JupiterMcpError::SolanaSdk(format!("Failed to deserialize transaction: {}", e)))?;
        
        // Send the transaction
        use solana_client::rpc_config::RpcSendTransactionConfig;
        let send_config = RpcSendTransactionConfig {
            skip_preflight: false,
            ..Default::default()
        };
        
        let signature = connection.send_transaction_with_config(&transaction, send_config)?;
        
        // Confirm the transaction
        connection.confirm_transaction(&signature)?;
        
        let explorer_url = get_explorer_url(&signature, config);
        
        let response_text = format!(
            "Swap executed successfully!\n\
            Signature: {}\n\
            Explorer: {}",
            signature,
            explorer_url
        );
        
        Ok(ToolResponse::text(response_text))
    }
}