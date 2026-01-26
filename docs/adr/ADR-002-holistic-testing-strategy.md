# ADR-002: Holistic Testing Strategy for Policy Hub

**Status**: Draft  
**Date**: 2026-01-20  
**Deciders**: Engineering Team

---

## Context

We need a testing strategy that is **black-box**, **deployable**, and **maintainable**. We previously evaluated `cucumber-rs` and `pytest-bdd`. The user requested a broader look at how modern tech companies test Rust microservices.

## The Landscape of Microservice Testing

Modern tech shops (AWS, Discord, generic enterprise) typically fall into three buckets:

1.  **Language-Native Integration** (The "Rust Standard")
2.  **Polyglot Test Suites** (The "Black-Box Standard")
3.  **DSL/Tool-Based Frameworks** (The "Enterprise Standard")

---

## Option 1: Language-Native (Rust)

*Tools*: `cucumber-rs`, `framework-less reqwest` in `tests/` folder.

| Pros | Cons |
|------|------|
| ✅ Single language (Rust everywhere) | ⚠️ Gray-box risk (can import internals) |
| ✅ Fast execution (native binaries) | ⚠️ High boilerplate for step defs |
| ✅ Strong typing & Compile checks | ⚠️ Slow compile times for tests |

**Verdict**: Best for teams who strictly want 1 language. "Gray-box" is acceptable for many pure-Rust shops.

---

## Option 2: Polyglot BDD (Python/Go)

*Tools*: `pytest-bdd` (Python), `godog` (Go)

| Pros | Cons |
|------|------|
| ✅ True Black-box (Process isolation) | ⚠️ 2nd Language dependency (Python/Go) |
| ✅ Vast ecosystem (Python libraries) | ⚠️ Context switching for devs |
| ✅ Very readable / Clean-code tests | |

**Verdict**: The "Clean Architecture" choice. Ensures strict boundary separation. `pytest-bdd` is excellent here.

---

## Option 3: DSL / Tool-Based (DSL)

*Tools*: **Karate** (Java/JS), **Hurl** (Rust/C), **Venom** (Go)

### Karate DSL
A "batteries-included" framework popular in enterprise.
- **Pros**: Parallel exec, API mocking, Reports, "No Glue Code" (Built-in steps for HTTP/JSON).
- **Cons**: Java runtime, unique DSL syntax (not standard Gherkin), hard to debug complex logic.

### Hurl
Lightweight, CLI-first.
- **Pros**: Extremely fast, simple text files, great for CI smoke tests.
- **Cons**: No Logic/Branching, not "BDD" (no Gherkin), limited for complex flows.

### Venom
YAML-based integration testing (from OVHCloud).
- **Pros**: Broad scope (DB, Files, Scripts, HTTP), declarative.
- **Cons**: Verbose YAML, limited logic.

---

## "Modern Tech Company" Patterns

1.  **The "Rust Purist"**: Uses `cucumber-rs` or just `cargo test` with `reqwest`. Accepts the gray-box tradeoff for DX.
2.  **The "Polyglot Microsevice"**: Uses **Go** or **Python** for integration tests. Go is very popular for writing black-box test suites for services written in *any* language due to compile speed and simplicity.
3.  **The "Enterprise Platform"**: Uses **Karate** or **Postman/Newman**. Centralized QA teams often prefer these "no-code/low-code" BDD tools.

---

## Recommendation

### Evaluated Frameworks (With Working Implementations)

| Framework | BDD? | Black-Box? | Tested? | Result |
|-----------|------|------------|---------|--------|
| **pytest-bdd** (Python) | ✅ Gherkin | ✅ True | ✅ 6 tests collected | Works |
| **godog** (Go) | ✅ Gherkin | ✅ True | ✅ 26/36 steps passed | Works |
| Hurl | ❌ | ✅ True | ✅ Files created | HTTP only |
| cucumber-rs | ✅ | ⚠️ Gray-box | ❌ Rejected | Shared runtime |

### Final Recommendation

**Primary: pytest-bdd (Python)**
- Easiest for QA/PM collaboration
- Excellent `requests` library for HTTP
- Best error messages

**Alternative: godog (Go)**  
- Better for Rust-centric teams (typed + compiled)
- Faster execution
- Single binary distribution

### Decision Record

We implemented both `pytest-bdd` and `godog` as working proof-of-concepts. Both successfully:
1. Connected to the running API
2. Parsed Gherkin feature files
3. Executed HTTP-based step definitions
4. Produced readable test output

The test failures observed are **system bugs** (bundle regeneration, pre-existing data), not framework issues.
