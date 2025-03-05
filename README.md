# SPL 2022 Token with Buy/Sell Fees and WBTC Rewards

This project implements a custom SPL 2022 token on the Solana blockchain with built-in buy/sell fees, WBTC rewards distribution, and automatic liquidity provision. The token includes:

- 5% fee on buy transactions
- 5% fee on sell transactions
- Automatic fee collection and conversion to WBTC
- WBTC rewards distribution to token holders every 30 minutes
- Reserve wallet for liquidity provision
- Automatic liquidity addition to DEX pools
- Standard SPL token functionality

## Project Structure

```
.
├── src/               # Source code for the token program
│   ├── lib.rs        # Main token program
│   └── rewards.rs    # Rewards distribution program
├── tests/            # Test files
├── Cargo.toml        # Rust dependencies and project configuration
└── README.md         # This file
```

## Features

- **Buy Fee**: 5% fee on all buy transactions
- **Sell Fee**: 5% fee on all sell transactions
- **Fee Collection**: Automatic collection and conversion to WBTC
- **Rewards Distribution**:
  - 50% of collected WBTC distributed to token holders
  - Distribution every 30 minutes
  - Proportional to holder's token balance
- **Reserve System**:
  - 50% of collected WBTC sent to reserve wallet
  - Automatic liquidity provision to DEX pools
  - Liquidity addition every 30 minutes
- **Standard SPL Features**:
  - Token minting
  - Token transfers
  - Associated token accounts
  - Decimals configuration

## Prerequisites

- Rust and Cargo
- Solana CLI tools
- Node.js (for testing)
- Jupiter DEX integration (for SOL to WBTC swaps)
- DEX integration (for liquidity provision)

## Building

```bash
cargo build-bpf
```

## Testing

```bash
cargo test
```

## Deployment

1. Build the program:

```bash
cargo build-bpf
```

2. Deploy to your chosen Solana network (devnet/testnet/mainnet):

```bash
solana program deploy target/deploy/spl_2022_token.so
```

## Usage Instructions

### 1. Initialize Token Mint

When initializing the token mint, you need to provide:

- Number of decimals
- Mint authority (optional)
- Fee collector account
- Rewards program account
- Reserve wallet account

The program will automatically set up:

- 5% buy fee
- 5% sell fee
- Fee collection mechanism
- Rewards distribution system
- Reserve wallet system

### 2. Mint Tokens

To mint new tokens:

```bash
solana program invoke <PROGRAM_ID> <INSTRUCTION_DATA> --keypair <KEYPAIR> <ACCOUNTS>
```

Required accounts:

- Mint account
- Destination account
- Authority account
- Token program

### 3. Transfer Tokens (Buy/Sell)

When transferring tokens, specify whether it's a buy or sell operation:

For Buy:

```bash
solana program invoke <PROGRAM_ID> <INSTRUCTION_DATA> --keypair <KEYPAIR> <ACCOUNTS>
```

- Set `is_buy = true`
- 5% fee will be collected
- Fee will be converted to WBTC

For Sell:

```bash
solana program invoke <PROGRAM_ID> <INSTRUCTION_DATA> --keypair <KEYPAIR> <ACCOUNTS>
```

- Set `is_buy = false`
- 5% fee will be collected
- Fee will be converted to WBTC

Required accounts for transfers:

- Source account
- Destination account
- Authority account
- Token program
- Mint account
- Fee collector account
- Rewards program account
- Reserve wallet account

### 4. Rewards and Reserve Distribution

The system automatically:

1. Collects fees from buy/sell transactions
2. Converts fees to WBTC using Jupiter DEX
3. Every 30 minutes:
   - Distributes 50% of WBTC to token holders
   - Sends 50% to reserve wallet
   - Adds liquidity to DEX pools from reserve wallet

## Fee and Rewards Calculation Example

For a transfer of 1000 tokens:

Buy Transaction:

- Amount: 1000 tokens
- Fee (5%): 50 tokens
- Net amount received: 950 tokens
- Fee converted to WBTC and added to rewards pool

Sell Transaction:

- Amount: 1000 tokens
- Fee (5%): 50 tokens
- Net amount received: 950 tokens
- Fee converted to WBTC and added to rewards pool

Rewards Distribution (every 30 minutes):

- 50% of collected WBTC distributed to holders
- 50% sent to reserve wallet
- Distribution proportional to holder's token balance
- Example: If you hold 10% of total tokens, you receive 10% of the distributed WBTC

Reserve and Liquidity:

- Reserve wallet receives 50% of collected WBTC
- Liquidity is added to DEX pools every 30 minutes
- Liquidity threshold: 0.1 WBTC minimum
- Automatic market making for token stability

## Security Considerations

- All fees are automatically collected and converted to WBTC
- Fee rates are fixed at 5% for both buy and sell operations
- Only the mint authority can mint new tokens
- All transfers require proper authorization
- Rewards distribution is time-locked to 30-minute intervals
- Holder balances are automatically tracked and updated
- Reserve wallet is program-controlled
- Liquidity provision is automated and time-locked

## License

MIT
