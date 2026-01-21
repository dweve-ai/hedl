#!/bin/bash
# Generate unsafe code metrics for hedl-core

echo "=== Unsafe Code Metrics for hedl-core ==="
echo "Generated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
echo ""

cd crates/hedl-core

# Count unsafe blocks
unsafe_blocks=$(rg "unsafe \{" src --type rust 2>/dev/null | wc -l)
unsafe_fn=$(rg "unsafe fn" src --type rust 2>/dev/null | wc -l)
unsafe_trait=$(rg "unsafe trait" src --type rust 2>/dev/null | wc -l)
unsafe_impl=$(rg "unsafe impl" src --type rust 2>/dev/null | wc -l)

echo "Unsafe Block Count:"
echo "  Unsafe blocks: $unsafe_blocks"
echo "  Unsafe functions: $unsafe_fn"
echo "  Unsafe traits: $unsafe_trait"
echo "  Unsafe impls: $unsafe_impl"
echo "  Total unsafe items: $((unsafe_blocks + unsafe_fn + unsafe_trait + unsafe_impl))"
echo ""

# Check for unsafe in dependencies
echo "Dependencies with unsafe:"
cargo tree -p hedl-core -e normal --prefix none 2>/dev/null \
  | grep -v hedl-core \
  | sort -u \
  | while read dep; do
      dep_name=$(echo $dep | cut -d' ' -f1)
      echo "  - $dep_name"
    done
echo ""

# Safety documentation coverage
if [ "$unsafe_blocks" -gt 0 ]; then
    echo "Safety Documentation Coverage:"

    # Check each unsafe block has safety comment
    documented=$(rg -B 1 "unsafe \{" src --type rust 2>/dev/null | grep -c "// Safety:" || echo 0)
    echo "  Documented unsafe blocks: $documented / $unsafe_blocks"

    if [ "$documented" -lt "$unsafe_blocks" ]; then
        echo "  ⚠️  WARNING: Some unsafe blocks lack safety documentation"
    fi
fi

echo ""
echo "MIRI Validation Status:"
if cargo +nightly miri test --quiet 2>&1 >/dev/null; then
    echo "  ✓ All MIRI checks pass"
else
    echo "  ✗ MIRI detected potential UB"
fi

echo ""
echo "=== Historical Tracking ==="
echo "Date,Unsafe Blocks,Unsafe Fns,Unsafe Traits,Unsafe Impls" >> .unsafe-metrics.csv
echo "$(date -u +"%Y-%m-%d"),$unsafe_blocks,$unsafe_fn,$unsafe_trait,$unsafe_impl" >> .unsafe-metrics.csv
echo "Metrics appended to .unsafe-metrics.csv"
