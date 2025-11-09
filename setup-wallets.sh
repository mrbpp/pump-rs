#!/bin/bash
# Multi-Wallet Setup Script for Bundle Launches

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=================================================="
echo "  Pump-RS Multi-Wallet Setup Script"
echo "=================================================="
echo -e "${NC}"

# Get number of wallets from user
read -p "How many wallets do you want to create? (recommended: 10): " WALLET_COUNT
WALLET_COUNT=${WALLET_COUNT:-10}

# Get funding amount
read -p "How much SOL to fund each wallet? (in SOL, e.g., 0.02): " SOL_AMOUNT
SOL_AMOUNT=${SOL_AMOUNT:-0.02}

# Convert SOL to lamports (1 SOL = 1,000,000,000 lamports)
LAMPORTS=$(echo "$SOL_AMOUNT * 1000000000" | bc | cut -d. -f1)

echo ""
echo -e "${YELLOW}Configuration:${NC}"
echo "  - Number of wallets: $WALLET_COUNT"
echo "  - Funding amount: $SOL_AMOUNT SOL ($LAMPORTS lamports) per wallet"
echo "  - Total needed: $(echo "$SOL_AMOUNT * $WALLET_COUNT + 0.01" | bc) SOL (including fees)"
echo ""

read -p "Continue? (y/n): " CONFIRM
if [ "$CONFIRM" != "y" ]; then
    echo "Cancelled."
    exit 0
fi

# Create wallets directory
echo ""
echo -e "${BLUE}Step 1: Creating wallets directory...${NC}"
mkdir -p wallets
echo -e "${GREEN}✅ Created ./wallets/${NC}"

# Generate wallets
echo ""
echo -e "${BLUE}Step 2: Generating $WALLET_COUNT wallet keypairs...${NC}"
for i in $(seq 1 $WALLET_COUNT); do
    if [ -f "wallets/wallet-$i.json" ]; then
        echo -e "${YELLOW}  ⚠️  Wallet $i already exists, skipping...${NC}"
    else
        solana-keygen new -o wallets/wallet-$i.json --no-bip39-passphrase --silent
        PUBKEY=$(solana-keygen pubkey wallets/wallet-$i.json)
        echo -e "${GREEN}  ✅ Created wallet $i: $PUBKEY${NC}"
    fi
done

# Update .env file
echo ""
echo -e "${BLUE}Step 3: Updating .env file...${NC}"
if grep -q "WALLET_DIRECTORY" .env 2>/dev/null; then
    echo -e "${YELLOW}  ⚠️  WALLET_DIRECTORY already in .env${NC}"
else
    echo "WALLET_DIRECTORY=./wallets" >> .env
    echo -e "${GREEN}  ✅ Added WALLET_DIRECTORY to .env${NC}"
fi

# Build the project if needed
echo ""
echo -e "${BLUE}Step 4: Checking if binary exists...${NC}"
if [ ! -f "target/release/pump-rs" ]; then
    echo -e "${YELLOW}  Building project (this may take 5-10 minutes)...${NC}"
    cargo build --release
    echo -e "${GREEN}  ✅ Build complete${NC}"
else
    echo -e "${GREEN}  ✅ Binary already exists${NC}"
fi

# Fund wallets
echo ""
echo -e "${BLUE}Step 5: Funding wallets...${NC}"
echo -e "${YELLOW}This will fund $WALLET_COUNT wallets with $LAMPORTS lamports each${NC}"
echo ""

read -p "Ready to fund? Make sure your FUND_KEYPAIR has enough SOL! (y/n): " FUND_CONFIRM
if [ "$FUND_CONFIRM" = "y" ]; then
    if ./target/release/pump-rs wallets-fund --lamports $LAMPORTS; then
        echo -e "${GREEN}✅ Wallets funded successfully!${NC}"
    else
        echo -e "${RED}❌ Funding failed! Check your main wallet balance.${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️  Skipped funding. You can fund later with:${NC}"
    echo "  ./target/release/pump-rs wallets-fund --lamports $LAMPORTS"
fi

# Check balances
echo ""
echo -e "${BLUE}Step 6: Checking wallet balances...${NC}"
./target/release/pump-rs wallets

# Summary
echo ""
echo -e "${GREEN}=================================================="
echo "  ✅ Setup Complete!"
echo "=================================================="
echo -e "${NC}"
echo "You now have $WALLET_COUNT wallets ready for bundle launches!"
echo ""
echo "Next steps:"
echo "  1. Read BUNDLE_GUIDE.md to understand bundle functionality"
echo "  2. Uncomment bundle code in src/launcher.rs (see guide)"
echo "  3. Rebuild: cargo build --release"
echo "  4. Test launch with small amounts"
echo ""
echo "Wallet management commands:"
echo "  - Check balances: ./target/release/pump-rs wallets"
echo "  - Check tokens: ./target/release/pump-rs wallets --token-balances"
echo "  - Drain wallets: ./target/release/pump-rs wallets-drain"
echo ""
echo -e "${YELLOW}⚠️  Keep your wallets/ directory secure!${NC}"
echo -e "${YELLOW}⚠️  Add wallets/ to .gitignore to prevent committing keys${NC}"
echo ""
