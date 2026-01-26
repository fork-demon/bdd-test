# pytest-bdd Tests for Policy Hub

Black-box BDD tests for the Policy Hub API.

## Prerequisites

```bash
# Install dependencies
pip install pytest pytest-bdd requests

# Optional: For Couchbase data validation
pip install couchbase
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `API_BASE_URL` | `http://localhost:8080` | API server URL |
| `CB_HOST` | `couchbase://localhost` | Couchbase connection string |
| `CB_USER` | `Administrator` | Couchbase username |
| `CB_PASSWORD` | `password` | Couchbase password |
| `CB_BUCKET` | `policy-hub` | Couchbase bucket name |

## Running Tests

### Against Local Development Server

```bash
# Start the server first
cd /path/to/policy-hub
STORAGE_TYPE=couchbase cargo run --features couchbase

# In another terminal, run tests
cd tests/bdd
pytest -v
```

### Against Staging

```bash
API_BASE_URL=https://staging.example.com \
CB_HOST=couchbase://staging-cb.example.com \
CB_USER=test-user \
CB_PASSWORD=test-password \
pytest -v
```

### With Coverage Report

```bash
pytest -v --html=report.html --self-contained-html
```

## Test Structure

```
tests/bdd/
├── conftest.py              # Fixtures (API client, Couchbase, env vars)
├── features/
│   ├── execution.feature    # Policy execution scenarios
│   └── rule_templates.feature
├── steps.py                 # Step definitions
├── test_execution.py        # pytest-bdd test module
└── test_rule_templates.py
```

## Adding New Tests

1. Create a `.feature` file in `features/`
2. Add step definitions in `steps.py`
3. Create a test module (e.g., `test_myfeature.py`)
4. Run `pytest -v` to verify

## Couchbase Data Validation

To verify data is stored correctly in Couchbase:

```gherkin
Scenario: Policy is persisted to Couchbase
  Given a rule template "test-template" exists
  And a policy "test-policy" uses template "test-template"
  When I create the policy via API
  Then the policy should be stored in Couchbase
  And the policy name in Couchbase should be "test-policy"
```
