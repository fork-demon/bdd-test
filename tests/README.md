# BDD Test Implementations

All BDD frameworks are consolidated in this directory for comparison.

## Implemented Frameworks

| Framework | Directory | Status | Run Command |
|-----------|-----------|--------|-------------|
| **pytest-bdd** | `pytest-bdd/` | ✅ Implemented | `cd pytest-bdd && pytest -v` |
| **godog** | `godog/` | ✅ Working | `cd godog && go test -v` |
| **cucumber-rs** | `cucumber-rs/` | ✅ Working | `cargo test --test bdd -p policy-hub-api` |
| **rust-rspec** | `rust-rspec/` | ✅ Implemented | See README in directory |
| **hurl** | `hurl/` | ✅ Implemented | `cd hurl && hurl --test *.hurl` |

## Directory Structure

```
tests/
├── README.md              # This file
├── run_all_tests.sh       # Script to run all frameworks
├── features/              # Shared Gherkin feature files
│   ├── execution.feature
│   ├── policies.feature
│   └── rule_templates.feature
├── pytest-bdd/            # Python BDD (RECOMMENDED)
├── godog/                 # Go BDD
├── cucumber-rs/           # Rust Cucumber
├── rust-rspec/            # Rust RSpec-style (describe/it)
└── hurl/                  # HTTP testing tool
```

## Quick Start

```bash
# 1. Ensure API is running
STORAGE_TYPE=couchbase cargo run --features couchbase

# 2. Run tests (pick one)
cd tests/godog && go test -v              # godog
cargo test --test bdd -p policy-hub-api   # cucumber-rs
cd tests/pytest-bdd && pytest -v          # pytest-bdd
```

## Framework Comparison Summary

| Aspect | pytest-bdd | godog | cucumber-rs | rust-rspec |
|--------|------------|-------|-------------|------------|
| **Syntax** | Gherkin | Gherkin | Gherkin | describe/it |
| **Black-box** | ✅ True | ✅ True | ⚠️ Gray | ⚠️ Gray |
| **Couchbase SDK** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes |
| **Setup** | pip install | go mod | Cargo.toml | Cargo.toml |
