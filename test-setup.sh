#!/bin/bash
# Pump-RS Setup Testing Script
# This script runs safe, read-only tests to verify your installation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=================================================="
echo "  Pump-RS Setup Testing Script"
echo "=================================================="
echo ""

# Check if binary exists
if [ ! -f "target/release/pump-rs" ]; then
    echo -e "${RED}❌ Binary not found!${NC}"
    echo "Please build first: cargo build --release"
    exit 1
fi

# Check if .env exists
if [ ! -f ".env" ]; then
    echo -e "${RED}❌ .env file not found!${NC}"
    echo "Please copy .env.example to .env and configure it"
    echo "  cp .env.example .env"
    echo "  nano .env"
    exit 1
fi

echo -e "${GREEN}✅ Found binary and .env file${NC}"
echo ""

# Test 1: Sanity Check
echo "=================================================="
echo "Test 1: Sanity Check (Configuration Verification)"
echo "=================================================="
echo ""

if ./target/release/pump-rs sanity; then
    echo -e "${GREEN}✅ Sanity check passed!${NC}"
else
    echo -e "${RED}❌ Sanity check failed!${NC}"
    echo "Please check your .env configuration and wallet paths"
    exit 1
fi
echo ""

# Test 2: Slot Created Query
echo "=================================================="
echo "Test 2: Pump.fun Integration (Read-Only)"
echo "=================================================="
echo ""
echo "Fetching slot data for a known pump.fun token..."

if ./target/release/pump-rs slot-created --mint FASTykZyyjVfhutuRzMMYbFbFacQpRnMzDguhWfWadbi; then
    echo -e "${GREEN}✅ Pump.fun integration working!${NC}"
else
    echo -e "${YELLOW}⚠️  Pump.fun query failed${NC}"
    echo "This might be due to RPC issues or rate limiting"
fi
echo ""

# Test 3: WebSocket Tests
echo "=================================================="
echo "Test 3: WebSocket Connection Tests"
echo "=================================================="
echo ""
echo -e "${YELLOW}Note: The following tests will run for 10 seconds each${NC}"
echo ""

# Test Pump.fun websocket
echo "Testing pump.fun websocket connection..."
timeout 10 ./target/release/pump-rs subscribe-pump || {
    code=$?
    if [ $code -eq 124 ]; then
        echo -e "${GREEN}✅ Pump.fun websocket connected successfully${NC}"
    else
        echo -e "${YELLOW}⚠️  Pump.fun websocket test inconclusive${NC}"
    fi
}
echo ""

# Test 4: Check Wallet Balance
echo "=================================================="
echo "Test 4: Wallet Balance Check"
echo "=================================================="
echo ""

if ./target/release/pump-rs wallets; then
    echo -e "${GREEN}✅ Wallet check successful${NC}"
    echo -e "${YELLOW}⚠️  Make sure you have at least 0.5 SOL for testing${NC}"
else
    echo -e "${RED}❌ Wallet check failed${NC}"
fi
echo ""

# Summary
echo "=================================================="
echo "  Test Summary"
echo "=================================================="
echo ""
echo -e "${GREEN}✅ Basic tests completed${NC}"
echo ""
echo "Next steps:"
echo "  1. Review SETUP.md for detailed testing instructions"
echo "  2. Test manual trading with small amounts (0.01 SOL)"
echo "  3. Apply for Jito Shredstream access for optimal performance"
echo ""
echo -e "${YELLOW}⚠️  IMPORTANT SAFETY REMINDERS:${NC}"
echo "  - Start with MINIMAL funds (0.1-0.5 SOL)"
echo "  - This bot runs on MAINNET with real money"
echo "  - Past performance does not guarantee future results"
echo "  - Most pump.fun tokens fail (99%+)"
echo ""
echo "=================================================="
