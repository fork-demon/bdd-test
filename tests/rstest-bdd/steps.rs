//! rstest-bdd Step Definitions for Policy Hub API
//!
//! Demonstrates `rstest-bdd` v0.5 with Gherkin `.feature` files,
//! `#[given]`/`#[when]`/`#[then]` macros, and `Slot`-based state.
//!
//! Runs via `cargo test --test rstest_bdd_steps -p policy-hub-api`

use reqwest::blocking::Client;
use rstest_bdd::state::{ScenarioState, Slot};
use rstest_bdd_macros::{given, when, then, scenario, ScenarioState};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

// ==================== State ====================

#[derive(Default, ScenarioState)]
struct PolicyState {
    client: Slot<Client>,
    base_url: Slot<String>,
    template_ids: Slot<HashMap<String, String>>,
    policy_ids: Slot<HashMap<String, String>>,
    last_response: Slot<Value>,
    last_status: Slot<u16>,
}

impl PolicyState {
    fn ensure_init(&self) {
        if self.client.is_empty() {
            self.client.set(
                Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()
                    .expect("Failed to create HTTP client"),
            );
            self.base_url.set(
                std::env::var("API_BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            );
            self.template_ids.set(HashMap::new());
            self.policy_ids.set(HashMap::new());
        }
    }

    fn base(&self) -> String {
        self.base_url
            .with_ref(|u| u.clone())
            .unwrap_or_else(|| "http://localhost:8080".to_string())
    }
}

// ==================== GIVEN Steps ====================

#[given("the API server is running")]
fn the_api_server_is_running(state: &PolicyState) {
    state.ensure_init();
    let base = state.base();
    state.client.with_ref(|c| {
        let resp = c
            .get(format!("{}/health", base))
            .send()
            .expect("API server is not running");
        assert!(resp.status().is_success(), "Health check failed");
    });
}

#[given("a rule template {name:string} exists with source:")]
fn a_rule_template_exists_with_source(state: &PolicyState, name: String, docstring: String) {
    state.ensure_init();
    let base = state.base();
    let (status, id) = state.client.with_ref(|c| {
        let resp = c
            .post(format!("{}/api/rule-templates", base))
            .json(&json!({
                "name": &name,
                "source": &docstring
            }))
            .send()
            .expect("Failed to create template");

        let status = resp.status().as_u16();
        let body: Value = resp.json().expect("Failed to parse response");
        let id = body["id"].as_str().expect("No id in response").to_string();
        (status, id)
    }).unwrap();

    assert_eq!(status, 201, "Template creation failed");
    state.template_ids.with_mut(|map| {
        map.insert(name, id);
    });
}

#[given("a policy {name:string} exists using template {template:string}")]
fn a_policy_exists_using_template(state: &PolicyState, name: String, template: String) {
    state.ensure_init();
    let base = state.base();
    let template_id = state.template_ids.with_ref(|map| {
        map.get(&template)
            .cloned()
            .unwrap_or_else(|| panic!("Template '{}' not found", template))
    }).unwrap();

    let (status, id) = state.client.with_ref(|c| {
        let resp = c
            .post(format!("{}/api/policies", base))
            .json(&json!({
                "name": &name,
                "rule_template_id": &template_id,
                "metadata": {}
            }))
            .send()
            .expect("Failed to create policy");

        let status = resp.status().as_u16();
        let body: Value = resp.json().expect("Failed to parse response");
        let id = body["id"].as_str().expect("No id in response").to_string();
        (status, id)
    }).unwrap();

    assert_eq!(status, 201, "Policy creation failed");
    state.policy_ids.with_mut(|map| {
        map.insert(name, id);
    });
}

// ==================== WHEN Steps ====================

#[when("I execute policy {name:string} with amount {amount:int}")]
fn i_execute_policy_with_amount(state: &PolicyState, name: String, amount: i64) {
    let base = state.base();
    let policy_id = state.policy_ids.with_ref(|map| {
        map.get(&name)
            .cloned()
            .unwrap_or_else(|| panic!("Policy '{}' not found", name))
    }).unwrap();

    let (status, body) = state.client.with_ref(|c| {
        let resp = c
            .post(format!("{}/api/execute", base))
            .json(&json!({
                "policy_id": &policy_id,
                "facts": { "amount": amount }
            }))
            .send()
            .expect("Failed to execute policy");

        let status = resp.status().as_u16();
        let body: Value = resp.json().expect("Failed to parse response");
        (status, body)
    }).unwrap();

    state.last_status.set(status);
    state.last_response.set(body);
}

// ==================== THEN Steps ====================

#[then("the execution should succeed")]
fn the_execution_should_succeed(state: &PolicyState) {
    state.last_response.with_ref(|resp| {
        assert_eq!(resp["success"], true, "Execution did not succeed: {:?}", resp);
    }).expect("No response");
}

#[then("the condition should be met")]
fn the_condition_should_be_met(state: &PolicyState) {
    state.last_response.with_ref(|resp| {
        assert_eq!(resp["condition_met"], true, "Condition was not met: {:?}", resp);
    }).expect("No response");
}

#[then("the condition should NOT be met")]
fn the_condition_should_not_be_met(state: &PolicyState) {
    state.last_response.with_ref(|resp| {
        assert_eq!(resp["condition_met"], false, "Condition was unexpectedly met: {:?}", resp);
    }).expect("No response");
}

#[then("the discount should be {expected:int}")]
fn the_discount_should_be(state: &PolicyState, expected: i64) {
    state.last_response.with_ref(|resp| {
        let discount = resp["output_facts"]["discount"]
            .as_i64()
            .expect("No discount in output_facts");
        assert_eq!(discount, expected, "Unexpected discount value");
    }).expect("No response");
}

// ==================== Fixture ====================

#[rstest::fixture]
fn state() -> PolicyState {
    let s = PolicyState::default();
    s.ensure_init();
    s
}

// ==================== Scenario Bindings ====================

#[scenario(
    "tests/rstest-bdd-features/execution.feature",
    name = "Execute policy when condition is met"
)]
fn execute_condition_met(state: PolicyState) {}

#[scenario(
    "tests/rstest-bdd-features/execution.feature",
    name = "Execute policy when condition is NOT met"
)]
fn execute_condition_not_met(state: PolicyState) {}

#[scenario(
    "tests/rstest-bdd-features/execution.feature",
    name = "Execute policy at boundary value"
)]
fn execute_boundary_value(state: PolicyState) {}
