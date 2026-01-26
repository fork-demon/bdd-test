# Policy Hub

A dynamic policy engine built in Rust with TypeScript rule templates for high-performance rule evaluation.

## Features

- 🚀 **TypeScript DSL** - Define rules using intuitive `when().then()` syntax
- ⚡ **Fast Execution** - QuickJS-based JavaScript runtime with LRU caching
- 📦 **Version Control** - Multiple versions of rule templates with policy binding
- 🔌 **REST API** - Clean HTTP API for rule management and execution
- 🗄️ **Flexible Storage** - In-memory (dev) or Couchbase (production)

## Quick Start

### Build
```bash
cargo build --release
```

### Run
```bash
cargo run --package policy-hub-api
```

### Test
```bash
cargo test
```

## API Usage

### 1. Create a Rule Template

```bash
curl -X POST http://localhost:8080/api/rule-templates \
  -H "Content-Type: application/json" \
  -d '{
    "name": "loyalty-discount",
    "source": "rule(\"loyalty-discount\").when(function(facts) { return facts.customer.tier === \"GOLD\" && facts.cart.total > 100; }).then(function(facts) { return { discount: 0.15, message: \"15% Gold member discount!\" }; });"
  }'
```

### 2. Create a Policy

```bash
curl -X POST http://localhost:8080/api/policies \
  -H "Content-Type: application/json" \
  -d '{
    "name": "holiday-promo-2026",
    "rule_template_id": "<TEMPLATE_ID>",
    "metadata": {
      "campaign": "holiday-2026",
      "created_by": "marketing"
    }
  }'
```

### 3. Execute the Policy

```bash
curl -X POST http://localhost:8080/api/execute \
  -H "Content-Type: application/json" \
  -d '{
    "policy_id": "<POLICY_ID>",
    "facts": {
      "customer": { "name": "John", "tier": "GOLD" },
      "cart": { "total": 150, "items": ["item1", "item2"] }
    }
  }'
```

## Rule Template DSL

Rules are defined using a simple JavaScript DSL:

```javascript
// Simple condition
rule("my-rule")
  .when(function(facts, metadata) {
    return facts.value > 100;
  })
  .then(function(facts, metadata) {
    return { result: "approved" };
  });

// Multiple rules in one template
rule("rule-1")
  .when(facts => facts.type === "A")
  .then(facts => ({ handler: "process-a" }));

rule("rule-2")
  .when(facts => facts.type === "B")
  .then(facts => ({ handler: "process-b" }));
```

## Project Structure

```
policy-hub/
├── Cargo.toml              # Workspace configuration
├── policy-hub-core/        # Domain models
├── policy-hub-storage/     # Persistence layer
├── policy-hub-compiler/    # TypeScript to JS compilation
├── policy-hub-executor/    # QuickJS runtime
└── policy-hub-api/         # REST API server
```

## Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `HOST` | `127.0.0.1` | Server host |
| `PORT` | `8080` | Server port |
| `RUST_LOG` | `info` | Log level |

## License

MIT
