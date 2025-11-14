# Pump-RS Command Reference

Complete guide to all available commands in pump-rs.

---

## 📋 **Table of Contents**

1. [Configuration & Testing](#configuration--testing)
2. [Wallet Management](#wallet-management)
3. [Token Operations](#token-operations)
4. [Monitoring & Analysis](#monitoring--analysis)
5. [Sniping & Trading](#sniping--trading)
6. [Services](#services)
7. [Advanced/Development](#advanceddevelopment)

---

## 🔧 **Configuration & Testing**

### **`sanity`** - Configuration Check
**Purpose:** Verify your `.env` configuration is correct

**Usage:**
```bash
./target/release/pump-rs sanity
```

**What it does:**
- ✅ Checks wallet keypair paths are valid
- ✅ Verifies RPC connectivity
- ✅ Tests Jito Block Engine connection
- ✅ Shows wallet balance

**Example output:**
```
Wallet: 7xKXt...9xyz
Balance: 500000000 (0.5 SOL)
Auth: 9yKLp...3abc
RPC: https://api.mainnet-beta.solana.com
Block Engine: https://mainnet.block-engine.jito.wtf
```

**Use when:** First time setup, troubleshooting connection issues

---

### **`is-on-curve`** - Check Pubkey Validity
**Purpose:** Check if a public key is a valid on-curve point

**Usage:**
```bash
./target/release/pump-rs is-on-curve --pubkey <PUBKEY>
```

**Example:**
```bash
./target/release/pump-rs is-on-curve --pubkey 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
```

**Output:** `true` or `false`

**Use when:** Validating addresses before sending transactions

---

## 💰 **Wallet Management**

### **`wallets`** - Wallet Management
**Purpose:** Create and manage trading wallets, check balances

**Usage:**
```bash
# Create new wallets
./target/release/pump-rs wallets --create <COUNT>

# View SOL balances
./target/release/pump-rs wallets

# View all token balances
./target/release/pump-rs wallets --token-balances
```

**Examples:**
```bash
# Create 10 new wallets
./target/release/pump-rs wallets --create 10

# Check balances
./target/release/pump-rs wallets
```

**What it does:**
- `--create <COUNT>`: Creates N new keypair wallets in WALLET_DIRECTORY
- Default: Shows SOL balances of all wallets
- `--token-balances`: Shows all SPL token holdings

**Created wallets are saved as:**
- Directory: `./wallets/` (or WALLET_DIRECTORY from .env)
- Filename format: `{PUBKEY}.json`

**Use when:** Setting up multi-wallet bundle launches, checking available funds

---

### **`wallets-fund`** - Fund Multiple Wallets
**Purpose:** Distribute SOL from main wallet to sub-wallets (for bundle launches)

**Usage:**
```bash
./target/release/pump-rs wallets-fund --lamports <AMOUNT>
```

**Example:**
```bash
# Fund each wallet with 0.02 SOL (20,000,000 lamports)
./target/release/pump-rs wallets-fund --lamports 20000000
```

**Requirements:**
- `WALLET_DIRECTORY` set in .env
- Multiple wallet keypairs in the directory
- Sufficient balance in FUND_KEYPAIR

**Use when:** Setting up bundle launch wallets

---

### **`wallets-drain`** - Collect SOL from Sub-Wallets
**Purpose:** Drain all SOL from sub-wallets back to main wallet

**Usage:**
```bash
./target/release/pump-rs wallets-drain
```

**What it does:**
- Sends all SOL from each sub-wallet to FUND_KEYPAIR
- Sent as Jito bundles for efficiency
- Leaves ~5000 lamports for rent

**Use when:** Consolidating funds after trading

---

### **`close-token-accounts`** - Close Empty Token Accounts
**Purpose:** Close empty SPL token accounts to recover rent

**Usage:**
```bash
# Close empty accounts, send rent to wallet
./target/release/pump-rs close-token-accounts --wallet-path <PATH>

# Burn tokens instead of closing
./target/release/pump-rs close-token-accounts --wallet-path <PATH> --burn
```

**Example:**
```bash
./target/release/pump-rs close-token-accounts \
  --wallet-path ~/.config/solana/fund-wallet.json
```

**Recovers:** ~0.002 SOL per closed account

**Use when:** Cleaning up after many trades

---

## 🪙 **Token Operations**

### **`slot-created`** - Check Token Creation Slot
**Purpose:** Find the slot number when a token was created

**Usage:**
```bash
./target/release/pump-rs slot-created --mint <MINT_ADDRESS>
```

**Example:**
```bash
./target/release/pump-rs slot-created \
  --mint FASTykZyyjVfhutuRzMMYbFbFacQpRnMzDguhWfWadbi
```

**Output:** `FASTy...adbi: created 379023813`

**Use when:** Analyzing token launch timing

---

### **`swap-mode`** - Manual Buy/Sell Mode
**Purpose:** Interactive trading interface for manual buys/sells

**Usage:**
```bash
# Buy mode (enter amount in lamports)
./target/release/pump-rs swap-mode --lamports <AMOUNT>

# Sell mode (sells entire balance)
./target/release/pump-rs swap-mode --lamports 0 --sell
```

**Example:**
```bash
# Buy mode with 0.01 SOL per trade
./target/release/pump-rs swap-mode --lamports 10000000

# Sell mode
./target/release/pump-rs swap-mode --lamports 0 --sell
```

**How it works:**
1. Enter mint address when prompted
2. Transaction is built and sent
3. Type 'q' to quit

**Use when:** Manual trading specific tokens

---

### **`sweep-pump`** - Sell All Pump.fun Tokens
**Purpose:** Automatically sell all pump.fun tokens in a wallet

**Usage:**
```bash
./target/release/pump-rs sweep-pump --wallet-path <PATH>
```

**Example:**
```bash
./target/release/pump-rs sweep-pump \
  --wallet-path ~/.config/solana/fund-wallet.json
```

**What it does:**
- Finds all pump.fun token holdings from API
- **FILTER 1**: Skips non-pump.fun tokens (USDC, etc.) via HashSet lookup
- **FILTER 2**: Skips graduated tokens (bonding curve closed)
- **Error Handler**: Catches any sell failures and continues to next token
- Waits 300ms between sells

**How it works:**
1. **FILTER 1** (fast): Checks if token is in pump.fun API response (no RPC call)
2. **FILTER 2** (optimization): Checks if bonding curve exists (1 RPC call per token)
3. **Error Handler** (safety net): Catches network errors, RPC failures, etc.

**Filters are optional** - Comment them out in `src/main.rs` (lines 343-346, 353-356) if you want error handler to catch everything instead.

**Use when:** Liquidating entire portfolio

---

### **`sweep-jup`** - Sell All Tokens via Jupiter
**Purpose:** Sell any SPL tokens (not just pump.fun) via Jupiter aggregator

**Usage:**
```bash
./target/release/pump-rs sweep-jup --wallet-path <PATH>
```

**Example:**
```bash
./target/release/pump-rs sweep-jup \
  --wallet-path ~/.config/solana/fund-wallet.json
```

**Features:**
- Works with any SPL token
- Interactive confirmation for each token
- Uses Jupiter for best prices
- 0.5% slippage tolerance

**Use when:** Selling tokens not on pump.fun

---

### **`launch`** - Create Your Own Token
**Purpose:** Launch a new token on pump.fun

**Usage:**
```bash
./target/release/pump-rs launch \
  --name "Token Name" \
  --symbol "SYMBOL" \
  --description "Token description" \
  --telegram "telegram_handle" \
  --twitter "twitter_handle" \
  --website "https://website.com" \
  --image-path ./logo.png \
  --dev-buy <LAMPORTS> \
  --snipe-buy <LAMPORTS>
```

**Example:**
```bash
./target/release/pump-rs launch \
  --name "My Token" \
  --symbol "MYTOK" \
  --description "My awesome token" \
  --telegram "" \
  --twitter "" \
  --website "" \
  --image-path ./token-logo.png \
  --dev-buy 20000000 \
  --snipe-buy 10000000
```

**Parameters:**
- `--dev-buy`: Your initial buy (0.02 SOL = 20000000 lamports)
- `--snipe-buy`: Amount for sub-wallets to buy (if using bundles)

**Requires:**
- `INFURA_PROJECT` and `INFURA_SECRET` in .env
- Image file (PNG/JPG)

**Use when:** Launching your own pump.fun token

---

## 📊 **Monitoring & Analysis**

### **`subscribe-pump`** - Monitor New Token Launches
**Purpose:** Real-time monitoring of new pump.fun tokens

**Usage:**
```bash
./target/release/pump-rs subscribe-pump
```

**What you see:**
```
[INFO] Subscribed to pump logs
[DEBUG] Updated slot: 379023813
[INFO] 379023813: 5iDQ9V7wiEQhEYj9vrxe1f141AVjDduJGDexcHtwY7fV6VAv3NzYcWPqrtPnx7fCxiEeGMEe19ihHYnUjkfAPhbu
```

**Shows:**
- Current slot number
- Transaction signatures of new token creations

**Latency:** ~5-10 slots (pump.fun frontend API)

**Use when:** Monitoring market activity, testing connectivity

---

### **`bench-pump`** - Benchmark Pump.fun Connection
**Purpose:** Measure latency to pump.fun websocket API

**Usage:**
```bash
./target/release/pump-rs bench-pump
```

**What it does:**
- Connects to pump.fun Socket.io websocket
- Listens for new token events
- Calculates time difference between token creation and notification

**Output:**
```
<mint_address>: <latency_in_ms>
```

**Expected latency:** 2000-5000ms (5-10 slots)

**Use when:** Testing connection speed to pump.fun API

---

### **`bench-portal`** - Benchmark PumpPortal Connection
**Purpose:** Measure latency to pumpportal.fun API

**Usage:**
```bash
./target/release/pump-rs bench-portal
```

**Expected latency:** 500-2000ms (1-5 slots)

**Faster than:** bench-pump (pump.fun frontend)

**Use when:** Comparing data sources

---

### **`slot-subscribe`** - Monitor Current Slot
**Purpose:** Watch slot numbers update in real-time

**Usage:**
```bash
./target/release/pump-rs slot-subscribe
```

**Output:**
```
Subscribing to slot updates
Current slot: 379023813
Current slot: 379023814
Current slot: 379023815
...
```

**Updates:** Every ~400ms (Solana block time)

**Use when:** Testing WS_URL connectivity, timing tests

---

### **`subscribe-tip`** - Monitor Jito Tips
**Purpose:** Track Jito tip amounts in real-time

**Usage:**
```bash
./target/release/pump-rs subscribe-tip
```

**What it shows:**
- Current recommended tip amounts
- Updates from Jito tip stream
- MEV tip pricing dynamics

**Use when:** Determining optimal tip amounts for bundles

---

### **`get-tx`** - Fetch Transaction Details
**Purpose:** Get detailed information about a transaction

**Usage:**
```bash
./target/release/pump-rs get-tx --sig <SIGNATURE>
```

**Example:**
```bash
./target/release/pump-rs get-tx \
  --sig 5iDQ9V7wiEQhEYj9vrxe1f141AVjDduJGDexcHtwY7fV6VAv3NzYcWPqrtPnx7fCxiEeGMEe19ihHYnUjkfAPhbu
```

**Shows:** Full transaction details (accounts, instructions, logs)

**Use when:** Debugging transactions, analyzing trades

---

### **`analyze`** - Analyze Wallet Performance
**Purpose:** Analyze trading performance and P&L

**Usage:**
```bash
./target/release/pump-rs analyze \
  --wallet-path <PATH> \
  --address <ADDRESS>
```

**Example:**
```bash
./target/release/pump-rs analyze \
  --wallet-path ~/.config/solana/fund-wallet.json \
  --address 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
```

**Analyzes:** Transaction history, wins/losses, performance

**Use when:** Reviewing trading results

---

## 🎯 **Sniping & Trading**

### **`snipe-pump`** - Auto-Snipe from Pump.fun
**Purpose:** Automatically buy new tokens from pump.fun websocket

**Usage:**
```bash
./target/release/pump-rs snipe-pump --lamports <AMOUNT>
```

**Example:**
```bash
# Snipe with 0.05 SOL per token
./target/release/pump-rs snipe-pump --lamports 50000000
```

**How it works:**
1. Connects to pump.fun websocket
2. Listens for new token creation events
3. Automatically buys with specified amount
4. Sends via Jito bundles

**Latency:** ~5-10 slots (slow, no Shredstream)

**Use when:** Auto-sniping tokens (needs supervision)

---

### **`snipe-portal`** - Auto-Snipe from PumpPortal
**Purpose:** Automatically buy new tokens from pumpportal.fun API

**Usage:**
```bash
./target/release/pump-rs snipe-portal --lamports <AMOUNT>
```

**Example:**
```bash
# Snipe with 0.05 SOL per token
./target/release/pump-rs snipe-portal --lamports 50000000
```

**Latency:** ~1-5 slots (faster than snipe-pump)

**Use when:** Auto-sniping with better latency

---

## 🚀 **Services**

### **`pump-service`** - HTTP Service for Shredstream
**Purpose:** Run HTTP server to receive Shredstream webhooks

**Usage:**
```bash
./target/release/pump-rs pump-service --lamports <AMOUNT>
```

**Example:**
```bash
# Run service, buy with 0.05 SOL per token
./target/release/pump-rs pump-service --lamports 50000000
```

**Endpoints:**
- `GET /healthz` - Health check
- `GET /blockhash` - Current blockhash
- `POST /pump-buy` - Manual buy trigger
- `POST /v2/pump-buy` - Shredstream webhook (private)

**Runs on:** Port 6969

**Latency:** ~0.5 slots (with Shredstream webhook)

**Use when:** Running production bot with Shredstream access

---

### **`seller`** - Automated Sell Service
**Purpose:** Automatically sell tokens when buy transactions confirm

**Usage:**
```bash
./target/release/pump-rs seller
```

**How it works:**
1. Subscribes to your wallet's transaction logs
2. Detects successful buy transactions
3. Automatically sells immediately after confirmation
4. Uses Jito bundles for speed

**Strategy:** Scalp profits from other snipers

**Use when:** Running automated sell-after-buy strategy

---

### **`bundle-status-listener`** - Monitor Bundle Results
**Purpose:** Watch Jito bundle acceptance/rejection in real-time

**Usage:**
```bash
./target/release/pump-rs bundle-status-listener
```

**Output:**
```
Bundle abc123 accepted
Bundle def456 rejected: simulation failed
Bundle ghi789 processed
Bundle jkl012 finalized
```

**Use when:** Monitoring bundle performance, debugging

---

## 🔬 **Advanced/Development**

### **`bundle-status`** - Check Specific Bundle
**Purpose:** Get status of a specific Jito bundle

**Usage:**
```bash
./target/release/pump-rs bundle-status --bundle-id <ID>
```

**Shows:** Bundle state (accepted/rejected/landed)

**Use when:** Checking individual bundle outcomes

---

### **`bump-pump`** - Continuous Buy (Volume Bot)
**Purpose:** Repeatedly buy a token to create volume/activity

**Usage:**
```bash
./target/release/pump-rs bump-pump --mint <MINT>
```

**Example:**
```bash
./target/release/pump-rs bump-pump \
  --mint FASTykZyyjVfhutuRzMMYbFbFacQpRnMzDguhWfWadbi
```

**Requires:** `BUMP_KEYPAIR_PATH` in .env

**Interval:** Buys every 6 seconds

**Cost Calculation:**

Each bump performs a buy and immediate sell in a single Jito bundle. The cost per bump follows this formula:

```
Cost per bump ≈ (Bump Amount × 0.025) + 0.000055 SOL
```

**Examples:**
- 2 SOL bump: (2 × 0.025) + 0.000055 = ~0.05 SOL loss (~2.5%)
- 3 SOL bump: (3 × 0.025) + 0.000055 = ~0.075 SOL loss (~2.5%)

**Note:** Larger bumps may have higher slippage, so expect 2.5-4% on bigger amounts.

**Configuration:**
- **Delay between bumps:** `src/main.rs:300` (currently 6 seconds)
- **Bump amount & Jito tip:** Edit `lamports` and `tip` in `send_pump_bump` function in `src/pump.rs:833` and `src/pump.rs:895`
  - Default bump amount: 22,800,000 lamports (0.0228 SOL)
  - Default Jito tip: 50,000 lamports (0.00005 SOL)

**Use when:** Creating artificial volume (use responsibly)

---

### **`test-slot-program`** - Test Deadline Program
**Purpose:** Test the custom slot deadline program

**Usage:**
```bash
./target/release/pump-rs test-slot-program
```

**What it does:**
- Creates transaction with deadline instruction
- Sends to network
- Tests slot-based execution control

**Use when:** Testing custom on-chain program

---

### **`look-for-geyser`** - Search for Geyser Endpoints
**Purpose:** Find available Geyser plugin endpoints

**Usage:**
```bash
./target/release/pump-rs look-for-geyser
```

**Use when:** Researching alternative data sources

---

### **`subscribe`** - Raw Jito Subscription
**Purpose:** Test raw Jito Block Engine subscription

**Usage:**
```bash
./target/release/pump-rs subscribe
```

**Status:** Unimplemented (returns error)

**Use when:** Development/testing

---

## 🐛 **Troubleshooting Commands**

### **Why `bench-pump` Not Working?**

If `bench-pump` isn't working, it's likely due to:

**1. WebSocket Connection Issues**
```bash
# Check if you can connect to pump.fun
curl -I https://frontend-api-v3.pump.fun
```

**2. Socket.io Protocol Changes**
The bench commands use Socket.io websockets which can change. Check logs:
```bash
RUST_LOG=debug ./target/release/pump-rs bench-pump
```

**3. Rate Limiting**
Pump.fun may be rate-limiting your connection. Try:
- Using a VPN
- Waiting a few minutes
- Using `bench-portal` instead

**4. Missing Dependencies**
Ensure WS_URL is set:
```bash
# Check your .env
cat .env | grep WS_URL
```

**Common Error Messages:**

| Error | Cause | Solution |
|-------|-------|----------|
| "connection refused" | Pump.fun blocking | Try VPN, use bench-portal |
| "timeout" | Network issues | Check internet connection |
| "decode error" | Protocol changed | Check for updates |
| "handshake failed" | TLS issues | Update dependencies |

---

## 📝 **Quick Reference Card**

### **Most Used Commands:**

```bash
# Setup & Testing
pump-rs sanity                                    # Verify config
pump-rs wallets                                   # Check balance

# Monitoring
pump-rs subscribe-pump                            # Watch launches
pump-rs slot-subscribe                            # Watch slots

# Trading
pump-rs swap-mode --lamports 10000000            # Manual trading
pump-rs sweep-pump --wallet-path <path>          # Sell all

# Sniping (needs monitoring!)
pump-rs snipe-portal --lamports 50000000         # Auto-snipe

# Services
pump-rs pump-service --lamports 50000000         # HTTP service
pump-rs seller                                    # Auto-seller
```

---

## 💡 **Pro Tips**

1. **Always start with `sanity`** to verify configuration
2. **Test with small amounts** using `swap-mode` first
3. **Monitor with `subscribe-pump`** before auto-sniping
4. **Use `bench-portal`** over `bench-pump` (more reliable)
5. **Run `bundle-status-listener`** in separate terminal when testing bundles
6. **Check `slot-created`** to verify token authenticity before buying

---

## ⚠️ **Safety Reminders**

- 🔴 **Sniping commands spend real money automatically**
- 🔴 **Always use small amounts for testing**
- 🔴 **Monitor services - don't leave unattended**
- 🔴 **Most pump.fun tokens fail (99%+)**
- 🔴 **Without Shredstream, you'll be 5-10 slots slow**

---

**Need help with a specific command?** Run with `--help`:
```bash
./target/release/pump-rs <command> --help
```
