# Bundle Launch Strategy - Advanced Guide

## 🎯 What is the Bundle Functionality?

The bundle functionality in `launcher.rs` is a **commented-out feature** for launching tokens with coordinated multi-wallet buys sent as Jito MEV bundles.

### Current Behavior (Active Code)
When you launch a token with multiple wallets, each wallet sends its buy transaction **individually**:
- Wallet 1 → Individual Jito transaction
- Wallet 2 → Individual Jito transaction
- Wallet 3 → Individual Jito transaction
- ...and so on

### Bundle Behavior (Commented Out)
The original design was to **group transactions into two bundles**:
- **Bundle 1**: First 5 wallets buy simultaneously
- **Bundle 2**: Next 5 wallets buy simultaneously (2 seconds later)

---

## 📍 Where is the Bundle Code?

Location: `src/launcher.rs`, function `ladder_buys()` (lines 233-309)

### Commented-Out Sections:

**1. Bundle Variables (lines 241-242)**
```rust
// let mut first_buy_bundle = vec![];
// let mut second_buy_bundle = vec![];
```

**2. Bundle Assembly Logic (lines 277-283)**
```rust
// if i < 5 {
//     first_buy_bundle.push(buy_tx.clone());
// } else if i < 10 {
//     second_buy_bundle.push(buy_tx.clone());
// } else {
//     break;
// }
```

**3. Bundle Submission (lines 293-306)**
```rust
// #[cfg(not(feature = "dry-run"))]
// {
//     tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
//
//     info!(
//         "{:?}",
//         send_bundle_no_wait(&first_buy_bundle, searcher_client).await?
//     );
//
//     info!(
//         "{:?}",
//         send_bundle_no_wait(&second_buy_bundle, searcher_client).await?
//     );
// }
```

---

## 🔧 How the Bundle System Works

### Scenario: Launching with 10 Wallets

```
TOKEN CREATION (your main wallet)
         ↓
[Wait for new blockhash]
         ↓
┌─────────────────────────────┐
│  BUNDLE 1 (sent together)   │
│  - Wallet 1 buys 0.0095 SOL │
│  - Wallet 2 buys 0.0087 SOL │
│  - Wallet 3 buys 0.0092 SOL │
│  - Wallet 4 buys 0.0088 SOL │
│  - Wallet 5 buys 0.0091 SOL │
└─────────────────────────────┘
         ↓
    [Wait 2 seconds]
         ↓
┌─────────────────────────────┐
│  BUNDLE 2 (sent together)   │
│  - Wallet 6 buys 0.0089 SOL │
│  - Wallet 7 buys 0.0093 SOL │
│  - Wallet 8 buys 0.0090 SOL │
│  - Wallet 9 buys 0.0086 SOL │
│  - Wallet 10 buys 0.0094 SOL│
└─────────────────────────────┘
```

**Key Features:**
1. **Jitter**: Each wallet buys a slightly different amount (80-95% of specified amount)
2. **Atomic**: All transactions in a bundle execute together or fail together
3. **MEV Protection**: Jito ensures your bundles land without frontrunning
4. **Delay**: 2-second gap between bundles to simulate organic activity

---

## 🚀 Setting Up Multiple Wallets

### Step 1: Create Wallet Directory

```bash
mkdir -p wallets
```

### Step 2: Add Environment Variable

Edit your `.env`:
```bash
WALLET_DIRECTORY=./wallets
```

### Step 3: Generate Wallets

You have two options:

#### Option A: Manual Creation
```bash
# Create 10 wallets manually
for i in {1..10}; do
    solana-keygen new -o wallets/wallet-$i.json --no-bip39-passphrase
done
```

#### Option B: Use WalletManager (Programmatically)

Create a helper script `create-wallets.rs` or use the existing test:
```rust
// This functionality is in wallet.rs
let mut wallet_manager = make_manager().await?;
wallet_manager.create_wallets(10)?;  // Creates 10 wallets
```

### Step 4: Fund Your Wallets

```bash
# Fund all wallets with 0.02 SOL each (0.2 SOL total + tip)
./target/release/pump-rs wallets-fund --lamports 20000000
```

This sends a Jito bundle to fund all wallets at once.

### Step 5: Check Wallet Balances

```bash
# View all wallet balances
./target/release/pump-rs wallets
```

Expected output:
```
Read in 10 wallets
Wallets directory: ./wallets
Wallet balances: [
    (Pubkey1, 20000000),
    (Pubkey2, 20000000),
    ...
]
```

---

## ⚙️ How to Activate Bundle Functionality

### Method 1: Simple Uncommenting (Quick but Limited)

Edit `src/launcher.rs`:

**1. Uncomment bundle variables (line 241-242):**
```rust
let mut first_buy_bundle = vec![];
let mut second_buy_bundle = vec![];
```

**2. Comment out individual send (line 275):**
```rust
// send_jito_tx(buy_tx).await?;
```

**3. Uncomment bundle assembly (lines 277-283):**
```rust
if i < 5 {
    first_buy_bundle.push(buy_tx.clone());
} else if i < 10 {
    second_buy_bundle.push(buy_tx.clone());
} else {
    break;
}
```

**4. Uncomment bundle submission (lines 293-306):**
```rust
#[cfg(not(feature = "dry-run"))]
{
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    info!(
        "{:?}",
        send_bundle_no_wait(&first_buy_bundle, searcher_client).await?
    );

    info!(
        "{:?}",
        send_bundle_no_wait(&second_buy_bundle, searcher_client).await?
    );
}
```

**5. Rebuild:**
```bash
cargo build --release
```

---

### Method 2: Improved Implementation (Recommended)

I can create an enhanced version that:
- Makes bundle size configurable (not hardcoded to 5)
- Adds better error handling
- Makes delay configurable
- Adds a feature flag to toggle bundle mode

Would you like me to implement this improved version?

---

## 📊 Example Launch with Bundles

```bash
# Launch token with dev buy + 10 wallet snipe buys
./target/release/pump-rs launch \
  --name "Bundle Test Token" \
  --symbol "BTT" \
  --description "Testing bundle launches" \
  --image-path ./logo.png \
  --dev-buy 20000000 \
  --snipe-buy 10000000
```

**What happens:**
1. Main wallet creates token + dev buy (0.02 SOL)
2. Waits for new blockhash
3. **Bundle 1**: Wallets 1-5 each buy ~0.01 SOL (total ~0.05 SOL)
4. Wait 2 seconds
5. **Bundle 2**: Wallets 6-10 each buy ~0.01 SOL (total ~0.05 SOL)

**Total cost:**
- Token creation: ~0.002 SOL
- Dev buy: 0.02 SOL
- 10 snipe buys: 0.1 SOL
- Jito tips: ~0.0001 SOL
- **Grand total: ~0.122 SOL**

---

## ⚠️ Important Considerations

### Why Was It Commented Out?

Possible reasons:
1. **Complexity**: Individual transactions are simpler to debug
2. **Flexibility**: Individual sends allow partial success
3. **Bundle Rejection**: If one tx in bundle fails, entire bundle fails
4. **Testing**: Easier to test individual transactions
5. **Market Dead**: Author stopped development (September 2024)

### Pros of Bundle Mode

✅ **Atomic execution** - All or nothing
✅ **MEV protection** - Harder to sandwich
✅ **Coordination** - Precise timing control
✅ **Gas efficiency** - Single tip for multiple txs
✅ **Appears organic** - Multiple wallets buying "together"

### Cons of Bundle Mode

❌ **All-or-nothing** - One bad tx kills entire bundle
❌ **Bundle rejection** - MEV auction can reject bundles
❌ **Complexity** - Harder to debug failures
❌ **Blockhash timing** - All txs must use same blockhash
❌ **Size limits** - Jito bundles have size/compute limits

---

## 🧪 Testing Bundle Mode Safely

### Step 1: Test with Dry-Run Feature

```bash
# Build with dry-run to see bundle structure without sending
cargo build --release --features dry-run

# Launch (won't actually send bundles)
./target/release/pump-rs launch \
  --name "Test" \
  --symbol "TST" \
  --description "Test" \
  --image-path ./test.png \
  --dev-buy 10000000 \
  --snipe-buy 5000000
```

You'll see debug output showing bundle contents without actually sending.

### Step 2: Test with Small Amounts

```bash
# Test with minimal amounts first (0.005 SOL per wallet)
./target/release/pump-rs launch \
  --name "Small Test" \
  --symbol "SMOL" \
  --description "Small test launch" \
  --dev-buy 5000000 \
  --snipe-buy 5000000
```

### Step 3: Monitor Bundle Results

Check Jito bundle status:
```bash
# Listen to bundle results in separate terminal
./target/release/pump-rs bundle-status-listener
```

---

## 🎛️ Configuration Options

You can modify these in `launcher.rs`:

**Jitter Range (line 435):**
```rust
let jitter = rng.gen_range(0.8..0.95);  // Buy 80-95% of target amount
```

**Bundle Split (lines 277-283):**
```rust
if i < 5 {              // First 5 wallets → Bundle 1
if i < 10 {             // Next 5 wallets → Bundle 2
```

**Delay Between Bundles (line 295):**
```rust
tokio::time::sleep(Duration::from_millis(2000)).await;  // 2 seconds
```

**Tips (lines 262-264):**
```rust
// Uncomment to add tips to specific bundles
if i == 4 || i == 9 {  // Add tip to wallet 5 and 10
    ixs.push(transfer(&wallet.pubkey(), &get_jito_tip_pubkey(), 25000));
}
```

---

## 💡 Pro Tips

1. **Start with 2-3 wallets** for testing
2. **Use dry-run first** to verify bundle structure
3. **Monitor Jito bundles** to understand rejection reasons
4. **Keep amounts small** during testing
5. **Check blockhash timing** - bundles must use valid blockhash
6. **Have backup strategy** - Individual txs as fallback

---

## 🆘 Troubleshooting

**"No wallets available"**
→ Create wallets in `./wallets/` directory

**"Insufficient funds"**
→ Fund wallets: `./target/release/pump-rs wallets-fund --lamports 20000000`

**"Bundle rejected"**
→ Check bundle size, compute limits, and tip amount

**"Transaction too large"**
→ Reduce number of wallets per bundle (split into 3+ bundles)

**"Invalid blockhash"**
→ Bundles taking too long to assemble, blockhash expired

---

## 📝 Summary

| Aspect | Individual Txs (Current) | Bundles (Commented) |
|--------|--------------------------|---------------------|
| **Setup** | Simple | Needs multiple wallets |
| **Reliability** | High | Medium (all-or-nothing) |
| **Cost** | Higher (tip per tx) | Lower (tip per bundle) |
| **Coordination** | Sequential | Atomic/simultaneous |
| **Debugging** | Easy | Harder |
| **MEV Protection** | Standard | Better |

---

## 🚀 Quick Start Checklist

- [ ] Create `wallets/` directory
- [ ] Generate 10 wallet keypairs
- [ ] Add `WALLET_DIRECTORY=./wallets` to `.env`
- [ ] Fund wallets with 0.02 SOL each
- [ ] Uncomment bundle code in `launcher.rs`
- [ ] Rebuild with `cargo build --release`
- [ ] Test with dry-run first
- [ ] Test with small amounts
- [ ] Monitor bundle results

---

**Do you want me to create an improved, configurable version of the bundle system?**
