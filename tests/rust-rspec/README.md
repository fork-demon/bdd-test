# rust-rspec Style Tests

This directory contains tests written in RSpec style (`describe`/`it` pattern) 
for comparison with Gherkin-based frameworks.

## Important Note

**rust-rspec is NOT recommended** for new projects because:
- Last updated 5+ years ago (unmaintained)
- No Gherkin support (uses describe/it syntax)
- Limited community support

However, the `describe`/`it` pattern can be implemented using standard Rust 
test modules as shown in `rspec_tests.rs`.

## Running Tests

```bash
# From project root
cargo test --test rspec --features couchbase

# With verbose output
cargo test --test rspec --features couchbase -- --nocapture
```

## Syntax Comparison

### RSpec/describe-it Style (this implementation)
```rust
mod policy_execution {
    #[test]
    fn it_executes_when_condition_is_met() {
        // Setup
        let result = execute_policy(...);
        // Verify
        assert_eq!(result["success"], true);
    }
}
```

### Gherkin Style (cucumber-rs)
```gherkin
Scenario: Execute policy when condition is met
  Given a policy exists
  When I execute with amount 150
  Then the condition should be met
```

## Verdict

The describe/it pattern is just standard Rust tests with organized module names.
No special framework needed. Gherkin (cucumber-rs, pytest-bdd) provides better 
separation between specification and implementation.
