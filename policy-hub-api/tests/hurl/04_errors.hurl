# Error Handling Tests

# Test: Create template with invalid source
POST http://localhost:8080/api/rule-templates
Content-Type: application/json
{
  "name": "invalid-template",
  "source": "this is not valid javascript {{{{"
}

HTTP 400


# Test: Create policy with non-existent template
POST http://localhost:8080/api/policies
Content-Type: application/json
{
  "name": "orphan-policy",
  "rule_template_id": "00000000-0000-0000-0000-000000000000",
  "metadata": {}
}

HTTP 404


# Test: Execute non-existent policy
POST http://localhost:8080/api/execute
Content-Type: application/json
{
  "policy_id": "00000000-0000-0000-0000-000000000000",
  "facts": {"amount": 100}
}

HTTP 404


# Test: Get non-existent template
GET http://localhost:8080/api/rule-templates/00000000-0000-0000-0000-000000000000

HTTP 404


# Test: Get non-existent policy
GET http://localhost:8080/api/policies/00000000-0000-0000-0000-000000000000

HTTP 404
