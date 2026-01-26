# Policy API Tests

# Setup: Create a template first
POST http://localhost:8080/api/rule-templates
Content-Type: application/json
{
  "name": "policy-test-template-{{timestamp}}",
  "source": "rule(\"policy-test\").when(f => f.enabled).then(f => ({status: \"active\"}))"
}

HTTP 201
[Captures]
template_id: jsonpath "$.id"


# Test: Create a policy
POST http://localhost:8080/api/policies
Content-Type: application/json
{
  "name": "hurl-test-policy-{{timestamp}}",
  "rule_template_id": "{{template_id}}",
  "metadata": {
    "region": "US",
    "environment": "test"
  }
}

HTTP 201
[Asserts]
jsonpath "$.name" startsWith "hurl-test-policy"
jsonpath "$.rule_template_id" == "{{template_id}}"
jsonpath "$.is_active" == true
jsonpath "$.metadata.region" == "US"

[Captures]
policy_id: jsonpath "$.id"
policy_name: jsonpath "$.name"


# Test: Get policy by ID
GET http://localhost:8080/api/policies/{{policy_id}}

HTTP 200
[Asserts]
jsonpath "$.id" == "{{policy_id}}"
jsonpath "$.name" == "{{policy_name}}"


# Test: List all policies
GET http://localhost:8080/api/policies

HTTP 200
[Asserts]
jsonpath "$" isCollection
