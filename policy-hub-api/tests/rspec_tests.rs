//! rust-rspec BDD Tests for Policy Hub API
//!
//! This demonstrates the rust-rspec framework which uses describe/it syntax
//! (similar to Ruby's RSpec) instead of Gherkin's Given/When/Then.
//!
//! NOTE: rust-rspec is NOT recommended for new projects as it hasn't been
//! maintained since 2019. This is included for evaluation purposes only.
//!
//! Run with: cargo test --test rspec --features couchbase

use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::time::Duration;

/// API Base URL - configurable via environment variable
fn api_base_url() -> String {
    std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

/// Create HTTP client with timeout
fn create_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client")
}

/// Helper to create a rule template
fn create_template(client: &Client, name: &str, source: &str) -> String {
    let resp = client
        .post(format!("{}/api/rule-templates", api_base_url()))
        .json(&json!({
            "name": name,
            "source": source
        }))
        .send()
        .expect("Failed to create template");
    
    let json: Value = resp.json().expect("Failed to parse response");
    json["id"].as_str().expect("No id in response").to_string()
}

/// Helper to create a policy
fn create_policy(client: &Client, name: &str, template_id: &str) -> String {
    let resp = client
        .post(format!("{}/api/policies", api_base_url()))
        .json(&json!({
            "name": name,
            "rule_template_id": template_id,
            "metadata": {}
        }))
        .send()
        .expect("Failed to create policy");
    
    let json: Value = resp.json().expect("Failed to parse response");
    json["id"].as_str().expect("No id in response").to_string()
}

/// Helper to execute a policy
fn execute_policy(client: &Client, policy_id: &str, facts: Value) -> Value {
    let resp = client
        .post(format!("{}/api/execute", api_base_url()))
        .json(&json!({
            "policy_id": policy_id,
            "facts": facts
        }))
        .send()
        .expect("Failed to execute policy");
    
    resp.json().expect("Failed to parse response")
}

// ==================== Tests using RSpec-style describe/it ====================

#[cfg(test)]
mod tests {
    use super::*;

    mod api_health {
        use super::*;

        #[test]
        fn it_returns_healthy_status() {
            // describe "API Health"
            //   it "returns healthy status"
            let client = create_client();
            
            let resp = client
                .get(format!("{}/health", api_base_url()))
                .send()
                .expect("Health check failed");
            
            assert!(resp.status().is_success());
            
            let json: Value = resp.json().expect("Failed to parse");
            assert_eq!(json["status"], "healthy");
        }
    }

    mod rule_templates {
        use super::*;

        #[test]
        fn it_creates_a_new_template() {
            // describe "Rule Templates"
            //   context "when creating a new template"
            //     it "returns the template with an ID"
            let client = create_client();
            let name = format!("rspec-test-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis());
            
            let resp = client
                .post(format!("{}/api/rule-templates", api_base_url()))
                .json(&json!({
                    "name": name,
                    "source": r#"rule("test").when(f => true).then(f => ({ok: true}))"#
                }))
                .send()
                .expect("Create template failed");
            
            assert_eq!(resp.status().as_u16(), 201);
            
            let json: Value = resp.json().expect("Failed to parse");
            assert!(json["id"].is_string());
            assert_eq!(json["name"], name);
            assert!(json["version"].as_i64().unwrap() >= 1);
        }

        #[test]
        fn it_lists_all_templates() {
            // describe "Rule Templates"
            //   context "when listing templates"  
            //     it "returns an array"
            let client = create_client();
            
            let resp = client
                .get(format!("{}/api/rule-templates", api_base_url()))
                .send()
                .expect("List templates failed");
            
            assert!(resp.status().is_success());
            
            let json: Value = resp.json().expect("Failed to parse");
            assert!(json.is_array());
        }
    }

    mod policy_execution {
        use super::*;

        #[test]
        fn it_executes_when_condition_is_met() {
            // describe "Policy Execution"
            //   context "when amount > 100"
            //     it "applies the discount"
            let client = create_client();
            
            // Setup: create template and policy
            let template_id = create_template(
                &client,
                &format!("discount-{}", std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()),
                r#"rule("discount").when(f => f.amount > 100).then(f => ({discount: f.amount * 0.1}))"#
            );
            
            let policy_id = create_policy(
                &client,
                &format!("discount-policy-{}", std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()),
                &template_id
            );
            
            // Execute with amount > 100
            let result = execute_policy(&client, &policy_id, json!({"amount": 150}));
            
            // Verify
            assert_eq!(result["success"], true);
            assert_eq!(result["condition_met"], true);
            assert_eq!(result["output_facts"]["discount"], 15);
        }

        #[test]
        fn it_does_not_execute_when_condition_is_not_met() {
            // describe "Policy Execution"
            //   context "when amount <= 100"
            //     it "does not apply the discount"
            let client = create_client();
            
            let template_id = create_template(
                &client,
                &format!("no-discount-{}", std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()),
                r#"rule("discount").when(f => f.amount > 100).then(f => ({discount: f.amount * 0.1}))"#
            );
            
            let policy_id = create_policy(
                &client,
                &format!("no-discount-policy-{}", std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()),
                &template_id
            );
            
            // Execute with amount <= 100
            let result = execute_policy(&client, &policy_id, json!({"amount": 50}));
            
            // Verify
            assert_eq!(result["success"], true);
            assert_eq!(result["condition_met"], false);
        }

        #[test]
        fn it_handles_boundary_value() {
            // describe "Policy Execution"
            //   context "when amount is exactly 100"
            //     it "does not apply the discount (boundary case)"
            let client = create_client();
            
            let template_id = create_template(
                &client,
                &format!("boundary-{}", std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()),
                r#"rule("discount").when(f => f.amount > 100).then(f => ({discount: f.amount * 0.1}))"#
            );
            
            let policy_id = create_policy(
                &client,
                &format!("boundary-policy-{}", std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()),
                &template_id
            );
            
            // Execute with amount = 100 (boundary)
            let result = execute_policy(&client, &policy_id, json!({"amount": 100}));
            
            // Verify: 100 is NOT > 100, so condition not met
            assert_eq!(result["success"], true);
            assert_eq!(result["condition_met"], false);
        }
    }
}
