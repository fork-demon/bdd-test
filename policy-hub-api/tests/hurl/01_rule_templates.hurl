# Rule Template API Tests

# Test: Create a new rule template
POST http://localhost:8080/api/rule-templates
Content-Type: application/json
{
  "name": "hurl-test-template-{{timestamp}}",
  "source": "rule(\"test\").when(f => f.value > 10).then(f => ({result: f.value * 2}))"
}

HTTP 201
[Asserts]
jsonpath "$.name" startsWith "hurl-test-template"
jsonpath "$.version" == 1
jsonpath "$.compiled_js" == null
jsonpath "$.source" contains "when"

[Captures]
template_id: jsonpath "$.id"
template_name: jsonpath "$.name"


# Test: Get template by ID
GET http://localhost:8080/api/rule-templates/{{template_id}}

HTTP 200
[Asserts]
jsonpath "$.id" == "{{template_id}}"
jsonpath "$.name" == "{{template_name}}"


# Test: List all templates
GET http://localhost:8080/api/rule-templates

HTTP 200
[Asserts]
jsonpath "$" isCollection


# Test: Create new version of same template
POST http://localhost:8080/api/rule-templates
Content-Type: application/json
{
  "name": "{{template_name}}",
  "source": "rule(\"v2\").when(f => f.value > 20).then(f => ({result: f.value * 3}))"
}

HTTP 201
[Asserts]
jsonpath "$.name" == "{{template_name}}"
jsonpath "$.version" == 2


# Test: Get template versions
GET http://localhost:8080/api/rule-templates/{{template_name}}/versions

HTTP 200
[Asserts]
jsonpath "$" isCollection
jsonpath "$" count >= 2
