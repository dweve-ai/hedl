# hedl-wasm optimization Makefile

.PHONY: all clean test benchmark validate help install-deps
.DEFAULT_GOAL := help

# Build targets
PROFILE := release
TARGET := wasm32-unknown-unknown
PACKAGE := hedl-wasm
SCRIPT_DIR := ../../scripts

# Optimization variants
VARIANTS := minimal standard full

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
BLUE := \033[0;34m
NC := \033[0m # No Color

## help: Show this help message
help:
	@echo "$(BLUE)hedl-wasm Optimization Build System$(NC)"
	@echo ""
	@echo "$(GREEN)Available targets:$(NC)"
	@grep -E '^## ' $(MAKEFILE_LIST) | sed 's/^## /  /'
	@echo ""
	@echo "$(GREEN)Variants:$(NC)"
	@echo "  minimal  - Core parsing only (smallest)"
	@echo "  standard - JSON + common features"
	@echo "  full     - All features enabled"

## all: Build all optimization variants
all: $(VARIANTS)
	@echo "$(GREEN)✓ All variants built successfully$(NC)"

## minimal: Build minimal variant (smallest binary)
minimal:
	@echo "$(BLUE)Building minimal variant...$(NC)"
	@FEATURES=minimal $(SCRIPT_DIR)/build-optimized-wasm.sh
	@echo ""

## standard: Build standard variant (JSON + common features)
standard:
	@echo "$(BLUE)Building standard variant...$(NC)"
	@FEATURES=standard $(SCRIPT_DIR)/build-optimized-wasm.sh
	@echo ""

## full: Build full variant (all features)
full:
	@echo "$(BLUE)Building full variant...$(NC)"
	@FEATURES=full,query-api,statistics,token-tools,full-validation $(SCRIPT_DIR)/build-optimized-wasm.sh
	@echo ""

## test: Run optimization correctness tests
test:
	@echo "$(BLUE)Running optimization tests...$(NC)"
	@cargo test --package $(PACKAGE) --test optimization_tests
	@echo "$(GREEN)✓ Tests passed$(NC)"

## test-all-features: Test all feature combinations
test-all-features:
	@echo "$(BLUE)Testing all feature combinations...$(NC)"
	@for features in "minimal" "standard" "json" "query-api,statistics,token-tools" "full,query-api,statistics,token-tools,full-validation"; do \
		echo "$(YELLOW)Testing with features: $$features$(NC)"; \
		cargo test --package $(PACKAGE) --no-default-features --features "$$features" || exit 1; \
	done
	@echo "$(GREEN)✓ All feature tests passed$(NC)"

## benchmark: Run performance benchmarks (requires Node.js)
benchmark:
	@echo "$(BLUE)Running benchmarks...$(NC)"
	@if command -v node > /dev/null; then \
		echo "$(YELLOW)Benchmark implementation TODO$(NC)"; \
	else \
		echo "$(RED)Error: Node.js not found$(NC)"; \
		exit 1; \
	fi

## validate: Validate binary sizes meet targets
validate:
	@echo "$(BLUE)Validating binary sizes...$(NC)"
	@if [ -f target/$(TARGET)/$(PROFILE)/hedl_wasm.optimized.wasm ]; then \
		SIZE=$$(stat -c%s target/$(TARGET)/$(PROFILE)/hedl_wasm.optimized.wasm); \
		SIZE_KB=$$((SIZE / 1024)); \
		echo "  Binary size: $$SIZE_KB KB"; \
		if [ $$SIZE_KB -lt 500 ]; then \
			echo "$(GREEN)✓ Size validation passed$(NC)"; \
		else \
			echo "$(YELLOW)⚠ Warning: Binary larger than target (500 KB)$(NC)"; \
		fi; \
	else \
		echo "$(RED)Error: Optimized binary not found$(NC)"; \
		echo "Run 'make standard' first"; \
		exit 1; \
	fi

## analyze: Analyze WASM binary for dead code
analyze:
	@echo "$(BLUE)Analyzing WASM binary...$(NC)"
	@if [ -f target/$(TARGET)/$(PROFILE)/hedl_wasm.wasm ]; then \
		$(SCRIPT_DIR)/wasm-analyze.sh target/$(TARGET)/$(PROFILE)/hedl_wasm.wasm target/wasm-analysis; \
	else \
		echo "$(RED)Error: WASM binary not found$(NC)"; \
		echo "Run 'cargo build --target $(TARGET) --package $(PACKAGE)' first"; \
		exit 1; \
	fi

## check: Run clippy and format checks
check:
	@echo "$(BLUE)Running clippy...$(NC)"
	@cargo clippy --package $(PACKAGE) --target $(TARGET) -- -D warnings
	@echo "$(BLUE)Checking formatting...$(NC)"
	@cargo fmt --package $(PACKAGE) -- --check
	@echo "$(GREEN)✓ Check passed$(NC)"

## fmt: Format code
fmt:
	@echo "$(BLUE)Formatting code...$(NC)"
	@cargo fmt --package $(PACKAGE)
	@echo "$(GREEN)✓ Code formatted$(NC)"

## clean: Remove build artifacts
clean:
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	@rm -rf target/$(TARGET)/$(PROFILE)
	@rm -rf target/wasm-optimization
	@rm -rf target/wasm-analysis
	@echo "$(GREEN)✓ Clean complete$(NC)"

## install-deps: Install required dependencies
install-deps:
	@echo "$(BLUE)Installing dependencies...$(NC)"
	@echo ""
	@echo "$(YELLOW)Checking for required tools...$(NC)"
	@MISSING=0; \
	for tool in cargo wasm-opt python3 wasm-validate wasm-objdump; do \
		if command -v $$tool > /dev/null 2>&1; then \
			echo "$(GREEN)✓$(NC) $$tool"; \
		else \
			echo "$(RED)✗$(NC) $$tool (missing)"; \
			MISSING=1; \
		fi; \
	done; \
	if [ $$MISSING -eq 1 ]; then \
		echo ""; \
		echo "$(YELLOW)Installation instructions:$(NC)"; \
		echo "  Rust/Cargo:   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"; \
		echo "  wasm-opt:     Download binaryen from https://github.com/WebAssembly/binaryen"; \
		echo "  python3:      apt-get install python3 (Ubuntu) or brew install python3 (macOS)"; \
		echo "  WABT tools:   Download from https://github.com/WebAssembly/wabt"; \
		echo ""; \
		echo "Or use your system package manager:"; \
		echo "  Ubuntu/Debian: apt-get install python3 wabt"; \
		echo "  macOS:         brew install python3 wabt binaryen"; \
		exit 1; \
	fi
	@echo ""
	@echo "$(YELLOW)Checking for optional tools...$(NC)"
	@for tool in wasm-snip node; do \
		if command -v $$tool > /dev/null 2>&1; then \
			echo "$(GREEN)✓$(NC) $$tool"; \
		else \
			echo "$(YELLOW)○$(NC) $$tool (optional, recommended)"; \
		fi; \
	done
	@echo ""
	@echo "$(GREEN)✓ All required dependencies installed$(NC)"
	@echo ""
	@echo "Optional installations:"
	@echo "  wasm-snip: cargo install wasm-snip"
	@echo "  Node.js:   https://nodejs.org"

## ci: Run all CI checks
ci: clean install-deps all test validate
	@echo ""
	@echo "$(GREEN)✓✓✓ All CI checks passed ✓✓✓$(NC)"

## size-report: Generate detailed size report
size-report:
	@echo "$(BLUE)Generating size report...$(NC)"
	@echo ""
	@echo "$(YELLOW)Binary Sizes:$(NC)"
	@for variant in $(VARIANTS); do \
		echo ""; \
		echo "$(BLUE)$$variant variant:$(NC)"; \
		WASM_FILE="target/$(TARGET)/$(PROFILE)/hedl_wasm.optimized.wasm"; \
		if [ -f "$$WASM_FILE" ]; then \
			SIZE=$$(stat -c%s "$$WASM_FILE" 2>/dev/null || stat -f%z "$$WASM_FILE" 2>/dev/null); \
			SIZE_KB=$$((SIZE / 1024)); \
			GZIP_SIZE=$$(gzip -c "$$WASM_FILE" | wc -c); \
			GZIP_KB=$$((GZIP_SIZE / 1024)); \
			echo "  Raw:     $$SIZE_KB KB"; \
			echo "  Gzipped: $$GZIP_KB KB"; \
		else \
			echo "  $(YELLOW)Not built yet$(NC)"; \
		fi; \
	done
	@echo ""
