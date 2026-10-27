require('dotenv').config();
const { Wallet } = require('./near');

function generateAnswer(prompt) {
  return `Answer to "${prompt}"`;
}

async function answerRequests(wallet, contractId) {
  if (!contractId) throw new Error('CONTRACT_ID is required');
  const requests = await wallet.viewMethod({ contractId, method: 'list_requests' });
  if (requests.length === 0) {
    console.log('There are no requests, going back to sleep');
    return;
  }

  const last = requests[requests.length - 1];
  console.log('There is a request:', last.prompt);
  return wallet.callMethod({
    contractId,
    method: 'respond',
    args: { yield_id: last.yield_id, response: generateAnswer(last.prompt) },
  });
}

if (require.main === module) {
  const { NETWORK_ID = 'testnet', ACCOUNT_ID, PRIVATE_KEY, CONTRACT_ID } = process.env;
  Promise.resolve().then(() => answerRequests(new Wallet({
    networkId: NETWORK_ID,
    accountId: ACCOUNT_ID,
    privateKey: PRIVATE_KEY,
  }), CONTRACT_ID)).catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}

module.exports = { answerRequests };
