"""
Test module for Policy Execution scenarios.
"""
import pytest
from pytest_bdd import scenarios
from steps import *  # Import all step definitions

# Load all scenarios from the feature file
scenarios("features/execution.feature")
