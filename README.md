# Yield / Resume Example

NEAR smart contracts have the ability to yield execution until an external service provides them with an answer.

This repository shows an example smart contract that uses the yield/resume feature to take a request (in the form of a string), and yields until an external service responds to it.

## Build and test

The contract uses near-sdk 5.29.1 and Rust 1.96. From the contract directory:

```bash
cd contract
cargo build --target wasm32-unknown-unknown --release
cargo test
```

The integration tests run on a local NEAR sandbox and cover successful resumption, timeout, request cleanup, and late responses.

The contract creates yielded promises with `Promise::new_yield` and resumes them with `YieldId::resume`. Yield IDs are base64 strings in JSON. Pass the `yield_id` from `list_requests` unchanged to `respond`.

The callback receives `Result<String, PromiseError>`. A response returns `Response::Answer`; a timeout returns `Response::TimeOutError`. Both paths remove the stored request without panicking, so cleanup is preserved.

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
