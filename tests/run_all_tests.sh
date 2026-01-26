#!/bin/bash
# Run all BDD test frameworks
# Usage: ./run_all_tests.sh

set -e

API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"
echo "Testing against: $API_BASE_URL"

# Check if API is running
echo "Checking API health..."
if ! curl -s "$API_BASE_URL/health" > /dev/null 2>&1; then
    echo "❌ API not available at $API_BASE_URL"
    echo "   Start with: STORAGE_TYPE=couchbase cargo run --features couchbase"
    exit 1
fi
echo "✅ API is healthy"

# Run each framework
echo ""
echo "=========================================="
echo "1. Hurl (HTTP tests)"
echo "=========================================="
cd "$(dirname "$0")/hurl"
if command -v hurl &> /dev/null; then
    hurl --test *.hurl --variable base_url="$API_BASE_URL" || echo "⚠️ Hurl tests had failures"
else
    echo "⚠️ Hurl not installed. Install with: brew install hurl"
fi
cd ..

echo ""
echo "=========================================="
echo "2. pytest-bdd (Python)"
echo "=========================================="
cd pytest-bdd
if [ -d ".venv" ]; then
    source .venv/bin/activate
    API_BASE_URL="$API_BASE_URL" pytest -v --tb=short || echo "⚠️ pytest-bdd had failures"
    deactivate
else
    echo "⚠️ Python venv not setup. Run: python -m venv .venv && source .venv/bin/activate && pip install pytest pytest-bdd requests"
fi
cd ..

echo ""
echo "=========================================="
echo "3. godog (Go)"
echo "=========================================="
cd godog
if command -v go &> /dev/null; then
    API_BASE_URL="$API_BASE_URL" go test -v || echo "⚠️ godog had failures"
else
    echo "⚠️ Go not installed"
fi
cd ..

echo ""
echo "=========================================="
echo "4. cucumber-rs (Rust) - in policy-hub-api"
echo "=========================================="
cd "$(dirname "$0")/.."
cargo test --test bdd --features couchbase -p policy-hub-api 2>/dev/null || echo "⚠️ cucumber-rs had failures"

echo ""
echo "=========================================="
echo "5. rust-rspec style (Rust)"
echo "=========================================="
# Note: These are just standard Rust tests organized in rspec style
# They would need to be added to Cargo.toml properly
echo "⚠️ rust-rspec tests need Cargo.toml setup (see tests/rust-rspec/README.md)"

echo ""
echo "=========================================="
echo "Test run complete!"
echo "=========================================="
