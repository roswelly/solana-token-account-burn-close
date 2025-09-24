# Solana Token Account Burn & Close (Rust)

A Rust tool for burning and closing unnecessary token accounts on Solana to recover SOL. This is a Rust port of the original TypeScript implementation.

## Features

- **Automatic Token Account Cleanup**: Identifies and processes all token accounts for a given wallet
- **USDC Protection**: Option to preserve USDC token accounts (common stablecoin)
- **SOL Recovery**: Closes empty token accounts to recover rent-exempt SOL
- **Batch Processing**: Handles multiple accounts in a single transaction (configurable limit)
- **Transaction Simulation**: Tests transactions before execution to prevent failures
- **Comprehensive Error Handling**: Robust error handling with detailed logging
- **Environment Configuration**: Uses environment variables for configuration
- **Compute Optimization**: Configurable compute unit limits and pricing

## Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- A Solana wallet with token accounts to clean up
- Access to a Solana RPC endpoint

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd solana-token-account-burn-close
```

2. Build the project:
```bash
cargo build --release
```

## Configuration

1. Copy the example environment file:
```bash
cp env.example .env
```

2. Edit `.env` with your configuration:
```bash
# Required
RPC_ENDPOINT=https://api.mainnet-beta.solana.com
PRIVATE_KEY=your_base58_encoded_private_key_here

# Optional
SKIP_USDC=true
MAX_INSTRUCTIONS=22
COMPUTE_UNIT_PRICE=220000
COMPUTE_UNIT_LIMIT=350000
```

### Environment Variables

- `RPC_ENDPOINT`: Solana RPC endpoint URL (required)
- `PRIVATE_KEY`: Base58-encoded private key for your wallet (required)
- `SKIP_USDC`: Skip USDC token accounts (default: true)
- `MAX_INSTRUCTIONS`: Maximum instructions per transaction (default: 22)
- `COMPUTE_UNIT_PRICE`: Compute unit price in micro-lamports (default: 220000)
- `COMPUTE_UNIT_LIMIT`: Compute unit limit (default: 350000)

## Usage

### Command Line Arguments

You can override environment variables using command line arguments:

```bash
cargo run -- --help
```

### Basic Usage

```bash
# Using environment variables
cargo run

# Using command line arguments
cargo run -- --rpc-endpoint https://api.mainnet-beta.solana.com --private-key your_private_key

# Skip USDC accounts
cargo run -- --skip-usdc true

# Custom batch size
cargo run -- --max-instructions 15
```

### Production Usage

```bash
# Build optimized release binary
cargo build --release

# Run the optimized binary
./target/release/solana-token-burn-close
```

## How It Works

1. **Wallet Connection**: Connects to Solana using the provided RPC endpoint
2. **Token Account Discovery**: Retrieves all token accounts owned by the wallet
3. **Account Processing**: 
   - Optionally skips USDC token accounts
   - Burns tokens with non-zero balances
   - Closes all token accounts to recover SOL
4. **Transaction Execution**: 
   - Batches instructions per transaction (configurable limit)
   - Sets compute unit price and limit
   - Simulates transactions before execution
   - Provides Solscan transaction links upon completion

## Safety Features

- **Transaction Simulation**: All transactions are simulated before execution
- **USDC Protection**: Option to preserve USDC accounts by default
- **Error Handling**: Comprehensive error handling with detailed logging
- **Batch Processing**: Prevents oversized transactions
- **Compute Budget**: Configurable compute unit limits to prevent failures

## Logging

The tool provides detailed logging at different levels:

- `INFO`: General progress and successful operations
- `WARN`: Non-critical issues (e.g., simulation failures)
- `ERROR`: Critical errors that prevent execution

Set the log level using the `RUST_LOG` environment variable:

```bash
RUST_LOG=info cargo run
RUST_LOG=debug cargo run  # For more detailed output
```

## Dependencies

- `solana-client`: Solana RPC client
- `solana-sdk`: Core Solana SDK
- `spl-token`: SPL Token program utilities
- `tokio`: Async runtime
- `anyhow`: Error handling
- `clap`: Command line argument parsing
- `env_logger`: Logging
- `dotenv`: Environment variable loading

## Security Considerations

- **Private Key**: Never commit your private key to version control
- **RPC Endpoint**: Use a reliable RPC provider
- **Testnet First**: Test on devnet/testnet before using on mainnet
- **Backup**: Ensure you have backups of important token accounts

## Troubleshooting

### Common Issues

1. **"Failed to decode base58 private key"**
   - Ensure your private key is base58 encoded
   - Check for typos in the private key

2. **"Failed to fetch token accounts"**
   - Verify your RPC endpoint is correct and accessible
   - Check your internet connection

3. **"Transaction simulation failed"**
   - Your wallet may not have enough SOL for transaction fees
   - Some token accounts may be frozen or have special restrictions

4. **"Failed to send and confirm transaction"**
   - Network congestion may cause timeouts
   - Try reducing the batch size with `--max-instructions`

### Getting Help

- Check the logs for detailed error messages
- Verify your configuration in the `.env` file
- Test with a smaller batch size first
- Ensure you have sufficient SOL for transaction fees

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Disclaimer

This tool interacts with the Solana blockchain and can permanently burn tokens and close accounts. Use at your own risk. Always test on devnet/testnet first and ensure you understand the implications of burning tokens and closing accounts.