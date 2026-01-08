#!/usr/bin/env bash
# Quick mypy validation for HEDL type stubs

set -euo pipefail

cd "$(dirname "$0")/.."

echo "Testing HEDL type stubs with mypy..."
echo

# Check if mypy is installed
if ! command -v mypy &> /dev/null; then
    echo "Error: mypy is not installed"
    echo "Install with: pip install mypy"
    exit 1
fi

# Test the stubs themselves
echo "1. Checking stub files..."
if mypy hedl --config-file mypy.ini --no-error-summary 2>&1 | grep -q "Success"; then
    echo "✓ Stub files are valid"
else
    echo "✗ Stub validation failed"
    mypy hedl --config-file mypy.ini
    exit 1
fi

# Test typed example
echo
echo "2. Checking typed example..."
if [ -f "examples/typed_usage.py" ]; then
    if mypy examples/typed_usage.py --config-file mypy.ini; then
        echo "✓ Example type checks passed"
    else
        echo "✗ Example type checking failed"
        exit 1
    fi
else
    echo "! Example file not found (non-fatal)"
fi

echo
echo "All mypy checks passed!"
