# BDD Framework Samples — One Scenario Per Framework

> A single scenario — **"Execute policy when condition is met"** — implemented in each evaluated BDD framework. This shows the feature file (where applicable) and corresponding step definitions side-by-side, so you can compare the developer experience at a glance.

---

## Scenario Under Test

> **Given** a discount rule template exists (amount > 100 → 10% discount)  
> **And** a policy is configured with that template  
> **When** I execute the policy with amount 150  
> **Then** the execution succeeds, condition is met, and discount = 15

---

## 1. pytest-bdd (Python) ✅ Recommended

**Language:** Python &nbsp;|&nbsp; **Gherkin:** Yes &nbsp;|&nbsp; **Type:** True Black-Box

### Feature File

`tests/pytest-bdd/features/execution.feature`

```gherkin
Feature: Policy Execution

  Background:
    Given the API is available

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
```

### Step Definitions

`tests/pytest-bdd/steps.py`

```python
from pytest_bdd import given, when, then, parsers
import requests, os

API_URL = os.environ.get("API_BASE_URL", "http://localhost:8080")

@given("the API is available")
def api_available():
    resp = requests.get(f"{API_URL}/health")
    assert resp.status_code == 200

@given(parsers.parse('a rule template "{name}" exists with source:'))
def create_template(context, name, docstring):
    response = requests.post(f"{API_URL}/api/rule-templates",
        json={"name": name, "source": docstring})
    assert response.status_code == 201
    context["template_id"] = response.json()["id"]

@given(parsers.parse('a policy "{name}" uses template "{template}"'))
def create_policy(context, name, template):
    response = requests.post(f"{API_URL}/api/policies",
        json={"name": name, "rule_template_id": context["template_id"], "metadata": {}})
    assert response.status_code == 201
    context["policy_id"] = response.json()["id"]

@when("I execute the policy with facts:")
def execute_policy(context, datatable):
    amount = int(datatable[0]["amount"])
    response = requests.post(f"{API_URL}/api/execute",
        json={"policy_id": context["policy_id"], "facts": {"amount": amount}})
    context["result"] = response.json()

@then("the execution should succeed")
def execution_succeeds(context):
    assert context["result"]["success"] == True

@then(parsers.parse('the output field "{field}" should be {value:d}'))
def check_output(context, field, value):
    assert context["result"]["output_facts"][field] == value
```

### How to Run

```bash
cd tests/pytest-bdd && source .venv/bin/activate && pytest -v
```

---

## 2. godog (Go)

**Language:** Go &nbsp;|&nbsp; **Gherkin:** Yes &nbsp;|&nbsp; **Type:** True Black-Box

### Feature File

`tests/godog/features/execution.feature` *(same Gherkin as pytest-bdd)*

```gherkin
Feature: Policy Execution

  Background:
    Given the API server is running

  Scenario: Execute policy when condition is met
    Given a rule template "discount-rule" exists with source:
      """
      rule("discount").when(f => f.amount > 100).then(f => ({ discount: f.amount * 0.1 }))
      """
    And a policy "exec-test" exists using template "discount-rule"
    When I execute policy "exec-test" with amount 150
    Then the execution should succeed
    And the condition should be met
    And the discount should be 15
```

### Step Definitions

`tests/godog/api_test.go`

```go
package main

import (
    "bytes"; "encoding/json"; "fmt"; "net/http"; "os"
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
    body, _ := json.Marshal(map[string]string{"name": name, "source": source.Content})
    resp, err := http.Post(c.baseURL+"/api/rule-templates", "application/json", bytes.NewBuffer(body))
    if err != nil { return err }
    var result map[string]interface{}
    json.NewDecoder(resp.Body).Decode(&result)
    c.templateID = result["id"].(string)
    return nil
}

func (c *apiContext) iExecutePolicyWithAmount(name string, amount int) error {
    body, _ := json.Marshal(map[string]interface{}{
        "policy_id": c.policyID,
        "facts":     map[string]int{"amount": amount},
    })
    resp, _ := http.Post(c.baseURL+"/api/execute", "application/json", bytes.NewBuffer(body))
    json.NewDecoder(resp.Body).Decode(&c.lastResponse)
    return nil
}

func (c *apiContext) theDiscountShouldBe(expected int) error {
    output := c.lastResponse["output_facts"].(map[string]interface{})
    if int(output["discount"].(float64)) != expected {
        return fmt.Errorf("expected %d, got %v", expected, output["discount"])
    }
    return nil
}

func InitializeScenario(ctx *godog.ScenarioContext) {
    api := &apiContext{baseURL: os.Getenv("API_BASE_URL")}
    if api.baseURL == "" { api.baseURL = "http://localhost:8080" }
    ctx.Step(`^the API server is running$`, api.theAPIIsAvailable)
    ctx.Step(`^a rule template "([^"]*)" exists with source:$`, api.aRuleTemplateExistsWithSource)
    ctx.Step(`^I execute policy "([^"]*)" with amount (\d+)$`, api.iExecutePolicyWithAmount)
    ctx.Step(`^the discount should be (\d+)$`, api.theDiscountShouldBe)
}
```

### How to Run

```bash
cd tests/godog && go test -v
```

---

## 3. cucumber-rs (Rust)

**Language:** Rust &nbsp;|&nbsp; **Gherkin:** Yes &nbsp;|&nbsp; **Type:** Gray-Box (same process)

### Feature File

`tests/cucumber-rs/features/execution.feature` *(same Gherkin)*

```gherkin
Feature: Policy Execution

  Background:
    Given the API server is running
    And a rule template "exec-discount" exists with source:
      """
      rule("discount").when(f => f.amount > 100).then(f => ({ discount: f.amount * 0.1 }))
      """

  Scenario: Execute policy when condition is met
    Given a policy "exec-test-policy" exists using template "exec-discount"
    When I execute policy "exec-test-policy" with amount 150
    Then the execution should succeed
    And the condition should be met
    And the discount should be 15
```

### Step Definitions

`tests/cucumber-rs/bdd.rs`

```rust
use cucumber::{given, when, then, World};
use serde_json::{json, Value};

#[derive(Debug, Default, World)]
pub struct PolicyHubWorld {
    client: reqwest::Client,
    template_ids: std::collections::HashMap<String, String>,
    policy_ids: std::collections::HashMap<String, String>,
    last_response: Option<Value>,
}

#[given(expr = "the API server is running")]
async fn server_is_running(world: &mut PolicyHubWorld) {
    let resp = world.client.get("http://localhost:8080/health")
        .send().await.expect("API not running");
    assert!(resp.status().is_success());
}

#[given(expr = "a rule template {string} exists with source:")]
async fn template_exists(world: &mut PolicyHubWorld, name: String, docstring: String) {
    let resp = world.client.post("http://localhost:8080/api/rule-templates")
        .json(&json!({"name": name, "source": docstring}))
        .send().await.unwrap();
    let json: Value = resp.json().await.unwrap();
    world.template_ids.insert(name, json["id"].as_str().unwrap().to_string());
}

#[when(expr = "I execute policy {string} with amount {int}")]
async fn execute_policy(world: &mut PolicyHubWorld, name: String, amount: i32) {
    let pid = world.policy_ids.get(&name).unwrap();
    let resp = world.client.post("http://localhost:8080/api/execute")
        .json(&json!({"policy_id": pid, "facts": {"amount": amount}}))
        .send().await.unwrap();
    world.last_response = Some(resp.json().await.unwrap());
}

#[then("the discount should be {int}")]
async fn discount_value(world: &mut PolicyHubWorld, expected: i64) {
    let resp = world.last_response.as_ref().unwrap();
    assert_eq!(resp["output_facts"]["discount"].as_i64().unwrap(), expected);
}

fn main() {
    futures::executor::block_on(PolicyHubWorld::run("tests/cucumber-rs/features"));
}
```

### How to Run

```bash
cargo test --test bdd --features couchbase
```

---

## 4. rstest-bdd (Rust)

**Language:** Rust &nbsp;|&nbsp; **Gherkin:** Yes &nbsp;|&nbsp; **Type:** Gray-Box (same process)

### Feature File

`tests/rstest-bdd/features/execution.feature` *(same Gherkin)*

```gherkin
Feature: Policy Execution

  Background:
    Given the API server is running
    And a rule template "discount-rule" exists with source:
      """
      rule("discount").when(f => f.amount > 100).then(f => ({ discount: f.amount * 0.1 }))
      """

  Scenario: Execute policy when condition is met
    Given a policy "exec-test" exists using template "discount-rule"
    When I execute policy "exec-test" with amount 150
    Then the execution should succeed
    And the condition should be met
    And the discount should be 15
```

### Step Definitions

`policy-hub-api/tests/rstest_bdd_steps.rs`

```rust
use reqwest::blocking::Client;
use rstest_bdd::state::{ScenarioState, Slot};
use rstest_bdd_macros::{given, when, then, scenario, ScenarioState};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

// State uses Slot<T> for interior mutability across steps
#[derive(Default, ScenarioState)]
struct PolicyState {
    client: Slot<Client>,
    base_url: Slot<String>,
    template_ids: Slot<HashMap<String, String>>,
    policy_ids: Slot<HashMap<String, String>>,
    last_response: Slot<Value>,
}

impl PolicyState {
    fn ensure_init(&self) {
        if self.client.is_empty() {
            self.client.set(Client::builder().timeout(Duration::from_secs(10)).build().unwrap());
            self.base_url.set(std::env::var("API_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()));
            self.template_ids.set(HashMap::new());
            self.policy_ids.set(HashMap::new());
        }
    }
}

#[given("the API server is running")]
fn the_api_server_is_running(state: &PolicyState) {
    state.ensure_init();
    let base = state.base_url.with_ref(|u| u.clone()).unwrap();
    state.client.with_ref(|c| {
        let resp = c.get(format!("{}/health", base)).send().unwrap();
        assert!(resp.status().is_success());
    });
}

#[when("I execute policy {name:string} with amount {amount:int}")]
fn execute_policy(state: &PolicyState, name: String, amount: i64) {
    let base = state.base_url.with_ref(|u| u.clone()).unwrap();
    let pid = state.policy_ids.with_ref(|m| m.get(&name).cloned().unwrap()).unwrap();
    let (_, body) = state.client.with_ref(|c| {
        let resp = c.post(format!("{}/api/execute", base))
            .json(&json!({"policy_id": &pid, "facts": {"amount": amount}}))
            .send().unwrap();
        (resp.status().as_u16(), resp.json::<Value>().unwrap())
    }).unwrap();
    state.last_response.set(body);
}

#[then("the discount should be {expected:int}")]
fn discount_value(state: &PolicyState, expected: i64) {
    state.last_response.with_ref(|resp| {
        assert_eq!(resp["output_facts"]["discount"].as_i64().unwrap(), expected);
    });
}

// Fixture provides initialized state to each scenario
#[rstest::fixture]
fn state() -> PolicyState { let s = PolicyState::default(); s.ensure_init(); s }

#[scenario("tests/rstest-bdd-features/execution.feature",
           name = "Execute policy when condition is met")]
fn execute_condition_met(state: PolicyState) {}
```

### How to Run

```bash
cargo test --test rstest_bdd_steps -p policy-hub-api
```

---

## 5. rust-rspec (Rust describe/it)

**Language:** Rust &nbsp;|&nbsp; **Gherkin:** No &nbsp;|&nbsp; **Type:** Gray-Box (same process)

> **No feature file.** Specification is embedded in module names and test function names using the `describe`/`it` pattern.

### Step Definitions (Test Code Only)

`tests/rust-rspec/rspec_tests.rs`

```rust
use reqwest::blocking::Client;
use serde_json::{json, Value};

fn create_client() -> Client {
    Client::builder().timeout(Duration::from_secs(10)).build().unwrap()
}

fn create_template(client: &Client, name: &str, source: &str) -> String {
    let resp = client.post("http://localhost:8080/api/rule-templates")
        .json(&json!({"name": name, "source": source})).send().unwrap();
    resp.json::<Value>().unwrap()["id"].as_str().unwrap().to_string()
}

fn create_policy(client: &Client, name: &str, tid: &str) -> String {
    let resp = client.post("http://localhost:8080/api/policies")
        .json(&json!({"name": name, "rule_template_id": tid, "metadata": {}}))
        .send().unwrap();
    resp.json::<Value>().unwrap()["id"].as_str().unwrap().to_string()
}

#[cfg(test)]
mod policy_execution {
    use super::*;

    #[test]
    fn it_executes_when_condition_is_met() {
        // describe "Policy Execution"
        //   context "when amount > 100"
        //     it "applies the discount"
        let client = create_client();
        let tid = create_template(&client, "rspec-discount",
            r#"rule("discount").when(f => f.amount > 100).then(f => ({discount: f.amount * 0.1}))"#);
        let pid = create_policy(&client, "rspec-policy", &tid);

        let result: Value = client.post("http://localhost:8080/api/execute")
            .json(&json!({"policy_id": pid, "facts": {"amount": 150}}))
            .send().unwrap().json().unwrap();

        assert_eq!(result["success"], true);
        assert_eq!(result["condition_met"], true);
        assert_eq!(result["output_facts"]["discount"], 15);
    }
}
```

### How to Run

```bash
cargo test --test rspec_tests -p policy-hub-api
```

---

## 6. Hurl (HTTP Testing)

**Language:** Hurl DSL &nbsp;|&nbsp; **Gherkin:** No &nbsp;|&nbsp; **Type:** True Black-Box

> **No feature file.** Hurl uses its own declarative HTTP DSL. Not BDD, but useful for API smoke tests.

### Test File (Code Only)

`tests/hurl/01_execute.hurl`

```hurl
# Create template
POST http://localhost:8080/api/rule-templates
Content-Type: application/json
{
  "name": "hurl-discount",
  "source": "rule(\"discount\").when(f => f.amount > 100).then(f => ({discount: f.amount * 0.1}))"
}
HTTP 201
[Captures]
template_id: jsonpath "$.id"

# Create policy
POST http://localhost:8080/api/policies
Content-Type: application/json
{
  "name": "hurl-policy",
  "rule_template_id": "{{template_id}}",
  "metadata": {}
}
HTTP 201
[Captures]
policy_id: jsonpath "$.id"

# Execute — condition met
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
jsonpath "$.output_facts.discount" == 15
```

### How to Run

```bash
cd tests/hurl && hurl --test *.hurl
```

---

## Quick Comparison

| Framework | Language | Gherkin | Feature File | Black-Box | Runner |
|-----------|----------|---------|--------------|-----------|--------|
| **pytest-bdd** | Python | ✅ | Separate `.feature` | ✅ True | `pytest` |
| **godog** | Go | ✅ | Separate `.feature` | ✅ True | `go test` |
| **cucumber-rs** | Rust | ✅ | Separate `.feature` | ❌ Gray | Custom `main()` |
| **rstest-bdd** | Rust | ✅ | Separate `.feature` | ❌ Gray | `cargo test` |
| **rust-rspec** | Rust | ❌ | N/A (in code) | ❌ Gray | `cargo test` |
| **Hurl** | Hurl DSL | ❌ | N/A (`.hurl` files) | ✅ True | `hurl` CLI |
