use bitcoin::Network;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DescriptorType {
    Wpkh, // BIP84 Native SegWit
    Tr,   // BIP86 Taproot
}

impl std::fmt::Display for DescriptorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DescriptorType::Wpkh => write!(f, "wpkh"),
            DescriptorType::Tr => write!(f, "tr"),
        }
    }
}

impl std::str::FromStr for DescriptorType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "wpkh" | "bip84" | "segwit" => Ok(DescriptorType::Wpkh),
            "tr" | "bip86" | "taproot" => Ok(DescriptorType::Tr),
            other => anyhow::bail!("Unsupported descriptor type: {}. Use 'wpkh' or 'tr'", other),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub url: String,
    pub user: Option<String>,
    pub pass: Option<String>,
    pub cookie_file: Option<PathBuf>,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:18443".to_string(),
            user: Some("user".to_string()),
            pass: Some("password".to_string()),
            cookie_file: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub network: Network,
    pub descriptor_type: DescriptorType,
    pub db_path: PathBuf,
    pub rpc: RpcConfig,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            network: Network::Regtest,
            descriptor_type: DescriptorType::Wpkh,
            db_path: PathBuf::from("wallet.db"),
            rpc: RpcConfig::default(),
        }
    }
}
