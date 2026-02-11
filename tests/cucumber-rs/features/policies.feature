Feature: Policy Management
  As an API consumer
  I want to manage policies
  So that I can apply rule templates with specific metadata

  Background:
    Given the API server is running
    And a rule template "policy-test-template" exists

  @create
  Scenario: Create a new policy
    When I create a policy named "my-test-policy" using template "policy-test-template"
    Then the response status should be 201
    And the response should contain "my-test-policy"

  @get
  Scenario: Get policy by ID
    Given a policy "get-test-policy" exists using template "policy-test-template"
    When I get the policy "get-test-policy"
    Then the response status should be 200
    And the response should contain "get-test-policy"

  @list
  Scenario: List all policies
    Given a policy "list-policy-alpha" exists using template "policy-test-template"
    And a policy "list-policy-beta" exists using template "policy-test-template"
    When I list all policies
    Then the response should contain "list-policy-alpha"
    And the response should contain "list-policy-beta"
