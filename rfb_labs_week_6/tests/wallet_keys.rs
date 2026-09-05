use bitcoin::Network;
use rfb_labs_week_6::config::DescriptorType;
use rfb_labs_week_6::keys::WalletKeys;

#[test]
fn test_mnemonic_generation_and_recovery() {
    let keys = WalletKeys::new_random(Network::Regtest, DescriptorType::Wpkh).unwrap();
    let words = keys.mnemonic.to_string();
    assert_eq!(words.split_whitespace().count(), 12);

    let recovered =
        WalletKeys::from_mnemonic_str(&words, "", Network::Regtest, DescriptorType::Wpkh).unwrap();
    assert_eq!(keys.master_fingerprint, recovered.master_fingerprint);
    assert_eq!(keys.account_xpub, recovered.account_xpub);
}

#[test]
fn test_bip84_segwit_descriptors_and_addresses() {
    let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let keys =
        WalletKeys::from_mnemonic_str(test_mnemonic, "", Network::Regtest, DescriptorType::Wpkh)
            .unwrap();

    let ext_desc = keys.descriptor(false);
    let int_desc = keys.descriptor(true);

    assert!(ext_desc.starts_with("wpkh([73c5da0a/84'/1'/0']"));
    assert!(ext_desc.ends_with("/0/*)"));
    assert!(int_desc.starts_with("wpkh([73c5da0a/84'/1'/0']"));
    assert!(int_desc.ends_with("/1/*)"));

    let receive_0 = keys.derive_address(false, 0).unwrap();
    let receive_1 = keys.derive_address(false, 1).unwrap();
    let change_0 = keys.derive_address(true, 0).unwrap();

    assert!(receive_0.to_string().starts_with("bcrt1q"));
    assert!(receive_1.to_string().starts_with("bcrt1q"));
    assert!(change_0.to_string().starts_with("bcrt1q"));
    assert_ne!(receive_0, receive_1);
    assert_ne!(receive_0, change_0);
}

#[test]
fn test_bip86_taproot_descriptors_and_addresses() {
    let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let keys =
        WalletKeys::from_mnemonic_str(test_mnemonic, "", Network::Regtest, DescriptorType::Tr)
            .unwrap();

    let ext_desc = keys.descriptor(false);
    let int_desc = keys.descriptor(true);

    assert!(ext_desc.starts_with("tr([73c5da0a/86'/1'/0']"));
    assert!(ext_desc.ends_with("/0/*)"));
    assert!(int_desc.starts_with("tr([73c5da0a/86'/1'/0']"));
    assert!(int_desc.ends_with("/1/*)"));

    let receive_0 = keys.derive_address(false, 0).unwrap();
    let receive_1 = keys.derive_address(false, 1).unwrap();
    let change_0 = keys.derive_address(true, 0).unwrap();

    assert!(receive_0.to_string().starts_with("bcrt1p"));
    assert!(receive_1.to_string().starts_with("bcrt1p"));
    assert!(change_0.to_string().starts_with("bcrt1p"));
    assert_ne!(receive_0, receive_1);
    assert_ne!(receive_0, change_0);
}
