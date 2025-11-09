# Pump-RS Installation & Testing Guide

## ⚠️ SAFETY WARNING

**THIS BOT RUNS ON MAINNET AND CAN CAUSE REAL FINANCIAL LOSSES!**

- Start with MINIMAL funds (0.1-0.5 SOL for testing)
- The memecoin market has changed significantly since September 2024
- Success in the past does NOT guarantee current profitability
- Test thoroughly before deploying with real capital

---

## Prerequisites

### 1. System Requirements

```bash
# Rust (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Solana CLI (optional but recommended)

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Verify
solana --version

# Set to mainnet
solana config set --url https://api.mainnet-beta.solana.com
```

---

## Installation Steps

### Step 1: Install Protobuf Compiler

The project requires `protoc` (Protocol Buffers compiler) for the Jito dependencies:

```bash
# Install protoc via cargo (one-time setup)
cargo install protoc-bin-vendored

# Verify installation
protoc-bin-which
```

### Step 2: Build the Project

```bash
cd /home/user/pump-rs

# Use the build script (automatically sets PROTOC)
./build.sh release

# OR build manually
export PROTOC=$(protoc-bin-which)
cargo build --release

# Verify build succeeded
ls -lh target/release/pump-rs
```

### Step 3: Create Wallet Keypairs

You need TWO wallets:

#### A. Main Trading Wallet (FUND_KEYPAIR)

```bash
# Option 1: Create new wallet
solana-keygen new -o ~/.config/solana/fund-wallet.json

# Option 2: Use existing wallet
# Just copy your existing keypair.json to a known location

# Check the wallet address
solana-keygen pubkey ~/.config/solana/fund-wallet.json

# Check balance (should have at least 0.5 SOL for testing)
solana balance $(solana-keygen pubkey ~/.config/solana/fund-wallet.json)
```

#### B. Jito Authentication Wallet (AUTH_KEYPAIR)

```bash
# Create a new keypair for Jito authentication
solana-keygen new -o ~/.config/solana/jito-auth.json

# This wallet doesn't need SOL, it's just for authentication
```

### Step 4: Configure Environment Variables

```bash
# Copy the example file
cp .env.example .env

# Edit with your favorite editor
nano .env
# OR
vim .env
```

**Minimal .env configuration:**

```bash
# Use your wallet paths
FUND_KEYPAIR_PATH=/home/user/.config/solana/fund-wallet.json
AUTH_KEYPAIR_PATH=/home/user/.config/solana/jito-auth.json

# RPC URLs - HIGHLY RECOMMENDED to use paid RPC
RPC_URL=https://api.mainnet-beta.solana.com
WS_URL=wss://api.mainnet-beta.solana.com

# Jito Block Engine
BLOCK_ENGINE_URL=https://mainnet.block-engine.jito.wtf

# Logging
RUST_LOG=info
```

### Step 5: Fund Your Wallet

```bash
# Check your wallet address
solana-keygen pubkey ~/.config/solana/fund-wallet.json

# Send 0.5 SOL to this address for testing
# You can use any Solana wallet (Phantom, Solflare, etc.)
```

---

## Testing & Verification

### Test 1: Sanity Check (SAFE - No transactions)

This verifies your configuration without spending money:

```bash
./target/release/pump-rs sanity
```

**Expected output:**
```
Wallet: <your_wallet_address>
Balance: <your_balance_in_lamports>
Auth: <jito_auth_address>
RPC: https://api.mainnet-beta.solana.com
Block Engine: https://mainnet.block-engine.jito.wtf
```

**If this fails:** Check your .env file paths and RPC connectivity.

---

### Test 2: Check Pump.fun Integration (SAFE - Read only)

Test fetching data from a known pump.fun token:

```bash
# Get slot created for a known token
./target/release/pump-rs slot-created --mint FASTykZyyjVfhutuRzMMYbFbFacQpRnMzDguhWfWadbi
```

**Expected output:**
```
FASTykZyyjVfhutuRzMMYbFbFacQpRnMzDguhWfWadbi: created <slot_number>
```

**If this fails:** Pump.fun API or RPC issues.

---

### Test 3: WebSocket Connection Tests (SAFE - Read only)

Test connection to pump.fun and pumpportal websockets:

```bash
# Test pump.fun websocket (Ctrl+C to exit after ~10 seconds)
./target/release/pump-rs subscribe-pump

# Test pumpportal connection speed
./target/release/pump-rs bench-portal

# Test pump.fun connection speed
./target/release/pump-rs bench-pump
```

**Expected behavior:**
- You should see token creation events streaming
- Connection latency should be displayed
- Ctrl+C to exit

**If this fails:** Network/firewall issues or websocket endpoints changed.

---

### Test 4: Jito Bundle Listener (SAFE - Read only)

```bash
# Listen to Jito bundle results (Ctrl+C to exit)
./target/release/pump-rs bundle-status-listener
```

**Expected behavior:**
- Should connect without errors
- May show bundle results if other traders are active
- Ctrl+C to exit

---

### Test 5: Check Your Token Holdings (SAFE - Read only)

```bash
# See all your pump.fun token holdings
./target/release/pump-rs wallets --token-balances
```

---

## Test Trading (⚠️ COSTS REAL MONEY)

### Test 6: Manual Buy Test (SMALL AMOUNT)

**WARNING:** This will spend real SOL!

```bash
# Enter swap mode to manually buy tokens
./target/release/pump-rs swap-mode --lamports 10000000

# This is 0.01 SOL per trade
# When prompted, paste a pump.fun token mint address
# Type 'q' to quit
```

**How to use:**
1. Find a pump.fun token on https://pump.fun
2. Copy its mint address (starts with capital letter, ~44 chars)
3. Paste it when prompted
4. Confirm the transaction

**Expected outcome:**
- Transaction signature printed
- Check on Solana Explorer: https://solscan.io

---

### Test 7: Manual Sell Test (IF YOU HAVE TOKENS)

```bash
# Enter sell mode
./target/release/pump-rs swap-mode --lamports 0 --sell

# Paste the mint address of a token you own
# It will sell your entire balance
```

---

## Understanding the Sniping Features

### ⚠️ Critical Limitation: NO SHREDSTREAM WEBHOOK

The code has these sniping modes, but **they rely on a private Shredstream webhook** that's NOT included:

```bash
# These commands exist but won't work optimally without Shredstream:
./target/release/pump-rs snipe-pump --lamports 50000000
./target/release/pump-rs snipe-portal --lamports 50000000
./target/release/pump-rs pump-service --lamports 50000000
```

**What they do:**
- `snipe-pump`: Listens to pump.fun websocket API (5-10 slots delay)
- `snipe-portal`: Listens to pumpportal.fun API (1-5 slots delay)
- `pump-service`: HTTP service expecting Shredstream webhook (0.5 slots delay)

**Without Shredstream:**
- You'll be 3-10 slots slower than the original performance
- Other bots with Shredstream will beat you
- Still works, but significantly less profitable

---

## Advanced: Running the HTTP Service

The `/v2/pump-buy` endpoint expects a private Shredstream webhook:

```bash
# Start the service (port 6969)
./target/release/pump-rs pump-service --lamports 50000000

# In another terminal, test health endpoint
curl http://localhost:6969/healthz
```

**Expected response:**
```json
{"status":"im ok, hit me with pump stuff"}
```

**To actually use this:** You need to build a Shredstream webhook deserializer that sends POST requests to `http://your-server:6969/v2/pump-buy`

---

## Troubleshooting

### Error: "RPC rate limit exceeded"

**Solution:** Use a paid RPC provider:
- Helius: https://helius.dev (recommended)
- QuickNode: https://quicknode.com
- Triton: https://triton.one

Update your .env:
```bash
RPC_URL=https://mainnet.helius-rpc.com/?api-key=YOUR_KEY
```

### Error: "AUTH_KEYPAIR_PATH env var not set"

**Solution:** Check your .env file exists and is in the current directory:
```bash
cat .env
pwd
```

### Error: "Failed to send bundle"

**Possible causes:**
1. Network issues with Jito
2. Insufficient balance for transaction + tip
3. Transaction errors (slippage, bonding curve full, etc.)

### Bundles Rejected/Dropped

**This is normal!** Jito bundles can be:
- Accepted → Processed → Finalized ✅
- Rejected (simulation failed, tip too low, etc.)
- Dropped (didn't win MEV auction)

You'll need to monitor the `bundle-status-listener` to understand why.

---

## Next Steps

1. ✅ **If basic tests pass:** Your setup is correct
2. ⚠️ **Before live trading:**
   - Apply for Jito Shredstream access (https://docs.jito.wtf/lowlatencytxnfeed/)
   - Build the Shredstream webhook deserializer
   - Update dependencies to Solana SDK 2.3+
3. 💰 **For production:**
   - Use paid RPC with high rate limits
   - Monitor performance metrics
   - Start with small position sizes (0.01-0.05 SOL)
   - Set up proper logging and monitoring

---

## Safety Reminders

- ⚠️ Never commit your .env file or wallet keypairs to git
- ⚠️ Test with minimal funds first
- ⚠️ The memecoin market is extremely risky
- ⚠️ Bot performance from July-August 2024 may not reflect current conditions
- ⚠️ Most pump.fun tokens fail (99%+ according to the README)

**Good luck, and trade responsibly!** 🚀
