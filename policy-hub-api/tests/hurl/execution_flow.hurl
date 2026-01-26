# Test: Create a rule template
POST http://localhost:8080/api/rule-templates
Content-Type: application/json
{
  "name": "discount-rule",
  "source": "rule(\"discount\").when(f => f.amount > 100).then(f => ({ discount: f.amount * 0.1 }))"
}

HTTP 201
[Asserts]
jsonpath "$.name" == "discount-rule"
jsonpath "$.version" == 1
jsonpath "$.compiled_js" == null

[Captures]
template_id: jsonpath "$.id"


# Test: Create a policy using the template
POST http://localhost:8080/api/policies
Content-Type: application/json
{
  "name": "premium-pricing-policy",
  "rule_template_id": "{{template_id}}",
  "metadata": {"region": "US"}
}

HTTP 201
[Asserts]
jsonpath "$.name" == "premium-pricing-policy"
jsonpath "$.is_active" == true

[Captures]
policy_id: jsonpath "$.id"


# Test: Execute policy - condition MET
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


# Test: Execute policy - condition NOT MET
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
