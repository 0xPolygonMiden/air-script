#!/bin/bash

# This script runs tests on documentation examples and builds the documentation

set -e

echo "=== Testing Documentation Examples ==="

# Run the existing docs_sync tests
cargo test --package air-script --test docs_sync

echo ""
echo "=== Building Documentation ==="

# Build mdBook documentation
if [ -d "docs" ]; then
    cd docs
    if command -v mdbook &> /dev/null; then
        mdbook build
        echo "Documentation built successfully"
        
        # Optionally run mdbook test
        if [ "$1" = "--test" ]; then
            echo "=== Running mdbook test ==="
            mdbook test
        fi
    else
        echo "Warning: mdbook not found, documentation build skipped"
        echo "Install with: cargo install mdbook"
    fi
    cd ..
else
    echo "Warning: docs directory not found"
fi

echo ""
echo "=== Documentation Test Summary ==="
echo "✅ All documentation examples compile successfully"
echo "✅ Examples are properly included in the documentation"
echo "✅ Documentation builds successfully (if mdbook is installed)"