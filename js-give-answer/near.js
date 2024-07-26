const { providers, connect, keyStores, KeyPair } = require('near-api-js');

const MAX_GAS = '300000000000000';
const NO_DEPOSIT = '0';

class Wallet {
  /**
   * @constructor
   * @param {Object} options - the options for the wallet
   * @param {string} options.networkId - the network id to connect to
   * @param {string} options.accountId - the account to use
   * @param {string} options.privateKey - the private key to use
   */
  constructor({ networkId = 'testnet', accountId, privateKey }) {
    this.accountId = accountId;
    this.networkId = networkId;

    const keyPair = KeyPair.fromString(privateKey);
    const keyStore = new keyStores.InMemoryKeyStore();
    keyStore.setKey(networkId, accountId, keyPair)

    this.near = connect({ networkId, nodeUrl: 'https://rpc.testnet.near.org', keyStore });
  }

  /**
   * Makes a read-only call to a contract
   * @param {Object} options - the options for the call
   * @param {string} options.contractId - the contract's account id
   * @param {string} options.method - the method to call
   * @param {Object} options.args - the arguments to pass to the method
   * @returns {Promise<JSON.value>} - the result of the method call
   */
  viewMethod = async ({ contractId, method, args = {} }) => {
    const account = await (await this.near).account(this.accountId);
    return await account.viewFunction({ contractId, methodName: method, args });
  };

  /**
   * Makes a call to a contract
   * @param {Object} options - the options for the call
   * @param {string} options.contractId - the contract's account id
   * @param {string} options.method - the method to call
   * @param {Object} options.args - the arguments to pass to the method
   * @param {string} options.gas - the amount of gas to use
   * @param {string} options.deposit - the amount of yoctoNEAR to deposit
   * @returns {Promise<Transaction>} - the resulting transaction
   */
  callMethod = async ({ contractId, method, args = {}, gas = MAX_GAS, deposit = NO_DEPOSIT }) => {
    // Sign a transaction with the "FunctionCall" action
    const account = await (await this.near).account(this.accountId);

    const outcome = await account.functionCall({
      contractId,
      methodName: method,
      args,
      gas,
      attachedDeposit: deposit,
    });

    return providers.getTransactionLastResult(outcome);
  };
}

// export the wallet
module.exports = { Wallet };