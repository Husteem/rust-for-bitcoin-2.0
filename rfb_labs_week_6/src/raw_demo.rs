use anyhow::Result;
use bitcoin::blockdata::script::Builder;
use bitcoin::ecdsa::Signature as EcdsaSig;
use bitcoin::hashes::{Hash, sha256};
use bitcoin::locktime::absolute::LockTime;
use bitcoin::opcodes::all::*;
use bitcoin::secp256k1::{Message, PublicKey as SecpPublicKey, Secp256k1, SecretKey};
use bitcoin::sighash::{EcdsaSighashType, SighashCache};
use bitcoin::transaction::Version;
use bitcoin::{
    Address, Amount, CompressedPublicKey, Network, OutPoint, PublicKey, ScriptBuf, Sequence,
    Transaction, TxIn, TxOut, Txid, Witness,
};
use std::str::FromStr;

#[derive(Debug)]
pub struct RawScriptDemoReport {
    pub script_hex: String,
    pub script_asm: String,
    pub funding_txid: Txid,
    pub spending_txid: Txid,
    pub spending_raw_hex: String,
    pub witness_items: Vec<String>,
    pub explanation: String,
}

/// Demonstrates building and spending a custom Bitcoin SegWit script using low-level rust-bitcoin primitives.
///
/// Script logic: Hash Lock + Signature Lock
/// Script: OP_SHA256 <secret_hash> OP_EQUALVERIFY <pubkey> OP_CHECKSIG
pub fn run_raw_script_demo(network: Network) -> Result<RawScriptDemoReport> {
    let secp = Secp256k1::new();

    // 1. Generate low-level keypair and secret preimage
    let secret_key = SecretKey::from_slice(&[0x11; 32])?;
    let secp_pubkey = SecpPublicKey::from_secret_key(&secp, &secret_key);
    let btc_pubkey = PublicKey::new(secp_pubkey);

    let secret_preimage = b"rust-for-bitcoin-week-6-demo-secret";
    let preimage_hash = sha256::Hash::hash(secret_preimage);

    // 2. Build custom witness script: OP_SHA256 <hash> OP_EQUALVERIFY <pubkey> OP_CHECKSIG
    let witness_script = Builder::new()
        .push_opcode(OP_SHA256)
        .push_slice(preimage_hash.as_byte_array())
        .push_opcode(OP_EQUALVERIFY)
        .push_key(&btc_pubkey)
        .push_opcode(OP_CHECKSIG)
        .into_script();

    let script_hex = hex::encode(witness_script.as_bytes());
    let script_asm = format!("{:?}", witness_script);

    // 3. Wrap into P2WSH scriptPubkey: OP_0 <sha256(witness_script)>
    let dummy_funding_txid =
        Txid::from_str("0000000000000000000000000000000000000000000000000000000000000001")?;
    let funding_outpoint = OutPoint::new(dummy_funding_txid, 0);
    let utxo_amount = Amount::from_sat(100_000);

    // 4. Construct spending transaction
    let recipient_secret = SecretKey::from_slice(&[0x22; 32])?;
    let recipient_pubkey =
        CompressedPublicKey(SecpPublicKey::from_secret_key(&secp, &recipient_secret));
    let recipient_address = Address::p2wpkh(&recipient_pubkey, network);

    let spend_amount = Amount::from_sat(98_000); // 2,000 sat fee

    let mut spend_tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: funding_outpoint,
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: spend_amount,
            script_pubkey: recipient_address.script_pubkey(),
        }],
    };

    // 5. Compute BIP143 SegWit v0 sighash for P2WSH spending
    let mut sighash_cache = SighashCache::new(&spend_tx);
    let sighash = sighash_cache.p2wsh_signature_hash(
        0,
        &witness_script,
        utxo_amount,
        EcdsaSighashType::All,
    )?;

    let msg = Message::from_digest(sighash.to_byte_array());
    let sig = secp.sign_ecdsa(&msg, &secret_key);
    let ecdsa_sig = EcdsaSig::sighash_all(sig);

    // 6. Assemble witness stack: [<sig>, <preimage>, <witness_script>]
    let mut witness = Witness::new();
    witness.push(ecdsa_sig.to_vec());
    witness.push(secret_preimage);
    witness.push(witness_script.as_bytes());

    spend_tx.input[0].witness = witness.clone();

    let spending_txid = spend_tx.compute_txid();
    let spending_raw_hex = hex::encode(bitcoin::consensus::serialize(&spend_tx));

    let witness_items = witness.iter().map(hex::encode).collect::<Vec<_>>();

    let explanation = "Low-Level Script vs Wallet Descriptors:\n\
        1. Low-Level Script: Manual opcode sequence assembly, custom hash preimage verification, explicit BIP143 sighash calculation, and exact witness stack serialization.\n\
        2. Wallet Descriptors: Standardized expression (wpkh / tr), hierarchical derivation paths (BIP32/BIP84/BIP86), automated change and coin selection, eliminating manual script mistakes.".to_string();

    Ok(RawScriptDemoReport {
        script_hex,
        script_asm,
        funding_txid: dummy_funding_txid,
        spending_txid,
        spending_raw_hex,
        witness_items,
        explanation,
    })
}
