//! Rule template compiler
//!
//! Compiles TypeScript-like DSL into executable JavaScript with
//! the when/then runtime helpers.

use crate::CompilerError;

/// Runtime JavaScript that provides the when/then DSL
const RUNTIME_JS: &str = r#"
// Policy Hub Runtime - when/then DSL
var __PolicyHub = {
    rules: [],
    
    // Create a new rule builder
    rule: function(name) {
        return {
            name: name,
            condition: null,
            action: null,
            
            when: function(conditionFn) {
                this.condition = conditionFn;
                return this;
            },
            
            then: function(actionFn) {
                this.action = actionFn;
                __PolicyHub.rules.push({
                    name: this.name,
                    condition: this.condition,
                    action: this.action
                });
                return this;
            }
        };
    },
    
    // Execute all registered rules
    execute: function(facts, metadata) {
        var results = [];
        for (var i = 0; i < this.rules.length; i++) {
            var rule = this.rules[i];
            try {
                var conditionMet = rule.condition(facts, metadata);
                if (conditionMet) {
                    var output = rule.action(facts, metadata);
                    results.push({
                        rule: rule.name,
                        conditionMet: true,
                        output: output
                    });
                } else {
                    results.push({
                        rule: rule.name,
                        conditionMet: false,
                        output: null
                    });
                }
            } catch (e) {
                results.push({
                    rule: rule.name,
                    error: e.toString()
                });
            }
        }
        return results;
    },
    
    // Reset rules (for reuse)
    reset: function() {
        this.rules = [];
    }
};

// Expose rule function globally
function rule(name) {
    return __PolicyHub.rule(name);
}

// Expose when/then for standalone usage
function when(conditionFn) {
    return {
        then: function(actionFn) {
            return {
                condition: conditionFn,
                action: actionFn,
                evaluate: function(facts, metadata) {
                    var conditionMet = conditionFn(facts, metadata);
                    if (conditionMet) {
                        return {
                            conditionMet: true,
                            output: actionFn(facts, metadata)
                        };
                    }
                    return { conditionMet: false, output: null };
                }
            };
        }
    };
}
"#;

/// Compiler for rule templates
pub struct RuleCompiler;

impl RuleCompiler {
    pub fn new() -> Self {
        Self
    }

    /// Compile a TypeScript-like source into executable JavaScript
    /// 
    /// Note: This is a simplified compiler that handles the basic DSL.
    /// For full TypeScript support, we would integrate swc or similar.
    pub fn compile(&self, source: &str) -> Result<String, CompilerError> {
        // Validate basic structure
        if !source.contains("when") && !source.contains("rule") {
            return Err(CompilerError::InvalidRuleStructure(
                "Source must contain 'when' or 'rule' definitions".to_string(),
            ));
        }

        // For now, we accept JavaScript-compatible syntax directly
        // In a full implementation, we would transpile TypeScript → JavaScript
        let compiled = format!(
            r#"
// === Policy Hub Runtime ===
{}

// === User Rule Definition ===
{}

// === Execution Entry Point ===
function __execute(factsJson, metadataJson) {{
    var facts = JSON.parse(factsJson);
    var metadata = JSON.parse(metadataJson);
    var results = __PolicyHub.execute(facts, metadata);
    return JSON.stringify(results);
}}
"#,
            RUNTIME_JS, source
        );

        Ok(compiled)
    }

    /// Validate rule source without compiling
    pub fn validate(&self, source: &str) -> Result<(), CompilerError> {
        // Basic validation
        if source.trim().is_empty() {
            return Err(CompilerError::InvalidRuleStructure(
                "Source cannot be empty".to_string(),
            ));
        }

        // Check for balanced braces
        let open_braces = source.matches('{').count();
        let close_braces = source.matches('}').count();
        if open_braces != close_braces {
            return Err(CompilerError::SyntaxError(format!(
                "Unbalanced braces: {} open, {} close",
                open_braces, close_braces
            )));
        }

        // Check for balanced parentheses
        let open_parens = source.matches('(').count();
        let close_parens = source.matches(')').count();
        if open_parens != close_parens {
            return Err(CompilerError::SyntaxError(format!(
                "Unbalanced parentheses: {} open, {} close",
                open_parens, close_parens
            )));
        }

        Ok(())
    }
}

impl Default for RuleCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_rule() {
        let compiler = RuleCompiler::new();
        let source = r#"
            rule("discount-rule")
                .when(function(facts) { return facts.total > 100; })
                .then(function(facts) { return { discount: 0.1 }; });
        "#;

        let result = compiler.compile(source);
        assert!(result.is_ok());
        let compiled = result.unwrap();
        assert!(compiled.contains("__PolicyHub"));
        assert!(compiled.contains("discount-rule"));
    }

    #[test]
    fn test_validate_empty_source() {
        let compiler = RuleCompiler::new();
        let result = compiler.validate("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_unbalanced_braces() {
        let compiler = RuleCompiler::new();
        let result = compiler.validate("function() { when(true) ");
        assert!(result.is_err());
    }
}
