#!/usr/bin/env bash
# Dweve HEDL - WASM Optimization Script
# Single-pass wasm-opt optimization for WASM binaries

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

# Get file size (works on both macOS and Linux)
get_file_size() {
    local file="$1"
    if [[ -f "$file" ]]; then
        stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null
    else
        echo "0"
    fi
}

# Format size in human readable format
format_size() {
    local bytes="$1"
    if [[ "$bytes" -ge 1048576 ]]; then
        echo "$((bytes / 1048576))MB"
    elif [[ "$bytes" -ge 1024 ]]; then
        echo "$((bytes / 1024))KB"
    else
        echo "${bytes}B"
    fi
}

# Check if wasm-opt is installed
check_wasm_opt() {
    if ! command -v wasm-opt &> /dev/null; then
        log_error "wasm-opt is not installed."
        log_info "Install options:"
        log_info "  - apt-get install binaryen     (Debian/Ubuntu)"
        log_info "  - brew install binaryen        (macOS)"
        log_info "  - cargo install wasm-opt       (via Cargo)"
        exit 1
    fi
    log_info "wasm-opt version: $(wasm-opt --version 2>&1 | head -1)"
}

# Check if wasm-validate is installed (optional but recommended)
check_wasm_validate() {
    if command -v wasm-validate &> /dev/null; then
        return 0
    else
        log_warn "wasm-validate not found. Skipping binary validation."
        return 1
    fi
}

# Optimize a single WASM file
optimize_wasm() {
    local input="$1"
    local output="$2"

    log_info "Optimizing WASM binary: $input -> $output"

    # Validate input exists
    if [[ ! -f "$input" ]]; then
        log_error "Input file not found: $input"
        exit 1
    fi

    local original_size
    original_size=$(get_file_size "$input")
    log_info "Original size: $(format_size "$original_size")"

    # Run wasm-opt with size optimization flags
    wasm-opt "$input" -Oz \
        --enable-mutable-globals \
        --enable-sign-ext \
        --enable-bulk-memory \
        --converge \
        --strip-debug \
        --strip-producers \
        --dce \
        --remove-unused-names \
        --remove-unused-module-elements \
        --vacuum \
        -o "$output"

    local optimized_size
    optimized_size=$(get_file_size "$output")
    local reduction=$((original_size - optimized_size))
    local percent=0
    if [[ "$original_size" -gt 0 ]]; then
        percent=$((reduction * 100 / original_size))
    fi

    log_success "Optimized size: $(format_size "$optimized_size")"
    log_success "Reduction: $(format_size "$reduction") ($percent%)"

    # Validate output if wasm-validate is available
    if check_wasm_validate; then
        if wasm-validate "$output" 2>/dev/null; then
            log_success "Valid WASM binary"
        else
            log_error "Invalid WASM binary produced!"
            exit 1
        fi
    fi
}

# Show usage
usage() {
    echo "Usage: $0 <input.wasm> [output.wasm]"
    echo ""
    echo "Optimizes a WASM binary using wasm-opt for size reduction."
    echo ""
    echo "Arguments:"
    echo "  input.wasm   - Input WASM file to optimize"
    echo "  output.wasm  - Output file (default: input.optimized.wasm)"
    echo ""
    echo "Examples:"
    echo "  $0 target/wasm32-unknown-unknown/release-wasm/hedl_wasm.wasm"
    echo "  $0 input.wasm optimized.wasm"
}

# Main
main() {
    if [[ $# -lt 1 ]]; then
        usage
        exit 1
    fi

    local input="$1"
    local output="${2:-${input%.wasm}.optimized.wasm}"

    check_wasm_opt
    optimize_wasm "$input" "$output"

    log_success "Optimization complete!"
    log_info "Output: $output"
}

main "$@"
