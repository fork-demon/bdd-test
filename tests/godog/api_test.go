package main

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"testing"

	"github.com/cucumber/godog"
)

// API Context
type apiContext struct {
	BaseURL      string
	Resp         *http.Response
	TemplateIDs  map[string]string
	PolicyIDs    map[string]string
	ResponseBody interface{}
}

func (c *apiContext) reset() {
	c.Resp = nil
	c.ResponseBody = nil
}

// Step Definitions

func (c *apiContext) theAPIIsAvailableAt(url string) error {
	c.BaseURL = url
	resp, err := http.Get(url + "/health")
	if err != nil {
		return fmt.Errorf("API check failed: %v", err)
	}
	if resp.StatusCode != 200 {
		return fmt.Errorf("API responded with status %d", resp.StatusCode)
	}
	return nil
}

func (c *apiContext) aRuleTemplateExists(name string) error {
	source := `rule("default").when(f => true).then(f => ({result: "ok"}))`
	return c.createTemplate(name, source)
}

func (c *apiContext) aRuleTemplateExistsWithSource(name, source string) error {
	return c.createTemplate(name, source)
}

func (c *apiContext) createTemplate(name, source string) error {
	payload := map[string]string{
		"name":   name,
		"source": strings.TrimSpace(source),
	}
	return c.sendPostRequest("/api/rule-templates", payload)
}

func (c *apiContext) aPolicyExists(name, templateName string) error {
	templateID, ok := c.TemplateIDs[templateName]
	if !ok {
		return fmt.Errorf("template '%s' not found", templateName)
	}

	payload := map[string]interface{}{
		"name":             name,
		"rule_template_id": templateID,
		"metadata":         map[string]interface{}{},
	}
	return c.sendPostRequest("/api/policies", payload)
}

func (c *apiContext) iPostToWith(endpoint string, docstring *godog.DocString) error {
	var payload interface{}
	if err := json.Unmarshal([]byte(docstring.Content), &payload); err != nil {
		return err
	}
	return c.sendPostRequest(endpoint, payload)
}

func (c *apiContext) iGet(endpoint string) error {
	resp, err := http.Get(c.BaseURL + endpoint)
	if err != nil {
		return err
	}
	c.Resp = resp
	return c.parseBody()
}

func (c *apiContext) iExecutePolicyWithFacts(name string, docstring *godog.DocString) error {
	policyID, ok := c.PolicyIDs[name]
	if !ok {
		return fmt.Errorf("policy '%s' not found", name)
	}

	var facts interface{}
	if err := json.Unmarshal([]byte(docstring.Content), &facts); err != nil {
		return err
	}

	payload := map[string]interface{}{
		"policy_id": policyID,
		"facts":     facts,
	}
	return c.sendPostRequest("/api/execute", payload)
}

// Helpers

func (c *apiContext) sendPostRequest(endpoint string, payload interface{}) error {
	body, _ := json.Marshal(payload)
	resp, err := http.Post(c.BaseURL+endpoint, "application/json", strings.NewReader(string(body)))
	if err != nil {
		return err
	}
	c.Resp = resp
	if err := c.parseBody(); err != nil {
		return err
	}

	// Store IDs if present
	bodyMap, ok := c.ResponseBody.(map[string]interface{})
	if ok {
		if id, ok := bodyMap["id"].(string); ok {
			// Try to get name from payload - handle both map types
			var name string
			if payloadMap, ok := payload.(map[string]interface{}); ok {
				if n, ok := payloadMap["name"].(string); ok {
					name = n
				}
			} else if payloadMap, ok := payload.(map[string]string); ok {
				name = payloadMap["name"]
			}
			if name != "" {
				if strings.Contains(endpoint, "policies") {
					c.PolicyIDs[name] = id
				} else {
					c.TemplateIDs[name] = id
				}
			}
		}
	}
	return nil
}

func (c *apiContext) parseBody() error {
	defer c.Resp.Body.Close()
	return json.NewDecoder(c.Resp.Body).Decode(&c.ResponseBody)
}

// Assertions

func (c *apiContext) theResponseStatusShouldBe(code int) error {
	if c.Resp.StatusCode != code {
		return fmt.Errorf("expected status %d, got %d", code, c.Resp.StatusCode)
	}
	return nil
}

func (c *apiContext) theResponseShouldContain(text string) error {
	bodyBytes, _ := json.Marshal(c.ResponseBody)
	if !strings.Contains(string(bodyBytes), text) {
		return fmt.Errorf("response body does not contain '%s'", text)
	}
	return nil
}

func (c *apiContext) theResponseFieldShouldBe(field string, value int) error {
	bodyMap, ok := c.ResponseBody.(map[string]interface{})
	if !ok {
		return fmt.Errorf("response is not an object")
	}
	val, ok := bodyMap[field].(float64) // JSON numbers are float64
	if !ok {
		return fmt.Errorf("field '%s' not found or not a number", field)
	}
	if int(val) != value {
		return fmt.Errorf("expected field '%s' to be %d, got %d", field, value, int(val))
	}
	return nil
}

func (c *apiContext) theOutputFieldShouldBe(field string, value int) error {
	bodyMap, ok := c.ResponseBody.(map[string]interface{})
	if !ok {
		return fmt.Errorf("response is not an object")
	}
	output, ok := bodyMap["output_facts"].(map[string]interface{})
	if !ok {
		return fmt.Errorf("output_facts not found")
	}
	val, ok := output[field].(float64)
	if !ok {
		return fmt.Errorf("output field '%s' not found or not a number", field)
	}
	if int(val) != value {
		return fmt.Errorf("expected output field '%s' to be %d, got %d", field, value, int(val))
	}
	return nil
}

func (c *apiContext) theExecutionShouldSucceed() error {
	bodyMap, ok := c.ResponseBody.(map[string]interface{})
	if !ok {
		return fmt.Errorf("response is not an object")
	}
	if success, ok := bodyMap["success"].(bool); !ok || !success {
		return fmt.Errorf("execution failed: %v", c.ResponseBody)
	}
	return nil
}

func (c *apiContext) theConditionShouldBeMet() error {
	bodyMap, ok := c.ResponseBody.(map[string]interface{})
	if !ok {
		return fmt.Errorf("response is not an object")
	}
	if met, ok := bodyMap["condition_met"].(bool); !ok || !met {
		return fmt.Errorf("condition not met: %v", c.ResponseBody)
	}
	return nil
}

func (c *apiContext) theConditionShouldNotBeMet() error {
	bodyMap, ok := c.ResponseBody.(map[string]interface{})
	if !ok {
		return fmt.Errorf("response is not an object")
	}
	if met, ok := bodyMap["condition_met"].(bool); !ok || met {
		return fmt.Errorf("condition unexpectedly met: %v", c.ResponseBody)
	}
	return nil
}

func (c *apiContext) theResponseFieldShouldBeNull(field string) error {
	bodyMap, ok := c.ResponseBody.(map[string]interface{})
	if !ok {
		return fmt.Errorf("response is not an object")
	}
	val, ok := bodyMap[field]
	if !ok {
		return nil
	}
	if val != nil {
		return fmt.Errorf("expected field '%s' to be null, got %v", field, val)
	}
	return nil
}

func (c *apiContext) theResponseShouldBeAList() error {
	_, ok := c.ResponseBody.([]interface{})
	if !ok {
		return fmt.Errorf("response is not a list, got %T", c.ResponseBody)
	}
	return nil
}

// Test Runner

func TestFeatures(t *testing.T) {
	suite := godog.TestSuite{
		ScenarioInitializer: InitializeScenario,
		Options: &godog.Options{
			Format:   "pretty",
			Paths:    []string{"features"},
			TestingT: t,
		},
	}

	if suite.Run() != 0 {
		t.Fatal("non-zero status returned, failed to run feature tests")
	}
}

func InitializeScenario(ctx *godog.ScenarioContext) {
	api := &apiContext{
		TemplateIDs: make(map[string]string),
		PolicyIDs:   make(map[string]string),
	}

	ctx.Before(func(ctx context.Context, sc *godog.Scenario) (context.Context, error) {
		api.reset()
		return ctx, nil
	})

	ctx.Step(`^the API is available at "([^"]*)"$`, api.theAPIIsAvailableAt)
	ctx.Step(`^a rule template "([^"]*)" exists$`, api.aRuleTemplateExists)
	ctx.Step(`^a rule template "([^"]*)" exists with source:$`, api.aRuleTemplateExistsWithSource)
	ctx.Step(`^a policy "([^"]*)" exists using template "([^"]*)"$`, api.aPolicyExists)
	ctx.Step(`^I POST to "([^"]*)" with:$`, api.iPostToWith)
	ctx.Step(`^I GET "([^"]*)"$`, api.iGet)
	ctx.Step(`^I execute policy "([^"]*)" with facts:$`, api.iExecutePolicyWithFacts)
	ctx.Step(`^the response status should be (\d+)$`, api.theResponseStatusShouldBe)
	ctx.Step(`^the response should contain "([^"]*)"$`, api.theResponseShouldContain)
	ctx.Step(`^the response field "([^"]*)" should be (\d+)$`, api.theResponseFieldShouldBe)
	ctx.Step(`^the output field "([^"]*)" should be (\d+)$`, api.theOutputFieldShouldBe)
	ctx.Step(`^the execution should succeed$`, api.theExecutionShouldSucceed)
	ctx.Step(`^the condition should be met$`, api.theConditionShouldBeMet)
	ctx.Step(`^the condition should NOT be met$`, api.theConditionShouldNotBeMet)
	ctx.Step(`^the response field "([^"]*)" should be null$`, api.theResponseFieldShouldBeNull)
	ctx.Step(`^the response should be a list$`, api.theResponseShouldBeAList)
}
