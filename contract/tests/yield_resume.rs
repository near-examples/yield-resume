use near_api::{Contract, NearToken, NetworkConfig, Signer};
use near_sandbox::Sandbox;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

async fn deploy() -> anyhow::Result<(Sandbox, NetworkConfig, Contract, Arc<Signer>)> {
    let artifact = cargo_near_build::build_with_cli(cargo_near_build::BuildOpts {
        no_abi: true,
        no_embed_abi: true,
        ..Default::default()
    })
    .expect("build yield/resume contract");
    let wasm = std::fs::read(artifact)?;
    let sandbox = Sandbox::start_sandbox().await?;
    let network = NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);
    let id = "yield.sandbox".parse()?;
    sandbox
        .create_account(id)
        .initial_balance(NearToken::from_near(10))
        .send()
        .await?;
    let contract = Contract("yield.sandbox".parse()?);
    let signer = Signer::from_secret_key(
        near_sandbox::config::DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY.parse()?,
    )?;
    Contract::deploy(contract.account_id().clone())
        .use_code(wasm)
        .without_init_call()
        .with_signer(signer.clone())
        .send_to(&network)
        .await?
        .assert_success();
    Ok((sandbox, network, contract, signer))
}

async fn pending_request(
    contract: &Contract,
    network: &NetworkConfig,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let requests: Vec<Value> = contract
                .call_function("list_requests", json!({}))
                .read_only()
                .fetch_from(network)
                .await?
                .data;
            if let Some(request) = requests.into_iter().next() {
                return Ok(request);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await?
}

async fn assert_no_requests(
    contract: &Contract,
    network: &NetworkConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let requests: Vec<Value> = contract
        .call_function("list_requests", json!({}))
        .read_only()
        .fetch_from(network)
        .await?
        .data;
    assert!(requests.is_empty());
    Ok(())
}

#[tokio::test]
async fn external_response_resumes_request_and_clears_state(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (worker, network, contract, signer) = deploy().await?;
    let caller: near_api::AccountId = "caller.sandbox".parse()?;
    worker
        .create_account(caller.clone())
        .initial_balance(NearToken::from_near(10))
        .send()
        .await?;
    let pending_call = contract
        .call_function("request", json!({"prompt": "What is 2+2?"}))
        .transaction()
        .max_gas()
        .with_signer(caller.clone(), signer.clone());
    let pending_network = network.clone();
    let pending = tokio::spawn(async move { pending_call.send_to(&pending_network).await });
    let request = pending_request(&contract, &network).await?;
    assert_eq!(request["prompt"], "What is 2+2?");
    assert!(
        request["yield_id"].is_string(),
        "YieldId is encoded as base64 JSON"
    );

    contract
        .call_function(
            "respond",
            json!({"yield_id": request["yield_id"], "response": "4"}),
        )
        .transaction()
        .max_gas()
        .with_signer(caller.clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();
    let outcome = tokio::time::timeout(Duration::from_secs(30), pending)
        .await???
        .assert_success();
    assert_eq!(outcome.json::<Value>()?, json!({"Answer": "4"}));
    assert_no_requests(&contract, &network).await?;

    // Once the callback finishes, its yield ID cannot resume another request.
    let late = contract
        .call_function(
            "respond",
            json!({"yield_id": request["yield_id"], "response": "late"}),
        )
        .transaction()
        .max_gas()
        .with_signer(caller.clone(), signer.clone())
        .send_to(&network)
        .await?;
    late.assert_failure();
    assert_no_requests(&contract, &network).await?;
    Ok(())
}

#[tokio::test]
async fn timeout_returns_timeout_error_and_clears_state(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (worker, network, contract, signer) = deploy().await?;
    let caller: near_api::AccountId = "caller.sandbox".parse()?;
    worker
        .create_account(caller.clone())
        .initial_balance(NearToken::from_near(10))
        .send()
        .await?;
    let pending_call = contract
        .call_function("request", json!({"prompt": "No service will answer"}))
        .transaction()
        .max_gas()
        .with_signer(caller.clone(), signer.clone());
    let pending_network = network.clone();
    let pending = tokio::spawn(async move { pending_call.send_to(&pending_network).await });
    let request = pending_request(&contract, &network).await?;
    worker.fast_forward(210).await?;

    let outcome = tokio::time::timeout(Duration::from_secs(30), pending)
        .await???
        .assert_success();
    assert_eq!(outcome.json::<Value>()?, json!("TimeOutError"));
    assert_no_requests(&contract, &network).await?;

    let late = contract
        .call_function(
            "respond",
            json!({"yield_id": request["yield_id"], "response": "too late"}),
        )
        .transaction()
        .max_gas()
        .with_signer(caller.clone(), signer.clone())
        .send_to(&network)
        .await?;
    late.assert_failure();
    assert_no_requests(&contract, &network).await?;
    Ok(())
}
