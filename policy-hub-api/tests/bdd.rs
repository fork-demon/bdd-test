//! BDD Test Harness for Policy Hub API
//!
//! Run with: cargo test --test bdd --features couchbase
//!
//! Prerequisites:
//! - Couchbase running (docker-compose up -d)
//! - API server running (STORAGE_TYPE=couchbase cargo run --features couchbase)

use cucumber::{given, then, when, World};
use reqwest::StatusCode;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Base URL for the API server
const API_BASE_URL: &str = "http://localhost:8080";

/// World state shared across steps
#[derive(Debug, Default, World)]
pub struct PolicyHubWorld {
    /// HTTP client for API requests
    #[world(default)]
    client: reqwest::Client,
    
    /// Last HTTP response status
    last_status: Option<StatusCode>,
    
    /// Last response body as JSON
    last_response: Option<Value>,
    
    /// Template name -> ID mapping
    template_ids: HashMap<String, String>,
    
    /// Policy name -> ID mapping
    policy_ids: HashMap<String, String>,
}

impl PolicyHubWorld {
    fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

// ==================== GIVEN Steps ====================

#[given("the API server is running")]
async fn server_is_running(world: &mut PolicyHubWorld) {
    let resp = world
        .client()
        .get(format!("{}/health", API_BASE_URL))
        .send()
        .await
        .expect("API server is not running. Start it with: STORAGE_TYPE=couchbase cargo run --features couchbase");
    
    assert!(resp.status().is_success(), "Health check failed");
}

#[given(expr = "a rule template {string} exists")]
async fn template_exists_simple(world: &mut PolicyHubWorld, name: String) {
    let source = r#"rule("default").when(f => true).then(f => ({result: "ok"}))"#;
    create_template(world, &name, source).await;
}

#[given(expr = "a rule template {string} exists with source:")]
async fn template_exists_with_source(world: &mut PolicyHubWorld, step: &cucumber::gherkin::Step, name: String) {
    let source = step.docstring().expect("docstring not found");
    create_template(world, &name, source).await;
}

#[given(expr = "a policy {string} exists using template {string}")]
async fn policy_exists(world: &mut PolicyHubWorld, policy_name: String, template_name: String) {
    let template_id = world
        .template_ids
        .get(&template_name)
        .cloned()
        .expect(&format!("Template '{}' not found. Create it first.", template_name));

    let body = json!({
        "name": policy_name,
        "rule_template_id": template_id,
        "metadata": {}
    });

    let resp = world
        .client()
        .post(format!("{}/api/policies", API_BASE_URL))
        .json(&body)
        .send()
        .await
        .expect("Failed to create policy");

    let json: Value = resp.json().await.expect("Failed to parse response");
    let policy_id = json["id"].as_str().expect("No id in response").to_string();
    world.policy_ids.insert(policy_name, policy_id);
}

// ==================== WHEN Steps ====================

#[when(expr = "I create a rule template named {string} with source {string}")]
async fn create_template_named(world: &mut PolicyHubWorld, name: String, source: String) {
    let body = json!({
        "name": name,
        "source": source
    });

    let resp = world
        .client()
        .post(format!("{}/api/rule-templates", API_BASE_URL))
        .json(&body)
        .send()
        .await
        .expect("Failed to create template");

    world.last_status = Some(resp.status());
    let json: Value = resp.json().await.expect("Failed to parse response");
    
    if let Some(id) = json["id"].as_str() {
        world.template_ids.insert(name.clone(), id.to_string());
    }
    world.last_response = Some(json);
}

#[when("I list all rule templates")]
async fn list_templates(world: &mut PolicyHubWorld) {
    let resp = world
        .client()
        .get(format!("{}/api/rule-templates", API_BASE_URL))
        .send()
        .await
        .expect("Failed to list templates");

    world.last_status = Some(resp.status());
    world.last_response = Some(resp.json().await.expect("Failed to parse response"));
}

#[when(expr = "I create a policy named {string} using template {string}")]
async fn create_policy_named(world: &mut PolicyHubWorld, name: String, template_name: String) {
    let template_id = world
        .template_ids
        .get(&template_name)
        .cloned()
        .expect(&format!("Template '{}' not found", template_name));

    let body = json!({
        "name": name,
        "rule_template_id": template_id,
        "metadata": {}
    });

    let resp = world
        .client()
        .post(format!("{}/api/policies", API_BASE_URL))
        .json(&body)
        .send()
        .await
        .expect("Failed to create policy");

    world.last_status = Some(resp.status());
    let json: Value = resp.json().await.expect("Failed to parse response");
    
    if let Some(id) = json["id"].as_str() {
        world.policy_ids.insert(name.clone(), id.to_string());
    }
    world.last_response = Some(json);
}

#[when(expr = "I get the policy {string}")]
async fn get_policy(world: &mut PolicyHubWorld, name: String) {
    let policy_id = world
        .policy_ids
        .get(&name)
        .cloned()
        .expect(&format!("Policy '{}' not found", name));

    let resp = world
        .client()
        .get(format!("{}/api/policies/{}", API_BASE_URL, policy_id))
        .send()
        .await
        .expect("Failed to get policy");

    world.last_status = Some(resp.status());
    world.last_response = Some(resp.json().await.expect("Failed to parse response"));
}

#[when("I list all policies")]
async fn list_policies(world: &mut PolicyHubWorld) {
    let resp = world
        .client()
        .get(format!("{}/api/policies", API_BASE_URL))
        .send()
        .await
        .expect("Failed to list policies");

    world.last_status = Some(resp.status());
    world.last_response = Some(resp.json().await.expect("Failed to parse response"));
}

#[when(expr = "I execute policy {string} with amount {int}")]
async fn execute_policy_with_amount(world: &mut PolicyHubWorld, name: String, amount: i32) {
    let policy_id = world
        .policy_ids
        .get(&name)
        .cloned()
        .expect(&format!("Policy '{}' not found", name));

    let body = json!({
        "policy_id": policy_id,
        "facts": { "amount": amount }
    });

    let resp = world
        .client()
        .post(format!("{}/api/execute", API_BASE_URL))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute policy");

    world.last_status = Some(resp.status());
    world.last_response = Some(resp.json().await.expect("Failed to parse response"));
}

// ==================== THEN Steps ====================

#[then(expr = "the response status should be {int}")]
async fn response_status(world: &mut PolicyHubWorld, expected: u16) {
    let status = world.last_status.expect("No response received");
    assert_eq!(status.as_u16(), expected, "Unexpected status code");
}

#[then(expr = "the response should contain {string}")]
async fn response_contains(world: &mut PolicyHubWorld, expected: String) {
    let resp = world.last_response.as_ref().expect("No response received");
    let resp_str = resp.to_string();
    assert!(resp_str.contains(&expected), "Response does not contain '{}': {}", expected, resp_str);
}

#[then(expr = "the template should have version {int}")]
async fn template_version(world: &mut PolicyHubWorld, expected: i64) {
    let resp = world.last_response.as_ref().expect("No response received");
    let version = resp["version"].as_i64().expect("No version in response");
    assert_eq!(version, expected, "Unexpected version");
}

#[then("the execution should succeed")]
async fn execution_succeeds(world: &mut PolicyHubWorld) {
    let resp = world.last_response.as_ref().expect("No response received");
    let success = resp["success"].as_bool().unwrap_or(false);
    assert!(success, "Execution did not succeed: {:?}", resp);
}

#[then("the condition should be met")]
async fn condition_met(world: &mut PolicyHubWorld) {
    let resp = world.last_response.as_ref().expect("No response received");
    let met = resp["condition_met"].as_bool().unwrap_or(false);
    assert!(met, "Condition was not met: {:?}", resp);
}

#[then("the condition should NOT be met")]
async fn condition_not_met(world: &mut PolicyHubWorld) {
    let resp = world.last_response.as_ref().expect("No response received");
    let met = resp["condition_met"].as_bool().unwrap_or(true);
    assert!(!met, "Condition was unexpectedly met: {:?}", resp);
}

#[then(expr = "the discount should be {int}")]
async fn discount_value(world: &mut PolicyHubWorld, expected: i64) {
    let resp = world.last_response.as_ref().expect("No response received");
    let discount = resp["output_facts"]["discount"].as_i64()
        .expect("No discount in output_facts");
    assert_eq!(discount, expected, "Unexpected discount value");
}

// ==================== Helper Functions ====================

async fn create_template(world: &mut PolicyHubWorld, name: &str, source: &str) {
    let body = json!({
        "name": name,
        "source": source
    });

    let resp = world
        .client()
        .post(format!("{}/api/rule-templates", API_BASE_URL))
        .json(&body)
        .send()
        .await
        .expect("Failed to create template");

    let json: Value = resp.json().await.expect("Failed to parse response");
    let id = json["id"].as_str().expect("No id in response").to_string();
    world.template_ids.insert(name.to_string(), id);
}

// ==================== Main ====================

#[tokio::main]
async fn main() {
    PolicyHubWorld::run("tests/features").await;
}
