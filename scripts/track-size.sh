#!/usr/bin/env bash
# Dweve HEDL - WASM Size Tracking Script
# Tracks WASM binary sizes and reports metrics without git dependencies

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HISTORY_FILE="$PROJECT_ROOT/.wasm-size-history.csv"

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

# Get gzipped size
get_gzip_size() {
    local file="$1"
    if [[ -f "$file" ]]; then
        gzip -c "$file" | wc -c | tr -d ' '
    else
        echo "0"
    fi
}

# Get brotli size (if available)
get_brotli_size() {
    local file="$1"
    if [[ -f "$file" ]] && command -v brotli &> /dev/null; then
        brotli -c "$file" | wc -c | tr -d ' '
    else
        echo "N/A"
    fi
}

# Check if size is within target
check_target() {
    local actual_kb="$1"
    local target_kb="$2"
    local name="$3"

    if [[ "$actual_kb" -le "$target_kb" ]]; then
        echo -e "${GREEN}PASS${NC} $name: ${actual_kb}KB <= ${target_kb}KB target"
        return 0
    else
        local over=$((actual_kb - target_kb))
        echo -e "${RED}FAIL${NC} $name: ${actual_kb}KB > ${target_kb}KB target (${over}KB over)"
        return 1
    fi
}

# Track a single WASM file
track_file() {
    local wasm_file="$1"
    local variant="${2:-unknown}"

    if [[ ! -f "$wasm_file" ]]; then
        log_warn "File not found: $wasm_file"
        return 1
    fi

    local raw_size
    raw_size=$(get_file_size "$wasm_file")
    local gzip_size
    gzip_size=$(get_gzip_size "$wasm_file")
    local brotli_size
    brotli_size=$(get_brotli_size "$wasm_file")
    local timestamp
    timestamp=$(date -u +"%Y-%m-%d %H:%M:%S UTC")

    # Create header if file doesn't exist
    if [[ ! -f "$HISTORY_FILE" ]]; then
        echo "timestamp,variant,raw_bytes,gzip_bytes,brotli_bytes" > "$HISTORY_FILE"
    fi

    # Append measurement
    echo "$timestamp,$variant,$raw_size,$gzip_size,$brotli_size" >> "$HISTORY_FILE"

    # Display current size
    echo ""
    echo "Variant: $variant"
    echo "  Raw:     $(format_size "$raw_size")"
    echo "  Gzipped: $(format_size "$gzip_size")"
    if [[ "$brotli_size" != "N/A" ]]; then
        echo "  Brotli:  $(format_size "$brotli_size")"
    fi

    return 0
}

# Check size regression against previous measurement
check_regression() {
    local current_gzip="$1"
    local variant="$2"

    if [[ ! -f "$HISTORY_FILE" ]]; then
        return 0
    fi

    # Get previous measurement for this variant
    local prev_size
    prev_size=$(grep ",$variant," "$HISTORY_FILE" | tail -2 | head -1 | cut -d',' -f4)

    if [[ -z "$prev_size" || "$prev_size" == "gzip_bytes" ]]; then
        return 0
    fi

    local diff=$((current_gzip - prev_size))
    local percent=0
    if [[ "$prev_size" -gt 0 ]]; then
        percent=$((diff * 100 / prev_size))
    fi

    if [[ $percent -gt 5 ]]; then
        log_warn "Size increased by $percent% (+$(format_size "$diff")) for $variant"
        return 1
    elif [[ $percent -lt -5 ]]; then
        log_success "Size decreased by ${percent#-}% (-$(format_size "${diff#-}")) for $variant"
    else
        log_info "Size stable (${percent}%) for $variant"
    fi

    return 0
}

# Validate all build variants against targets
validate_targets() {
    local output_dir="$PROJECT_ROOT/target/wasm-builds"
    local all_pass=true

    echo ""
    echo "========================================"
    echo "      WASM Size Target Validation       "
    echo "========================================"
    echo ""

    # Check minimal variant
    local minimal_wasm="$output_dir/hedl-wasm-minimal/hedl_wasm_bg.wasm"
    if [[ -f "$minimal_wasm" ]]; then
        local minimal_gzip
        minimal_gzip=$(get_gzip_size "$minimal_wasm")
        local minimal_kb=$((minimal_gzip / 1024))
        check_target "$minimal_kb" "$TARGET_MINIMAL_KB" "minimal" || all_pass=false
    else
        echo -e "${YELLOW}SKIP${NC} minimal: not built yet"
    fi

    # Check standard variant
    local standard_wasm="$output_dir/hedl-wasm-standard/hedl_wasm_bg.wasm"
    if [[ -f "$standard_wasm" ]]; then
        local standard_gzip
        standard_gzip=$(get_gzip_size "$standard_wasm")
        local standard_kb=$((standard_gzip / 1024))
        check_target "$standard_kb" "$TARGET_STANDARD_KB" "standard" || all_pass=false
    else
        echo -e "${YELLOW}SKIP${NC} standard: not built yet"
    fi

    # Check full variant
    local full_wasm="$output_dir/hedl-wasm-full/hedl_wasm_bg.wasm"
    if [[ -f "$full_wasm" ]]; then
        local full_gzip
        full_gzip=$(get_gzip_size "$full_wasm")
        local full_kb=$((full_gzip / 1024))
        check_target "$full_kb" "$TARGET_FULL_KB" "full" || all_pass=false
    else
        echo -e "${YELLOW}SKIP${NC} full: not built yet"
    fi

    echo ""

    if $all_pass; then
        log_success "All size targets met!"
        return 0
    else
        log_error "Some size targets exceeded."
        log_info "Run './scripts/wasm-build.sh' to rebuild variants."
        return 1
    fi
}

# Show usage
usage() {
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  track <file> [variant]  - Track size of a WASM file"
    echo "  validate                - Validate all variants against targets"
    echo "  history                 - Show size history"
    echo ""
    echo "Examples:"
    echo "  $0 track target/wasm-builds/hedl-wasm-minimal/hedl_wasm_bg.wasm minimal"
    echo "  $0 validate"
    echo "  $0 history"
}

# Show history
show_history() {
    if [[ ! -f "$HISTORY_FILE" ]]; then
        log_info "No size history found."
        return 0
    fi

    echo ""
    echo "========================================"
    echo "         WASM Size History              "
    echo "========================================"
    echo ""

    # Show last 10 entries
    echo "Recent measurements (last 10):"
    echo ""
    head -1 "$HISTORY_FILE"
    tail -10 "$HISTORY_FILE"
}

# Main
main() {
    local command="${1:-validate}"

    case "$command" in
        track)
            if [[ $# -lt 2 ]]; then
                log_error "Missing file argument"
                usage
                exit 1
            fi
            track_file "$2" "${3:-unknown}"
            ;;
        validate)
            validate_targets
            ;;
        history)
            show_history
            ;;
        -h|--help|help)
            usage
            ;;
        *)
            log_error "Unknown command: $command"
            usage
            exit 1
            ;;
    esac
}

main "$@"
