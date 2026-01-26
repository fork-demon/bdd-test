# Policy Execution E2E Tests

# Setup: Create discount template
POST http://localhost:8080/api/rule-templates
Content-Type: application/json
{
  "name": "e2e-discount-template-{{timestamp}}",
  "source": "rule(\"discount\").when(f => f.amount > 100).then(f => ({ discount: f.amount * 0.1, applied: true }))"
}

HTTP 201
[Captures]
template_id: jsonpath "$.id"


# Setup: Create policy
POST http://localhost:8080/api/policies
Content-Type: application/json
{
  "name": "e2e-discount-policy-{{timestamp}}",
  "rule_template_id": "{{template_id}}",
  "metadata": {"tier": "premium"}
}

HTTP 201
[Captures]
policy_id: jsonpath "$.id"


# Test: Execute policy - condition MET (amount > 100)
POST http://localhost:8080/api/execute
Content-Type: application/json
{
  "policy_id": "{{policy_id}}",
  "facts": {"amount": 150}
}

HTTP 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.condition_met" == true
jsonpath "$.output_facts.discount" == 15
jsonpath "$.output_facts.applied" == true
jsonpath "$.execution_time_ms" >= 0


# Test: Execute policy - condition NOT MET (amount <= 100)
POST http://localhost:8080/api/execute
Content-Type: application/json
{
  "policy_id": "{{policy_id}}",
  "facts": {"amount": 50}
}

HTTP 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.condition_met" == false


# Test: Execute policy - boundary value (amount = 100)
POST http://localhost:8080/api/execute
Content-Type: application/json
{
  "policy_id": "{{policy_id}}",
  "facts": {"amount": 100}
}

HTTP 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.condition_met" == false


# Test: Execute policy - just above boundary (amount = 101)
POST http://localhost:8080/api/execute
Content-Type: application/json
{
  "policy_id": "{{policy_id}}",
  "facts": {"amount": 101}
}

HTTP 200
[Asserts]
jsonpath "$.success" == true
jsonpath "$.condition_met" == true
jsonpath "$.output_facts.discount" == 10.1
