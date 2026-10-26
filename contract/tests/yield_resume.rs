use near_workspaces::{network::Sandbox, Contract, Worker};
use serde_json::{json, Value};
use std::time::Duration;

async fn deploy() -> Result<(Worker<Sandbox>, Contract), Box<dyn std::error::Error>> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read("target/wasm32-unknown-unknown/release/yield_resume.wasm")
        .expect("Build the contract first: cargo build --target wasm32-unknown-unknown --release");
    let contract = worker.dev_deploy(&wasm).await?;
    Ok((worker, contract))
}

async fn pending_request(contract: &Contract) -> Result<Value, Box<dyn std::error::Error>> {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let requests: Vec<Value> = contract.view("list_requests").await?.json()?;
            if let Some(request) = requests.into_iter().next() {
                return Ok(request);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await?
}

async fn assert_no_requests(contract: &Contract) -> Result<(), Box<dyn std::error::Error>> {
    let requests: Vec<Value> = contract.view("list_requests").await?.json()?;
    assert!(requests.is_empty());
    Ok(())
}

#[tokio::test]
async fn external_response_resumes_request_and_clears_state(
) -> Result<(), Box<dyn std::error::Error>> {
    let (worker, contract) = deploy().await?;
    let caller = worker.dev_create_account().await?;
    let pending = caller
        .call(contract.id(), "request")
        .args_json(json!({"prompt": "What is 2+2?"}))
        .max_gas()
        .transact_async()
        .await?;
    let request = pending_request(&contract).await?;
    assert_eq!(request["prompt"], "What is 2+2?");
    assert!(
        request["yield_id"].is_string(),
        "YieldId is encoded as base64 JSON"
    );

    caller
        .call(contract.id(), "respond")
        .args_json(json!({"yield_id": request["yield_id"], "response": "4"}))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    let outcome = tokio::time::timeout(Duration::from_secs(30), pending)
        .await??
        .into_result()?;
    assert_eq!(outcome.json::<Value>()?, json!({"Answer": "4"}));
    assert_no_requests(&contract).await?;

    // Once the callback finishes, its yield ID cannot resume another request.
    let late = caller
        .call(contract.id(), "respond")
        .args_json(json!({"yield_id": request["yield_id"], "response": "late"}))
        .max_gas()
        .transact()
        .await?;
    assert!(late.is_failure());
    assert_no_requests(&contract).await?;
    Ok(())
}

#[tokio::test]
async fn timeout_returns_timeout_error_and_clears_state() -> Result<(), Box<dyn std::error::Error>>
{
    let (worker, contract) = deploy().await?;
    let caller = worker.dev_create_account().await?;
    let pending = caller
        .call(contract.id(), "request")
        .args_json(json!({"prompt": "No service will answer"}))
        .max_gas()
        .transact_async()
        .await?;
    let request = pending_request(&contract).await?;
    worker.fast_forward(210).await?;

    let outcome = tokio::time::timeout(Duration::from_secs(30), pending)
        .await??
        .into_result()?;
    assert_eq!(outcome.json::<Value>()?, json!("TimeOutError"));
    assert_no_requests(&contract).await?;

    let late = caller
        .call(contract.id(), "respond")
        .args_json(json!({"yield_id": request["yield_id"], "response": "too late"}))
        .max_gas()
        .transact()
        .await?;
    assert!(late.is_failure());
    assert_no_requests(&contract).await?;
    Ok(())
}
