//! Rule executor using QuickJS runtime

use crate::ExecutorError;
use lru::LruCache;
use parking_lot::Mutex;
use policy_hub_core::ExecutionResult;
use rquickjs::{Context, Runtime};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Instant;

/// Cached compiled script
struct CompiledScript {
    source: String,
}

/// Rule executor with LRU caching for compiled scripts
pub struct RuleExecutor {
    cache: Arc<Mutex<LruCache<String, CompiledScript>>>,
}

impl RuleExecutor {
    pub fn new(cache_size: usize) -> Self {
        let cache_size = NonZeroUsize::new(cache_size).unwrap_or(NonZeroUsize::new(100).unwrap());
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
        }
    }

    /// Execute a compiled JavaScript rule with the provided facts and metadata
    pub fn execute(
        &self,
        compiled_js: &str,
        facts: &serde_json::Value,
        metadata: &serde_json::Value,
    ) -> Result<ExecutionResult, ExecutorError> {
        let start = Instant::now();

        // Create QuickJS runtime and context
        let runtime = Runtime::new().map_err(|e| ExecutorError::RuntimeError(e.to_string()))?;
        let context = Context::full(&runtime).map_err(|e| ExecutorError::RuntimeError(e.to_string()))?;

        // Serialize inputs
        let facts_json = serde_json::to_string(facts)?;
        let metadata_json = serde_json::to_string(metadata)?;

        // Execute the script
        let result: Result<String, ExecutorError> = context.with(|ctx| {
            // Load and evaluate the compiled script
            ctx.eval::<(), _>(compiled_js.as_bytes().to_vec())
                .map_err(|e| ExecutorError::RuntimeError(e.to_string()))?;

            // Call the execution entry point
            let call_script = format!(
                r#"__execute('{}', '{}')"#,
                facts_json.replace('\'', "\\'").replace('\n', "\\n"),
                metadata_json.replace('\'', "\\'").replace('\n', "\\n")
            );

            let result: String = ctx
                .eval(call_script.into_bytes())
                .map_err(|e| ExecutorError::RuntimeError(e.to_string()))?;

            Ok(result)
        });

        let result_json = result?;
        let execution_time_ms = start.elapsed().as_millis() as u64;

        // Parse the result
        let results: Vec<RuleResult> = serde_json::from_str(&result_json)?;

        // Aggregate results
        let any_condition_met = results.iter().any(|r| r.condition_met);
        let output_facts: serde_json::Value = if any_condition_met {
            let outputs: Vec<_> = results
                .iter()
                .filter(|r| r.condition_met && r.output.is_some())
                .map(|r| r.output.clone().unwrap())
                .collect();
            
            if outputs.len() == 1 {
                outputs.into_iter().next().unwrap()
            } else {
                serde_json::json!(outputs)
            }
        } else {
            serde_json::Value::Null
        };

        // Check for any errors
        let error = results.iter().find_map(|r| r.error.clone());
        if let Some(err) = error {
            return Ok(ExecutionResult::failure(err, execution_time_ms));
        }

        Ok(ExecutionResult::success(
            any_condition_met,
            output_facts,
            execution_time_ms,
        ))
    }

    /// Cache a compiled script for a rule template
    pub fn cache_script(&self, template_id: &str, source: String) {
        let mut cache = self.cache.lock();
        cache.put(template_id.to_string(), CompiledScript { source });
    }

    /// Get a cached script
    pub fn get_cached_script(&self, template_id: &str) -> Option<String> {
        let mut cache = self.cache.lock();
        cache.get(template_id).map(|s| s.source.clone())
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock();
        cache.clear();
    }
}

impl Default for RuleExecutor {
    fn default() -> Self {
        Self::new(100)
    }
}

#[derive(Debug, serde::Deserialize)]
struct RuleResult {
    #[serde(default)]
    rule: String,
    #[serde(default, rename = "conditionMet")]
    condition_met: bool,
    output: Option<serde_json::Value>,
    error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use policy_hub_compiler::RuleCompiler;

    #[test]
    fn test_execute_simple_rule() {
        let compiler = RuleCompiler::new();
        let executor = RuleExecutor::new(10);

        let source = r#"
            rule("discount-rule")
                .when(function(facts, metadata) { 
                    return facts.total > 100; 
                })
                .then(function(facts, metadata) { 
                    return { discount: 0.1, message: "10% discount applied" }; 
                });
        "#;

        let compiled = compiler.compile(source).expect("Compilation failed");

        let facts = serde_json::json!({ "total": 150 });
        let metadata = serde_json::json!({});

        let result = executor.execute(&compiled, &facts, &metadata).expect("Execution failed");

        assert!(result.success);
        assert!(result.condition_met);
        assert_eq!(result.output_facts["discount"], 0.1);
    }

    #[test]
    fn test_execute_condition_not_met() {
        let compiler = RuleCompiler::new();
        let executor = RuleExecutor::new(10);

        let source = r#"
            rule("discount-rule")
                .when(function(facts, metadata) { 
                    return facts.total > 100; 
                })
                .then(function(facts, metadata) { 
                    return { discount: 0.1 }; 
                });
        "#;

        let compiled = compiler.compile(source).expect("Compilation failed");

        let facts = serde_json::json!({ "total": 50 });
        let metadata = serde_json::json!({});

        let result = executor.execute(&compiled, &facts, &metadata).expect("Execution failed");

        assert!(result.success);
        assert!(!result.condition_met);
    }
}
