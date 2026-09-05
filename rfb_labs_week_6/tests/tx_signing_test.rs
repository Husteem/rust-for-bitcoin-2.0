use bitcoin::{Network, Txid};
use rfb_labs_week_6::config::DescriptorType;
use rfb_labs_week_6::db::UtxoRecord;
use rfb_labs_week_6::keys::WalletKeys;
use rfb_labs_week_6::tx::{create_unsigned_tx, sign_transaction};
use std::str::FromStr;

#[test]
fn test_sign_segwit_p2wpkh_transaction() {
    let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let keys =
        WalletKeys::from_mnemonic_str(test_mnemonic, "", Network::Regtest, DescriptorType::Wpkh)
            .unwrap();

    let receive_addr = keys.derive_address(false, 0).unwrap();
    let change_addr = keys.derive_address(true, 0).unwrap();

    let utxo = UtxoRecord {
        txid: Txid::from_str("3333333333333333333333333333333333333333333333333333333333333333")
            .unwrap(),
        vout: 0,
        amount_sats: 100_000,
        script_pubkey: receive_addr.script_pubkey(),
        address: receive_addr.to_string(),
        is_change: false,
        derivation_index: 0,
        height: Some(50),
        is_spent: false,
    };

    let unsigned = create_unsigned_tx(
        std::slice::from_ref(&utxo),
        &receive_addr,
        60_000,
        Some(&change_addr),
        39_000,
    )
    .unwrap();

    assert_eq!(unsigned.input.len(), 1);
    assert_eq!(unsigned.output.len(), 2);
    assert!(unsigned.input[0].witness.is_empty());

    let signed = sign_transaction(unsigned, &[utxo], &keys).unwrap();

    assert_eq!(signed.transaction.input[0].witness.len(), 2); // [sig, pubkey]
    assert_eq!(signed.fee_sats, 1000);
    assert!(signed.vsize > 0);
    assert!(signed.weight.to_wu() > 0);
}

#[test]
fn test_sign_taproot_p2tr_transaction() {
    let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let keys =
        WalletKeys::from_mnemonic_str(test_mnemonic, "", Network::Regtest, DescriptorType::Tr)
            .unwrap();

    let receive_addr = keys.derive_address(false, 0).unwrap();
    let change_addr = keys.derive_address(true, 0).unwrap();

    let utxo = UtxoRecord {
        txid: Txid::from_str("4444444444444444444444444444444444444444444444444444444444444444")
            .unwrap(),
        vout: 0,
        amount_sats: 100_000,
        script_pubkey: receive_addr.script_pubkey(),
        address: receive_addr.to_string(),
        is_change: false,
        derivation_index: 0,
        height: Some(50),
        is_spent: false,
    };

    let unsigned = create_unsigned_tx(
        std::slice::from_ref(&utxo),
        &receive_addr,
        60_000,
        Some(&change_addr),
        39_000,
    )
    .unwrap();

    assert_eq!(unsigned.input.len(), 1);
    assert_eq!(unsigned.output.len(), 2);
    assert!(unsigned.input[0].witness.is_empty());

    let signed = sign_transaction(unsigned, &[utxo], &keys).unwrap();

    assert_eq!(signed.transaction.input[0].witness.len(), 1); // [schnorr_sig]
    assert_eq!(signed.transaction.input[0].witness[0].len(), 64); // 64-byte Schnorr signature
    assert_eq!(signed.fee_sats, 1000);
    assert!(signed.vsize > 0);
}
