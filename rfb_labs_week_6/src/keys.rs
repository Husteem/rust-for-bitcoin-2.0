use anyhow::{Context, Result, anyhow};
use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpriv, Xpub};
use bitcoin::key::Keypair;
use bitcoin::secp256k1::rand::RngCore;
use bitcoin::secp256k1::{Secp256k1, Signing, Verification};
use bitcoin::{Address, CompressedPublicKey, Network};
use std::str::FromStr;

use crate::config::DescriptorType;

#[derive(Debug, Clone)]
pub struct WalletKeys {
    pub network: Network,
    pub descriptor_type: DescriptorType,
    pub mnemonic: Mnemonic,
    pub master_xpriv: Xpriv,
    pub account_xpriv: Xpriv,
    pub account_xpub: Xpub,
    pub master_fingerprint: Fingerprint,
    pub derivation_path_prefix: DerivationPath,
}

impl WalletKeys {
    pub fn new_random(network: Network, descriptor_type: DescriptorType) -> Result<Self> {
        let mut entropy = [0u8; 16];
        bitcoin::secp256k1::rand::thread_rng().fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy(&entropy)
            .map_err(|e| anyhow!("Failed to generate mnemonic from entropy: {:?}", e))?;
        Self::from_mnemonic(mnemonic, "", network, descriptor_type)
    }

    pub fn from_mnemonic_str(
        words: &str,
        passphrase: &str,
        network: Network,
        descriptor_type: DescriptorType,
    ) -> Result<Self> {
        let mnemonic = Mnemonic::from_str(words).context("Invalid BIP39 mnemonic words")?;
        Self::from_mnemonic(mnemonic, passphrase, network, descriptor_type)
    }

    pub fn from_mnemonic(
        mnemonic: Mnemonic,
        passphrase: &str,
        network: Network,
        descriptor_type: DescriptorType,
    ) -> Result<Self> {
        let secp = Secp256k1::new();
        let seed = mnemonic.to_seed(passphrase);
        let master_xpriv = Xpriv::new_master(network, &seed)
            .map_err(|e| anyhow!("Failed to derive master xpriv: {:?}", e))?;
        let master_fingerprint = master_xpriv.fingerprint(&secp);

        let coin_type = match network {
            Network::Bitcoin => 0,
            _ => 1,
        };

        let purpose = match descriptor_type {
            DescriptorType::Wpkh => 84,
            DescriptorType::Tr => 86,
        };

        let path_str = format!("m/{}'/{}'/0'", purpose, coin_type);
        let derivation_path_prefix = DerivationPath::from_str(&path_str)
            .context("Failed to parse derivation path prefix")?;

        let account_xpriv = master_xpriv
            .derive_priv(&secp, &derivation_path_prefix)
            .map_err(|e| anyhow!("Failed to derive account xpriv: {:?}", e))?;
        let account_xpub = Xpub::from_priv(&secp, &account_xpriv);

        Ok(Self {
            network,
            descriptor_type,
            mnemonic,
            master_xpriv,
            account_xpriv,
            account_xpub,
            master_fingerprint,
            derivation_path_prefix,
        })
    }

    /// Returns descriptor string for external (receive) or internal (change) keychain.
    pub fn descriptor(&self, is_change: bool) -> String {
        let keychain_idx = if is_change { 1 } else { 0 };
        let coin_type = match self.network {
            Network::Bitcoin => 0,
            _ => 1,
        };
        let purpose = match self.descriptor_type {
            DescriptorType::Wpkh => 84,
            DescriptorType::Tr => 86,
        };

        let origin = format!("{}/{}'/{}'/0'", self.master_fingerprint, purpose, coin_type);

        match self.descriptor_type {
            DescriptorType::Wpkh => {
                format!("wpkh([{}]{}/{}/*)", origin, self.account_xpub, keychain_idx)
            }
            DescriptorType::Tr => {
                format!("tr([{}]{}/{}/*)", origin, self.account_xpub, keychain_idx)
            }
        }
    }

    /// Derive child private key for specific keychain and index
    pub fn derive_private_key<C: Signing>(
        &self,
        secp: &Secp256k1<C>,
        is_change: bool,
        index: u32,
    ) -> Result<Xpriv> {
        let keychain_idx = if is_change { 1 } else { 0 };
        let path = [
            ChildNumber::from_normal_idx(keychain_idx)
                .map_err(|e| anyhow!("Invalid keychain child number: {:?}", e))?,
            ChildNumber::from_normal_idx(index)
                .map_err(|e| anyhow!("Invalid address index child number: {:?}", e))?,
        ];
        self.account_xpriv
            .derive_priv(secp, &path)
            .map_err(|e| anyhow!("Failed to derive child private key: {:?}", e))
    }

    /// Derive child public key for specific keychain and index
    pub fn derive_public_key<C: Verification>(
        &self,
        secp: &Secp256k1<C>,
        is_change: bool,
        index: u32,
    ) -> Result<Xpub> {
        let keychain_idx = if is_change { 1 } else { 0 };
        let path = [
            ChildNumber::from_normal_idx(keychain_idx)
                .map_err(|e| anyhow!("Invalid keychain child number: {:?}", e))?,
            ChildNumber::from_normal_idx(index)
                .map_err(|e| anyhow!("Invalid address index child number: {:?}", e))?,
        ];
        self.account_xpub
            .derive_pub(secp, &path)
            .map_err(|e| anyhow!("Failed to derive child public key: {:?}", e))
    }

    /// Derive Address for a specific keychain and index
    pub fn derive_address(&self, is_change: bool, index: u32) -> Result<Address> {
        let secp = Secp256k1::new();
        let child_xpub = self.derive_public_key(&secp, is_change, index)?;

        match self.descriptor_type {
            DescriptorType::Wpkh => {
                let compressed_pubkey = CompressedPublicKey(child_xpub.public_key);
                Ok(Address::p2wpkh(&compressed_pubkey, self.network))
            }
            DescriptorType::Tr => {
                let (x_only, _parity) = child_xpub.public_key.x_only_public_key();
                Ok(Address::p2tr(&secp, x_only, None, self.network))
            }
        }
    }

    /// Get Keypair for Taproot signing at a specific keychain and index
    pub fn derive_keypair<C: Signing>(
        &self,
        secp: &Secp256k1<C>,
        is_change: bool,
        index: u32,
    ) -> Result<Keypair> {
        let child_xpriv = self.derive_private_key(secp, is_change, index)?;
        Ok(Keypair::from_secret_key(secp, &child_xpriv.private_key))
    }

    /// Get CompressedPublicKey and SecretKey for SegWit signing
    pub fn derive_secp_keypair<C: Signing>(
        &self,
        secp: &Secp256k1<C>,
        is_change: bool,
        index: u32,
    ) -> Result<(bitcoin::secp256k1::SecretKey, CompressedPublicKey)> {
        let child_xpriv = self.derive_private_key(secp, is_change, index)?;
        let child_xpub = Xpub::from_priv(secp, &child_xpriv);
        let pubkey = CompressedPublicKey(child_xpub.public_key);
        Ok((child_xpriv.private_key, pubkey))
    }
}
