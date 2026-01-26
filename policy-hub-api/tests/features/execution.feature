Feature: Policy Execution
  As an API consumer
  I want to execute policies with input facts
  So that I can get computed output based on business rules

  Background:
    Given the API server is running
    And a rule template "exec-discount" exists with source:
      """
      rule("discount").when(f => f.amount > 100).then(f => ({ discount: f.amount * 0.1 }))
      """

  @execute @condition-met
  Scenario: Execute policy when condition is met
    Given a policy "exec-test-policy" exists using template "exec-discount"
    When I execute policy "exec-test-policy" with amount 150
    Then the execution should succeed
    And the condition should be met
    And the discount should be 15

  @execute @condition-not-met
  Scenario: Execute policy when condition is NOT met
    Given a policy "exec-test-policy-2" exists using template "exec-discount"
    When I execute policy "exec-test-policy-2" with amount 50
    Then the execution should succeed
    And the condition should NOT be met

  @execute @boundary
  Scenario: Execute policy at boundary value
    Given a policy "boundary-test" exists using template "exec-discount"
    When I execute policy "boundary-test" with amount 100
    Then the execution should succeed
    And the condition should NOT be met

  @execute @above-boundary
  Scenario: Execute policy above boundary value
    Given a policy "above-boundary-test" exists using template "exec-discount"
    When I execute policy "above-boundary-test" with amount 101
    Then the execution should succeed
    And the condition should be met
