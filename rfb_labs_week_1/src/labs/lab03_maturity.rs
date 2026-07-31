//! Lab 03 — demonstrate coinbase maturity.

use crate::labs::lab01_network::get_block_height;
use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::{parse_cli_value, required_f64, RpcClient};
use crate::{LabError, LabResult};

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_owned()],
    )?;
    let val = parse_cli_value(&raw)?;
    let array = val
        .as_array()
        .ok_or_else(|| LabError::Parse("expected array of block hashes".to_owned()))?;

    array
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| LabError::Parse("expected block hash string".to_owned()))
        })
        .collect()
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    let raw = client.call(Some(wallet_name), "getbalances", &[])?;
    let val = parse_cli_value(&raw)?;
    let mine = val.get("mine").ok_or(LabError::MissingField("mine"))?;

    let trusted = required_f64(mine, "trusted")?;
    let untrusted_pending = required_f64(mine, "untrusted_pending")?;
    let immature = required_f64(mine, "immature")?;

    Ok(WalletBalances {
        trusted,
        untrusted_pending,
        immature,
    })
}

/// Attempt a wallet payment and return either its TXID or the Bitcoin Core error.
pub fn attempt_payment<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
    amount_btc: f64,
) -> LabResult<String> {
    let amount_str = amount_btc.to_string();
    let raw = client.call(
        Some(wallet_name),
        "sendtoaddress",
        &[address.to_owned(), amount_str],
    )?;
    let val = parse_cli_value(&raw)?;
    val.as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse("expected TXID string".to_owned()))
}

/// Mine one block, prove the reward is immature, then mine 100 more blocks.
pub fn demonstrate_coinbase_maturity<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    miner_address: &str,
    receiver_address: &str,
) -> LabResult<CoinbaseMaturityReport> {
    mine_blocks(client, miner_address, 1)?;
    let height_after_first_block = get_block_height(client)?;
    let balance_after_first_block = get_balances(client, miner_wallet)?;

    let premature_spend_error = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Err(LabError::Rpc(message)) => message,
        Err(err) => return Err(err),
        Ok(_) => return Err(LabError::Rpc("expected premature payment error".to_owned())),
    };

    mine_blocks(client, miner_address, 100)?;
    let final_height = get_block_height(client)?;
    let final_balance = get_balances(client, miner_wallet)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block,
        balance_after_first_block,
        premature_spend_error,
        final_height,
        final_balance,
    })
}
