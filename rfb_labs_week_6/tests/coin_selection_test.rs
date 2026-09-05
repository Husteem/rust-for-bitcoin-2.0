use bitcoin::{ScriptBuf, Txid};
use rfb_labs_week_6::coin_selection::{CoinSelectionStrategy, select_coins};
use rfb_labs_week_6::config::DescriptorType;
use rfb_labs_week_6::db::UtxoRecord;
use std::str::FromStr;

fn make_utxo(vout: u32, amount_sats: u64) -> UtxoRecord {
    let txid =
        Txid::from_str("2222222222222222222222222222222222222222222222222222222222222222").unwrap();
    UtxoRecord {
        txid,
        vout,
        amount_sats,
        script_pubkey: ScriptBuf::new(),
        address: "bcrt1qtest".to_string(),
        is_change: false,
        derivation_index: vout,
        height: Some(100),
        is_spent: false,
    }
}

#[test]
fn test_coin_selection_largest_first() {
    let utxos = vec![
        make_utxo(0, 10_000),
        make_utxo(1, 50_000),
        make_utxo(2, 20_000),
    ];

    let result = select_coins(
        &utxos,
        25_000,
        2,
        DescriptorType::Wpkh,
        CoinSelectionStrategy::LargestFirst,
    )
    .unwrap();

    assert_eq!(result.selected_utxos.len(), 1);
    assert_eq!(result.selected_utxos[0].amount_sats, 50_000);
    assert!(result.create_change_output);
    assert!(result.change_sats > 0);
}

#[test]
fn test_coin_selection_smallest_first() {
    let utxos = vec![
        make_utxo(0, 10_000),
        make_utxo(1, 50_000),
        make_utxo(2, 20_000),
    ];

    let result = select_coins(
        &utxos,
        25_000,
        2,
        DescriptorType::Wpkh,
        CoinSelectionStrategy::SmallestFirst,
    )
    .unwrap();

    // 10k + 20k = 30k >= 25k + fee
    assert_eq!(result.selected_utxos.len(), 2);
    assert_eq!(result.total_input_sats, 30_000);
    assert!(result.create_change_output);
}

#[test]
fn test_coin_selection_insufficient_funds() {
    let utxos = vec![make_utxo(0, 10_000)];

    let err = select_coins(
        &utxos,
        20_000,
        2,
        DescriptorType::Wpkh,
        CoinSelectionStrategy::LargestFirst,
    );

    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("Insufficient funds"));
}

#[test]
fn test_dust_protection_drops_change() {
    // If change is less than 546 sats, change output is dropped and added to miner fee
    let utxos = vec![make_utxo(0, 10_200)];

    let result = select_coins(
        &utxos,
        10_000,
        1,
        DescriptorType::Wpkh,
        CoinSelectionStrategy::LargestFirst,
    )
    .unwrap();

    assert_eq!(result.selected_utxos.len(), 1);
    // Leftover ~200 - fee is less than 546 -> no change output created
    assert!(!result.create_change_output);
    assert_eq!(result.change_sats, 0);
    assert_eq!(result.fee_sats, 200);
}
