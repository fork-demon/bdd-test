//! Policy Hub Bundler
//!
//! Aggregates policies and rule templates into a single execution bundle (simulated WASM).

use anyhow::Result;
use policy_hub_core::{Policy, RuleTemplate};
use std::collections::HashMap;
use uuid::Uuid;

pub struct Bundler;

impl Bundler {
    /// Bundle all policies and templates into a single artifact
    ///
    /// In a real-world scenario using `wizer` or `javy`, this would:
    /// 1. Take a base WASM binary (containing JS engine)
    /// 2. Inject the generated JS code into it
    /// 3. Snapshot the memory state
    /// 4. Return the new WASM binary
    ///
    /// Here, we simply return the "JS Source Bundle" as bytes,
    /// which our WasmExecutor knows how to load into its engine.
    pub fn bundle_all(
        policies: &[Policy],
        templates: &HashMap<Uuid, RuleTemplate>,
    ) -> Result<Vec<u8>> {
        let mut js_code = String::new();

        // 1. Add Runtime Helpers (if not already included in templates)
        // We assume templates have the helper code or we add it once here.
        // For simplicity, we'll rely on the templates being compiled with helpers 
        // OR better: we define a global runtime here.
        
        js_code.push_str(r#"
            // Global Runtime Map
            var __POLICY_MAP = {};
            
            // Helper to register policies
            function __register_policy(id, ruleFn, metadata) {
                __POLICY_MAP[id] = {
                    rule: ruleFn,
                    metadata: metadata
                };
            }

            // Global Execute Function (Entry Point)
            function __execute_bundle(policyId, factsJson) {
                var policy = __POLICY_MAP[policyId];
                if (!policy) {
                     return JSON.stringify({ error: "Policy not found in bundle: " + policyId });
                }
                
                var facts = JSON.parse(factsJson);
                var metadata = policy.metadata;
                
                // Execute rule (assuming ruleFn follows the { condition, action } pattern or simpler)
                // Our compiled templates usually return a rule object or builder.
                // We need to adapt based on how `compiled_js` looks.
                
                // Let's assume the compiled_js defines a `rule` variable or similar.
                // To isolate them, we wrap each in a closure.
                
                try {
                    var result = policy.rule(facts, metadata);
                    return JSON.stringify(result);
                } catch (e) {
                    return JSON.stringify({ error: e.toString() });
                }
            }
        "#);

        // 2. Add Rule Templates and Policies
        // We need to handle the fact that multiple policies might use the same template.
        // But the user wants "One Bundle".
        // Strategy:
        // - Embed all unique RuleTemplates as functions.
        // - Map PolicyID -> TemplateFunction + Metadata.

        // Map template_id -> function_name
        let mut template_fn_map = HashMap::new();

        for (id, template) in templates {
            let fn_name = format!("__template_{}", id.simple());
            template_fn_map.insert(*id, fn_name.clone());

            let source = template.compiled_js.as_deref().unwrap_or("");
            
            // Wrap template source in an isolated function
            // The compiled source defines `function __execute(factsJson, metaJson)` globally
            // We need to call it after defining it
            js_code.push_str(&format!(
                r#"
                function {}(facts, metadata) {{
                    // Define __execute within this scope
                    {}
                    
                    // Call the internal __execute function with JSON strings
                    return __execute(JSON.stringify(facts), JSON.stringify(metadata));
                }}
                "#, 
                fn_name, 
                source
            ));
        }

        // 3. Register Policies
        for policy in policies {
            if let Some(fn_name) = template_fn_map.get(&policy.rule_template_id) {
                let metadata_json = serde_json::to_string(&policy.metadata).unwrap_or("{}".into());
                js_code.push_str(&format!(
                    r#"
                    __register_policy("{}", {}, {});
                    "#,
                    policy.id,
                    fn_name,
                    metadata_json
                ));
            }
        }

        Ok(js_code.into_bytes())
    }
}
