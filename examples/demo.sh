#!/bin/bash
# Example script to demonstrate Policy Hub usage

set -e

BASE_URL="http://localhost:8080"

echo "=== Policy Hub Demo ==="
echo ""

# 1. Create a rule template for loyalty discounts
echo "1. Creating rule template 'loyalty-discount'..."
TEMPLATE_RESPONSE=$(curl -s -X POST "$BASE_URL/api/rule-templates" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "loyalty-discount",
    "source": "rule(\"loyalty-discount\").when(function(facts, metadata) { return facts.customer.tier === \"GOLD\" && facts.cart.total > metadata.min_purchase; }).then(function(facts, metadata) { return { discount: metadata.discount_rate, message: \"Loyalty discount applied for \" + facts.customer.name }; });"
  }')

echo "$TEMPLATE_RESPONSE" | jq .
TEMPLATE_ID=$(echo "$TEMPLATE_RESPONSE" | jq -r '.id')
echo "Template ID: $TEMPLATE_ID"
echo ""

# 2. Create a policy using this template
echo "2. Creating policy 'holiday-promo-2026'..."
POLICY_RESPONSE=$(curl -s -X POST "$BASE_URL/api/policies" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"holiday-promo-2026\",
    \"rule_template_id\": \"$TEMPLATE_ID\",
    \"metadata\": {
      \"min_purchase\": 100,
      \"discount_rate\": 0.15,
      \"campaign\": \"holiday-2026\"
    }
  }")

echo "$POLICY_RESPONSE" | jq .
POLICY_ID=$(echo "$POLICY_RESPONSE" | jq -r '.id')
echo "Policy ID: $POLICY_ID"
echo ""

# 3. Execute the policy with qualifying facts
echo "3. Executing policy with GOLD customer (should get discount)..."
curl -s -X POST "$BASE_URL/api/execute" \
  -H "Content-Type: application/json" \
  -d "{
    \"policy_id\": \"$POLICY_ID\",
    \"facts\": {
      \"customer\": { \"name\": \"Alice\", \"tier\": \"GOLD\" },
      \"cart\": { \"total\": 150 }
    }
  }" | jq .
echo ""

# 4. Execute with non-qualifying facts
echo "4. Executing policy with SILVER customer (should NOT get discount)..."
curl -s -X POST "$BASE_URL/api/execute" \
  -H "Content-Type: application/json" \
  -d "{
    \"policy_id\": \"$POLICY_ID\",
    \"facts\": {
      \"customer\": { \"name\": \"Bob\", \"tier\": \"SILVER\" },
      \"cart\": { \"total\": 200 }
    }
  }" | jq .
echo ""

echo "=== Demo Complete ==="
