use std::fmt;

/// Expected failures produced by transaction validation and coin selection.
#[derive(Debug, PartialEq, Eq)]
pub enum TransactionError {
    NoInputs,
    NoOutputs,
    ZeroValueOutput,
    OutputsExceedInputs {
        total_inputs: u64,
        total_outputs: u64,
    },
    CoinbaseMixedWithRegularInputs,
    MultipleCoinbaseInputs,
    InvalidTxid,
    InsufficientFunds {
        available: u64,
        required: u64,
    },
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoInputs => write!(f, "Transaction must have at least one input"),
            Self::NoOutputs => write!(f, "Transaction must have at least one output"),
            Self::ZeroValueOutput => write!(
                f,
                "Non-OpReturn outputs must have a value greater than zero"
            ),
            Self::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => {
                write!(
                    f,
                    "Outputs exceed inputs: total inputs = {} sats, total outputs = {} sats",
                    total_inputs, total_outputs
                )
            }
            Self::CoinbaseMixedWithRegularInputs => {
                write!(
                    f,
                    "Coinbase inputs cannot be mixed with regular inputs in a single transaction"
                )
            }
            Self::MultipleCoinbaseInputs => {
                write!(f, "Transaction cannot contain multiple coinbase inputs")
            }
            Self::InvalidTxid => write!(f, "Regular input has an empty or invalid TXID"),
            Self::InsufficientFunds {
                available,
                required,
            } => {
                write!(
                    f,
                    "Insufficient funds: available = {} sats, required = {} sats",
                    available, required
                )
            }
        }
    }
}

impl std::error::Error for TransactionError {}
