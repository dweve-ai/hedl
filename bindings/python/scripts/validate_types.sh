#!/usr/bin/env bash
# Production-grade type validation script for HEDL Python bindings

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

echo "==================================="
echo "HEDL Python Type Validation"
echo "==================================="
echo

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track overall success
OVERALL_SUCCESS=true

# Function to print status
print_status() {
    if [ "$1" -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
        OVERALL_SUCCESS=false
    fi
}

# Function to run a check
run_check() {
    local name="$1"
    shift
    echo -e "${YELLOW}Running: $name${NC}"
    if "$@"; then
        print_status 0 "$name passed"
        return 0
    else
        print_status 1 "$name failed"
        return 1
    fi
    echo
}

# Check if mypy is installed
if ! command -v mypy &> /dev/null; then
    echo -e "${RED}Error: mypy is not installed${NC}"
    echo "Install with: pip install mypy"
    exit 1
fi

echo "1. Checking stub file syntax..."
echo "--------------------------------"
for stub in hedl/*.pyi; do
    if [ -f "$stub" ]; then
        echo "  Checking $stub..."
        if python -m py_compile "$stub"; then
            print_status 0 "$(basename "$stub") syntax valid"
        else
            print_status 1 "$(basename "$stub") has syntax errors"
        fi
    fi
done
echo

echo "2. Validating py.typed marker..."
echo "--------------------------------"
if [ -f "hedl/py.typed" ]; then
    print_status 0 "py.typed marker exists"
else
    print_status 1 "py.typed marker missing"
fi
echo

echo "3. Running mypy on hedl package..."
echo "--------------------------------"
if mypy hedl --config-file mypy.ini 2>&1 | head -20; then
    print_status 0 "Package type checking passed"
else
    print_status 1 "Package type checking failed"
fi
echo

echo "4. Running mypy on typed examples..."
echo "--------------------------------"
if [ -f "examples/typed_usage.py" ]; then
    if mypy examples/typed_usage.py --config-file mypy.ini; then
        print_status 0 "Example type checking passed"
    else
        print_status 1 "Example type checking failed"
    fi
else
    echo -e "${YELLOW}Warning: examples/typed_usage.py not found${NC}"
fi
echo

echo "5. Running mypy on tests..."
echo "--------------------------------"
# Less strict for tests
if mypy tests --config-file mypy.ini --allow-untyped-defs --allow-untyped-calls 2>&1 | head -20; then
    print_status 0 "Test type checking passed"
else
    # Tests may have some type issues, warn but don't fail
    echo -e "${YELLOW}Warning: Test type checking had issues (non-fatal)${NC}"
fi
echo

echo "6. Checking type completeness..."
echo "--------------------------------"
# Check that all .py files have corresponding .pyi files for public API
echo "  Checking core.py -> core.pyi"
if [ -f "hedl/core.pyi" ]; then
    print_status 0 "core.pyi exists"
else
    print_status 1 "core.pyi missing"
fi

echo "  Checking lib.py -> lib.pyi"
if [ -f "hedl/lib.pyi" ]; then
    print_status 0 "lib.pyi exists"
else
    print_status 1 "lib.pyi missing"
fi

echo "  Checking __init__.py -> __init__.pyi"
if [ -f "hedl/__init__.pyi" ]; then
    print_status 0 "__init__.pyi exists"
else
    print_status 1 "__init__.pyi missing"
fi
echo

echo "7. Running stub tests..."
echo "--------------------------------"
if [ -f "tests/test_type_stubs.py" ]; then
    if python -m pytest tests/test_type_stubs.py -v --tb=short; then
        print_status 0 "Stub tests passed"
    else
        print_status 1 "Stub tests failed"
    fi
else
    echo -e "${YELLOW}Warning: tests/test_type_stubs.py not found${NC}"
fi
echo

echo "==================================="
if [ "$OVERALL_SUCCESS" = true ]; then
    echo -e "${GREEN}All type validations passed!${NC}"
    exit 0
else
    echo -e "${RED}Some type validations failed${NC}"
    exit 1
fi
