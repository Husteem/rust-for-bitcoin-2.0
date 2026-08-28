//! Lab 09 — decode BIP44 paths and derive the selected address.

use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network};
use std::str::FromStr;

use crate::model::Bip44PathInfo;
use crate::{LabError, LabResult};

fn raw_child_info(cn: ChildNumber) -> (u32, bool) {
    match cn {
        ChildNumber::Normal { index } => (index, false),
        ChildNumber::Hardened { index } => (index, true),
    }
}

/// Parse `m / purpose' / coin' / account' / change / index`.
pub fn decode_bip44_path(path: &str) -> LabResult<Bip44PathInfo> {
    let derivation_path =
        DerivationPath::from_str(path).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    let steps: Vec<ChildNumber> = derivation_path.into_iter().copied().collect();

    if steps.len() != 5 {
        return Err(LabError::InvalidPath(
            "BIP44 path must have exactly 5 levels (purpose, coin, account, change, index)".into(),
        ));
    }

    let (purpose, p_hard) = raw_child_info(steps[0]);
    let (coin_type, c_hard) = raw_child_info(steps[1]);
    let (account, a_hard) = raw_child_info(steps[2]);
    let (change, ch_hard) = raw_child_info(steps[3]);
    let (index, i_hard) = raw_child_info(steps[4]);

    if !p_hard || !c_hard || !a_hard || ch_hard || i_hard {
        return Err(LabError::InvalidPath(
            "BIP44 requires hardened purpose, coin, account and unhardened change, index".into(),
        ));
    }

    Ok(Bip44PathInfo {
        purpose,
        coin_type,
        account,
        change,
        index,
    })
}

fn ordinal_word(n: u32) -> String {
    match n {
        0 => "first".to_string(),
        1 => "second".to_string(),
        2 => "third".to_string(),
        3 => "fourth".to_string(),
        4 => "fifth".to_string(),
        5 => "sixth".to_string(),
        6 => "seventh".to_string(),
        7 => "eighth".to_string(),
        8 => "ninth".to_string(),
        9 => "tenth".to_string(),
        other => format!("{}th", other + 1),
    }
}

/// Translate a decoded path into a concise English explanation.
pub fn describe_bip44_path(info: &Bip44PathInfo) -> String {
    format!(
        "BIP44 path: purpose {}', coin type {}', {} account ({}), {} chain ({}), {} address ({})",
        info.purpose,
        info.coin_type,
        ordinal_word(info.account),
        info.account,
        if info.change == 1 {
            "change"
        } else {
            "receive"
        },
        info.change,
        ordinal_word(info.index),
        info.index
    )
}

/// Return the same BIP44 path with only its final address index changed.
pub fn with_address_index(path: &str, new_index: u32) -> LabResult<String> {
    let mut parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 2 {
        return Err(LabError::InvalidPath(
            "Path must contain at least one step".into(),
        ));
    }
    let new_idx_str = new_index.to_string();
    let last_idx = parts.len() - 1;
    parts[last_idx] = &new_idx_str;
    let new_path = parts.join("/");
    let _ =
        DerivationPath::from_str(&new_path).map_err(|e| LabError::InvalidPath(e.to_string()))?;
    Ok(new_path)
}

/// Derive the P2PKH address selected by a BIP44 path.
pub fn derive_bip44_address(
    mnemonic: &str,
    passphrase: &str,
    path: &str,
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
    let public_key = child_xpub.to_pub();

    Ok(Address::p2pkh(public_key, network).to_string())
}
