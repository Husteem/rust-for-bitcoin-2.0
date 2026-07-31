//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::labs::lab03_maturity::get_balances;
use crate::model::{MempoolObservation, WalletTransactionStatus};
use crate::rpc::{parse_cli_value, required_f64, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Send bitcoin from one wallet and return the TXID.
pub fn send_btc<C: RpcClient>(
    client: &C,
    from_wallet: &str,
    destination: &str,
    amount_btc: f64,
) -> LabResult<String> {
    let amount_str = amount_btc.to_string();
    let raw = client.call(
        Some(from_wallet),
        "sendtoaddress",
        &[destination.to_owned(), amount_str],
    )?;
    let val = parse_cli_value(&raw)?;
    val.as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse("expected TXID string".to_owned()))
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "getrawmempool", &[])?;
    let val = parse_cli_value(&raw)?;
    let array = val
        .as_array()
        .ok_or_else(|| LabError::Parse("expected mempool TXID array".to_owned()))?;

    array
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| LabError::Parse("expected TXID string".to_owned()))
        })
        .collect()
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let val = parse_cli_value(&raw)?;

    let parsed_txid = required_string(&val, "txid")?;
    let confirmations = val
        .get("confirmations")
        .and_then(serde_json::Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;
    let amount = required_f64(&val, "amount")?;
    let fee = val.get("fee").and_then(serde_json::Value::as_f64);
    let block_hash = val
        .get("blockhash")
        .or_else(|| val.get("block_hash"))
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned);

    Ok(WalletTransactionStatus {
        txid: parsed_txid,
        confirmations,
        amount,
        fee,
        block_hash,
    })
}

/// Send a payment without mining and capture its mempool and receiver-wallet state.
pub fn observe_unconfirmed_payment<C: RpcClient>(
    client: &C,
    sender_wallet: &str,
    receiver_wallet: &str,
    receiver_address: &str,
    amount_btc: f64,
) -> LabResult<MempoolObservation> {
    let txid = send_btc(client, sender_wallet, receiver_address, amount_btc)?;
    let mempool = get_raw_mempool(client)?;
    let mempool_contains_tx = mempool.contains(&txid);
    let sender_status = get_transaction_status(client, sender_wallet, &txid)?;
    let receiver_balance = get_balances(client, receiver_wallet)?;

    Ok(MempoolObservation {
        txid,
        mempool_contains_tx,
        sender_status,
        receiver_balance,
    })
}
