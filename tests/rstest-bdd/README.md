# rstest-bdd Tests

BDD tests for Policy Hub using [rstest-bdd](https://docs.rs/rstest-bdd) — a native Rust BDD framework built on `rstest`.

## Why rstest-bdd?

- **Gherkin `.feature` files** with `Given`/`When`/`Then` (like cucumber-rs)
- **Attribute macros**: `#[given]`, `#[when]`, `#[then]`, `#[scenario]`
- **`cargo test` native** — no separate runner needed
- **Compile-time validation** — missing step definitions fail at build time
- **rstest fixtures** — share fixtures between unit, integration, and BDD tests

## Structure

```
rstest-bdd/
├── features/
│   └── execution.feature    # Gherkin scenarios
├── steps.rs                 # Step definitions + scenario bindings
└── README.md
```

## Running

```bash
# From project root
cargo test --test rstest_bdd_steps -p policy-hub-api

# With verbose output
cargo test --test rstest_bdd_steps -p policy-hub-api -- --nocapture
```

## Key Difference from cucumber-rs

| | cucumber-rs | rstest-bdd |
|---|---|---|
| Runner | Custom main() | cargo test |
| Async | First-class (tokio) | Sync by default |
| Fixtures | World struct | rstest fixtures |
| Step errors | Runtime | Compile-time |
| Maturity | 689★, established | Newer, 0.5.0 |
