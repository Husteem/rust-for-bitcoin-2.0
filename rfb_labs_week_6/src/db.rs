use anyhow::{Context, Result};
use bitcoin::{OutPoint, ScriptBuf, Txid};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoRecord {
    pub txid: Txid,
    pub vout: u32,
    pub amount_sats: u64,
    pub script_pubkey: ScriptBuf,
    pub address: String,
    pub is_change: bool,
    pub derivation_index: u32,
    pub height: Option<u32>,
    pub is_spent: bool,
}

impl UtxoRecord {
    pub fn outpoint(&self) -> OutPoint {
        OutPoint {
            txid: self.txid,
            vout: self.vout,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxRecord {
    pub txid: Txid,
    pub raw_tx_hex: String,
    pub fee_sats: Option<u64>,
    pub height: Option<u32>,
    pub is_outgoing: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedAddressRecord {
    pub address: String,
    pub script_pubkey_hex: String,
    pub is_change: bool,
    pub index_num: u32,
    pub used: bool,
}

pub struct WalletDb {
    conn: Connection,
}

impl WalletDb {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open SQLite database")?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn =
            Connection::open_in_memory().context("Failed to open in-memory SQLite database")?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS wallet_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS keychain_indices (
                keychain INTEGER PRIMARY KEY, -- 0: external/receive, 1: internal/change
                next_index INTEGER NOT NULL DEFAULT 0
            );

            INSERT OR IGNORE INTO keychain_indices (keychain, next_index) VALUES (0, 0);
            INSERT OR IGNORE INTO keychain_indices (keychain, next_index) VALUES (1, 0);

            CREATE TABLE IF NOT EXISTS derived_addresses (
                address TEXT PRIMARY KEY,
                script_pubkey_hex TEXT NOT NULL,
                is_change INTEGER NOT NULL,
                index_num INTEGER NOT NULL,
                used INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS utxos (
                txid TEXT NOT NULL,
                vout INTEGER NOT NULL,
                amount_sats INTEGER NOT NULL,
                script_pubkey_hex TEXT NOT NULL,
                address TEXT NOT NULL,
                is_change INTEGER NOT NULL,
                derivation_index INTEGER NOT NULL,
                height INTEGER,
                is_spent INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (txid, vout)
            );

            CREATE TABLE IF NOT EXISTS transactions (
                txid TEXT PRIMARY KEY,
                raw_tx_hex TEXT NOT NULL,
                fee_sats INTEGER,
                height INTEGER,
                is_outgoing INTEGER NOT NULL,
                timestamp INTEGER NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO wallet_meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_meta(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM wallet_meta WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            let val: String = row.get(0)?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    pub fn get_next_index(&self, is_change: bool) -> Result<u32> {
        let keychain = if is_change { 1 } else { 0 };
        let mut stmt = self
            .conn
            .prepare("SELECT next_index FROM keychain_indices WHERE keychain = ?1")?;
        let next_idx: u32 = stmt.query_row(params![keychain], |row| row.get(0))?;
        Ok(next_idx)
    }

    pub fn advance_index(&self, is_change: bool) -> Result<u32> {
        let keychain = if is_change { 1 } else { 0 };
        let current = self.get_next_index(is_change)?;
        self.conn.execute(
            "UPDATE keychain_indices SET next_index = next_index + 1 WHERE keychain = ?1",
            params![keychain],
        )?;
        Ok(current)
    }

    pub fn insert_address(
        &self,
        address: &str,
        script_pubkey_hex: &str,
        is_change: bool,
        index_num: u32,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO derived_addresses (address, script_pubkey_hex, is_change, index_num, used)
             VALUES (?1, ?2, ?3, ?4, 0)",
            params![address, script_pubkey_hex, is_change as i32, index_num],
        )?;
        Ok(())
    }

    pub fn mark_address_used(&self, address: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE derived_addresses SET used = 1 WHERE address = ?1",
            params![address],
        )?;
        Ok(())
    }

    pub fn get_derived_address_by_script_hex(
        &self,
        script_pubkey_hex: &str,
    ) -> Result<Option<DerivedAddressRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT address, script_pubkey_hex, is_change, index_num, used
             FROM derived_addresses WHERE script_pubkey_hex = ?1",
        )?;
        let mut rows = stmt.query(params![script_pubkey_hex])?;
        if let Some(row) = rows.next()? {
            Ok(Some(DerivedAddressRecord {
                address: row.get(0)?,
                script_pubkey_hex: row.get(1)?,
                is_change: row.get::<_, i32>(2)? != 0,
                index_num: row.get(3)?,
                used: row.get::<_, i32>(4)? != 0,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_addresses(&self) -> Result<Vec<DerivedAddressRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT address, script_pubkey_hex, is_change, index_num, used
             FROM derived_addresses ORDER BY is_change ASC, index_num ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DerivedAddressRecord {
                address: row.get(0)?,
                script_pubkey_hex: row.get(1)?,
                is_change: row.get::<_, i32>(2)? != 0,
                index_num: row.get(3)?,
                used: row.get::<_, i32>(4)? != 0,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn insert_or_update_utxo(&self, utxo: &UtxoRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO utxos (txid, vout, amount_sats, script_pubkey_hex, address, is_change, derivation_index, height, is_spent)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(txid, vout) DO UPDATE SET
                amount_sats = ?3,
                script_pubkey_hex = ?4,
                address = ?5,
                is_change = ?6,
                derivation_index = ?7,
                height = ?8,
                is_spent = ?9",
            params![
                utxo.txid.to_string(),
                utxo.vout,
                utxo.amount_sats as i64,
                hex::encode(utxo.script_pubkey.as_bytes()),
                utxo.address,
                utxo.is_change as i32,
                utxo.derivation_index,
                utxo.height,
                utxo.is_spent as i32,
            ],
        )?;
        Ok(())
    }

    pub fn mark_utxo_spent(&self, txid: &Txid, vout: u32) -> Result<()> {
        self.conn.execute(
            "UPDATE utxos SET is_spent = 1 WHERE txid = ?1 AND vout = ?2",
            params![txid.to_string(), vout],
        )?;
        Ok(())
    }

    pub fn get_unspent_utxos(&self) -> Result<Vec<UtxoRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT txid, vout, amount_sats, script_pubkey_hex, address, is_change, derivation_index, height, is_spent
             FROM utxos WHERE is_spent = 0 ORDER BY amount_sats DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let txid_str: String = row.get(0)?;
            let vout: u32 = row.get(1)?;
            let amount_sats: i64 = row.get(2)?;
            let script_hex: String = row.get(3)?;
            let address: String = row.get(4)?;
            let is_change: i32 = row.get(5)?;
            let derivation_index: u32 = row.get(6)?;
            let height: Option<u32> = row.get(7)?;
            let is_spent: i32 = row.get(8)?;

            let txid = Txid::from_str(&txid_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            let script_bytes = hex::decode(&script_hex).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            Ok(UtxoRecord {
                txid,
                vout,
                amount_sats: amount_sats as u64,
                script_pubkey: ScriptBuf::from(script_bytes),
                address,
                is_change: is_change != 0,
                derivation_index,
                height,
                is_spent: is_spent != 0,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_balance(&self) -> Result<(u64, u64)> {
        let unspent = self.get_unspent_utxos()?;
        let mut confirmed = 0u64;
        let mut unconfirmed = 0u64;

        for utxo in unspent {
            if utxo.height.is_some() {
                confirmed += utxo.amount_sats;
            } else {
                unconfirmed += utxo.amount_sats;
            }
        }
        Ok((confirmed, unconfirmed))
    }

    pub fn insert_transaction(&self, tx: &TxRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO transactions (txid, raw_tx_hex, fee_sats, height, is_outgoing, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(txid) DO UPDATE SET
                raw_tx_hex = ?2,
                fee_sats = ?3,
                height = ?4,
                is_outgoing = ?5,
                timestamp = ?6",
            params![
                tx.txid.to_string(),
                tx.raw_tx_hex,
                tx.fee_sats.map(|f| f as i64),
                tx.height,
                tx.is_outgoing as i32,
                tx.timestamp as i64,
            ],
        )?;
        Ok(())
    }

    pub fn get_transactions(&self) -> Result<Vec<TxRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT txid, raw_tx_hex, fee_sats, height, is_outgoing, timestamp
             FROM transactions ORDER BY timestamp DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let txid_str: String = row.get(0)?;
            let raw_tx_hex: String = row.get(1)?;
            let fee_sats: Option<i64> = row.get(2)?;
            let height: Option<u32> = row.get(3)?;
            let is_outgoing: i32 = row.get(4)?;
            let timestamp: i64 = row.get(5)?;

            let txid = Txid::from_str(&txid_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            Ok(TxRecord {
                txid,
                raw_tx_hex,
                fee_sats: fee_sats.map(|f| f as u64),
                height,
                is_outgoing: is_outgoing != 0,
                timestamp: timestamp as u64,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }
}
