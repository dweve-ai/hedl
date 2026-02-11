#!/bin/bash
# =============================================================================
# HEDL WASM Build Script
# Open Source (Apache 2.0)
#
# Copyright (c) Dweve B.V.
# IP holder: Dweve IP B.V.
# =============================================================================
#
# This script builds the hedl-wasm crate for WebAssembly.
#
# Workaround for Rust 1.82+ / wasm-bindgen reference-types issue:
# Rust 1.82+ adds 'reference-types' to the target_features metadata section
# of WASM binaries by default. wasm-bindgen detects this and enables externref
# support, but then fails because the required runtime functions aren't present.
#
# Solution: Strip the target_features section before running wasm-bindgen.
#
# See: https://github.com/rustwasm/wasm-bindgen/issues/4211
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CRATE_DIR="$SCRIPT_DIR"
TARGET_DIR="$PROJECT_ROOT/target"
WASM_TARGET="wasm32-unknown-unknown"
OUT_DIR="$CRATE_DIR/pkg"

# Default target for wasm-bindgen
WB_TARGET="${1:-web}"

echo "Building hedl-wasm for target: $WB_TARGET"

# Step 1: Build the WASM binary with all features
echo "Step 1: Building WASM binary with full features..."
cd "$PROJECT_ROOT"
cargo build --package hedl-wasm --target "$WASM_TARGET" --release --features "full,full-validation,statistics,token-tools,query-api"

WASM_FILE="$TARGET_DIR/$WASM_TARGET/release/hedl_wasm.wasm"

# Step 2: Strip target_features section (workaround for Rust 1.82+ reference-types)
# We strip in-place by writing to a temp file then replacing the original
echo "Step 2: Stripping target_features section..."
if command -v wasm-tools &> /dev/null; then
    TEMP_WASM="$TARGET_DIR/$WASM_TARGET/release/hedl_wasm_temp.wasm"
    wasm-tools strip "$WASM_FILE" --delete "target_features" -o "$TEMP_WASM"
    mv "$TEMP_WASM" "$WASM_FILE"
    echo "  Stripped target_features from $WASM_FILE"
else
    echo "ERROR: wasm-tools not found. Install with: cargo install wasm-tools"
    echo "This is required to work around the Rust 1.82+ reference-types issue."
    exit 1
fi

# Step 3: Run wasm-bindgen on the stripped wasm (using original filename)
echo "Step 3: Running wasm-bindgen..."
mkdir -p "$OUT_DIR"
wasm-bindgen "$WASM_FILE" \
    --out-dir "$OUT_DIR" \
    --typescript \
    --target "$WB_TARGET"

echo ""
echo "Build complete! Output in: $OUT_DIR"
ls -la "$OUT_DIR"
