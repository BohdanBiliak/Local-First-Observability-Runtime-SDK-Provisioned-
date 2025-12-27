#!/usr/bin/env node

/**
 * Message Classification & Versioning Example
 *
 * Demonstrates:
 * - Publishing events with explicit versions
 * - Handling different event versions
 * - Error classification (transient vs permanent)
 * - DLQ inspection
 */

const axios = require('axios');

const API_URL = process.env.API_URL || 'http://localhost:3000';

async function publishLog(payload: any, description: string) {
  console.log(`\n📤 ${description}`);
  try {
    const response = await axios.post(`${API_URL}/test/telemetry`, payload);
    console.log('✅ Response:', response.data);
    return response.data;
  } catch (error: any) {
    console.log('❌ Error:', error.response?.data || error.message);
    throw error;
  }
}

async function inspectDLQ(queue: string = 'telemetry') {
  console.log(`\n🔍 Inspecting DLQ: ${queue}`);
  try {
    const response = await axios.get(`${API_URL}/dlq/${queue}?limit=10`);
    const data = response.data;

    console.log(`\n📊 DLQ Stats:`);
    console.log(`   Queue: ${data.queue}`);
    console.log(`   Message Count: ${data.messageCount}`);

    if (data.messages.length > 0) {
      console.log(`\n📋 Messages:`);
      data.messages.forEach((msg: any, index: number) => {
        console.log(`\n   [${index + 1}] ${msg.id}`);
        console.log(`       Error Type: ${msg.errorType || 'N/A'}`);
        console.log(`       Error Reason: ${msg.errorReason || 'N/A'}`);
        console.log(`       Event Version: ${msg.eventVersion || 'N/A'}`);
        console.log(`       Retry Count: ${msg.retryCount || 0}`);
        console.log(`       Original Queue: ${msg.originalQueue}`);
      });
    }

    return data;
  } catch (error: any) {
    console.log('❌ Error inspecting DLQ:', error.message);
    throw error;
  }
}

async function replayMessage(queue: string, messageId: string, operator: string = 'test-script') {
  console.log(`\n🔄 Replaying message: ${messageId}`);
  try {
    const response = await axios.post(`${API_URL}/dlq/${queue}/replay`, {
      messageId,
      operator,
    });
    console.log('✅ Replay result:', response.data);
    return response.data;
  } catch (error: any) {
    console.log('❌ Replay error:', error.response?.data || error.message);
    throw error;
  }
}

async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function main() {
  console.log('🧪 Message Classification & Versioning Demo');
  console.log('==========================================\n');

  try {
    // Test 1: Valid v1 event
    await publishLog(
      {
        level: 'info',
        message: 'This is a valid log message',
        serviceName: 'demo-app',
        environment: 'development',
      },
      'Publishing valid v1 log event',
    );

    await sleep(2000);

    // Test 2: Transient error
    await publishLog(
      {
        level: 'error',
        message: 'Simulating a transient network failure',
        serviceName: 'demo-app',
        environment: 'development',
        fail: 'transient',
      },
      'Publishing event that will trigger transient error (will retry)',
    );

    await sleep(2000);

    // Test 3: Permanent error
    await publishLog(
      {
        level: 'error',
        message: 'Simulating a permanent validation failure',
        serviceName: 'demo-app',
        environment: 'development',
        fail: 'permanent',
      },
      'Publishing event that will trigger permanent error (DLQ immediately)',
    );

    // Wait for processing
    console.log('\n⏳ Waiting 8 seconds for message processing...');
    await sleep(8000);

    // Inspect DLQ
    const dlqData = await inspectDLQ('telemetry');

    // Optionally replay first message
    if (dlqData.messages.length > 0) {
      const firstMessage = dlqData.messages[0];
      console.log(`\n💡 To replay this message, run:`);
      console.log(`   node ${process.argv[1]} replay ${firstMessage.id}`);
    }

    console.log('\n✅ Demo complete!');
    console.log('\n📊 Next steps:');
    console.log('   1. Check metrics: http://localhost:9090/metrics');
    console.log('   2. View DLQ: curl http://localhost:3000/dlq/telemetry');
    console.log('   3. Replay messages using the API or script');
  } catch (error) {
    console.error('\n❌ Demo failed:', error);
    process.exit(1);
  }
}

// Handle command-line arguments
const args = process.argv.slice(2);
if (args[0] === 'replay' && args[1]) {
  replayMessage('telemetry', args[1])
    .then(() => process.exit(0))
    .catch(() => process.exit(1));
} else if (args[0] === 'inspect') {
  inspectDLQ(args[1] || 'telemetry')
    .then(() => process.exit(0))
    .catch(() => process.exit(1));
} else {
  main()
    .then(() => process.exit(0))
    .catch(() => process.exit(1));
}
