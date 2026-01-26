Feature: Rule Template Management
  As an API consumer
  I want to manage rule templates
  So that I can define reusable business rules

  @create
  Scenario: Create a new rule template
    Given the API is available at "http://localhost:8080"
    When I POST to "/api/rule-templates" with:
      """
      {
        "name": "bdd-discount-rule",
        "source": "rule(\"discount\").when(f => f.amount > 100).then(f => ({discount: f.amount * 0.1}))"
      }
      """
    Then the response status should be 201
    And the response should contain "bdd-discount-rule"
    And the response field "version" should be 1
    And the response field "compiled_js" should be null

  @version
  Scenario: Create a new version of existing template
    Given the API is available at "http://localhost:8080"
    And a rule template "version-test-template" exists
    When I POST to "/api/rule-templates" with:
      """
      {
        "name": "version-test-template",
        "source": "rule(\"v2\").when(f => true).then(f => ({version: 2}))"
      }
      """
    Then the response status should be 201
    And the response field "version" should be 2

  @list
  Scenario: List all rule templates
    Given the API is available at "http://localhost:8080"
    And a rule template "list-template-alpha" exists
    And a rule template "list-template-beta" exists
    When I GET "/api/rule-templates"
    Then the response status should be 200
    And the response should be a list
