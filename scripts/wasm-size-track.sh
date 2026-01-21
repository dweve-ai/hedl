#!/usr/bin/env bash
# Dweve HEDL - WASM Size Tracking Script
# Tracks WASM binary sizes and reports against targets

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/target/wasm-builds"

# Target sizes (in KB, gzipped)
TARGET_MINIMAL_KB=100
TARGET_STANDARD_KB=200
TARGET_FULL_KB=250

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Get gzipped size in KB
get_gzip_kb() {
    local file="$1"
    if [[ -f "$file" ]]; then
        local gzip_size=$(gzip -c "$file" | wc -c)
        echo $((gzip_size / 1024))
    else
        echo "0"
    fi
}

# Check if size is within target
check_target() {
    local actual="$1"
    local target="$2"
    local name="$3"

    if [[ "$actual" -le "$target" ]]; then
        echo -e "${GREEN}PASS${NC} $name: ${actual}KB <= ${target}KB target"
        return 0
    else
        local over=$((actual - target))
        echo -e "${RED}FAIL${NC} $name: ${actual}KB > ${target}KB target (${over}KB over)"
        return 1
    fi
}

# Main
main() {
    local all_pass=true

    echo ""
    echo "========================================"
    echo "      WASM Size Target Validation       "
    echo "========================================"
    echo ""

    # Check minimal
    local minimal_wasm="$OUTPUT_DIR/hedl-wasm-minimal/hedl_wasm_bg.wasm"
    if [[ -f "$minimal_wasm" ]]; then
        local minimal_kb=$(get_gzip_kb "$minimal_wasm")
        check_target "$minimal_kb" "$TARGET_MINIMAL_KB" "minimal" || all_pass=false
    else
        echo -e "${YELLOW}SKIP${NC} minimal: not built yet"
    fi

    # Check standard
    local standard_wasm="$OUTPUT_DIR/hedl-wasm-standard/hedl_wasm_bg.wasm"
    if [[ -f "$standard_wasm" ]]; then
        local standard_kb=$(get_gzip_kb "$standard_wasm")
        check_target "$standard_kb" "$TARGET_STANDARD_KB" "standard" || all_pass=false
    else
        echo -e "${YELLOW}SKIP${NC} standard: not built yet"
    fi

    # Check full
    local full_wasm="$OUTPUT_DIR/hedl-wasm-full/hedl_wasm_bg.wasm"
    if [[ -f "$full_wasm" ]]; then
        local full_kb=$(get_gzip_kb "$full_wasm")
        check_target "$full_kb" "$TARGET_FULL_KB" "full" || all_pass=false
    else
        echo -e "${YELLOW}SKIP${NC} full: not built yet"
    fi

    echo ""

    if $all_pass; then
        echo -e "${GREEN}All size targets met!${NC}"
        exit 0
    else
        echo -e "${RED}Some size targets exceeded.${NC}"
        echo "Run './scripts/wasm-build.sh' to rebuild variants."
        exit 1
    fi
}

main "$@"
