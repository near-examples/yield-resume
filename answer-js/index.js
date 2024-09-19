require('dotenv').config();
const { connect, keyStores, KeyPair } = require('near-api-js');
const { Wallet } = require('./near');

const networkId = process.env.NETWORK_ID;
const accountId = process.env.ACCOUNT_ID;
const privateKey = process.env.PRIVATE_KEY;

function generateAnswer(prompt) {
  return `Answer to "${prompt}"`;
}

async function main() {
  const wallet = new Wallet({ networkId, accountId, privateKey });

  const requests = await wallet.viewMethod(
    {
      contractId: process.env.CONTRACT_ID,
      method: 'list_requests',
    }
  )

  if (requests.length === 0) {
    console.log("There are no requests, going back to sleep");
    return;
  }

  const last = requests[requests.length - 1];
  console.log("There is a request:", last.prompt);
  console.log("I am going to answer");

  const answer = generateAnswer(last.prompt);
  console.log(last);
  await wallet.callMethod(
    {
      contractId: process.env.CONTRACT_ID,
      method: 'respond',
      args: { yield_id: last.yield_id, response: answer },
    }
  )

  // For the basic example
  // console.log("Answered, now deleting the request");
  // await wallet.callMethod(
  //   {
  //     contractId: process.env.CONTRACT_ID,
  //     method: 'remove_request',
  //     args: { id: requests[0].id },
  //   }
  // )
}

main()