"""
Step definitions for pytest-bdd.
All steps are pure HTTP-based - no domain knowledge required.
"""
import json
import requests
from pytest_bdd import given, when, then, parsers


# ==================== GIVEN Steps ====================

@given(parsers.parse('the API is available at "{url}"'))
def api_available(context, url):
    """Verify API is running and set base URL."""
    context["base_url"] = url
    resp = requests.get(f"{url}/health")
    assert resp.status_code == 200, f"API not available at {url}"


@given(parsers.parse('a rule template "{name}" exists'))
def template_exists(context, name):
    """Create a rule template with default source."""
    source = 'rule("default").when(f => true).then(f => ({result: "ok"}))'
    _create_template(context, name, source)


@given(parsers.parse('a rule template "{name}" exists with source:'))
def template_exists_with_source(context, name, docstring):
    """Create a rule template with specified source."""
    _create_template(context, name, docstring.strip())


@given(parsers.parse('a policy "{name}" exists using template "{template_name}"'))
def policy_exists(context, name, template_name):
    """Create a policy using specified template."""
    template_id = context["template_ids"].get(template_name)
    assert template_id, f"Template '{template_name}' not found"
    
    payload = {
        "name": name,
        "rule_template_id": template_id,
        "metadata": {}
    }
    resp = requests.post(
        f"{context['base_url']}/api/policies",
        json=payload
    )
    assert resp.status_code == 201, f"Failed to create policy: {resp.text}"
    data = resp.json()
    context["policy_ids"][name] = data["id"]


# ==================== WHEN Steps ====================

@when(parsers.parse('I POST to "{endpoint}" with:'))
def post_to_endpoint(context, endpoint, docstring):
    """Make a POST request with JSON body."""
    payload = json.loads(docstring)
    url = f"{context['base_url']}{endpoint}"
    resp = requests.post(url, json=payload)
    context["last_response"] = resp
    
    # Store IDs for templates/policies
    if resp.status_code in (200, 201):
        data = resp.json()
        if "id" in data:
            if "rule_template_id" in data:
                context["policy_ids"][data.get("name", "")] = data["id"]
            else:
                context["template_ids"][data.get("name", "")] = data["id"]


@when(parsers.parse('I GET "{endpoint}"'))
def get_endpoint(context, endpoint):
    """Make a GET request."""
    url = f"{context['base_url']}{endpoint}"
    resp = requests.get(url)
    context["last_response"] = resp


@when(parsers.parse('I execute policy "{name}" with facts:'))
def execute_policy(context, name, docstring):
    """Execute a policy with given facts."""
    policy_id = context["policy_ids"].get(name)
    assert policy_id, f"Policy '{name}' not found"
    
    facts = json.loads(docstring)
    payload = {
        "policy_id": policy_id,
        "facts": facts
    }
    url = f"{context['base_url']}/api/execute"
    resp = requests.post(url, json=payload)
    context["last_response"] = resp


# ==================== THEN Steps ====================

@then(parsers.parse('the response status should be {status:d}'))
def response_status(context, status):
    """Assert response status code."""
    resp = context["last_response"]
    assert resp.status_code == status, f"Expected {status}, got {resp.status_code}: {resp.text}"


@then(parsers.parse('the response should contain "{text}"'))
def response_contains(context, text):
    """Assert response body contains text."""
    resp = context["last_response"]
    assert text in resp.text, f"Response does not contain '{text}': {resp.text}"


@then(parsers.parse('the response field "{field}" should be {value:d}'))
def response_field_int(context, field, value):
    """Assert integer field value."""
    data = context["last_response"].json()
    assert data.get(field) == value, f"Field '{field}' is {data.get(field)}, expected {value}"


@then(parsers.parse('the response field "{field}" should be null'))
def response_field_null(context, field):
    """Assert field is null."""
    data = context["last_response"].json()
    assert data.get(field) is None, f"Field '{field}' is not null: {data.get(field)}"


@then("the response should be a list")
def response_is_list(context):
    """Assert response is a list."""
    data = context["last_response"].json()
    assert isinstance(data, list), f"Response is not a list: {type(data)}"


@then("the execution should succeed")
def execution_succeeds(context):
    """Assert execution succeeded."""
    data = context["last_response"].json()
    assert data.get("success") is True, f"Execution failed: {data}"


@then("the condition should be met")
def condition_met(context):
    """Assert condition was met."""
    data = context["last_response"].json()
    assert data.get("condition_met") is True, f"Condition not met: {data}"


@then("the condition should NOT be met")
def condition_not_met(context):
    """Assert condition was not met."""
    data = context["last_response"].json()
    assert data.get("condition_met") is False, f"Condition unexpectedly met: {data}"


@then(parsers.parse('the output field "{field}" should be {value:d}'))
def output_field_value(context, field, value):
    """Assert output_facts field value."""
    data = context["last_response"].json()
    output = data.get("output_facts", {})
    assert output.get(field) == value, f"Output field '{field}' is {output.get(field)}, expected {value}"


# ==================== Helpers ====================

def _create_template(context, name, source):
    """Helper to create a rule template."""
    payload = {"name": name, "source": source}
    resp = requests.post(
        f"{context['base_url']}/api/rule-templates",
        json=payload
    )
    assert resp.status_code == 201, f"Failed to create template: {resp.text}"
    data = resp.json()
    context["template_ids"][name] = data["id"]
