//! Lab 06 — decode a transaction and prove value conservation.

use crate::model::{DecodedInput, DecodedOutput, DecodedTransaction, OutPoint, PaymentAndChange};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Decode a transaction with enough verbosity to include every spent output's value.
pub fn decode_verbose_transaction<C: RpcClient>(
    client: &C,
    txid: &str,
) -> LabResult<DecodedTransaction> {
    let raw = client.call(
        None,
        "getrawtransaction",
        &[txid.to_owned(), "2".to_owned()],
    )?;
    let val = parse_cli_value(&raw)?;

    let parsed_txid = required_string(&val, "txid")?;
    let vsize = required_u64(&val, "vsize")?;

    let vin_array = val
        .get("vin")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("vin"))?;

    let inputs = vin_array
        .iter()
        .map(|vin_obj| {
            let prev_txid = required_string(vin_obj, "txid")?;
            let prev_vout = required_u64(vin_obj, "vout")? as u32;
            let prevout = vin_obj
                .get("prevout")
                .ok_or(LabError::MissingField("prevout"))?;
            let previous_value = required_f64(prevout, "value")?;

            Ok(DecodedInput {
                previous_output: OutPoint {
                    txid: prev_txid,
                    vout: prev_vout,
                },
                previous_value,
            })
        })
        .collect::<LabResult<Vec<DecodedInput>>>()?;

    let vout_array = val
        .get("vout")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("vout"))?;

    let outputs = vout_array
        .iter()
        .map(|vout_obj| {
            let vout_n = required_u64(vout_obj, "n")? as u32;
            let value = required_f64(vout_obj, "value")?;
            let spk = vout_obj
                .get("scriptPubKey")
                .ok_or(LabError::MissingField("scriptPubKey"))?;

            let script_pub_key_hex = spk
                .get("hex")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .ok_or(LabError::MissingField("scriptPubKey.hex"))?;

            let address = spk
                .get("address")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .or_else(|| {
                    spk.get("addresses")
                        .and_then(serde_json::Value::as_array)
                        .and_then(|arr| arr.first())
                        .and_then(serde_json::Value::as_str)
                        .map(ToOwned::to_owned)
                });

            Ok(DecodedOutput {
                vout: vout_n,
                value,
                address,
                script_pub_key_hex,
            })
        })
        .collect::<LabResult<Vec<DecodedOutput>>>()?;

    Ok(DecodedTransaction {
        txid: parsed_txid,
        inputs,
        outputs,
        vsize,
    })
}

/// Return every previous output consumed by the transaction.
pub fn input_outpoints(transaction: &DecodedTransaction) -> Vec<OutPoint> {
    transaction
        .inputs
        .iter()
        .map(|i| i.previous_output.clone())
        .collect()
}

/// Identify the receiver payment and optional change output.
pub fn identify_payment_and_change(
    transaction: &DecodedTransaction,
    receiver_address: &str,
) -> LabResult<PaymentAndChange> {
    let payment = transaction
        .outputs
        .iter()
        .find(|out| out.address.as_deref() == Some(receiver_address))
        .cloned()
        .ok_or(LabError::MissingField("payment output"))?;

    let change = transaction
        .outputs
        .iter()
        .find(|out| out.address.as_deref() != Some(receiver_address))
        .cloned();

    Ok(PaymentAndChange { payment, change })
}

/// Calculate `sum(inputs) - sum(outputs)`.
pub fn calculate_fee(transaction: &DecodedTransaction) -> LabResult<f64> {
    let sum_inputs: f64 = transaction.inputs.iter().map(|i| i.previous_value).sum();
    let sum_outputs: f64 = transaction.outputs.iter().map(|o| o.value).sum();
    let fee = sum_inputs - sum_outputs;

    if fee < -1e-9 {
        return Err(LabError::Rpc("negative fee calculated".to_owned()));
    }

    let rounded_fee = (fee * 100_000_000.0).round() / 100_000_000.0;
    Ok(rounded_fee)
}
