# Troubleshooting Guide - pump-rs

## ❌ `bench-pump` Not Working

### **Error:** "failed to lookup address information"

```
Error: Custom { kind: Uncategorized, error: "failed to lookup address information:
Temporary failure in name resolution" }
```

**Cause:** Cannot resolve `frontend-api-v3.pump.fun` DNS

**Solutions:**

#### 1. Check Internet Connection
```bash
# Test basic connectivity
ping -c 3 8.8.8.8

# Test DNS resolution
nslookup frontend-api-v3.pump.fun
```

#### 2. Try Alternative DNS
```bash
# Temporary DNS change (Linux)
sudo systemctl stop systemd-resolved
echo "nameserver 8.8.8.8" | sudo tee /etc/resolv.conf
echo "nameserver 1.1.1.1" | sudo tee -a /etc/resolv.conf
```

#### 3. Use `bench-portal` Instead
`bench-portal` is more reliable and faster:
```bash
./target/release/pump-rs bench-portal
```

#### 4. Check if pump.fun is Accessible
```bash
# Test HTTPS connection
curl -I https://frontend-api-v3.pump.fun

# If blocked, try with VPN or wait
```

---

## 🌐 **Network/Firewall Issues**

### Symptoms:
- DNS resolution failures
- Connection timeouts
- "Connection refused" errors

### Solutions:

**Check Proxy Settings:**
```bash
echo $HTTP_PROXY
echo $HTTPS_PROXY
```

**Disable Proxy Temporarily:**
```bash
unset HTTP_PROXY
unset HTTPS_PROXY
unset http_proxy
unset https_proxy
```

**Test Direct Connection:**
```bash
curl -v https://api.mainnet-beta.solana.com
curl -v https://frontend-api-v3.pump.fun
```

---

## 🔌 **WebSocket Connection Issues**

### Error: "WebSocket handshake failed"

**Check WS_URL:**
```bash
# Verify WS_URL is set correctly
cat .env | grep WS_URL

# Test websocket connection (if wscat is installed)
wscat -c wss://api.mainnet-beta.solana.com
```

**Common Issues:**
- Using `http://` instead of `https://`
- Using `ws://` instead of `wss://`
- Mismatched RPC and WS providers

**Fix:**
```bash
# Make sure both match in .env
RPC_URL=https://api.mainnet-beta.solana.com
WS_URL=wss://api.mainnet-beta.solana.com
```

---

## 💰 **RPC Rate Limiting**

### Error: "429 Too Many Requests"

**Solutions:**

1. **Use Paid RPC Provider:**
```bash
# Helius (recommended)
RPC_URL=https://mainnet.helius-rpc.com/?api-key=YOUR_KEY
WS_URL=wss://mainnet.helius-rpc.com/?api-key=YOUR_KEY

# QuickNode
RPC_URL=https://your-endpoint.quiknode.pro/YOUR_KEY
WS_URL=wss://your-endpoint.quiknode.pro/YOUR_KEY
```

2. **Add Delays Between Requests:**
Use slower commands or add delays in your scripts

3. **Check Rate Limits:**
```bash
# Test RPC with verbose output
curl -v -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
  $RPC_URL
```

---

## 🔑 **Keypair/Authentication Issues**

### Error: "AUTH_KEYPAIR_PATH env var not set"

**Fix:**
```bash
# Check .env file exists
ls -la .env

# Verify AUTH_KEYPAIR_PATH is set
cat .env | grep AUTH_KEYPAIR_PATH

# Create keypair if missing
solana-keygen new -o ~/.config/solana/jito-auth.json
```

### Error: "No such file or directory"

**Check paths are correct:**
```bash
# Verify wallet files exist
ls -l ~/.config/solana/fund-wallet.json
ls -l ~/.config/solana/jito-auth.json

# Check .env has correct paths
cat .env
```

---

## 💸 **Transaction Failures**

### Error: "Insufficient funds"

**Check balance:**
```bash
./target/release/pump-rs wallets
```

**Fund wallet:**
```bash
solana-keygen pubkey ~/.config/solana/fund-wallet.json
# Send SOL to this address
```

### Error: "Blockhash not found"

**Cause:** Transaction took too long, blockhash expired

**Solutions:**
- Use faster RPC endpoint
- Reduce transaction size
- Send with `skip_preflight: true`

### Error: "Transaction simulation failed"

**Debug:**
```bash
# Get detailed logs
RUST_LOG=debug ./target/release/pump-rs <command>

# Check transaction details
./target/release/pump-rs get-tx --sig <SIGNATURE>
```

---

## 📦 **Jito Bundle Issues**

### Error: "Bundle rejected"

**Common causes:**
1. **Simulation failed** - Check transaction validity
2. **Tip too low** - Increase tip amount
3. **Expired blockhash** - Bundle took too long to create
4. **Conflicting transactions** - Another tx used same account

**Debug bundle status:**
```bash
# Monitor bundles in real-time
./target/release/pump-rs bundle-status-listener

# Check specific bundle
./target/release/pump-rs bundle-status --bundle-id <ID>
```

**Increase tips:**
Edit source or use higher priority transactions

---

## 🐛 **Build/Compilation Issues**

### Error: "protoc failed"

**Solution:**
```bash
# Install protoc
cargo install protoc-bin-vendored

# Set PROTOC and rebuild
export PROTOC=$(protoc-bin-which)
cargo clean
cargo build --release
```

**Or use build script:**
```bash
./build.sh release
```

---

## 🔍 **Debugging Tips**

### Enable Debug Logging
```bash
RUST_LOG=debug ./target/release/pump-rs <command>
```

### Enable Trace Logging (Very Verbose)
```bash
RUST_LOG=trace ./target/release/pump-rs <command>
```

### Check Specific Module
```bash
# Only show pump-rs logs
RUST_LOG=pump_rs=debug ./target/release/pump-rs <command>

# Multiple modules
RUST_LOG=pump_rs=debug,solana_client=info ./target/release/pump-rs <command>
```

---

## 📞 **Getting Help**

### Collect Diagnostic Information

```bash
# System info
uname -a
rustc --version
cargo --version

# Check config
cat .env | grep -v SECRET | grep -v PROJECT

# Check connectivity
curl -I https://api.mainnet-beta.solana.com
curl -I https://frontend-api-v3.pump.fun

# Test protoc
protoc-bin-which
$(protoc-bin-which) --version

# Check wallet
./target/release/pump-rs sanity
```

---

## 🚀 **Quick Fixes**

### Most Common Issues & Solutions:

| Issue | Quick Fix |
|-------|-----------|
| DNS errors | Use `bench-portal` instead of `bench-pump` |
| Rate limiting | Use paid RPC (Helius/QuickNode) |
| Missing protoc | Run `cargo install protoc-bin-vendored` |
| Connection timeout | Check firewall, try VPN |
| Invalid keypair | Regenerate with `solana-keygen new` |
| Bundle rejected | Increase tip, check bundle size |
| WS errors | Verify `WS_URL` matches `RPC_URL` provider |

---

## 🛠️ **Advanced Troubleshooting**

### Network Packet Inspection
```bash
# Monitor network traffic (requires tcpdump)
sudo tcpdump -i any host frontend-api-v3.pump.fun

# Check TLS handshake
openssl s_client -connect frontend-api-v3.pump.fun:443
```

### Test Individual Components
```bash
# Test RPC
./target/release/pump-rs slot-subscribe

# Test WS
./target/release/pump-rs subscribe-pump

# Test Jito
./target/release/pump-rs bundle-status-listener
```

### Clean Rebuild
```bash
# Nuclear option - complete clean rebuild
rm -rf target/
cargo clean
./build.sh release
```

---

## 📝 **Reporting Issues**

When asking for help, include:

1. **Command run:** `./target/release/pump-rs bench-pump`
2. **Error message:** Full error output
3. **Environment:**
   ```bash
   uname -a
   cargo --version
   cat .env | grep -v SECRET
   ```
4. **Logs:** Run with `RUST_LOG=debug`
5. **Network:** Can you access pump.fun in browser?

---

## ✅ **Verification Checklist**

Before running commands, verify:

- [ ] `.env` file exists and is configured
- [ ] All keypair files exist at specified paths
- [ ] RPC_URL and WS_URL are set and match
- [ ] Wallet has sufficient balance (check with `sanity`)
- [ ] Internet connection is working
- [ ] Can access Solana RPC (test with `slot-subscribe`)
- [ ] `protoc` is installed (for rebuilds)

---

**Remember:** Most issues are configuration or network-related. Always start with `sanity` command!
