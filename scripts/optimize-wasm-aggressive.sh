#!/usr/bin/env bash
# Dweve HEDL - Aggressive WASM Optimization Script
# Multi-pass wasm-opt optimization for maximum size reduction

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }
log_pass() { echo -e "${CYAN}[PASS]${NC} $*"; }

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

# Check if wasm-validate is installed
check_wasm_validate() {
    if command -v wasm-validate &> /dev/null; then
        return 0
    else
        return 1
    fi
}

# Run a single optimization pass
run_pass() {
    local pass_name="$1"
    local input="$2"
    local output="$3"
    shift 3
    local flags=("$@")

    local input_size
    input_size=$(get_file_size "$input")

    log_pass "$pass_name"
    wasm-opt "$input" -Oz "${flags[@]}" -o "$output"

    local output_size
    output_size=$(get_file_size "$output")
    local reduction=$((input_size - output_size))

    echo "    $(format_size "$input_size") -> $(format_size "$output_size") (-$(format_size "$reduction"))"
}

# Multi-pass optimization
optimize_aggressive() {
    local input="$1"
    local output="$2"
    local temp_dir
    temp_dir=$(mktemp -d)

    log_info "Starting multi-pass optimization: $input -> $output"

    # Validate input exists
    if [[ ! -f "$input" ]]; then
        log_error "Input file not found: $input"
        exit 1
    fi

    local original_size
    original_size=$(get_file_size "$input")
    log_info "Original size: $(format_size "$original_size")"
    echo ""

    # Pass 1: Initial Size Optimization
    run_pass "Pass 1: Initial Size Optimization" \
        "$input" "${temp_dir}/pass1.wasm" \
        --enable-mutable-globals \
        --enable-sign-ext \
        --enable-bulk-memory \
        --strip-debug \
        --strip-producers

    # Pass 2: Dead Code Elimination
    run_pass "Pass 2: Dead Code Elimination" \
        "${temp_dir}/pass1.wasm" "${temp_dir}/pass2.wasm" \
        --dce \
        --remove-unused-brs \
        --remove-unused-module-elements \
        --remove-unused-nonfunction-module-elements

    # Pass 3: Code Simplification
    run_pass "Pass 3: Code Simplification" \
        "${temp_dir}/pass2.wasm" "${temp_dir}/pass3.wasm" \
        --simplify-globals \
        --simplify-locals \
        --flatten \
        --rereloop \
        --merge-blocks

    # Pass 4: Instruction Optimization
    run_pass "Pass 4: Instruction Optimization" \
        "${temp_dir}/pass3.wasm" "${temp_dir}/pass4.wasm" \
        --optimize-instructions \
        --optimize-added-constants \
        --precompute \
        --precompute-propagate

    # Pass 5: Final Cleanup with Convergence
    run_pass "Pass 5: Final Cleanup (converge)" \
        "${temp_dir}/pass4.wasm" "$output" \
        --vacuum \
        --remove-unused-names \
        --duplicate-function-elimination \
        --duplicate-import-elimination \
        --directize \
        --coalesce-locals \
        --reorder-functions \
        --reorder-locals \
        --converge

    # Cleanup intermediate files
    rm -rf "$temp_dir"

    echo ""
    local final_size
    final_size=$(get_file_size "$output")
    local total_reduction=$((original_size - final_size))
    local percent=0
    if [[ "$original_size" -gt 0 ]]; then
        percent=$((total_reduction * 100 / original_size))
    fi

    log_success "Final size: $(format_size "$final_size")"
    log_success "Total reduction: $(format_size "$total_reduction") ($percent%)"

    # Validate output if wasm-validate is available
    if check_wasm_validate; then
        if wasm-validate "$output" 2>/dev/null; then
            log_success "Valid WASM binary"
        else
            log_error "Invalid WASM binary produced!"
            exit 1
        fi
    fi

    # Show compressed sizes
    echo ""
    log_info "Compressed sizes:"
    local gzip_size
    gzip_size=$(gzip -c "$output" | wc -c)
    echo "    Gzip:   $(format_size "$gzip_size")"

    if command -v brotli &> /dev/null; then
        local brotli_size
        brotli_size=$(brotli -c "$output" | wc -c)
        echo "    Brotli: $(format_size "$brotli_size")"
    fi
}

# Show usage
usage() {
    echo "Usage: $0 <input.wasm> <output.wasm>"
    echo ""
    echo "Aggressively optimizes a WASM binary using multi-pass wasm-opt."
    echo "This script runs 5 optimization passes for maximum size reduction."
    echo ""
    echo "Arguments:"
    echo "  input.wasm   - Input WASM file to optimize"
    echo "  output.wasm  - Output file for optimized binary"
    echo ""
    echo "Optimization Passes:"
    echo "  1. Initial size optimization (strip debug, producers)"
    echo "  2. Dead code elimination"
    echo "  3. Code simplification (flatten, rereloop, merge)"
    echo "  4. Instruction optimization (precompute, constants)"
    echo "  5. Final cleanup with convergence"
    echo ""
    echo "Example:"
    echo "  $0 target/wasm32-unknown-unknown/release-wasm/hedl_wasm.wasm hedl_wasm.optimized.wasm"
}

# Main
main() {
    if [[ $# -lt 2 ]]; then
        usage
        exit 1
    fi

    local input="$1"
    local output="$2"

    echo ""
    echo "========================================"
    echo "   WASM Aggressive Optimization         "
    echo "========================================"
    echo ""

    check_wasm_opt
    optimize_aggressive "$input" "$output"

    echo ""
    log_success "Multi-pass optimization complete!"
    log_info "Output: $output"
}

main "$@"
