#!/bin/bash
# Run all HEDL core fuzz targets
# Usage: ./run_all_fuzzers.sh [duration_seconds]

set -e

DURATION=${1:-60}  # Default: 60 seconds per target
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "============================================"
echo "HEDL Core Fuzzing Suite"
echo "============================================"
echo "Duration per target: ${DURATION} seconds"
echo "Total targets: 4"
echo "Estimated total time: $((DURATION * 4)) seconds"
echo "============================================"
echo ""

TARGETS=(
    "fuzz_parse"
    "fuzz_limits"
    "fuzz_references"
    "fuzz_nest_depth"
)

SUCCESS=0
FAILED=0

for target in "${TARGETS[@]}"; do
    echo ">>> Running $target for ${DURATION} seconds..."
    echo ""

    if cargo fuzz run "$target" -- -max_total_time="$DURATION" -print_final_stats=1; then
        echo ""
        echo "✅ $target completed successfully"
        ((SUCCESS++))
    else
        echo ""
        echo "❌ $target failed or found crashes"
        ((FAILED++))
    fi

    echo ""
    echo "============================================"
    echo ""
done

echo ""
echo "============================================"
echo "SUMMARY"
echo "============================================"
echo "Successful: $SUCCESS/4"
echo "Failed:     $FAILED/4"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "✅ All fuzzers ran successfully!"
    echo ""
    echo "Note: Check fuzz/artifacts/ for any crashes found"
    exit 0
else
    echo "❌ Some fuzzers failed or found crashes"
    echo ""
    echo "Check fuzz/artifacts/ for crash inputs"
    exit 1
fi
