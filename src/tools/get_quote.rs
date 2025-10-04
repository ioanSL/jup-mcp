use crate::mcp::{Tool, ToolInputSchema, ToolResponse};
use crate::utils::parse_pubkey;
use crate::{Config, JupiterMcpError, Result};
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
    pub taker: String,
    #[serde(rename = "swapMode")]
    pub swap_mode: Option<String>,
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
            description: "Get a price quote for swapping tokens on Solana using Jupiter aggregator. This shows you how much of the output token you'll receive for a given amount of input token, including price impact and the best route.".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: json!({
                    "inputMint": {
                        "type": "string",
                        "description": "The token address (mint) you want to swap FROM (e.g., USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v)"
                    },
                    "outputMint": {
                        "type": "string", 
                        "description": "The token address (mint) you want to swap TO (e.g., SOL: So11111111111111111111111111111111111111112)"
                    },
                    "amount": {
                        "type": "string",
                        "description": "How much of the input token to swap (in the token's smallest unit - for USDC with 6 decimals, 1000000 = 1 USDC)"
                    },
                    "taker": {
                        "type": "string",
                        "description": "The wallet address that will perform the swap"
                    },
                    "swapMode": {
                        "type": "string",
                        "description": "Whether you want to specify an exact input amount (ExactIn) or exact output amount (ExactOut). Default is ExactIn."
                    },
                    "slippageBps": {
                        "type": "number",
                        "description": "Maximum acceptable slippage in basis points (100 bps = 1%). Default is 50 bps (0.5%). Higher values allow more price movement but ensure the swap completes."
                    }
                }),
                required: Some(vec![
                    "inputMint".to_string(),
                    "outputMint".to_string(), 
                    "amount".to_string(),
                    "taker".to_string()
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

        // Validate taker address
        parse_pubkey(&request.taker)?;

        // Validate amount
        request
            .amount
            .parse::<u64>()
            .map_err(|e| JupiterMcpError::InvalidInput(format!("Invalid amount: {}", e)))?;

        let slippage_bps = request.slippage_bps.unwrap_or(50);
        let swap_mode = request.swap_mode.unwrap_or_else(|| "ExactIn".to_string());

        // Build query parameters
        let mut params = HashMap::new();
        params.insert("inputMint", request.input_mint.clone());
        params.insert("outputMint", request.output_mint.clone());
        params.insert("amount", request.amount.clone());
        params.insert("taker", request.taker.clone());
        params.insert("swapMode", swap_mode);
        params.insert("slippageBps", slippage_bps.to_string());

        // Make request to Jupiter Ultra API
        let client = reqwest::Client::new();
        let response = client
            .get("https://ultra-api.jup.ag/order")
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(JupiterMcpError::JupiterApi(format!(
                "Jupiter API error {}: {}",
                status, error_text
            )));
        }

        let quote: QuoteResponse = response.json().await?;

        // Format route information
        let route_labels: Vec<String> = quote
            .route_plan
            .iter()
            .map(|r| r.swap_info.label.clone())
            .collect();

        let response_text = format!(
            "✅ Quote received for your swap:\n\n\
            📥 You will send: {} tokens\n\
            📤 You will receive: {} tokens\n\
            💹 Price impact: {}%\n\
            ⚡ Slippage tolerance: {} bps ({}%)\n\
            🛣️  Best route: {}\n\n\
            This quote is ready to use for executing the swap.",
            quote.in_amount,
            quote.out_amount,
            quote.price_impact_pct,
            quote.slippage_bps,
            (quote.slippage_bps as f64) / 100.0,
            route_labels.join(" → ")
        );

        Ok(ToolResponse::text(response_text))
    }
}
