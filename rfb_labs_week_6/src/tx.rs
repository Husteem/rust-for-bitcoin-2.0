use anyhow::{Result, anyhow};
use bitcoin::ecdsa::Signature as EcdsaSig;
use bitcoin::hashes::Hash;
use bitcoin::key::TapTweak;
use bitcoin::locktime::absolute::LockTime;
use bitcoin::secp256k1::{Message, Secp256k1};
use bitcoin::sighash::{EcdsaSighashType, Prevouts, SighashCache, TapSighashType};
use bitcoin::transaction::Version;
use bitcoin::{Address, Amount, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness};

use crate::config::DescriptorType;
use crate::db::UtxoRecord;
use crate::keys::WalletKeys;

#[derive(Debug, Clone)]
pub struct BuiltTransaction {
    pub transaction: Transaction,
    pub txid: bitcoin::Txid,
    pub fee_sats: u64,
    pub vsize: usize,
    pub weight: bitcoin::Weight,
}

pub fn create_unsigned_tx(
    selected_utxos: &[UtxoRecord],
    recipient_address: &Address,
    send_amount_sats: u64,
    change_address: Option<&Address>,
    change_amount_sats: u64,
) -> Result<Transaction> {
    let mut tx_ins = Vec::new();
    for utxo in selected_utxos {
        tx_ins.push(TxIn {
            previous_output: utxo.outpoint(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
        });
    }

    let mut tx_outs = Vec::new();
    tx_outs.push(TxOut {
        value: Amount::from_sat(send_amount_sats),
        script_pubkey: recipient_address.script_pubkey(),
    });

    if let Some(change_addr) = change_address.filter(|_| change_amount_sats > 0) {
        tx_outs.push(TxOut {
            value: Amount::from_sat(change_amount_sats),
            script_pubkey: change_addr.script_pubkey(),
        });
    }

    Ok(Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: tx_ins,
        output: tx_outs,
    })
}

pub fn sign_transaction(
    unsigned_tx: Transaction,
    selected_utxos: &[UtxoRecord],
    keys: &WalletKeys,
) -> Result<BuiltTransaction> {
    let secp = Secp256k1::new();
    let mut signed_tx = unsigned_tx;

    // Collect spent TxOuts for sighash calculation
    let spent_txouts: Vec<TxOut> = selected_utxos
        .iter()
        .map(|u| TxOut {
            value: Amount::from_sat(u.amount_sats),
            script_pubkey: u.script_pubkey.clone(),
        })
        .collect();

    match keys.descriptor_type {
        DescriptorType::Wpkh => {
            let mut sighash_cache = SighashCache::new(&signed_tx);
            let mut witnesses = Vec::new();

            for (idx, utxo) in selected_utxos.iter().enumerate() {
                let (privkey, pubkey) =
                    keys.derive_secp_keypair(&secp, utxo.is_change, utxo.derivation_index)?;

                let sighash = sighash_cache
                    .p2wpkh_signature_hash(
                        idx,
                        &utxo.script_pubkey,
                        Amount::from_sat(utxo.amount_sats),
                        EcdsaSighashType::All,
                    )
                    .map_err(|e| anyhow!("Failed to compute P2WPKH sighash: {:?}", e))?;

                let msg = Message::from_digest(sighash.to_byte_array());
                let sig = secp.sign_ecdsa(&msg, &privkey);
                let ecdsa_sig = EcdsaSig::sighash_all(sig);

                let mut witness = Witness::new();
                witness.push(ecdsa_sig.to_vec());
                witness.push(pubkey.to_bytes());
                witnesses.push(witness);
            }

            for (idx, witness) in witnesses.into_iter().enumerate() {
                signed_tx.input[idx].witness = witness;
            }
        }
        DescriptorType::Tr => {
            let mut sighash_cache = SighashCache::new(&signed_tx);
            let prevouts = Prevouts::All(&spent_txouts);
            let mut witnesses = Vec::new();

            for (idx, utxo) in selected_utxos.iter().enumerate() {
                let keypair = keys.derive_keypair(&secp, utxo.is_change, utxo.derivation_index)?;
                let tweaked_keypair = keypair.tap_tweak(&secp, None).to_keypair();

                let sighash = sighash_cache
                    .taproot_key_spend_signature_hash(idx, &prevouts, TapSighashType::Default)
                    .map_err(|e| anyhow!("Failed to compute Taproot sighash: {:?}", e))?;

                let msg = Message::from_digest(sighash.to_byte_array());
                let schnorr_sig = secp.sign_schnorr_no_aux_rand(&msg, &tweaked_keypair);

                let mut witness = Witness::new();
                witness.push(schnorr_sig.as_ref());
                witnesses.push(witness);
            }

            for (idx, witness) in witnesses.into_iter().enumerate() {
                signed_tx.input[idx].witness = witness;
            }
        }
    }

    let total_in: u64 = selected_utxos.iter().map(|u| u.amount_sats).sum();
    let total_out: u64 = signed_tx.output.iter().map(|o| o.value.to_sat()).sum();
    let fee_sats = total_in.saturating_sub(total_out);

    let txid = signed_tx.compute_txid();
    let weight = signed_tx.weight();
    let vsize = signed_tx.vsize();

    Ok(BuiltTransaction {
        transaction: signed_tx,
        txid,
        fee_sats,
        vsize,
        weight,
    })
}
