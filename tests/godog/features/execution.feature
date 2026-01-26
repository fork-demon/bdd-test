Feature: Policy Execution
  As an API consumer
  I want to execute policies with input facts
  So that I can get computed output based on business rules

  Background:
    Given the API is available at "http://localhost:8080"

  @execute @condition-met
  Scenario: Execute policy when condition is met
    Given a rule template "exec-discount-template" exists with source:
      """
      rule("discount").when(f => f.amount > 100).then(f => ({ discount: f.amount * 0.1 }))
      """
    And a policy "exec-test-policy" exists using template "exec-discount-template"
    When I execute policy "exec-test-policy" with facts:
      """
      {"amount": 150}
      """
    Then the execution should succeed
    And the condition should be met
    And the output field "discount" should be 15

  @execute @condition-not-met
  Scenario: Execute policy when condition is NOT met
    Given a rule template "exec-discount-template-2" exists with source:
      """
      rule("discount").when(f => f.amount > 100).then(f => ({ discount: f.amount * 0.1 }))
      """
    And a policy "exec-test-policy-2" exists using template "exec-discount-template-2"
    When I execute policy "exec-test-policy-2" with facts:
      """
      {"amount": 50}
      """
    Then the execution should succeed
    And the condition should NOT be met

  @execute @boundary
  Scenario: Execute policy at boundary value
    Given a rule template "boundary-template" exists with source:
      """
      rule("boundary").when(f => f.amount > 100).then(f => ({ result: "above" }))
      """
    And a policy "boundary-policy" exists using template "boundary-template"
    When I execute policy "boundary-policy" with facts:
      """
      {"amount": 100}
      """
    Then the execution should succeed
    And the condition should NOT be met
