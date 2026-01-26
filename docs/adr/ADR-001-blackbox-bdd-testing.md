# ADR-001: Black-Box BDD Testing Strategy for Policy Hub

**Status**: Accepted  
**Date**: 2026-01-19  
**Deciders**: Engineering Team

---

## Context

Policy Hub requires a comprehensive test suite to validate API behavior. The key question was:

> **What is the correct approach for true black-box testing?**

---

## Decision Journey

### Initial Attempt: cucumber-rs (Rust)

We first explored `cucumber-rs` - Rust's native Gherkin framework.

```
Gherkin (.feature)
      ↓
Rust step definitions
      ↓
Rust system under test
```

**Problem Discovered**: The step definitions required domain knowledge:
- Constructing `RuleTemplate` sources
- Understanding internal API contracts
- Sharing types with the system under test

Even using only `reqwest` for HTTP calls, the tests were **gray-box** because:
- They run in the same Rust process context
- They could boot Axum directly
- They share compilation with the system

### The Insight: Gray-Box vs Black-Box

| Characteristic | Gray-Box | True Black-Box |
|---------------|----------|----------------|
| Runs in same process | ✅ Yes | ❌ No |
| Shares language/runtime | ✅ Possible | ❌ Never |
| Can import domain types | ✅ Possible | ❌ Never |
| Only touches public APIs | ⚠️ Partially | ✅ Always |

**Key Realization**:
> If a test is written in Rust and can boot Axum or Wasmtime, it is **not** truly black-box.

### The Correct Architecture

```
Gherkin (spec)
    ↓
Python / JS steps
    ↓
HTTP / CLI / WASM
    ↓
Rust system under test (running separately)
```

This stack is:
- ✅ **Language-agnostic** - Python cannot import Rust types
- ✅ **Boundary-only** - Only HTTP/CLI/WASM is reachable
- ✅ **Deployment-realistic** - Mirrors production usage

---

## Decision

**Adopt Gherkin BDD with Python (pytest-bdd)** for black-box testing.

### Why Gherkin?

| Requirement | Gherkin Answer |
|------------|----------------|
| Business-readable | ✅ Given/When/Then syntax |
| True black-box | ✅ Python cannot access Rust internals |
| Boundary-only | ✅ Steps only call HTTP endpoints |
| Language-agnostic | ✅ Separate from Rust codebase |
| Deployment-realistic | ✅ Tests run against deployed API |

### Why pytest-bdd?

- Most popular Python BDD framework
- Standard Gherkin parser
- Easy HTTP with `requests` library
- Excellent pytest integration

---

## Implementation

### Directory Structure

```
policy-hub/
└── tests/
    └── bdd/                    # Separate from Rust tests
        ├── features/           # Gherkin specs (business language)
        │   ├── rule_templates.feature
        │   └── execution.feature
        ├── steps.py           # HTTP-only step definitions
        ├── conftest.py        # pytest fixtures
        └── pyproject.toml     # Python dependencies
```

### Test Execution Flow

```
1. Start Rust API server (separate process)
   └── STORAGE_TYPE=couchbase cargo run --features couchbase

2. Run Python BDD tests (separate process)
   └── cd tests/bdd && pytest -v

3. Tests communicate only via HTTP
   └── requests.post("http://localhost:8080/api/...")
```

---

## What We Rejected

### ❌ cucumber-rs (Rust Gherkin)

Even though Gherkin syntax is correct, step definitions in Rust:
- Share runtime with system under test
- Can accidentally import domain types
- Not truly language-agnostic

### ❌ Hurl (HTTP Testing Tool)

While great for HTTP testing, Hurl is:
- Not BDD (no Given/When/Then)
- Not business-readable
- Just HTTP assertions, not behavior specs

### ❌ Rust Integration Tests

Traditional `#[test]` in Rust:
- Gray-box by definition
- Boot the system in-process
- Not deployment-realistic

---

## Consequences

### Positive

- ✅ True black-box: Python cannot access Rust internals
- ✅ Business-readable: PM/QA can write Gherkin specs
- ✅ Deployment-realistic: Tests against running server
- ✅ API contract documentation via feature files

### Trade-offs

- ⚠️ Separate language (Python) for tests
- ⚠️ Requires running API server before tests
- ⚠️ Slightly slower than in-process tests

### Mitigations

- Python is ubiquitous and easy to learn
- CI/CD can orchestrate server + tests
- Black-box tests run after unit tests

---

## Summary

> **True black-box tests must run outside the Rust process and touch only HTTP/CLI/WASM.**

Under this definition:
- ✅ Gherkin makes **more** sense, not less
- ✅ Python step definitions are **correct**, not a compromise
- ✅ Separate process is a **feature**, not a bug

The test boundary matches the deployment boundary, which is exactly what black-box testing should achieve.
