use bitcoin::{ScriptBuf, Txid};
use rfb_labs_week_6::db::{TxRecord, UtxoRecord, WalletDb};
use std::str::FromStr;
use tempfile::NamedTempFile;

#[test]
fn test_db_persistence_across_reconnect() {
    let tmp_file = NamedTempFile::new().unwrap();
    let db_path = tmp_file.path();

    // 1. Initial write
    {
        let db = WalletDb::open(db_path).unwrap();
        db.set_meta("network", "regtest").unwrap();
        db.set_meta("mnemonic", "test phrase words").unwrap();

        let idx0 = db.advance_index(false).unwrap();
        assert_eq!(idx0, 0);
        let idx1 = db.advance_index(false).unwrap();
        assert_eq!(idx1, 1);

        db.insert_address("bcrt1qtestaddress", "0014aabb", false, 0)
            .unwrap();

        let txid =
            Txid::from_str("1111111111111111111111111111111111111111111111111111111111111111")
                .unwrap();
        let utxo = UtxoRecord {
            txid,
            vout: 0,
            amount_sats: 50_000,
            script_pubkey: ScriptBuf::from(vec![0x00, 0x14, 0xaa, 0xbb]),
            address: "bcrt1qtestaddress".to_string(),
            is_change: false,
            derivation_index: 0,
            height: Some(101),
            is_spent: false,
        };
        db.insert_or_update_utxo(&utxo).unwrap();

        let (confirmed, unconfirmed) = db.get_balance().unwrap();
        assert_eq!(confirmed, 50_000);
        assert_eq!(unconfirmed, 0);
    }

    // 2. Reopen and verify persistence
    {
        let db = WalletDb::open(db_path).unwrap();
        let net = db.get_meta("network").unwrap().unwrap();
        assert_eq!(net, "regtest");

        let phrase = db.get_meta("mnemonic").unwrap().unwrap();
        assert_eq!(phrase, "test phrase words");

        let next_idx = db.get_next_index(false).unwrap();
        assert_eq!(next_idx, 2);

        let utxos = db.get_unspent_utxos().unwrap();
        assert_eq!(utxos.len(), 1);
        assert_eq!(utxos[0].amount_sats, 50_000);

        let txid =
            Txid::from_str("1111111111111111111111111111111111111111111111111111111111111111")
                .unwrap();
        db.mark_utxo_spent(&txid, 0).unwrap();

        let (confirmed_after, _) = db.get_balance().unwrap();
        assert_eq!(confirmed_after, 0);

        let tx_rec = TxRecord {
            txid,
            raw_tx_hex: "0200000000".to_string(),
            fee_sats: Some(150),
            height: Some(102),
            is_outgoing: true,
            timestamp: 1700000000,
        };
        db.insert_transaction(&tx_rec).unwrap();

        let txs = db.get_transactions().unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].fee_sats, Some(150));
    }
}
