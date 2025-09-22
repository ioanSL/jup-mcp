use crate::{Config, JupiterMcpError, Result};
use crate::mcp::{McpRequest, McpResponse, Tool, ToolCallParams, ToolResponse};
use crate::tools::{GetQuoteTool, ExecuteSwapTool, GetBalanceTool};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tracing::{error, info, warn};

pub struct McpServer {
    config: Config,
}

impl McpServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// Get list of available tools
    fn get_tools(&self) -> Vec<Tool> {
        vec![
            GetQuoteTool::definition(),
            ExecuteSwapTool::definition(),
            GetBalanceTool::definition(),
        ]
    }
    
    /// Handle tools/list request
    async fn handle_tools_list(&self) -> Result<Value> {
        let tools = self.get_tools();
        Ok(json!({ "tools": tools }))
    }
    
    /// Handle tools/call request
    async fn handle_tools_call(&self, params: Value) -> Result<ToolResponse> {
        let tool_params: ToolCallParams = serde_json::from_value(params)
            .map_err(|e| JupiterMcpError::InvalidInput(format!("Invalid tool call params: {}", e)))?;
        
        let args = tool_params.arguments.unwrap_or(json!({}));
        
        match tool_params.name.as_str() {
            "get_quote" => GetQuoteTool::execute(&self.config, args).await,
            "execute_swap" => ExecuteSwapTool::execute(&self.config, args).await,
            "get_token_balance" => GetBalanceTool::execute(&self.config, args).await,
            _ => Err(JupiterMcpError::InvalidInput(
                format!("Unknown tool: {}", tool_params.name)
            )),
        }
    }
    
    /// Handle incoming MCP request
    async fn handle_request(&self, request: McpRequest) -> McpResponse {
        let result = match request.method.as_str() {
            "tools/list" => {
                match self.handle_tools_list().await {
                    Ok(result) => Some(result),
                    Err(e) => {
                        error!("Error in tools/list: {}", e);
                        return McpResponse::error(request.id, -32603, e.to_string());
                    }
                }
            }
            "tools/call" => {
                match request.params {
                    Some(params) => {
                        match self.handle_tools_call(params).await {
                            Ok(tool_response) => Some(serde_json::to_value(tool_response).unwrap()),
                            Err(e) => {
                                error!("Error in tools/call: {}", e);
                                return McpResponse::error(request.id, -32603, e.to_string());
                            }
                        }
                    }
                    None => {
                        warn!("tools/call request missing params");
                        return McpResponse::error(request.id, -32602, "Missing params".to_string());
                    }
                }
            }
            "initialize" => {
                info!("Client initializing MCP connection");
                Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "jupiter-ag-mcp",
                        "version": "1.0.0"
                    }
                }))
            }
            _ => {
                warn!("Unknown method: {}", request.method);
                return McpResponse::error(request.id, -32601, "Method not found".to_string());
            }
        };
        
        match result {
            Some(result) => McpResponse::success(request.id, result),
            None => McpResponse::error(request.id, -32603, "Internal error".to_string()),
        }
    }
    
    /// Run the MCP server using stdio transport
    pub async fn run_stdio(&self) -> Result<()> {
        info!("Jupiter AG MCP Server starting on stdio");
        
        let stdin = tokio::io::stdin();
        let mut reader = AsyncBufReader::new(stdin);
        let mut stdout = tokio::io::stdout();
        
        let mut line = String::new();
        
        loop {
            line.clear();
            
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF reached
                    info!("Client disconnected");
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    
                    // Parse the JSON-RPC request
                    let request: McpRequest = match serde_json::from_str(trimmed) {
                        Ok(req) => req,
                        Err(e) => {
                            error!("Failed to parse request: {} - Input: {}", e, trimmed);
                            let error_response = McpResponse::error(
                                "unknown".to_string(),
                                -32700,
                                "Parse error".to_string(),
                            );
                            let response_json = serde_json::to_string(&error_response)?;
                            stdout.write_all(response_json.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                            continue;
                        }
                    };
                    
                    info!("Handling request: {}", request.method);
                    
                    // Handle the request
                    let response = self.handle_request(request).await;
                    
                    // Send the response
                    let response_json = serde_json::to_string(&response)?;
                    stdout.write_all(response_json.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                    
                    info!("Sent response");
                }
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }
        
        info!("Jupiter AG MCP Server shutting down");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SolanaNetwork;
    
    #[tokio::test]
    async fn test_tools_list() {
        let config = Config {
            network: SolanaNetwork::Devnet,
            rpc_url: "https://api.devnet.solana.com".to_string(),
            private_key: "test_key".to_string(),
            commitment: solana_sdk::commitment_config::CommitmentConfig::confirmed(),
        };
        
        let server = McpServer::new(config);
        let tools = server.get_tools();
        
        assert_eq!(tools.len(), 3);
        assert!(tools.iter().any(|t| t.name == "get_quote"));
        assert!(tools.iter().any(|t| t.name == "execute_swap"));
        assert!(tools.iter().any(|t| t.name == "get_token_balance"));
    }
}