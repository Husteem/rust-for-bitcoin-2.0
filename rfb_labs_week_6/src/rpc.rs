use anyhow::{Context, Result};
use bitcoin::{Transaction, Txid};
use bitcoincore_rpc::json::{GetBlockchainInfoResult, ScanTxOutRequest};
use bitcoincore_rpc::{Auth, Client, RpcApi};

use crate::config::RpcConfig;
use crate::db::{DerivedAddressRecord, UtxoRecord, WalletDb};

pub struct BitcoinRpcClient {
    client: Client,
}

impl BitcoinRpcClient {
    pub fn new(config: &RpcConfig) -> Result<Self> {
        let auth = if let Some(ref cookie) = config.cookie_file {
            Auth::CookieFile(cookie.clone())
        } else if let (Some(user), Some(pass)) = (&config.user, &config.pass) {
            Auth::UserPass(user.clone(), pass.clone())
        } else {
            Auth::None
        };

        let client = Client::new(&config.url, auth)
            .context("Failed to initialize Bitcoin Core RPC client")?;

        Ok(Self { client })
    }

    pub fn get_blockchain_info(&self) -> Result<GetBlockchainInfoResult> {
        self.client
            .get_blockchain_info()
            .context("Failed to get blockchain info from Bitcoin Core")
    }

    pub fn get_block_count(&self) -> Result<u64> {
        self.client
            .get_block_count()
            .context("Failed to get block count from Bitcoin Core")
    }

    pub fn broadcast_transaction(&self, tx: &Transaction) -> Result<Txid> {
        self.client
            .send_raw_transaction(tx)
            .context("Failed to broadcast raw transaction via Bitcoin Core RPC")
    }

    /// Sync UTXOs for all derived addresses using scantxoutset RPC
    pub fn sync_wallet_utxos(
        &self,
        db: &WalletDb,
        addresses: &[DerivedAddressRecord],
    ) -> Result<usize> {
        if addresses.is_empty() {
            return Ok(0);
        }

        let mut scan_descriptors = Vec::new();
        for addr_record in addresses {
            scan_descriptors.push(ScanTxOutRequest::Single(format!(
                "addr({})",
                addr_record.address
            )));
        }

        let scan_result = self
            .client
            .scan_tx_out_set_blocking(&scan_descriptors)
            .context("Failed executing scantxoutset on Bitcoin Core")?;

        let current_height = self.get_block_count()? as u32;
        let mut new_utxo_count = 0;

        for unspent in scan_result.unspents {
            let txid = unspent.txid;
            let vout = unspent.vout;
            let amount_sats = unspent.amount.to_sat();
            let script_pubkey = unspent.script_pub_key;
            let script_hex = hex::encode(script_pubkey.as_bytes());

            // Match with derived address
            if let Some(addr_rec) = db.get_derived_address_by_script_hex(&script_hex)? {
                let height = if unspent.height > 0 {
                    Some(unspent.height as u32)
                } else {
                    Some(current_height)
                };

                let utxo = UtxoRecord {
                    txid,
                    vout,
                    amount_sats,
                    script_pubkey,
                    address: addr_rec.address.clone(),
                    is_change: addr_rec.is_change,
                    derivation_index: addr_rec.index_num,
                    height,
                    is_spent: false,
                };

                db.insert_or_update_utxo(&utxo)?;
                db.mark_address_used(&addr_rec.address)?;
                new_utxo_count += 1;
            }
        }

        Ok(new_utxo_count)
    }
}
