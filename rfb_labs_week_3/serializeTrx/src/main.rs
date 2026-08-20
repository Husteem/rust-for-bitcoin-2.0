use clap::Parser;
use std::error::Error;

#[derive(Debug, Clone)]
struct TxInput {
    prev_txid: Vec<u8>,
    vout: u32,
    script_sig: Vec<u8>,
    sequence: u32,
    witness: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
struct TxOutput {
    value: u64,
    script_pubkey: Vec<u8>,
}

#[derive(Debug, Clone)]
struct Transaction {
    version: i32,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    locktime: u32,
    segwit: bool,
}

#[derive(Parser, Debug)]
#[command(
    name = "serializeTrx",
    version = "0.1.0",
    about = "Serializes Bitcoin transactions dynamically",
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, default_value_t = 2)]
    version: i32,

    #[arg(long)]
    segwit: bool,

    /// Inputs in the format 'prev_txid:vout:sequence[:script_sig_hex]'
    #[arg(long = "input", required = true)]
    inputs: Vec<String>,

    /// Outputs in the format 'value_satoshis:script_pubkey_hex'
    #[arg(long = "output", required = true)]
    outputs: Vec<String>,

    /// Witness items in the format 'input_index:witness_item_hex'
    #[arg(long = "witness")]
    witnesses: Vec<String>,

    #[arg(long, default_value_t = 0)]
    locktime: u32,
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let cleaned = hex.trim();
    if !cleaned.len().is_multiple_of(2) {
        return Err("Hex string must have an even length".into());
    }

    let mut bytes = Vec::with_capacity(cleaned.len() / 2);
    for i in (0..cleaned.len()).step_by(2) {
        let byte_str = &cleaned[i..i + 2];
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|_| format!("Invalid hex character sequence: '{}'", byte_str))?;
        bytes.push(byte);
    }

    Ok(bytes)
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn encode_varint(value: usize) -> Vec<u8> {
    match value {
        0..=0xfc => vec![value as u8],
        0xfd..=0xffff => {
            let mut result = vec![0xfd];
            result.extend_from_slice(&(value as u16).to_le_bytes());
            result
        }
        0x10000..=0xffff_ffff => {
            let mut result = vec![0xfe];
            result.extend_from_slice(&(value as u32).to_le_bytes());
            result
        }
        _ => {
            let mut result = vec![0xff];
            result.extend_from_slice(&(value as u64).to_le_bytes());
            result
        }
    }
}

fn serialize_transaction(trx: &Transaction) -> Vec<u8> {
    let mut result = Vec::new();

    // 1. Version
    result.extend_from_slice(&trx.version.to_le_bytes());

    // 2. SegWit Marker and Flag
    if trx.segwit {
        result.push(0x00); // marker
        result.push(0x01); // flag
    }

    // 3. Inputs count & data
    result.extend_from_slice(&encode_varint(trx.inputs.len()));
    for input in &trx.inputs {
        result.extend_from_slice(&input.prev_txid);
        result.extend_from_slice(&input.vout.to_le_bytes());
        result.extend_from_slice(&encode_varint(input.script_sig.len()));
        result.extend_from_slice(&input.script_sig);
        result.extend_from_slice(&input.sequence.to_le_bytes());
    }

    // 4. Outputs count & data
    result.extend_from_slice(&encode_varint(trx.outputs.len()));
    for output in &trx.outputs {
        result.extend_from_slice(&output.value.to_le_bytes());
        result.extend_from_slice(&encode_varint(output.script_pubkey.len()));
        result.extend_from_slice(&output.script_pubkey);
    }

    // 5. Witness data
    if trx.segwit {
        for input in &trx.inputs {
            result.extend_from_slice(&encode_varint(input.witness.len()));
            for item in &input.witness {
                result.extend_from_slice(&encode_varint(item.len()));
                result.extend_from_slice(item);
            }
        }
    }

    // 6. Locktime
    result.extend_from_slice(&trx.locktime.to_le_bytes());

    result
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Parse inputs
    let mut inputs = Vec::new();
    for (idx, in_str) in cli.inputs.iter().enumerate() {
        let parts: Vec<&str> = in_str.split(':').collect();
        if parts.len() < 3 || parts.len() > 4 {
            return Err(format!(
                "Invalid input format for arg {}: '{}'. Expected 'prev_txid:vout:sequence[:script_sig_hex]'",
                idx, in_str
            ).into());
        }

        let prev_txid = hex_to_bytes(parts[0])
            .map_err(|e| format!("Invalid prev_txid hex for input {}: {}", idx, e))?;
        if prev_txid.len() != 32 {
            return Err(format!(
                "Invalid prev_txid size for input {} (must be exactly 32 bytes/64 hex characters), got {} bytes",
                idx, prev_txid.len()
            ).into());
        }

        let vout = parts[1]
            .parse::<u32>()
            .map_err(|e| format!("Invalid vout for input {}: {}", idx, e))?;
        let sequence = parts[2]
            .parse::<u32>()
            .map_err(|e| format!("Invalid sequence for input {}: {}", idx, e))?;

        let script_sig = if parts.len() == 4 {
            hex_to_bytes(parts[3])
                .map_err(|e| format!("Invalid script_sig hex for input {}: {}", idx, e))?
        } else {
            Vec::new()
        };

        inputs.push(TxInput {
            prev_txid,
            vout,
            script_sig,
            sequence,
            witness: Vec::new(),
        });
    }

    // Parse outputs
    let mut outputs = Vec::new();
    for (idx, out_str) in cli.outputs.iter().enumerate() {
        let parts: Vec<&str> = out_str.split(':').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid output format for arg {}: '{}'. Expected 'value_satoshis:script_pubkey_hex'",
                idx, out_str
            ).into());
        }

        let value = parts[0]
            .parse::<u64>()
            .map_err(|e| format!("Invalid output value for output {}: {}", idx, e))?;
        let script_pubkey = hex_to_bytes(parts[1])
            .map_err(|e| format!("Invalid script_pubkey hex for output {}: {}", idx, e))?;

        outputs.push(TxOutput {
            value,
            script_pubkey,
        });
    }

    // Parse witness data and associate with inputs
    for (idx, wit_str) in cli.witnesses.iter().enumerate() {
        let parts: Vec<&str> = wit_str.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid witness format for arg {}: '{}'. Expected 'input_index:witness_item_hex'",
                idx, wit_str
            )
            .into());
        }

        let input_idx = parts[0]
            .parse::<usize>()
            .map_err(|e| format!("Invalid input index in witness {}: {}", idx, e))?;
        if input_idx >= inputs.len() {
            return Err(format!(
                "Witness references out-of-bounds input index: {} (total inputs: {})",
                input_idx,
                inputs.len()
            )
            .into());
        }

        let item = hex_to_bytes(parts[1])
            .map_err(|e| format!("Invalid witness item hex for witness {}: {}", idx, e))?;

        inputs[input_idx].witness.push(item);
    }

    let trx = Transaction {
        version: cli.version,
        inputs,
        outputs,
        locktime: cli.locktime,
        segwit: cli.segwit,
    };

    let serialized = serialize_transaction(&trx);

    println!("Serialized transaction:");
    println!("{:?}", &serialized);
    println!("Serialized Hex transaction:");
    println!("{}", bytes_to_hex(&serialized));
    println!("\nTransaction size: {} bytes", serialized.len());

    Ok(())
}
