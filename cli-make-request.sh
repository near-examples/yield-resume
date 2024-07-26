CONTRACT=requests.near-examples.testnet
USER=

TIMESTAMP=$(date +%s)

near call $CONTRACT request "{\"prompt\": \"This is a question ${TIMESTAMP}\"}" --accountId $USER