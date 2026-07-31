//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    let raw = client.call(None, "getblockchaininfo", &[])?;
    let val = parse_cli_value(&raw)?;
    required_string(&val, "chain")
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    let raw = client.call(None, "getblockcount", &[])?;
    let val = parse_cli_value(&raw)?;
    val.as_u64().ok_or(LabError::MissingField("blockcount"))
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    let raw = client.call(None, "getbestblockhash", &[])?;
    let val = parse_cli_value(&raw)?;
    val.as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse("expected best block hash string".to_owned()))
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    let chain = get_chain(client)?;
    if chain != "regtest" {
        return Err(LabError::Rpc("expected regtest network".to_owned()));
    }

    let block_height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    Ok(NetworkSnapshot {
        chain,
        block_height,
        best_block_hash,
    })
}
