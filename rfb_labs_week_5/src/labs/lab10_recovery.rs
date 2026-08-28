//! Lab 10 — prove deterministic recovery across BIP44, BIP49, and BIP84.

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network};
use std::str::FromStr;

use crate::model::{AddressFormat, DerivedAddressSet};
use crate::{LabError, LabResult};

/// Derive one address from an arbitrary full path and selected script family.
pub fn derive_address_for_path(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<String> {
    let parsed =
        Mnemonic::from_str(mnemonic).map_err(|e| LabError::InvalidMnemonic(e.to_string()))?;
    let seed = parsed.to_seed(passphrase);
    let master =
        Xpriv::new_master(network, &seed).map_err(|e| LabError::Derivation(e.to_string()))?;

    let derivation_path =
        DerivationPath::from_str(path).map_err(|e| LabError::InvalidPath(e.to_string()))?;

    let secp = Secp256k1::new();
    let child_xpriv = master
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| LabError::Derivation(e.to_string()))?;
    let child_xpub = Xpub::from_priv(&secp, &child_xpriv);
    let compressed = child_xpub.to_pub();

    let address = match format {
        AddressFormat::P2pkh => Address::p2pkh(compressed, network).to_string(),
        AddressFormat::P2sh => {
            let wpkh_script = Address::p2wpkh(&compressed, network).script_pubkey();
            Address::p2sh(&wpkh_script, network)
                .map_err(|e| LabError::InvalidScript(e.to_string()))?
                .to_string()
        }
        AddressFormat::P2wpkh => Address::p2wpkh(&compressed, network).to_string(),
        _ => {
            return Err(LabError::InvalidAddress(
                "unsupported address format".into(),
            ))
        }
    };

    Ok(address)
}

/// Derive index `n` on the BIP44, BIP49, and BIP84 receive branches.
pub fn derive_address_set(
    mnemonic: &str,
    passphrase: &str,
    account: u32,
    index: u32,
    network: Network,
) -> LabResult<DerivedAddressSet> {
    let coin_type = match network {
        Network::Bitcoin => 0,
        _ => 1,
    };

    let bip44_path = format!("m/44'/{coin_type}'/{account}'/0/{index}");
    let bip49_path = format!("m/49'/{coin_type}'/{account}'/0/{index}");
    let bip84_path = format!("m/84'/{coin_type}'/{account}'/0/{index}");

    let bip44_p2pkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &bip44_path,
        AddressFormat::P2pkh,
        network,
    )?;
    let bip49_p2sh_p2wpkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &bip49_path,
        AddressFormat::P2sh,
        network,
    )?;
    let bip84_p2wpkh = derive_address_for_path(
        mnemonic,
        passphrase,
        &bip84_path,
        AddressFormat::P2wpkh,
        network,
    )?;

    Ok(DerivedAddressSet {
        bip44_p2pkh,
        bip49_p2sh_p2wpkh,
        bip84_p2wpkh,
    })
}

/// Prove that identical mnemonic, passphrase, path, and network reproduce an address.
pub fn recovery_is_repeatable(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<bool> {
    let first = derive_address_for_path(mnemonic, passphrase, path, format, network)?;
    let second = derive_address_for_path(mnemonic, passphrase, path, format, network)?;
    Ok(first == second)
}

/// Prove that changing only the final index selects a different address.
pub fn changing_index_changes_address(
    mnemonic: &str,
    passphrase: &str,
    first_path: &str,
    second_path: &str,
    format: AddressFormat,
    network: Network,
) -> LabResult<bool> {
    let first = derive_address_for_path(mnemonic, passphrase, first_path, format, network)?;
    let second = derive_address_for_path(mnemonic, passphrase, second_path, format, network)?;
    Ok(first != second)
}
