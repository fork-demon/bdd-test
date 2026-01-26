"""
pytest-bdd fixtures and configuration
Includes infrastructure support for Couchbase, environment variables, etc.
"""
import os
import pytest
import requests

# Environment configuration
API_BASE_URL = os.environ.get("API_BASE_URL", "http://localhost:8080")
CB_HOST = os.environ.get("CB_HOST", "couchbase://localhost")
CB_USER = os.environ.get("CB_USER", "Administrator")
CB_PASSWORD = os.environ.get("CB_PASSWORD", "password")
CB_BUCKET = os.environ.get("CB_BUCKET", "policy-hub")


@pytest.fixture(scope="session")
def api_base_url():
    """Base URL for the API server (configurable via env var)."""
    return API_BASE_URL


@pytest.fixture(scope="session")
def api_client():
    """HTTP client session for API requests."""
    session = requests.Session()
    session.headers.update({"Content-Type": "application/json"})
    return session


@pytest.fixture
def context(api_base_url):
    """Shared context for step data passing."""
    return {
        "base_url": api_base_url,
        "last_response": None,
        "last_response_json": None,
        "template_ids": {},
        "policy_ids": {},
    }


# Optional Couchbase fixtures - only if couchbase SDK is installed
try:
    from couchbase.cluster import Cluster
    from couchbase.options import ClusterOptions
    from couchbase.auth import PasswordAuthenticator
    from datetime import timedelta
    
    COUCHBASE_AVAILABLE = True
    
    @pytest.fixture(scope="session")
    def couchbase_cluster():
        """Connect to Couchbase for test data verification."""
        auth = PasswordAuthenticator(CB_USER, CB_PASSWORD)
        options = ClusterOptions(auth)
        options.apply_profile("wan_development")  # For local dev
        cluster = Cluster(CB_HOST, options)
        cluster.wait_until_ready(timedelta(seconds=5))
        return cluster
    
    @pytest.fixture
    def policy_bucket(couchbase_cluster):
        """Get the policy-hub bucket for data validation."""
        return couchbase_cluster.bucket(CB_BUCKET)
    
    @pytest.fixture
    def policy_collection(policy_bucket):
        """Get the default collection for policy documents."""
        return policy_bucket.default_collection()

except ImportError:
    COUCHBASE_AVAILABLE = False
    
    @pytest.fixture
    def couchbase_cluster():
        pytest.skip("Couchbase SDK not installed - skipping DB validation tests")
    
    @pytest.fixture
    def policy_bucket():
        pytest.skip("Couchbase SDK not installed - skipping DB validation tests")
    
    @pytest.fixture
    def policy_collection():
        pytest.skip("Couchbase SDK not installed - skipping DB validation tests")


# Test lifecycle hooks
@pytest.fixture(autouse=True)
def check_api_available(api_base_url, api_client):
    """Ensure API is available before running tests."""
    try:
        response = api_client.get(f"{api_base_url}/health", timeout=5)
        if response.status_code != 200:
            pytest.skip(f"API not available at {api_base_url}")
    except requests.exceptions.ConnectionError:
        pytest.skip(f"Cannot connect to API at {api_base_url}")
