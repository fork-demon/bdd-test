# BDD Framework Evaluation for Policy Hub


## TL;DR

After evaluating several BDD frameworks for testing our Rust microservices, I'm recommending **pytest-bdd** (Python). It gives us true black-box testing, solid Couchbase/Kafka integration, and aligns with how Netflix and Uber approach polyglot testing.

The tests should live in the same repo as the service - this keeps features and tests in sync and matches what most teams at scale are doing.

---

## The Problem We're Solving

We need a testing approach that:

1. Tests our Rust API as a **black box** - no peeking at internals, no gray-box risks
2. Lets us verify data actually landed in **Couchbase** after API calls
3. Doesn't fall behind when we ship new features (tests can't be an afterthought)
4. Works for a team where **developers write the tests** - we don't have separate QA

After digging through engineering blogs from Uber, Netflix, Stripe, and Cloudflare, plus hands-on testing with several frameworks, here's what I found.

---

## What the Industry Is Doing

I spent time looking at how larger companies approach this. Some highlights:

**Uber** - They've been explicit about using BDD and report a 30% reduction in defects from the improved collaboration between dev and test. They gate every code change with E2E tests and have invested heavily in ephemeral test environments (their [SLATE system](https://www.uber.com/blog/simplify-local-testing-uber/)). See also their posts on [microservice architecture](https://www.uber.com/blog/microservice-architecture/) and [end-to-end testing at scale](https://www.uber.com/blog/continuous-integration/).

**Netflix** - Classic polyglot testing setup. Test harnesses in Python or Go, services in whatever makes sense. Their [Integration Test team](https://netflixtechblog.com/a-]software-quality-strategy-for-netflix-7c57d653b7ca) ensures E2E quality, but each dev team owns their deliverables. Big focus on [chaos engineering](https://netflixtechblog.com/the-netflix-simian-army-16e57fbab116) and resilience testing.

**Stripe** - API-first with [sandbox environments](https://docs.stripe.com/sandboxes). Everything is designed around the test mode / production mode split. They've made contract-driven testing a core part of their workflow. See their [API design docs](https://stripe.com/docs/api).

**Cloudflare** - Heavy Rust shop. They open-sourced [`h3i` for HTTP/3 testing](https://blog.cloudflare.com/h3i/). Interesting to see them building dedicated testing tools rather than adapting existing frameworks. More on their [Rust usage](https://blog.cloudflare.com/cloudflare-workers-as-a-web-framework/).

The pattern I kept seeing: **tests written in a different language than the service**. Python is the most common choice for test harnesses, Go is the second. This gives you true process isolation and access to mature SDK ecosystems.

---

## The Frameworks I Evaluated

### pytest-bdd (Python) — My Pick

**GitHub**: [1.4k ⭐](https://github.com/pytest-dev/pytest-bdd) | **PyPI**: ~5M downloads/month | **Status**: Actively maintained

This is where I landed. Here's why:

The Python ecosystem has mature SDKs for everything. Need Couchbase? `couchbase` package works great. Kafka? `kafka-python`. The `requests` library is simple and reliable for HTTP calls.

pytest-bdd lets you write Gherkin feature files and implement steps in Python. The tests run in a completely separate process from your Rust service - genuine black-box testing.

```gherkin
Feature: Policy Execution
  Scenario: Apply discount when amount exceeds threshold
    Given a discount rule template exists
    And a policy is configured with that template
    When I execute the policy with amount 150
    Then the discount should be 15
```

The step definitions are straightforward Python:

```python
@when('I execute the policy with amount {amount:d}')
def execute_with_amount(context, amount):
    response = requests.post(f"{API_URL}/api/execute", json={
        "policy_id": context.policy_id,
        "facts": {"amount": amount}
    })
    context.result = response.json()
```

**Trade-off**: You're adding Python to a Rust project. This means a `requirements.txt`, a virtual environment, maybe some extra CI setup. In my view, this is worth it for the testing quality you get.

---

### godog (Go) — Good Alternative

**GitHub**: [2.3k ⭐](https://github.com/cucumber/godog) | **Go Modules**: Widely used | **Status**: Official Cucumber implementation

If you really don't want Python in your codebase, godog is solid. Go's philosophy is close to Rust's, the step definitions are typed, and you get a single binary.

The Couchbase SDK (`gocb`) is well-maintained. The main downside is Go's more verbose syntax for step definitions compared to Python.

I'd pick this if the team is already comfortable with Go or if there's a strong preference for compiled test binaries.

---

### cucumber-rs (Rust) — When You Need All-Rust

**GitHub**: [689 ⭐](https://github.com/cucumber-rs/cucumber) | **Crates.io**: ~800k total downloads | **Status**: Actively maintained

This is the native Rust option using the `gherkin` crate for parsing. It works, and I got it running (after fixing a tokio runtime issue).

The catch: your tests compile with your service. They run in the same process. This creates a gray-box risk - you can accidentally access internal modules, and your tests are coupled to your compile cycle.

I'd only recommend this if there's a hard requirement for an all-Rust toolchain.

---

### Others I Looked At

**Hurl** - Not BDD (no Gherkin), but excellent for quick HTTP smoke tests in CI. We already have these in `tests/hurl/` and they're useful for post-deployment checks.

**rust-rspec** - Unmaintained for 5+ years. The `describe/it` pattern is just module organization anyway - you can do this with standard Rust test modules.

**Karate DSL** - Requires Java. Looked interesting for low-code testing but doesn't fit our stack.

---

## Same Repo or Separate Repo?

I looked into this because I've seen both approaches at different companies.

**My recommendation: keep tests in the same repo.**

Here's the reasoning:

When tests are in the same repo, a feature PR includes both code and tests. You can't ship a feature without updating tests. The CI pipeline is one thing, not two. Shared fixtures and feature files are easy to maintain.

The separate-repo approach makes sense when multiple consumers need to run the same tests against your API, or when there's a regulatory reason for separation. That's not our situation.

Companies like Google and Uber use monorepos for this exact reason - atomic commits where everything related to a change goes in together.

The structure I'd recommend:

```
policy-hub/
├── src/                    # Rust service
├── tests/
│   ├── features/           # Shared .feature files
│   ├── pytest-bdd/         # Python step definitions + fixtures
│   ├── godog/              # Go tests (if we want both)
│   └── hurl/               # HTTP smoke tests
├── docker-compose.yml
└── Cargo.toml
```

---

## Making Sure Tests Don't Get Stale

This is the real risk with any testing approach. The black-box test suite can fall behind while features ship, and then it's just checking the happy path from three releases ago.

What I've seen work:

**Block the merge** - CI checks that if you touch `handlers.rs` or add a new endpoint, there should be corresponding `.feature` file changes. If not, the PR fails.

```yaml
- name: Check test coverage for API changes
  run: |
    ENDPOINTS=$(git diff --name-only main | grep handlers | wc -l)
    FEATURES=$(git diff --name-only main | grep .feature | wc -l)
    if [ $ENDPOINTS -gt 0 ] && [ $FEATURES -eq 0 ]; then
      echo "New API endpoints without feature tests"
      exit 1
    fi
```

**Run tests as a deploy gate** - The staging deploy doesn't succeed unless BDD tests pass. This is what Uber does with their end-to-end gate.

**Track freshness** - If you want to get fancy, track when code vs tests were last modified. A test file that hasn't been touched in months while the handler has been edited 12 times is a red flag.

---

## Couchbase Integration

Since we need to verify data in Couchbase, here's what that looks like with pytest-bdd:

```python
# conftest.py
import os
from couchbase.cluster import Cluster
from couchbase.auth import PasswordAuthenticator

@pytest.fixture(scope="session")
def couchbase():
    auth = PasswordAuthenticator(
        os.environ.get("CB_USER", "Administrator"),
        os.environ.get("CB_PASSWORD", "password")
    )
    cluster = Cluster(os.environ.get("CB_HOST", "couchbase://localhost"))
    return cluster.bucket("policy-hub").default_collection()

# In step definitions
@then('the policy should be stored in Couchbase')
def verify_stored(couchbase, context):
    doc = couchbase.get(f"policy::{context.policy_id}")
    assert doc.content_as[dict]["name"] == context.policy_name
```

Environment variables for CI:

| Variable | Description |
|----------|-------------|
| `API_BASE_URL` | Where the service is running |
| `CB_HOST` | Couchbase connection string |
| `CB_USER` / `CB_PASSWORD` | Couchbase credentials |

---

## Getting Started

If you want to try the pytest-bdd setup:

```bash
cd tests/pytest-bdd
python -m venv .venv
source .venv/bin/activate
pip install pytest pytest-bdd requests

# Make sure the API is running
API_BASE_URL=http://localhost:8080 pytest -v
```

We already have working examples in `tests/pytest-bdd/` with tests for policy execution and rule templates.

---

## Summary

| Decision | Choice | Why |
|----------|--------|-----|
| Primary framework | pytest-bdd | True black-box, rich SDK ecosystem, industry-standard polyglot testing |
| Repository location | Same repo | Atomic commits, can't ship without tests, simpler CI |
| DB verification | Couchbase Python SDK | Works in fixtures, straightforward assertions |

The trade-off is adding Python to the project. Given what we get in return (proper black-box testing, infrastructure access, alignment with how companies like Uber and Netflix do this), I think it's worth it.

---

*Questions or concerns? Let's discuss before we commit to this approach.*

---

## Appendix: Detailed Code Examples

### pytest-bdd (Python)

**Feature file** (`features/execution.feature`):
```gherkin
Feature: Policy Execution
  As an API consumer
  I want to execute policies with input facts
  So that I can get computed results

  Background:
    Given the API is available

  @smoke
  Scenario: Execute policy when condition is met
    Given a rule template "discount-rule" exists with source:
      """
      rule("discount").when(f => f.amount > 100).then(f => ({discount: f.amount * 0.1}))
      """
    And a policy "test-policy" uses template "discount-rule"
    When I execute the policy with facts:
      | amount |
      | 150    |
    Then the execution should succeed
    And the output field "discount" should be 15

  Scenario: Execute policy when condition is NOT met
    Given a rule template "no-discount-rule" exists with source:
      """
      rule("discount").when(f => f.amount > 100).then(f => ({discount: f.amount * 0.1}))
      """
    And a policy "no-discount-policy" uses template "no-discount-rule"
    When I execute the policy with facts:
      | amount |
      | 50     |
    Then the execution should succeed
    And the condition should NOT be met
```

**Step definitions** (`steps.py`):
```python
from pytest_bdd import given, when, then, parsers
import requests
import os

API_URL = os.environ.get("API_BASE_URL", "http://localhost:8080")

@given("the API is available")
def api_available():
    resp = requests.get(f"{API_URL}/health")
    assert resp.status_code == 200

@given(parsers.parse('a rule template "{name}" exists with source:'))
def create_template(context, name, docstring):
    response = requests.post(f"{API_URL}/api/rule-templates", json={
        "name": name,
        "source": docstring
    })
    assert response.status_code == 201
    context["template_id"] = response.json()["id"]

@given(parsers.parse('a policy "{name}" uses template "{template}"'))
def create_policy(context, name, template):
    response = requests.post(f"{API_URL}/api/policies", json={
        "name": name,
        "rule_template_id": context["template_id"],
        "metadata": {}
    })
    assert response.status_code == 201
    context["policy_id"] = response.json()["id"]

@when("I execute the policy with facts:")
def execute_policy(context, datatable):
    facts = {row["amount"]: int(row["amount"]) for row in datatable}
    # Simplified - just use amount directly  
    amount = int(datatable[0]["amount"])
    response = requests.post(f"{API_URL}/api/execute", json={
        "policy_id": context["policy_id"],
        "facts": {"amount": amount}
    })
    context["result"] = response.json()

@then("the execution should succeed")
def execution_succeeds(context):
    assert context["result"]["success"] == True

@then(parsers.parse('the output field "{field}" should be {value:d}'))
def check_output(context, field, value):
    assert context["result"]["output_facts"][field] == value
```

---

### godog (Go)

**Step definitions** (`api_test.go`):
```go
package main

import (
    "bytes"
    "encoding/json"
    "fmt"
    "net/http"
    "os"
    "github.com/cucumber/godog"
)

type apiContext struct {
    baseURL      string
    templateID   string
    policyID     string
    lastResponse map[string]interface{}
}

func (c *apiContext) theAPIIsAvailable() error {
    resp, err := http.Get(c.baseURL + "/health")
    if err != nil || resp.StatusCode != 200 {
        return fmt.Errorf("API not available")
    }
    return nil
}

func (c *apiContext) aRuleTemplateExistsWithSource(name string, source *godog.DocString) error {
    body, _ := json.Marshal(map[string]string{
        "name":   name,
        "source": source.Content,
    })
    resp, err := http.Post(c.baseURL+"/api/rule-templates", "application/json", bytes.NewBuffer(body))
    if err != nil {
        return err
    }
    var result map[string]interface{}
    json.NewDecoder(resp.Body).Decode(&result)
    c.templateID = result["id"].(string)
    return nil
}

func (c *apiContext) iExecutePolicyWithFacts(name string, facts *godog.DocString) error {
    var factsMap map[string]interface{}
    json.Unmarshal([]byte(facts.Content), &factsMap)
    
    body, _ := json.Marshal(map[string]interface{}{
        "policy_id": c.policyID,
        "facts":     factsMap,
    })
    resp, _ := http.Post(c.baseURL+"/api/execute", "application/json", bytes.NewBuffer(body))
    json.NewDecoder(resp.Body).Decode(&c.lastResponse)
    return nil
}

func (c *apiContext) theOutputFieldShouldBe(field string, expected int) error {
    output := c.lastResponse["output_facts"].(map[string]interface{})
    if int(output[field].(float64)) != expected {
        return fmt.Errorf("expected %d, got %v", expected, output[field])
    }
    return nil
}

func InitializeScenario(ctx *godog.ScenarioContext) {
    api := &apiContext{baseURL: os.Getenv("API_BASE_URL")}
    if api.baseURL == "" {
        api.baseURL = "http://localhost:8080"
    }
    
    ctx.Step(`^the API is available$`, api.theAPIIsAvailable)
    ctx.Step(`^a rule template "([^"]*)" exists with source:$`, api.aRuleTemplateExistsWithSource)
    ctx.Step(`^I execute policy "([^"]*)" with facts:$`, api.iExecutePolicyWithFacts)
    ctx.Step(`^the output field "([^"]*)" should be (\d+)$`, api.theOutputFieldShouldBe)
}
```

---

### cucumber-rs (Rust)

**Step definitions** (`bdd.rs`):
```rust
use cucumber::{given, when, then, World};
use reqwest::StatusCode;
use serde_json::{json, Value};

const API_URL: &str = "http://localhost:8080";

#[derive(Debug, Default, World)]
pub struct PolicyWorld {
    client: reqwest::Client,
    template_id: Option<String>,
    policy_id: Option<String>,
    result: Option<Value>,
}

#[given(expr = "the API is available")]
async fn api_available(world: &mut PolicyWorld) {
    let resp = world.client.get(format!("{}/health", API_URL))
        .send().await.expect("API not available");
    assert!(resp.status().is_success());
}

#[given(expr = "a rule template {string} exists with source:")]
async fn create_template(world: &mut PolicyWorld, step: &cucumber::gherkin::Step, name: String) {
    let source = step.docstring().expect("no docstring");
    let resp = world.client.post(format!("{}/api/rule-templates", API_URL))
        .json(&json!({"name": name, "source": source}))
        .send().await.unwrap();
    let json: Value = resp.json().await.unwrap();
    world.template_id = Some(json["id"].as_str().unwrap().to_string());
}

#[when(expr = "I execute the policy with amount {int}")]
async fn execute_policy(world: &mut PolicyWorld, amount: i32) {
    let resp = world.client.post(format!("{}/api/execute", API_URL))
        .json(&json!({
            "policy_id": world.policy_id.as_ref().unwrap(),
            "facts": {"amount": amount}
        }))
        .send().await.unwrap();
    world.result = Some(resp.json().await.unwrap());
}

#[then("the execution should succeed")]
async fn check_success(world: &mut PolicyWorld) {
    let result = world.result.as_ref().unwrap();
    assert_eq!(result["success"], true);
}

#[tokio::main]
async fn main() {
    PolicyWorld::run("tests/features").await;
}
```

---

### Hurl (HTTP testing)

**Smoke test** (`smoke.hurl`):
```hurl
# Health check
GET http://localhost:8080/health
HTTP 200
[Asserts]
jsonpath "$.status" == "healthy"

# Create template and execute
POST http://localhost:8080/api/rule-templates
Content-Type: application/json
{
  "name": "hurl-test-template",
  "source": "rule(\"test\").when(f => f.amount > 100).then(f => ({ok: true}))"
}
HTTP 201
[Captures]
template_id: jsonpath "$.id"

POST http://localhost:8080/api/policies
Content-Type: application/json
{
  "name": "hurl-test-policy",
  "rule_template_id": "{{template_id}}",
  "metadata": {}
}
HTTP 201
[Captures]
policy_id: jsonpath "$.id"

POST http://localhost:8080/api/execute
Content-Type: application/json
{
  "policy_id": "{{policy_id}}",
  "facts": {"amount": 150}
}
HTTP 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.condition_met" == true
```

---

### Running Each Framework

```bash
# pytest-bdd
cd tests/pytest-bdd
source .venv/bin/activate
pytest -v

# godog
cd tests/godog
go test -v

# cucumber-rs
cargo test --test bdd -p policy-hub-api

# hurl
cd tests/hurl
hurl --test *.hurl
```
