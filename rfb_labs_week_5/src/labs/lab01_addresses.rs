//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use bitcoin::Address;
use bitcoin::Network;
use std::str::FromStr;

use crate::model::{AddressFormat, AddressReport};
use crate::{LabError, LabResult};

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    if address.starts_with('1') || address.starts_with('m') || address.starts_with('n') {
        AddressFormat::P2pkh
    } else if address.starts_with('3') || address.starts_with('2') {
        AddressFormat::P2sh
    } else if address.starts_with("bc1q")
        || address.starts_with("tb1q")
        || address.starts_with("bcrt1q")
    {
        AddressFormat::P2wpkh
    } else if address.starts_with("bc1p")
        || address.starts_with("tb1p")
        || address.starts_with("bcrt1p")
    {
        AddressFormat::P2tr
    } else {
        AddressFormat::Unknown
    }
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    match network {
        Network::Bitcoin => match format {
            AddressFormat::P2pkh => Some("1"),
            AddressFormat::P2sh => Some("3"),
            AddressFormat::P2wpkh => Some("bc1q"),
            AddressFormat::P2tr => Some("bc1p"),
            AddressFormat::Unknown => None,
        },
        Network::Testnet | Network::Signet => match format {
            AddressFormat::P2pkh => Some("m/n"),
            AddressFormat::P2sh => Some("2"),
            AddressFormat::P2wpkh => Some("tb1q"),
            AddressFormat::P2tr => Some("tb1p"),
            AddressFormat::Unknown => None,
        },
        Network::Regtest => match format {
            AddressFormat::P2pkh => Some("m/n"),
            AddressFormat::P2sh => Some("2"),
            AddressFormat::P2wpkh => Some("bcrt1q"),
            AddressFormat::P2tr => Some("bcrt1p"),
            AddressFormat::Unknown => None,
        },
        _ => None,
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    let parsed = Address::from_str(address).map_err(|e| LabError::InvalidAddress(e.to_string()))?;

    let checked = parsed
        .require_network(network)
        .map_err(|e| LabError::WrongNetwork(e.to_string()))?;

    let format = match checked.address_type() {
        Some(bitcoin::AddressType::P2pkh) => AddressFormat::P2pkh,
        Some(bitcoin::AddressType::P2sh) => AddressFormat::P2sh,
        Some(bitcoin::AddressType::P2wpkh) => AddressFormat::P2wpkh,
        Some(bitcoin::AddressType::P2tr) => AddressFormat::P2tr,
        _ => AddressFormat::Unknown,
    };

    let script_pubkey_hex = checked.script_pubkey().to_hex_string();
    let network_str = network.to_string().to_lowercase();

    Ok(AddressReport {
        address: address.to_string(),
        network: network_str,
        format,
        script_pubkey_hex,
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    let report = inspect_address(address, network)?;
    Ok(report.script_pubkey_hex)
}
