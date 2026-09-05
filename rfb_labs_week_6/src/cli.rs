use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "bitcoin-wallet")]
#[command(about = "A persistent Bitcoin wallet in Rust supporting BIP84 (P2WPKH) and BIP86 (Taproot)", long_about = None)]
pub struct Cli {
    /// SQLite database file path
    #[arg(short = 'd', long, default_value = "wallet.db")]
    pub db: PathBuf,

    /// Bitcoin Network (regtest, testnet, signet, bitcoin)
    #[arg(short = 'n', long, default_value = "regtest")]
    pub network: String,

    /// Descriptor type (wpkh for Native SegWit, tr for Taproot)
    #[arg(short = 't', long, default_value = "wpkh")]
    pub descriptor_type: String,

    /// Bitcoin Core RPC URL
    #[arg(long, default_value = "http://127.0.0.1:18443")]
    pub rpc_url: String,

    /// Bitcoin Core RPC Username
    #[arg(long, default_value = "user")]
    pub rpc_user: String,

    /// Bitcoin Core RPC Password
    #[arg(long, default_value = "password")]
    pub rpc_pass: String,

    /// Bitcoin Core RPC Cookie File Path
    #[arg(long)]
    pub rpc_cookie: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new wallet or import an existing mnemonic phrase
    Init(InitArgs),

    /// Derive the next unused external receive address (keychain /0/*)
    GetNewAddress,

    /// Derive the next internal change address (keychain /1/*)
    GetChangeAddress,

    /// Display wallet balance (confirmed and unconfirmed)
    GetBalance,

    /// List all tracked UTXOs in the wallet database
    ListUtxos,

    /// List all derived addresses and their status
    ListAddresses,

    /// Display wallet descriptors (BIP84 / BIP86)
    ShowDescriptors,

    /// Sync UTXOs with Bitcoin Core RPC node using scantxoutset
    Sync,

    /// Construct, sign, and optionally broadcast a transaction
    Send(SendArgs),

    /// Run low-level raw rust-bitcoin script creation and spend demo
    RawDemo,
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// BIP39 mnemonic words (if omitted, a new 12-word mnemonic is generated)
    #[arg(short, long)]
    pub mnemonic: Option<String>,

    /// Optional BIP39 passphrase
    #[arg(short, long, default_value = "")]
    pub passphrase: String,
}

#[derive(Args, Debug)]
pub struct SendArgs {
    /// Recipient Bitcoin address
    #[arg(short, long)]
    pub recipient: String,

    /// Amount to send in satoshis
    #[arg(short, long)]
    pub amount_sats: u64,

    /// Fee rate in sat/vB
    #[arg(short, long, default_value = "2")]
    pub fee_rate: u64,

    /// Coin selection strategy (largest-first, smallest-first, exact-match)
    #[arg(short, long, default_value = "largest-first")]
    pub strategy: String,

    /// Broadcast the transaction to the Bitcoin network via RPC
    #[arg(short, long)]
    pub broadcast: bool,
}
