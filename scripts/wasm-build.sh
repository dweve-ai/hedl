#!/usr/bin/env bash
# Dweve HEDL - WASM Build Script
# Builds hedl-wasm with various feature configurations for size comparison

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
WASM_CRATE="$PROJECT_ROOT/crates/hedl-wasm"
OUTPUT_DIR="$PROJECT_ROOT/target/wasm-builds"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

# Check if wasm-pack is installed
check_wasm_pack() {
    if ! command -v wasm-pack &> /dev/null; then
        log_error "wasm-pack is not installed. Install with: cargo install wasm-pack"
        exit 1
    fi
}

# Build a specific variant
build_variant() {
    local variant="$1"
    local features="$2"
    local output_name="hedl-wasm-$variant"

    log_info "Building $variant variant with features: $features"

    cd "$WASM_CRATE"

    if [[ -z "$features" ]]; then
        wasm-pack build --release --target web \
            --out-dir "$OUTPUT_DIR/$output_name" \
            -- --no-default-features
    else
        wasm-pack build --release --target web \
            --out-dir "$OUTPUT_DIR/$output_name" \
            -- --no-default-features --features "$features"
    fi

    log_success "Built $variant variant"
}

# Get file size in human readable format
get_size() {
    local file="$1"
    if [[ -f "$file" ]]; then
        local size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
        local kb=$((size / 1024))
        echo "${kb}KB ($size bytes)"
    else
        echo "N/A"
    fi
}

# Get gzipped size
get_gzip_size() {
    local file="$1"
    if [[ -f "$file" ]]; then
        local gzip_size=$(gzip -c "$file" | wc -c)
        local kb=$((gzip_size / 1024))
        echo "${kb}KB ($gzip_size bytes)"
    else
        echo "N/A"
    fi
}

# Print size report
print_size_report() {
    echo ""
    echo "========================================"
    echo "        WASM Build Size Report          "
    echo "========================================"
    echo ""

    for variant in minimal standard full; do
        local wasm_file="$OUTPUT_DIR/hedl-wasm-$variant/hedl_wasm_bg.wasm"
        if [[ -f "$wasm_file" ]]; then
            echo "$variant:"
            echo "  Raw size:    $(get_size "$wasm_file")"
            echo "  Gzip size:   $(get_gzip_size "$wasm_file")"
            echo ""
        fi
    done
}

# Main
main() {
    check_wasm_pack

    mkdir -p "$OUTPUT_DIR"

    log_info "Building HEDL WASM variants..."
    echo ""

    # Build minimal variant (parse + validate only)
    build_variant "minimal" "minimal"

    # Build standard variant (json + format + lint + stats)
    build_variant "standard" "standard"

    # Build full variant (all features including debug)
    build_variant "full" "full"

    print_size_report

    log_success "All variants built successfully!"
    log_info "Output directory: $OUTPUT_DIR"
}

main "$@"
