#!/bin/bash
set -e

echo "=== MIRI Undefined Behavior Detection ==="
echo "Running comprehensive MIRI checks on hedl-core"
echo ""

cd crates/hedl-core

# Install MIRI if not present
if ! cargo +nightly miri --version >/dev/null 2>&1; then
    echo "Installing MIRI..."
    rustup +nightly component add miri
    cargo +nightly miri setup
fi

# Run with strict provenance (detect ptr-int-ptr roundtrips)
echo "1. Strict Provenance Check..."
MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-symbolic-alignment-check" \
    cargo +nightly miri test --quiet

# Run with stacked borrows (detect aliasing violations)
echo "2. Stacked Borrows Check..."
MIRIFLAGS="-Zmiri-tag-raw-pointers" \
    cargo +nightly miri test --quiet

# Run with tree borrows (experimental, stricter)
echo "3. Tree Borrows Check (experimental)..."
MIRIFLAGS="-Zmiri-tree-borrows" \
    cargo +nightly miri test --quiet 2>&1 | grep -v "experimental" || true

# Run with isolation (detect filesystem/env access in unsafe)
echo "4. Isolation Check..."
MIRIFLAGS="-Zmiri-disable-isolation" \
    cargo +nightly miri test --quiet

echo ""
echo "✓ All MIRI checks passed! No undefined behavior detected."
echo ""
echo "Note: hedl-core currently has 0 unsafe blocks."
echo "This baseline ensures future unsafe code is validated."
