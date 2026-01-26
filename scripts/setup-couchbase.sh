#!/bin/bash
# Setup script for Couchbase bucket and indexes

set -e

COUCHBASE_HOST="localhost"
COUCHBASE_USER="admin"
COUCHBASE_PASS="password123"
BUCKET_NAME="policy-hub"

echo "Waiting for Couchbase to be ready..."
until curl -s http://$COUCHBASE_HOST:8091/pools > /dev/null 2>&1; do
    echo "Waiting for Couchbase..."
    sleep 2
done
sleep 5

echo "Initializing cluster..."
# Initialize cluster with memory quotas
curl -s -X POST http://$COUCHBASE_HOST:8091/pools/default \
  -d memoryQuota=512 \
  -d indexMemoryQuota=256 \
  -d ftsMemoryQuota=256

# Setup services (kv, n1ql, index)
curl -s -X POST http://$COUCHBASE_HOST:8091/node/controller/setupServices \
  -d services=kv%2Cn1ql%2Cindex

# Set admin credentials
curl -s -X POST http://$COUCHBASE_HOST:8091/settings/web \
  -d password=$COUCHBASE_PASS \
  -d username=$COUCHBASE_USER \
  -d port=8091

echo "Setting indexer storage mode..."
# Set indexer storage mode to forestdb (required for Community Edition)
curl -s -X POST http://$COUCHBASE_HOST:8091/settings/indexes \
  -u $COUCHBASE_USER:$COUCHBASE_PASS \
  -d 'storageMode=forestdb'

echo "Creating $BUCKET_NAME bucket..."
curl -s -X POST http://$COUCHBASE_HOST:8091/pools/default/buckets \
  -u $COUCHBASE_USER:$COUCHBASE_PASS \
  -d name=$BUCKET_NAME \
  -d ramQuotaMB=256 \
  -d bucketType=couchbase

echo "Waiting for bucket to be ready..."
sleep 10

echo "Creating primary index..."
curl -s -X POST http://$COUCHBASE_HOST:8093/query/service \
  -u $COUCHBASE_USER:$COUCHBASE_PASS \
  -d "statement=CREATE PRIMARY INDEX ON \`$BUCKET_NAME\`" || echo "Primary index may already exist"

sleep 2

echo "Creating rule template indexes..."
curl -s -X POST http://$COUCHBASE_HOST:8093/query/service \
  -u $COUCHBASE_USER:$COUCHBASE_PASS \
  -d "statement=CREATE INDEX idx_rule_template_name ON \`$BUCKET_NAME\`(name) WHERE type = 'rule_template'" || echo "Index may already exist"

curl -s -X POST http://$COUCHBASE_HOST:8093/query/service \
  -u $COUCHBASE_USER:$COUCHBASE_PASS \
  -d "statement=CREATE INDEX idx_rule_template_latest ON \`$BUCKET_NAME\`(name, is_latest) WHERE type = 'rule_template'" || echo "Index may already exist"

echo "Creating policy indexes..."
curl -s -X POST http://$COUCHBASE_HOST:8093/query/service \
  -u $COUCHBASE_USER:$COUCHBASE_PASS \
  -d "statement=CREATE INDEX idx_policy_rule_template ON \`$BUCKET_NAME\`(rule_template_id) WHERE type = 'policy'" || echo "Index may already exist"

echo ""
echo "========================================"
echo "Couchbase setup complete!"
echo "========================================"
echo ""
echo "Admin Console: http://localhost:8091"
echo "Username: $COUCHBASE_USER"
echo "Password: $COUCHBASE_PASS"
echo "Bucket: $BUCKET_NAME"
echo ""
