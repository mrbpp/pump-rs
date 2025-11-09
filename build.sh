#!/bin/bash
# Build script for pump-rs with protoc environment variable set

set -e

echo "====================================="
echo "  Pump-RS Build Script"
echo "====================================="
echo ""

# Set PROTOC environment variable
export PROTOC=/root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/protoc-bin-vendored-linux-x86_64-3.2.0/bin/protoc

if [ ! -f "$PROTOC" ]; then
    echo "Error: protoc binary not found at $PROTOC"
    echo "Run: cargo install protoc-bin-vendored"
    exit 1
fi

echo "Using protoc: $PROTOC"
$PROTOC --version
echo ""

# Check if we should build in release or debug mode
BUILD_MODE="${1:-release}"

if [ "$BUILD_MODE" = "release" ]; then
    echo "Building in RELEASE mode (optimized)..."
    cargo build --release
    echo ""
    echo "====================================="
    echo "  Build Complete!"
    echo "====================================="
    echo ""
    echo "Binary location: ./target/release/pump-rs"
    echo "Run with: ./target/release/pump-rs --help"
elif [ "$BUILD_MODE" = "debug" ]; then
    echo "Building in DEBUG mode (faster compilation)..."
    cargo build
    echo ""
    echo "====================================="
    echo "  Build Complete!"
    echo "====================================="
    echo ""
    echo "Binary location: ./target/debug/pump-rs"
    echo "Run with: ./target/debug/pump-rs --help"
elif [ "$BUILD_MODE" = "dry-run" ]; then
    echo "Building in RELEASE mode with dry-run feature..."
    cargo build --release --features dry-run
    echo ""
    echo "====================================="
    echo "  Build Complete! (Dry-run mode)"
    echo "====================================="
    echo ""
    echo "Binary location: ./target/release/pump-rs"
    echo "This build won't send real transactions."
else
    echo "Usage: ./build.sh [release|debug|dry-run]"
    echo "  release  - Optimized build (default, slower compile)"
    echo "  debug    - Fast build for development"
    echo "  dry-run  - Release build that won't send transactions"
    exit 1
fi
