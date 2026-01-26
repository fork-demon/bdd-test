use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use policy_hub_api::{create_router, AppState};
use policy_hub_storage::InMemoryStorage;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt; // for oneshot

#[tokio::test]
async fn test_full_policy_lifecycle() {
    // 1. Setup AppState with InMemoryStorage
    let storage = Arc::new(InMemoryStorage::new());
    let app_state = Arc::new(AppState::with_storage(storage));
    let app = create_router(app_state);

    // 2. Create Rule Template
    let rule_source = r#"
        rule("test-rule")
            .when(facts => facts.value > 10)
            .then(facts => ({ result: "high" }));
    "#;

    let req = Request::builder()
        .method("POST")
        .uri("/api/rule-templates")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "name": "test-rule",
            "source": rule_source,
            "schema_version": 1
        }).to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let template_body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let template_id = template_body["id"].as_str().unwrap();
    let template_version = template_body["version"].as_u64().unwrap();

    // 3. Create Policy (linking to template)
    let req = Request::builder()
        .method("POST")
        .uri("/api/policies")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "name": "test-policy",
            "rule_template_id": template_id,
            "rule_template_version": template_version,
            "metadata": { "environment": "test" }
        }).to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let policy_body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let policy_id = policy_body["id"].as_str().unwrap();

    // 4. Execute Policy (Success case)
    let req = Request::builder()
        .method("POST")
        .uri("/api/execute")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "policy_id": policy_id,
            "facts": { "value": 20 }
        }).to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let exec_body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    
    assert_eq!(exec_body["success"], true);
    assert_eq!(exec_body["output_facts"]["result"], "high");

    // 5. Execute Policy (No match case)
    let req = Request::builder()
        .method("POST")
        .uri("/api/execute")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "policy_id": policy_id,
            "facts": { "value": 5 }
        }).to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let exec_body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    
    assert_eq!(exec_body["success"], true);
    // output_facts should be null or similar depending on implementation
}

#[tokio::test]
async fn test_multiple_policies_bundling() {
    // 1. Setup
    let storage = Arc::new(InMemoryStorage::new());
    let app_state = Arc::new(AppState::with_storage(storage.clone()));
    let app = create_router(app_state);

    // Helper to create template
    async fn create_template(app: &axum::Router, name: &str, condition: &str, output: &str) -> (String, u64) {
        let source = format!(
            r#"rule("{}").when(facts => {}).then(facts => ({{ result: {} }}));"#,
            name, condition, output
        );
        let req = Request::builder()
            .method("POST")
            .uri("/api/rule-templates")
            .header("content-type", "application/json")
            .body(Body::from(json!({
                "name": name,
                "source": source,
                "schema_version": 1
            }).to_string()))
            .unwrap();
        
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED, "Failed to create template {}", name);
        let body: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
        (body["id"].as_str().unwrap().to_string(), body["version"].as_u64().unwrap())
    }

    // Helper to create policy
    async fn create_policy(app: &axum::Router, name: &str, tid: &str, tver: u64) -> String {
        let req = Request::builder()
            .method("POST")
            .uri("/api/policies")
            .header("content-type", "application/json")
            .body(Body::from(json!({
                "name": name,
                "rule_template_id": tid,
                "rule_template_version": tver,
                "metadata": {}
            }).to_string()))
            .unwrap();
        
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED, "Failed to create policy {}", name);
        let body: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
        body["id"].as_str().unwrap().to_string()
    }

    // 2. Create 3 distinct Templates & Policies
    // Policy A: Value > 10 -> "High"
    let (t1, v1) = create_template(&app, "T1", "facts.val > 10", "\"High\"").await;
    let p1 = create_policy(&app, "P1", &t1, v1).await;

    // Policy B: Value < 5 -> "Low"
    let (t2, v2) = create_template(&app, "T2", "facts.val < 5", "\"Low\"").await;
    let p2 = create_policy(&app, "P2", &t2, v2).await;

    // Policy C: Value == 7 -> "Lucky"
    let (t3, v3) = create_template(&app, "T3", "facts.val === 7", "\"Lucky\"").await;
    let p3 = create_policy(&app, "P3", &t3, v3).await;

    // 3. Execute ALL against the SAME bundle state
    // The bundler should have rebuilt after P3 execution (or each step).
    // Use app.clone() to simulate persistent server state (state is shared via Arc)

    // Exec P1 with 20 -> High
    let req = Request::builder()
        .method("POST") .uri("/api/execute") .header("content-type", "application/json")
        .body(Body::from(json!({ "policy_id": p1, "facts": { "val": 20 } }).to_string())).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let out: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(out["output_facts"]["result"], "High");

    // Exec P2 with 2 -> Low
    let req = Request::builder()
        .method("POST") .uri("/api/execute") .header("content-type", "application/json")
        .body(Body::from(json!({ "policy_id": p2, "facts": { "val": 2 } }).to_string())).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let out: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(out["output_facts"]["result"], "Low");

    // Exec P3 with 7 -> Lucky
    let req = Request::builder()
        .method("POST") .uri("/api/execute") .header("content-type", "application/json")
        .body(Body::from(json!({ "policy_id": p3, "facts": { "val": 7 } }).to_string())).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let out: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(out["output_facts"]["result"], "Lucky");

    // Verify Cross-Talk (Running P1 with P2's input shouldn't trigger P1)
    let req = Request::builder()
        .method("POST") .uri("/api/execute") .header("content-type", "application/json")
        .body(Body::from(json!({ "policy_id": p1, "facts": { "val": 2 } }).to_string())).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let out: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(out["conditionMet"], false);
}
