"""
Test module for Rule Template scenarios.
"""
import pytest
from pytest_bdd import scenarios
from steps import *  # Import all step definitions

# Load all scenarios from the feature file
scenarios("features/rule_templates.feature")
