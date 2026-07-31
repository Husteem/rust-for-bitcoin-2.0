//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let val = parse_cli_value(&raw)?;
    let array = val
        .as_array()
        .ok_or_else(|| LabError::Parse("expected array of UTXOs".to_owned()))?;

    array
        .iter()
        .map(|item| {
            let txid = required_string(item, "txid")?;
            let vout = required_u64(item, "vout")? as u32;
            let address = item
                .get("address")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned);
            let script_pub_key = item
                .get("scriptPubKey")
                .or_else(|| item.get("script_pub_key"))
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .ok_or(LabError::MissingField("scriptPubKey"))?;
            let amount = required_f64(item, "amount")?;
            let confirmations = required_u64(item, "confirmations")?;
            let spendable = item
                .get("spendable")
                .and_then(serde_json::Value::as_bool)
                .ok_or(LabError::MissingField("spendable"))?;

            Ok(Utxo {
                txid,
                vout,
                address,
                script_pub_key,
                amount,
                confirmations,
                spendable,
            })
        })
        .collect()
}

/// Select one spendable UTXO, preferring the one with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    utxos
        .iter()
        .filter(|u| u.spendable)
        .max_by_key(|u| u.confirmations)
        .cloned()
}

/// Convert a UTXO into its unique `txid:vout` coordinate.
pub fn outpoint(utxo: &Utxo) -> OutPoint {
    utxo.outpoint()
}

/// Sum only the spendable UTXOs.
pub fn sum_spendable_utxos(utxos: &[Utxo]) -> f64 {
    utxos.iter().filter(|u| u.spendable).map(|u| u.amount).sum()
}
