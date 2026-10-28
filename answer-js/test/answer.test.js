const assert = require('node:assert/strict');
const { test } = require('node:test');
const { answerRequests } = require('../index');

test('empty request queue does not submit a transaction', async () => {
  await answerRequests({
    viewMethod: async ({ contractId, method }) => {
      assert.equal(contractId, 'requests.testnet');
      assert.equal(method, 'list_requests');
      return [];
    },
    callMethod: () => assert.fail('no response should be sent'),
  }, 'requests.testnet');
});

test('response preserves the latest request yield ID exactly', async () => {
  const yieldId = 'a+/b==';
  const result = await answerRequests({
    viewMethod: async () => [
      { prompt: 'earlier question', yield_id: 'earlier' },
      { prompt: 'latest question', yield_id: yieldId },
    ],
    callMethod: async (params) => {
      assert.deepEqual(params, {
        contractId: 'requests.testnet', method: 'respond',
        args: { yield_id: yieldId, response: 'Answer to "latest question"' },
      });
      return 'resumed';
    },
  }, 'requests.testnet');
  assert.equal(result, 'resumed');
});

test('missing contract configuration rejects before querying', async () => {
  await assert.rejects(answerRequests({ viewMethod: () => assert.fail('no RPC') }), /CONTRACT_ID/);
});
