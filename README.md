# Jupiter AG MCP Server

A Model Context Protocol (MCP) server for Jupiter AG token swaps on Solana.

**⚠️ Disclaimer: This is a proof of concept. Use at your own risk and do not use with mainnet funds without proper testing and security audits.**

## Quick Start

### Prerequisites
- Rust (1.70+)
- Docker (optional)
- Solana wallet private key

### Setup

1. Create `.env` file:
```bash
SOLANA_PRIVATE_KEY=your_base58_private_key_here
SOLANA_NETWORK=devnet
SOLANA_RPC_URL=https://api.devnet.solana.com
RUST_LOG=info
```

### Run Locally

```bash
# Build and run
cargo run

# Or build release
cargo build --release
./target/release/jup-mcp
```

### Run with Docker

```bash
# Using docker-compose (recommended)
docker-compose up -d

# Or build and run manually
./scripts/build.sh
docker run --env-file .env jupiter-ag-mcp:latest
```

## Features

- Get token balances
- Get swap quotes
- Execute token swaps
- Solana wallet integration

## MCP Tools

The server provides these MCP tools:
- `get_balance` - Check token balances
- `get_quote` - Get swap quotes
- `execute_swap` - Perform token swaps

Connect this server to any MCP-compatible client to interact with Jupiter AG programmatically.