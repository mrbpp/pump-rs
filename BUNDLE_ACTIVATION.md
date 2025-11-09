# How to Activate Bundle Mode - Step by Step

This guide shows **exactly** what code to change in `src/launcher.rs` to enable bundle launches.

---

## ⚠️ Before You Start

1. **Backup your code**: `git stash` or `git commit`
2. **Set up wallets**: Run `./setup-wallets.sh` first
3. **Understand the risk**: Bundle mode is all-or-nothing (one failed tx = entire bundle fails)

---

## 📝 Code Changes Required

### Change 1: Uncomment Bundle Variables

**Location**: `src/launcher.rs`, lines 241-242

**BEFORE:**
```rust
) -> Result<(), Box<dyn Error>> {
    // let mut first_buy_bundle = vec![];
    // let mut second_buy_bundle = vec![];
    for wallet in wallet_manager.wallets.iter() {
```

**AFTER:**
```rust
) -> Result<(), Box<dyn Error>> {
    let mut first_buy_bundle = vec![];
    let mut second_buy_bundle = vec![];
    for wallet in wallet_manager.wallets.iter() {
```

---

### Change 2: Comment Out Individual Send & Uncomment Bundle Assembly

**Location**: `src/launcher.rs`, lines 272-283

**BEFORE:**
```rust
        );
        pool_state.virtual_sol_reserves += lamports_amount;
        pool_state.virtual_token_reserves -= token_amount;

        send_jito_tx(buy_tx).await?;

        // if i < 5 {
        //     first_buy_bundle.push(buy_tx.clone());
        // } else if i < 10 {
        //     second_buy_bundle.push(buy_tx.clone());
        // } else {
        //     break;
        // }
    }
```

**AFTER:**
```rust
        );
        pool_state.virtual_sol_reserves += lamports_amount;
        pool_state.virtual_token_reserves -= token_amount;

        // send_jito_tx(buy_tx).await?;

        if i < 5 {
            first_buy_bundle.push(VersionedTransaction::from(buy_tx.clone()));
        } else if i < 10 {
            second_buy_bundle.push(VersionedTransaction::from(buy_tx.clone()));
        } else {
            break;
        }
    }
```

**⚠️ Important**: Add the loop index variable! See "Change 2b" below.

---

### Change 2b: Add Loop Index (CRITICAL)

**Location**: `src/launcher.rs`, line 243

**BEFORE:**
```rust
    for wallet in wallet_manager.wallets.iter() {
        let lamports_amount = jittered_lamports_amount(snipe_buy);
```

**AFTER:**
```rust
    for (i, wallet) in wallet_manager.wallets.iter().enumerate() {
        let lamports_amount = jittered_lamports_amount(snipe_buy);
```

This adds the `i` variable needed for bundle splitting logic.

---

### Change 3: Uncomment Bundle Submission

**Location**: `src/launcher.rs`, lines 293-306

**BEFORE:**
```rust
    }

    #[cfg(feature = "dry-run")]
    {
        info!("first_buy_bundle: {:#?}", first_buy_bundle);
        info!("second_buy_bundle: {:#?}", second_buy_bundle);
        return Ok(());
    }

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

    Ok(())
```

**AFTER:**
```rust
    }

    #[cfg(feature = "dry-run")]
    {
        info!("first_buy_bundle: {:#?}", first_buy_bundle);
        info!("second_buy_bundle: {:#?}", second_buy_bundle);
        return Ok(());
    }

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

    Ok(())
```

---

## 🔧 Complete Modified Function

Here's the complete `ladder_buys` function after all changes:

```rust
async fn ladder_buys(
    mint: Pubkey,
    pool_state: &mut PoolState,
    wallet_manager: &WalletManager,
    snipe_buy: u64,
    latest_blockhash: Hash,
    searcher_client: &mut SearcherClient,
) -> Result<(), Box<dyn Error>> {
    let mut first_buy_bundle = vec![];
    let mut second_buy_bundle = vec![];

    for (i, wallet) in wallet_manager.wallets.iter().enumerate() {
        let lamports_amount = jittered_lamports_amount(snipe_buy);
        let token_amount = get_token_amount(
            pool_state.virtual_sol_reserves,
            pool_state.virtual_token_reserves,
            None,
            lamports_amount,
        )?;
        let ixs = _make_buy_ixs(
            wallet.pubkey(),
            mint,
            pool_state.bonding_curve,
            pool_state.associated_bonding_curve,
            token_amount,
            apply_fee(lamports_amount),
        )?;

        let buy_tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&wallet.pubkey()),
            &[wallet],
            latest_blockhash,
        );

        pool_state.virtual_sol_reserves += lamports_amount;
        pool_state.virtual_token_reserves -= token_amount;

        // send_jito_tx(buy_tx).await?;

        if i < 5 {
            first_buy_bundle.push(VersionedTransaction::from(buy_tx.clone()));
        } else if i < 10 {
            second_buy_bundle.push(VersionedTransaction::from(buy_tx.clone()));
        } else {
            break;
        }
    }

    #[cfg(feature = "dry-run")]
    {
        info!("first_buy_bundle: {:#?}", first_buy_bundle);
        info!("second_buy_bundle: {:#?}", second_buy_bundle);
        return Ok(());
    }

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

    Ok(())
}
```

---

## 🏗️ Build After Changes

```bash
# Regular build
cargo build --release

# Or build with dry-run to test without sending
cargo build --release --features dry-run
```

---

## 🧪 Testing Your Changes

### Test 1: Dry-Run (Safest)

```bash
# Build with dry-run
cargo build --release --features dry-run

# Launch (won't actually send bundles)
./target/release/pump-rs launch \
  --name "Bundle Test" \
  --symbol "BTEST" \
  --description "Testing bundle mode" \
  --dev-buy 10000000 \
  --snipe-buy 5000000
```

**Expected output:**
```
first_buy_bundle: [Transaction { ... }, Transaction { ... }, ...]
second_buy_bundle: [Transaction { ... }, Transaction { ... }, ...]
```

### Test 2: Live Test (Small Amount)

```bash
# Build normal (without dry-run)
cargo build --release

# Launch with minimal amounts (0.005 SOL per wallet)
./target/release/pump-rs launch \
  --name "Real Bundle Test" \
  --symbol "RBT" \
  --description "Real test" \
  --dev-buy 5000000 \
  --snipe-buy 5000000
```

Monitor in another terminal:
```bash
./target/release/pump-rs bundle-status-listener
```

---

## 🎛️ Optional: Customize Bundle Behavior

### Change Bundle Sizes

**Location**: Lines 277-283 in `ladder_buys()`

```rust
// Original: 5 wallets per bundle
if i < 5 {
    first_buy_bundle.push(...);
} else if i < 10 {
    second_buy_bundle.push(...);
}

// Custom: 3 wallets per bundle
if i < 3 {
    first_buy_bundle.push(...);
} else if i < 6 {
    second_buy_bundle.push(...);
}

// Or: All wallets in one bundle
first_buy_bundle.push(VersionedTransaction::from(buy_tx.clone()));
// (remove the if/else logic)
```

### Change Delay Between Bundles

**Location**: Line 295

```rust
// Original: 2 seconds
tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

// Faster: 1 second
tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

// Slower: 5 seconds
tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
```

### Add Jito Tips to Bundles

**Location**: After line 258 in `ladder_buys()`

```rust
let mut ixs = _make_buy_ixs(...)?;

// Add tip for last wallet in each bundle
if i == 4 || i == 9 {  // Wallet 5 and 10
    ixs.push(transfer(
        &wallet.pubkey(),
        &get_jito_tip_pubkey(),
        25000,  // 0.000025 SOL tip
    ));
}

let buy_tx = Transaction::new_signed_with_payer(&ixs, ...);
```

---

## ✅ Verification Checklist

After making changes, verify:

- [ ] All 3 changes applied correctly
- [ ] Loop has `.enumerate()` added
- [ ] `VersionedTransaction::from()` wrapper added
- [ ] Code compiles: `cargo build --release`
- [ ] Dry-run test passes
- [ ] At least 10 wallets created and funded
- [ ] `WALLET_DIRECTORY=./wallets` in `.env`

---

## 🔄 Reverting Changes

To go back to individual transaction mode:

```bash
# Restore original code
git checkout src/launcher.rs

# Or re-comment the bundle code
# (reverse all the changes above)

# Rebuild
cargo build --release
```

---

## 📊 Expected Behavior Comparison

| Aspect | Before (Individual) | After (Bundle) |
|--------|-------------------|----------------|
| **Execution** | Sequential, one by one | Atomic, grouped |
| **Speed** | ~500ms between txs | Simultaneous per bundle |
| **Failures** | Some txs can succeed | All or nothing |
| **Tips** | One per tx | One per bundle |
| **Logs** | One signature per tx | One bundle ID per group |

---

## 🆘 Troubleshooting

**Compile error: "cannot find value `i`"**
→ Add `.enumerate()` to the for loop (Change 2b)

**Error: "expected VersionedTransaction"**
→ Wrap in `VersionedTransaction::from(buy_tx.clone())`

**Bundles get rejected**
→ Try smaller bundle sizes (3 wallets instead of 5)
→ Increase tip amount
→ Check compute budget isn't exceeded

**Only first bundle executes**
→ Check second bundle isn't empty
→ Make sure you have at least 6 wallets

---

## 💡 Pro Tips

1. **Start with dry-run** to verify bundle structure
2. **Test with 2-3 wallets** before using 10
3. **Monitor bundle listener** to see acceptance/rejection
4. **Keep bundle sizes small** (3-5 txs) for better success rate
5. **Add tips** to improve bundle priority

---

## Summary

You've now activated bundle mode! Your token launches will send coordinated buy transactions in atomic bundles, providing better MEV protection and execution guarantees.

**Total changes**: 3 sections, ~10 lines of code
**Time to implement**: 5 minutes
**Time to test**: 10 minutes

Ready to launch with bundles! 🚀
