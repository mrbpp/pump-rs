# 🚀 Quick Start Guide

## Step-by-Step Setup (5 minutes)

### 1. Build the Project
```bash
cargo build --release
```

### 2. Create Wallets
```bash
# Create main trading wallet
solana-keygen new -o ~/.config/solana/fund-wallet.json

# Create Jito auth wallet (doesn't need SOL)
solana-keygen new -o ~/.config/solana/jito-auth.json

# Get your wallet address
solana-keygen pubkey ~/.config/solana/fund-wallet.json
```

### 3. Configure Environment
```bash
cp .env.example .env
nano .env  # Edit these lines:
```

**Required settings in .env:**
```bash
FUND_KEYPAIR_PATH=/home/user/.config/solana/fund-wallet.json
AUTH_KEYPAIR_PATH=/home/user/.config/solana/jito-auth.json
RPC_URL=https://api.mainnet-beta.solana.com
WS_URL=wss://api.mainnet-beta.solana.com
BLOCK_ENGINE_URL=https://mainnet.block-engine.jito.wtf
RUST_LOG=info
```

### 4. Fund Your Wallet
Send **0.5 SOL** to your wallet address (from step 2)

### 5. Run Tests
```bash
# Automated testing
./test-setup.sh

# OR manual testing
./target/release/pump-rs sanity
```

---

## 🧪 Safe Testing Commands (No Money Spent)

```bash
# Check configuration
./target/release/pump-rs sanity

# Check your balance
./target/release/pump-rs wallets

# Test pump.fun API (read-only)
./target/release/pump-rs slot-created --mint FASTykZyyjVfhutuRzMMYbFbFacQpRnMzDguhWfWadbi

# Monitor new token launches (Ctrl+C to exit)
./target/release/pump-rs subscribe-pump

# Test connection speeds
./target/release/pump-rs bench-pump
./target/release/pump-rs bench-portal
```

---

## 💰 Trading Commands (⚠️ Spends Real Money)

```bash
# Manual buy/sell mode (0.01 SOL per trade)
./target/release/pump-rs swap-mode --lamports 10000000

# Sell mode (sells entire balance)
./target/release/pump-rs swap-mode --lamports 0 --sell

# Auto-snipe from pump.fun websocket (50 SOL per token)
./target/release/pump-rs snipe-pump --lamports 50000000

# Auto-snipe from pumpportal (50 SOL per token)
./target/release/pump-rs snipe-portal --lamports 50000000

# Run HTTP service for Shredstream webhook (50 SOL per token)
./target/release/pump-rs pump-service --lamports 50000000
```

---

## 🔧 Utility Commands

```bash
# Close all token accounts (cleanup)
./target/release/pump-rs close-token-accounts --wallet-path ~/.config/solana/fund-wallet.json

# Sell all pump.fun tokens via pump.fun
./target/release/pump-rs sweep-pump --wallet-path ~/.config/solana/fund-wallet.json

# Sell all tokens via Jupiter (interactive)
./target/release/pump-rs sweep-jup --wallet-path ~/.config/solana/fund-wallet.json

# Launch your own token (0.02 SOL dev buy, 0.01 SOL snipe)
./target/release/pump-rs launch \
  --name "My Token" \
  --symbol "MYTOK" \
  --description "My cool token" \
  --image-path ./image.png \
  --dev-buy 20000000 \
  --snipe-buy 10000000
```

---

## ⚠️ Important Notes

### What Works Without Shredstream:
- ✅ Manual trading (`swap-mode`)
- ✅ Monitoring websockets (`subscribe-pump`)
- ✅ Token launching (`launch`)
- ✅ Portfolio management (`sweep-pump`, `sweep-jup`)

### What Needs Shredstream:
- ⚠️ `pump-service` - requires webhook (0.5 slot latency)
- ⚠️ `snipe-pump` - works but slow (5-10 slot latency)
- ⚠️ `snipe-portal` - works but slow (1-5 slot latency)

### Performance Expectations:
- **With Shredstream webhook**: 0.5-1 slot latency ⚡
- **Without Shredstream**: 3-10 slot latency 🐌
- **Other bots with Shredstream will beat you to trades**

---

## 🎯 Recommended Testing Path

1. **Day 1: Setup & Verification**
   ```bash
   cargo build --release
   ./test-setup.sh
   ```

2. **Day 2: Manual Trading Test**
   ```bash
   # Trade 0.01 SOL on a known token
   ./target/release/pump-rs swap-mode --lamports 10000000
   ```

3. **Day 3: Monitoring**
   ```bash
   # Watch the market without trading
   ./target/release/pump-rs subscribe-pump
   ```

4. **Day 4+: Auto-Sniping** (if you want)
   ```bash
   # Apply for Shredstream first!
   # Then build webhook deserializer
   # Then run pump-service
   ```

---

## 📚 Full Documentation

- **SETUP.md** - Detailed installation guide
- **README.md** - Project overview
- **.env.example** - Configuration template
- **roadmap.md** - Development history

---

## 🆘 Common Issues

**"RPC rate limit exceeded"**
→ Use paid RPC: Helius, QuickNode, or Triton

**"env var not set"**
→ Check `.env` file exists and paths are correct

**"Bundle rejected/dropped"**
→ Normal! Jito bundles compete in MEV auction

**"Connection refused"**
→ Check firewall, network, and RPC endpoint

---

## 💡 Pro Tips

1. **Always use paid RPC** - Free RPCs will rate-limit you
2. **Start small** - Test with 0.01-0.05 SOL per trade
3. **Monitor first** - Watch the market before auto-trading
4. **Shredstream is critical** - Without it, you're at a major disadvantage
5. **Most tokens fail** - 99%+ don't reach bonding curve
6. **Have an exit strategy** - Set stop-losses mentally

---

**Need help?** Read SETUP.md for troubleshooting 📖
