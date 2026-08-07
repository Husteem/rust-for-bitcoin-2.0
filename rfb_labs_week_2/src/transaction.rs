use std::fmt;

use crate::error::TransactionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    P2pkh,
    P2wpkh,
    P2tr,
    OpReturn,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TxOutput {
    pub value: u64,
    pub recipient: String,
    pub output_type: OutputType,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OutPoint {
    pub txid: String,
    pub vout: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InputKind {
    Regular {
        previous_output: OutPoint,
        value: u64,
        sequence: u32,
    },
    Coinbase {
        block_height: u32,
        reward: u64,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Transaction {
    pub version: i32,
    pub inputs: Vec<InputKind>,
    pub outputs: Vec<TxOutput>,
    pub locktime: u32,
}

pub trait BitcoinValue {
    fn value(&self) -> u64;

    fn value_in_btc(&self) -> f64 {
        self.value() as f64 / 100_000_000.0
    }
}

impl Transaction {
    pub fn new(version: i32, locktime: u32) -> Self {
        Self {
            version,
            inputs: Vec::new(),
            outputs: Vec::new(),
            locktime,
        }
    }

    pub fn add_input(&mut self, input: InputKind) {
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: TxOutput) {
        self.outputs.push(output);
    }

    pub fn total_input_value(&self) -> u64 {
        self.inputs
            .iter()
            .map(|input| match input {
                InputKind::Regular { value, .. } => *value,
                InputKind::Coinbase { reward, .. } => *reward,
            })
            .sum()
    }

    pub fn total_output_value(&self) -> u64 {
        self.outputs.iter().map(|output| output.value).sum()
    }

    pub fn fee(&self) -> Result<u64, TransactionError> {
        let total_inputs = self.total_input_value();
        let total_outputs = self.total_output_value();
        total_inputs
            .checked_sub(total_outputs)
            .ok_or(TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            })
    }

    pub fn validate(&self) -> Result<(), TransactionError> {
        if self.inputs.is_empty() {
            return Err(TransactionError::NoInputs);
        }

        if self.outputs.is_empty() {
            return Err(TransactionError::NoOutputs);
        }

        for output in &self.outputs {
            if output.output_type != OutputType::OpReturn && output.value == 0 {
                return Err(TransactionError::ZeroValueOutput);
            }
        }

        self.fee()?;

        let has_coinbase = self
            .inputs
            .iter()
            .any(|i| matches!(i, InputKind::Coinbase { .. }));
        let has_regular = self
            .inputs
            .iter()
            .any(|i| matches!(i, InputKind::Regular { .. }));
        if has_coinbase && has_regular {
            return Err(TransactionError::CoinbaseMixedWithRegularInputs);
        }

        if has_coinbase {
            let coinbase_count = self
                .inputs
                .iter()
                .filter(|i| matches!(i, InputKind::Coinbase { .. }))
                .count();
            if coinbase_count > 1 {
                return Err(TransactionError::MultipleCoinbaseInputs);
            }
        }

        for input in &self.inputs {
            if let InputKind::Regular {
                previous_output, ..
            } = input
            {
                if previous_output.txid.is_empty() {
                    return Err(TransactionError::InvalidTxid);
                }
            }
        }

        Ok(())
    }
}

impl BitcoinValue for TxOutput {
    fn value(&self) -> u64 {
        self.value
    }
}

impl BitcoinValue for InputKind {
    fn value(&self) -> u64 {
        match self {
            InputKind::Regular { value, .. } => *value,
            InputKind::Coinbase { reward, .. } => *reward,
        }
    }
}

pub fn highest_value_output(transaction: &Transaction) -> Option<&TxOutput> {
    transaction.outputs.iter().max_by_key(|out| out.value)
}

pub fn find_outputs_for_recipient<'a>(
    transaction: &'a Transaction,
    recipient: &str,
) -> Vec<&'a TxOutput> {
    transaction
        .outputs
        .iter()
        .filter(|out| out.recipient == recipient)
        .collect()
}

impl fmt::Display for OutPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.txid, self.vout)
    }
}

impl fmt::Display for TxOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Recipient: {}, Value: {} sats ({:.8} BTC), Type: {:?}",
            self.recipient,
            self.value,
            self.value_in_btc(),
            self.output_type
        )
    }
}

impl fmt::Display for InputKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputKind::Regular {
                previous_output,
                value,
                sequence,
            } => {
                write!(
                    f,
                    "Regular Input - PrevOut: {}, Value: {} sats ({:.8} BTC), Sequence: {}",
                    previous_output,
                    value,
                    self.value_in_btc(),
                    sequence
                )
            }
            InputKind::Coinbase {
                block_height,
                reward,
            } => {
                write!(
                    f,
                    "Coinbase Input - Block Height: {}, Reward: {} sats ({:.8} BTC)",
                    block_height,
                    reward,
                    self.value_in_btc()
                )
            }
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fee_str = match self.fee() {
            Ok(fee_val) => format!(
                "{} sats ({:.8} BTC)",
                fee_val,
                fee_val as f64 / 100_000_000.0
            ),
            Err(_) => "INVALID (outputs exceed inputs)".to_owned(),
        };
        write!(
            f,
            "Transaction (v{}, locktime: {}):\n  Inputs ({}): {}\n  Outputs ({}): {}\n  Total Input: {} sats ({:.8} BTC)\n  Total Output: {} sats ({:.8} BTC)\n  Fee: {}",
            self.version,
            self.locktime,
            self.inputs.len(),
            self.inputs.iter().map(|i| i.value()).sum::<u64>(),
            self.outputs.len(),
            self.outputs.iter().map(|o| o.value()).sum::<u64>(),
            self.total_input_value(),
            self.total_input_value() as f64 / 100_000_000.0,
            self.total_output_value(),
            self.total_output_value() as f64 / 100_000_000.0,
            fee_str
        )
    }
}
