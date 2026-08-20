#![allow(unused)]
#![allow(dead_code)]
use sha2::{Digest, Sha256};
use std::fmt;
use std::io::{Error, Read};
use transaction::{Amount, Input, Output, Transaction, Txid};

pub mod transaction;

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    let mut buf = [0u8; 4];
    bytes_slice.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    let mut buf = [0u8; 8];
    transaction_bytes.read_exact(&mut buf).unwrap();
    u64::from_le_bytes(buf)
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let mut buf = [0u8; 8];
    transaction_bytes.read_exact(&mut buf)?;
    Ok(Amount::from_sat(u64::from_le_bytes(buf)))
}

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let mut header = [0u8; 1];
    transaction_bytes.read_exact(&mut header)?;
    match header[0] {
        0..=0xfc => Ok(header[0] as u64),
        0xfd => {
            let mut buf = [0u8; 2];
            transaction_bytes.read_exact(&mut buf)?;
            Ok(u16::from_le_bytes(buf) as u64)
        }
        0xfe => {
            let mut buf = [0u8; 4];
            transaction_bytes.read_exact(&mut buf)?;
            Ok(u32::from_le_bytes(buf) as u64)
        }
        _ => {
            let mut buf = [0u8; 8];
            transaction_bytes.read_exact(&mut buf)?;
            Ok(u64::from_le_bytes(buf))
        }
    }
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    let mut buf = [0u8; 32];
    transaction_bytes.read_exact(&mut buf)?;
    Ok(Txid::from_bytes(buf))
}

fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let len = read_compact_size(transaction_bytes)? as usize;
    let mut buf = vec![0u8; len];
    transaction_bytes.read_exact(&mut buf)?;
    Ok(hex::encode(buf))
}

fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

fn read_version(transaction_hex: &str) -> u32 {
    if let Some(bytes) = hex::decode(transaction_hex).ok().filter(|b| b.len() >= 4) {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&bytes[0..4]);
        u32::from_le_bytes(buf)
    } else {
        0
    }
}

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let hash1 = Sha256::digest(row_transaction_bytes);
    let hash2 = Sha256::digest(hash1);
    let mut txid = [0u8; 32];
    txid.copy_from_slice(&hash2);
    Ok(Txid::from_bytes(txid))
}

fn write_varint(w: &mut Vec<u8>, val: u64) {
    match val {
        0..=0xfc => w.push(val as u8),
        0xfd..=0xffff => {
            w.push(0xfd);
            w.extend_from_slice(&(val as u16).to_le_bytes());
        }
        0x10000..=0xffffffff => {
            w.push(0xfe);
            w.extend_from_slice(&(val as u32).to_le_bytes());
        }
        _ => {
            w.push(0xff);
            w.extend_from_slice(&val.to_le_bytes());
        }
    }
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = hex::decode(transaction_hex.trim())?;
    let mut bytes_slice = &bytes[..];

    let version = read_u32(&mut bytes_slice)?;

    let is_segwit = if bytes_slice.len() >= 2 && bytes_slice[0] == 0x00 && bytes_slice[1] == 0x01 {
        bytes_slice = &bytes_slice[2..];
        true
    } else {
        false
    };

    let in_count = read_compact_size(&mut bytes_slice)?;
    let mut inputs = Vec::with_capacity(in_count as usize);
    for _ in 0..in_count {
        let txid = read_txid(&mut bytes_slice)?;
        let output_index = read_u32(&mut bytes_slice)?;
        let script_len = read_compact_size(&mut bytes_slice)? as usize;
        let mut script_sig = vec![0u8; script_len];
        bytes_slice.read_exact(&mut script_sig)?;
        let sequence = read_u32(&mut bytes_slice)?;
        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
        });
    }

    let out_count = read_compact_size(&mut bytes_slice)?;
    let mut outputs = Vec::with_capacity(out_count as usize);
    for _ in 0..out_count {
        let amount = read_amount(&mut bytes_slice)?;
        let script_len = read_compact_size(&mut bytes_slice)? as usize;
        let mut script_pubkey = vec![0u8; script_len];
        bytes_slice.read_exact(&mut script_pubkey)?;
        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    if is_segwit {
        for _ in 0..in_count {
            let witness_count = read_compact_size(&mut bytes_slice)?;
            for _ in 0..witness_count {
                let item_len = read_compact_size(&mut bytes_slice)? as usize;
                let mut item_buf = vec![0u8; item_len];
                bytes_slice.read_exact(&mut item_buf)?;
            }
        }
    }

    let lock_time = read_u32(&mut bytes_slice)?;

    let mut legacy_bytes = Vec::new();
    legacy_bytes.extend_from_slice(&version.to_le_bytes());
    write_varint(&mut legacy_bytes, in_count);
    for input in &inputs {
        legacy_bytes.extend_from_slice(&input.txid.0);
        legacy_bytes.extend_from_slice(&input.output_index.to_le_bytes());
        write_varint(&mut legacy_bytes, input.script_sig.len() as u64);
        legacy_bytes.extend_from_slice(&input.script_sig);
        legacy_bytes.extend_from_slice(&input.sequence.to_le_bytes());
    }
    write_varint(&mut legacy_bytes, out_count);
    for output in &outputs {
        legacy_bytes.extend_from_slice(&output.amount.0.to_le_bytes());
        write_varint(&mut legacy_bytes, output.script_pubkey.len() as u64);
        legacy_bytes.extend_from_slice(&output.script_pubkey);
    }
    legacy_bytes.extend_from_slice(&lock_time.to_le_bytes());

    let transaction_id = hash_row_transaction(&legacy_bytes)?;

    let transaction = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    let json = serde_json::to_string_pretty(&transaction)?;
    Ok(json)
}
