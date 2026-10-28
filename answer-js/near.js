const { Account, JsonRpcProvider, KeyPairSigner } = require('near-api-js');

const RPC_URLS = {
  mainnet: 'https://free.rpc.fastnear.com',
  testnet: 'https://test.rpc.fastnear.com',
};

class Wallet {
  constructor({ networkId = 'testnet', accountId, privateKey, rpcUrl = process.env.RPC_URL }) {
    const url = rpcUrl || RPC_URLS[networkId];
    if (!url) throw new Error(`Set RPC_URL for network ${networkId}`);
    if (!accountId || !privateKey) throw new Error('ACCOUNT_ID and PRIVATE_KEY are required');
    this.provider = new JsonRpcProvider({ url });
    this.account = new Account(accountId, this.provider, KeyPairSigner.fromSecretKey(privateKey));
  }

  // Reading requests does not require an account lookup or a signing key on the RPC.
  viewMethod = ({ contractId, method, args = {} }) =>
    this.provider.callFunction({ contractId, method, args });

  callMethod = async ({ contractId, method, args = {}, gas = '300000000000000', deposit = '0' }) => {
    return this.account.callFunction({
      contractId,
      methodName: method,
      args,
      gas: BigInt(gas),
      deposit: BigInt(deposit),
    });
  };
}

module.exports = { Wallet };
