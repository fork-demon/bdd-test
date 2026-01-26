# Policy Hub API Reference

## Rule Templates

### Create Rule Template
`POST /api/rule-templates`

Creates a new rule template (or new version if name exists).

**Request Body:**
```json
{
  "name": "price-promotion",
  "source": "rule('price-promotion').when(facts => facts.total > 100).then(facts => ({ discount: 0.1 }))",
  "schema_version": 1
}
```

**Response:**
```json
{
  "id": "uuid",
  "name": "price-promotion",
  "version": 1,
  "source": "...",
  "created_at": "timestamp",
  "is_latest": true
}
```

### Get Rule Template Versions
`GET /api/rule-templates/name/{name}/versions`

Returns all versions of a specific rule template.

**Response:**
```json
[
  { "id": "uuid", "version": 2, ... },
  { "id": "uuid", "version": 1, ... }
]
```

---

## Policies

### Create Policy
`POST /api/policies`

Creates a policy instance linking to a specific rule template version.

**Request Body:**
```json
{
  "name": "holiday-promo-2024",
  "rule_template_id": "uuid",
  "rule_template_version": 1,
  "metadata": {
    "campagin": "summer-sale",
    "priority": "high"
  }
}
```

**Response:**
```json
{
  "id": "uuid",
  "name": "holiday-promo-2024",
  "rule_template_id": "uuid",
  "created_at": "timestamp",
  ...
}
```

---

## Execution

### Execute Policy
`POST /api/execute`

Executes a policy against provided facts.

**Request Body:**
```json
{
  "policy_id": "uuid",
  "facts": {
    "total": 150,
    "customer_tier": "gold"
  }
}
```

**Response:**
```json
{
  "success": true,
  "condition_met": true,
  "output_facts": {
    "discount": 0.1
  },
  "execution_time_ms": 2
}
```
