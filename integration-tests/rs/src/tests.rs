use near_sdk::json_types::U128;
use near_units::{parse_gas, parse_near};
use serde_json::json;
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
use workspaces::{network::Sandbox, Account, Contract, Worker};

const NFT_WASM_FILEPATH: &str = "../../res/non_fungible_token.wasm";
const TR_WASM_FILEPATH: &str = "../../res/token_receiver.wasm";
const AR_WASM_FILEPATH: &str = "../../res/approval_receiver.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initiate environemnt
    let worker = workspaces::sandbox().await?;

    // deploy contracts
    let nft_wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let nft_contract = worker.dev_deploy(&nft_wasm).await?;
    let tr_wasm = std::fs::read(TR_WASM_FILEPATH)?;
    let tr_contract = worker.dev_deploy(&tr_wasm).await?;
    let ar_wasm = std::fs::read(AR_WASM_FILEPATH)?;
    let ar_contract = worker.dev_deploy(&ar_wasm).await?;

    // create accounts
    let owner = worker.root_account();
    let alice = owner
        .create_subaccount(&worker, "alice")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    // Initialize contracts
    nft_contract
        .call(&worker, "new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": owner.id()
        }))?
        .transact()
        .await?;
    tr_contract
        .call(&worker, "new")
        .args_json(serde_json::json!({
            "non_fungible_token_account_id": nft_contract.id()
        }))?
        .transact()
        .await?;
    ar_contract
        .call(&worker, "new")
        .args_json(serde_json::json!({
            "non_fungible_token_account_id": nft_contract.id()
        }))?
        .transact()
        .await?;

    // begin tests
    test_simple_approve(&owner, &alice, &nft_contract, &worker).await?;
    test_approval_simple_call(&owner, &nft_contract, &ar_contract, &worker).await?;
    Ok(())
}

async fn test_simple_approve(
    owner: &Account,
    user: &Account,
    nft_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    owner
        .call(&worker, nft_contract.id(), "nft_mint")
        .args_json(json!({
            "token_id": "0",
            "receiver_id": owner.id(),
            "token_metadata": {
                "title": "Olympus Mons",
                "description": "The tallest mountain in the charted solar system",
                "copies": 10000,
            }
        }))?
        .deposit(parse_gas!("5950000000000000000000"))
        .transact()
        .await?;

    // root approves alice
    owner
        .call(&worker, nft_contract.id(), "nft_approve")
        .args_json(json!({
            "token_id":  "0",
            "account_id": user.id(),
        }))?
        .deposit(parse_gas!("5950000000000000000000"))
        .transact()
        .await?;

    let approval_no_id: bool = nft_contract
        .call(&worker, "nft_is_approved")
        .args_json(json!({
            "token_id":  "0",
            "approved_account_id": user.id()
        }))?
        .transact()
        .await?
        .json()?;

    assert!(approval_no_id);

    let approval: bool = nft_contract
        .call(&worker, "nft_is_approved")
        .args_json(json!({
            "token_id":  "0",
            "approved_account_id": user.id(),
            "approval_id": 1
        }))?
        .transact()
        .await?
        .json()?;

    assert!(approval);

    let approval_wrong_id: bool = nft_contract
        .call(&worker, "nft_is_approved")
        .args_json(json!({
            "token_id":  "0",
            "approved_account_id": user.id(),
            "approval_id": 2
        }))?
        .transact()
        .await?
        .json()?;

    assert!(!approval_wrong_id);
    println!("      Passed ✅ test_simple_approve");
    Ok(())
}

async fn test_approval_simple_call(
    owner:  &Account,
    nft_contract: &Contract,
    approval_receiver: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    owner
        .call(&worker, nft_contract.id(), "nft_mint")
        .args_json(json!({
            "token_id": "1",
            "receiver_id": owner.id(),
            "token_metadata": {
                "title": "Olympus Mons 2",
                "description": "The tallest mountain in the charted solar system",
                "copies": 1,
            }
        }))?
        .deposit(parse_gas!("5950000000000000000000"))
        .transact()
        .await?;

    let outcome: String = owner
        .call(&worker, nft_contract.id(), "nft_approve")
        .args_json(json!({
            "token_id": "1",
            "account_id": approval_receiver.id(),
            "msg": "return-now"
        }))?
        .gas(parse_gas!("150 Tgas") as u64)
        .deposit(parse_gas!("450000000000000000000"))
        .transact()
        .await?
        .json()?;
    assert_eq!("cool", outcome);

    let msg = "test message";
    let outcome: String = owner
        .call(&worker, nft_contract.id(), "nft_approve")
        .args_json(json!({
            "token_id": "1",
            "account_id": approval_receiver.id(),
            "msg": msg.clone(),
        }))?
        .gas(parse_gas!("150 Tgas") as u64)
        .deposit(parse_gas!("450000000000000000000"))
        .transact()
        .await?
        .json()?;
    assert_eq!(msg, outcome);

    println!("      Passed ✅ test_approval_simple_call");
    Ok(())
}

async fn test_approved_account_transfers_token() -> anyhow::Result<()> {Ok(())}
async fn test_revoke() -> anyhow::Result<()> {Ok(())}
async fn test_revoke_all() -> anyhow::Result<()> {Ok(())}
async fn test_simple_transfer() -> anyhow::Result<()> {Ok(())}
async fn test_transfer_call_fast_return_to_sender() -> anyhow::Result<()> {Ok(())}
async fn test_transfer_call_slow_return_to_sender() -> anyhow::Result<()> {Ok(())}
async fn test_transfer_call_fast_keep_with_sender() -> anyhow::Result<()> {Ok(())}
async fn test_transfer_call_slow_keep_with_sender() -> anyhow::Result<()> {Ok(())}
async fn test_transfer_call_receiver_panics() -> anyhow::Result<()> {Ok(())}
async fn test_enum_total_supply() -> anyhow::Result<()> {Ok(())}
async fn test_enum_nft_supply_for_owner() -> anyhow::Result<()> {Ok(())}
async fn test_enum_nft_tokens_for_owner() -> anyhow::Result<()> {Ok(())}
