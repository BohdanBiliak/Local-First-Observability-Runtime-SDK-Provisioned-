#!/bin/bash

# Message Classification & Versioning Test Script
# Demonstrates error handling and version enforcement

set -e

API_URL="${API_URL:-http://localhost:3000}"
QUEUE="${QUEUE:-telemetry}"

echo "🧪 Testing Message Classification & Versioning"
echo "=============================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test 1: Successful message processing
echo -e "${BLUE}Test 1: Publishing valid v1 message${NC}"
curl -s -X POST "$API_URL/test/telemetry" \
  -H "Content-Type: application/json" \
  -d '{
    "level": "info",
    "message": "Test message - should succeed",
    "serviceName": "test-service",
    "environment": "development"
  }' | jq '.'
echo -e "${GREEN}✓ Valid message published${NC}\n"
sleep 2

# Test 2: Transient error (will retry)
echo -e "${YELLOW}Test 2: Publishing message with transient error${NC}"
curl -s -X POST "$API_URL/test/telemetry" \
  -H "Content-Type: application/json" \
  -d '{
    "level": "error",
    "message": "Simulating transient failure",
    "serviceName": "test-service",
    "environment": "development",
    "fail": "transient"
  }' | jq '.'
echo -e "${YELLOW}⚠ Transient error triggered - will retry up to 3 times${NC}\n"
sleep 2

# Test 3: Permanent error (goes to DLQ immediately)
echo -e "${RED}Test 3: Publishing message with permanent error${NC}"
curl -s -X POST "$API_URL/test/telemetry" \
  -H "Content-Type: application/json" \
  -d '{
    "level": "error",
    "message": "Simulating permanent failure",
    "serviceName": "test-service",
    "environment": "development",
    "fail": "permanent"
  }' | jq '.'
echo -e "${RED}✗ Permanent error triggered - sent to DLQ immediately${NC}\n"
sleep 2

# Test 4: Invalid JSON (permanent error)
echo -e "${RED}Test 4: Publishing invalid event (missing required fields)${NC}"
echo "Note: This would fail validation on the publisher side"
echo ""

# Wait for messages to be processed
echo -e "${BLUE}Waiting 5 seconds for message processing...${NC}"
sleep 5

# Inspect DLQ
echo ""
echo -e "${BLUE}Inspecting DLQ messages${NC}"
echo "========================"
DLQ_RESPONSE=$(curl -s "$API_URL/dlq/$QUEUE?limit=10")
MESSAGE_COUNT=$(echo "$DLQ_RESPONSE" | jq -r '.messageCount')

echo -e "DLQ Message Count: ${YELLOW}$MESSAGE_COUNT${NC}"
echo ""

if [ "$MESSAGE_COUNT" -gt 0 ]; then
  echo "DLQ Messages:"
  echo "$DLQ_RESPONSE" | jq -r '.messages[] | "
  📋 Message ID: \(.id)
  ❌ Error Type: \(.errorType // "unknown")
  💬 Error Reason: \(.errorReason // "none")
  📌 Version: \(.eventVersion // "none")
  🔄 Retry Count: \(.retryCount // 0)
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"'
  
  # Get first message ID for replay test
  FIRST_MSG_ID=$(echo "$DLQ_RESPONSE" | jq -r '.messages[0].id')
  
  if [ "$FIRST_MSG_ID" != "null" ] && [ -n "$FIRST_MSG_ID" ]; then
    echo ""
    echo -e "${BLUE}Test 5: Inspecting specific DLQ message${NC}"
    echo "========================================"
    curl -s "$API_URL/dlq/$QUEUE/$FIRST_MSG_ID" | jq '.'
    
    echo ""
    echo -e "${GREEN}Test 6: Replay demonstration (optional)${NC}"
    echo "========================================"
    echo "To replay message $FIRST_MSG_ID, run:"
    echo ""
    echo -e "${YELLOW}curl -X POST $API_URL/dlq/$QUEUE/replay \\"
    echo "  -H 'Content-Type: application/json' \\"
    echo "  -d '{\"messageId\": \"$FIRST_MSG_ID\", \"operator\": \"test-user\"}' | jq '.'${NC}"
    echo ""
  fi
else
  echo -e "${GREEN}No messages in DLQ${NC}"
fi

echo ""
echo -e "${BLUE}Metrics Summary${NC}"
echo "==============="
echo "View detailed metrics at: http://localhost:9090/metrics"
echo ""
echo "Key metrics to check:"
echo "  - messages_processed_total (successful)"
echo "  - messages_failed_total{error_type=\"transient\"}"
echo "  - messages_failed_total{error_type=\"permanent\"}"
echo "  - messages_retried_total"
echo "  - messages_dlq_total"
echo ""

echo -e "${GREEN}✓ Test script complete!${NC}"
