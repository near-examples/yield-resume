# Yield / Resume

NEAR smart contracts have recently gained the ability to yield execution until an external service provides them with an answer.

## Smart Contracts

This repository has two contracts as an example on using the yield/resume feature.

- [Basic Example](./basic-yield-resume): A barebones contract that simply takes a request (in the form of a string), and holds until an external service responds to it
- [Retry on Error](./retry-on-error): A simple contract that caches the result, so if a timeout happens, the user does not need to make the request again

---

## Making a request

I have included a simple bash script to make a request to the contract. You can run it with the following command:

```bash
./cli-make-request.sh
```

---

## Giving an Answer

I have included a small JS script to give an answer to the contract. You can run it with the following command:

```bash
cd js-give-answer
npm install
npm start
```