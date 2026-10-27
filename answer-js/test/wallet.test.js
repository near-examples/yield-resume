const assert = require('node:assert/strict');
const { test } = require('node:test');
const { KeyPair } = require('near-api-js');
const { Wallet } = require('../near');

const options = { accountId: 'answer.testnet', privateKey: KeyPair.fromRandom('ed25519').toString() };

test('contract calls use bigint units and preserve parsed SDK results', async () => {
  const wallet = new Wallet(options);
  wallet.account.callFunction = async (params) => {
    assert.equal(params.gas, 300000000000000n);
    assert.equal(params.deposit, 0n);
    assert.equal(params.methodName, 'respond');
    return { answer: 'resumed' };
  };
  assert.deepEqual(await wallet.callMethod({ contractId: 'request.testnet', method: 'respond' }), { answer: 'resumed' });
});

test('view requests use the provider without a signer transaction', async () => {
  const wallet = new Wallet({ ...options, networkId: 'localnet', rpcUrl: 'http://localhost:3030' });
  wallet.provider.callFunction = async (params) => {
    assert.deepEqual(params, { contractId: 'request.testnet', method: 'list_requests', args: {} });
    return [];
  };
  assert.deepEqual(await wallet.viewMethod({ contractId: 'request.testnet', method: 'list_requests' }), []);
});
