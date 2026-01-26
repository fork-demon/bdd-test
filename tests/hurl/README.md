# Hurl API Tests

Black-box HTTP tests for Policy Hub API using [Hurl](https://hurl.dev/).

## Prerequisites

```bash
# Install Hurl
brew install hurl

# Start Couchbase
docker-compose up -d

# Start API server
STORAGE_TYPE=couchbase cargo run --features couchbase
```

## Run Tests

```bash
# Run all tests
hurl --test --variable timestamp=$(date +%s) tests/hurl/*.hurl

# Run specific test file
hurl --test --variable timestamp=$(date +%s) tests/hurl/03_execution.hurl

# Verbose output
hurl --verbose --variable timestamp=$(date +%s) tests/hurl/03_execution.hurl
```

## Test Files

| File | Description |
|------|-------------|
| `01_rule_templates.hurl` | Template CRUD operations |
| `02_policies.hurl` | Policy CRUD operations |
| `03_execution.hurl` | E2E execution scenarios |
| `04_errors.hurl` | Error handling cases |
