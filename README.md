# Yield / Resume Example

NEAR smart contracts have the ability to yield execution until an external service provides them with an answer.

This repository shows an example smart contract that uses the yield/resume feature to take a request (in the form of a string), and yields until an external service responds to it.

---

## Making a request

Making a request is as simple as calling the contract with the request method:

```bash
# using near-cli-rs (https://docs.near.org/tools/near-cli)
near call requests.near-examples.testnet request "{\"prompt\": \"A simple question\"}" --accountId <your account>
```

---

## Giving an Answer

In the current implementation, anyone can give an answer to the contract. The contract will then resume execution and return the answer to the caller.

We have included a simple JavaScript application that pings the contract to see if there are any pending questions, and gives an answer. To run it, follow the steps below:

```bash
cd answer-js
npm i
npm start
```