use crate::{Config, JupiterMcpError, Result};
use crate::mcp::{Tool, ToolInputSchema, ToolResponse};
use crate::utils::parse_pubkey;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteRequest {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    pub amount: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteResponse {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    pub other_amount_threshold: String,
    #[serde(rename = "swapMode")]
    pub swap_mode: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: String,
    #[serde(rename = "routePlan")]
    pub route_plan: Vec<RoutePlan>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoutePlan {
    #[serde(rename = "swapInfo")]
    pub swap_info: SwapInfo,
    pub percent: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapInfo {
    #[serde(rename = "ammKey")]
    pub amm_key: String,
    pub label: String,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "feeAmount")]
    pub fee_amount: String,
    #[serde(rename = "feeMint")]
    pub fee_mint: String,
}

pub struct GetQuoteTool;

impl GetQuoteTool {
    pub fn definition() -> Tool {
        Tool {
            name: "get_quote".to_string(),
            description: "Get a swap quote from Jupiter AG".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: json!({
                    "inputMint": {
                        "type": "string",
                        "description": "Input token mint address"
                    },
                    "outputMint": {
                        "type": "string", 
                        "description": "Output token mint address"
                    },
                    "amount": {
                        "type": "string",
                        "description": "Amount in token units"
                    },
                    "slippageBps": {
                        "type": "number",
                        "description": "Slippage in basis points (default: 50)"
                    }
                }),
                required: Some(vec![
                    "inputMint".to_string(),
                    "outputMint".to_string(), 
                    "amount".to_string()
                ]),
            },
        }
    }
    
    pub async fn execute(_config: &Config, args: Value) -> Result<ToolResponse> {
        let request: QuoteRequest = serde_json::from_value(args)
            .map_err(|e| JupiterMcpError::InvalidInput(format!("Invalid arguments: {}", e)))?;
        
        // Validate mint addresses
        parse_pubkey(&request.input_mint)?;
        parse_pubkey(&request.output_mint)?;
        
        // Validate amount
        request.amount.parse::<u64>()
            .map_err(|e| JupiterMcpError::InvalidInput(format!("Invalid amount: {}", e)))?;
        
        let slippage_bps = request.slippage_bps.unwrap_or(50);
        
        // Build query parameters
        let mut params = HashMap::new();
        params.insert("inputMint", request.input_mint.clone());
        params.insert("outputMint", request.output_mint.clone());
        params.insert("amount", request.amount.clone());
        params.insert("slippageBps", slippage_bps.to_string());
        
        // Make request to Jupiter API
        let client = reqwest::Client::new();
        let response = client
            .get("https://quote-api.jup.ag/v6/quote")
            .query(&params)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(JupiterMcpError::JupiterApi(
                format!("Jupiter API error {}: {}", status, error_text)
            ));
        }
        
        let quote: QuoteResponse = response.json().await?;
        
        // Format route information
        let route_labels: Vec<String> = quote.route_plan
            .iter()
            .map(|r| r.swap_info.label.clone())
            .collect();
        
        let response_text = format!(
            "Quote received:\n\
            Input: {} tokens\n\
            Output: {} tokens\n\
            Price Impact: {}%\n\
            Slippage: {} bps\n\
            Route: {}",
            quote.in_amount,
            quote.out_amount,
            quote.price_impact_pct,
            quote.slippage_bps,
            route_labels.join(" → ")
        );
        
        Ok(ToolResponse::text(response_text))
    }
}