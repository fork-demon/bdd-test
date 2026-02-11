Feature: Rule Template Management
  As an API consumer
  I want to manage rule templates
  So that I can define reusable business rules

  Background:
    Given the API server is running

  @create
  Scenario: Create a new rule template
    When I create a rule template named "test-discount-rule" with source "rule(\"test\").when(f => f.amount > 100).then(f => ({ok:1}))"
    Then the response status should be 201
    And the response should contain "test-discount-rule"
    And the template should have version 1

  @version
  Scenario: Create a new version of existing template
    Given a rule template "version-test" exists with source:
      """
      rule("v1").when(f => true).then(f => ({v:1}))
      """
    When I create a rule template named "version-test" with source "rule(\"v2\").when(f => true).then(f => ({v:2}))"
    Then the response status should be 201
    And the template should have version 2

  @list
  Scenario: List all rule template names
    Given a rule template "list-test-alpha" exists
    And a rule template "list-test-beta" exists
    When I list all rule templates
    Then the response should contain "list-test-alpha"
    And the response should contain "list-test-beta"
