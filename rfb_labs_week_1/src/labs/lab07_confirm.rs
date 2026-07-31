//! Lab 07 — confirm a transaction and prove block membership.

use crate::labs::lab03_maturity::mine_blocks;
use crate::labs::lab05_mempool::get_raw_mempool;
use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let hashes = mine_blocks(client, miner_address, 1)?;
    hashes
        .into_iter()
        .next()
        .ok_or(LabError::MissingField("blockhash"))
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let mempool = get_raw_mempool(client)?;
    Ok(mempool.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let val = parse_cli_value(&raw)?;
    val.get("confirmations")
        .and_then(serde_json::Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))
}

/// Mine, locate the transaction's block, and prove that the block contains the TXID.
pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
    let block_hash = mine_one_block(client, miner_address)?;
    let is_empty = mempool_is_empty(client)?;

    let tx_raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let tx_val = parse_cli_value(&tx_raw)?;

    let confirmations = tx_val
        .get("confirmations")
        .and_then(serde_json::Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;

    let containing_block_hash = tx_val
        .get("blockhash")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&block_hash)
        .to_owned();

    let block_raw = client.call(
        None,
        "getblock",
        &[containing_block_hash.clone(), "1".to_owned()],
    )?;
    let block_val = parse_cli_value(&block_raw)?;
    let tx_array = block_val
        .get("tx")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("tx"))?;

    let transaction_is_in_block = tx_array.iter().any(|t| t.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash: containing_block_hash,
        confirmations,
        mempool_is_empty: is_empty,
        transaction_is_in_block,
    })
}
