use anyhow::{Context, Result, anyhow};
use clap::Parser;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use bitcoin::{Address, Network};
use rfb_labs_week_6::cli::{Cli, Commands, InitArgs, SendArgs};
use rfb_labs_week_6::coin_selection::{CoinSelectionStrategy, select_coins};
use rfb_labs_week_6::config::{DescriptorType, RpcConfig};
use rfb_labs_week_6::db::{TxRecord, WalletDb};
use rfb_labs_week_6::keys::WalletKeys;
use rfb_labs_week_6::raw_demo::run_raw_script_demo;
use rfb_labs_week_6::rpc::BitcoinRpcClient;
use rfb_labs_week_6::tx::{create_unsigned_tx, sign_transaction};

fn parse_network(s: &str) -> Result<Network> {
    match s.to_lowercase().as_str() {
        "regtest" => Ok(Network::Regtest),
        "testnet" | "testnet3" => Ok(Network::Testnet),
        "signet" => Ok(Network::Signet),
        "mainnet" | "bitcoin" => Ok(Network::Bitcoin),
        other => Err(anyhow!(
            "Unsupported network: {}. Use regtest, testnet, signet, or bitcoin",
            other
        )),
    }
}

fn load_keys_from_db(
    db: &WalletDb,
    network: Network,
    descriptor_type: DescriptorType,
) -> Result<WalletKeys> {
    let mnemonic_str = db
        .get_meta("mnemonic")?
        .ok_or_else(|| anyhow!("Wallet not initialized. Please run 'init' first."))?;
    let passphrase = db.get_meta("passphrase")?.unwrap_or_default();
    WalletKeys::from_mnemonic_str(&mnemonic_str, &passphrase, network, descriptor_type)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let network = parse_network(&cli.network)?;
    let descriptor_type = DescriptorType::from_str(&cli.descriptor_type)?;

    let rpc_config = RpcConfig {
        url: cli.rpc_url,
        user: Some(cli.rpc_user),
        pass: Some(cli.rpc_pass),
        cookie_file: cli.rpc_cookie,
    };

    match cli.command {
        Commands::Init(args) => handle_init(&cli.db, network, descriptor_type, args)?,
        Commands::GetNewAddress => handle_get_new_address(&cli.db, network, descriptor_type)?,
        Commands::GetChangeAddress => handle_get_change_address(&cli.db, network, descriptor_type)?,
        Commands::GetBalance => handle_get_balance(&cli.db)?,
        Commands::ListUtxos => handle_list_utxos(&cli.db)?,
        Commands::ListAddresses => handle_list_addresses(&cli.db)?,
        Commands::ShowDescriptors => handle_show_descriptors(&cli.db, network, descriptor_type)?,
        Commands::Sync => handle_sync(&cli.db, &rpc_config)?,
        Commands::Send(args) => handle_send(&cli.db, network, descriptor_type, &rpc_config, args)?,
        Commands::RawDemo => handle_raw_demo(network)?,
    }

    Ok(())
}

fn handle_init(
    db_path: &std::path::Path,
    network: Network,
    descriptor_type: DescriptorType,
    args: InitArgs,
) -> Result<()> {
    let db = WalletDb::open(db_path)?;

    let keys = if let Some(ref words) = args.mnemonic {
        WalletKeys::from_mnemonic_str(words, &args.passphrase, network, descriptor_type)?
    } else {
        WalletKeys::new_random(network, descriptor_type)?
    };

    db.set_meta("network", &network.to_string())?;
    db.set_meta("descriptor_type", &descriptor_type.to_string())?;
    db.set_meta("mnemonic", &keys.mnemonic.to_string())?;
    db.set_meta("passphrase", &args.passphrase)?;
    db.set_meta("master_fingerprint", &keys.master_fingerprint.to_string())?;

    // Pre-derive initial receive address (index 0)
    let initial_receive_addr = keys.derive_address(false, 0)?;
    let script_hex = hex::encode(initial_receive_addr.script_pubkey().as_bytes());
    db.insert_address(&initial_receive_addr.to_string(), &script_hex, false, 0)?;
    db.advance_index(false)?;

    println!("============================================================");
    println!("Wallet Initialized Successfully!");
    println!("============================================================");
    println!("Database Path:       {}", db_path.display());
    println!("Network:             {}", network);
    println!("Descriptor Type:     {}", descriptor_type);
    println!("Master Fingerprint:  {}", keys.master_fingerprint);
    println!("Mnemonic:            {}", keys.mnemonic);
    println!("------------------------------------------------------------");
    println!("External Descriptor: {}", keys.descriptor(false));
    println!("Internal Descriptor: {}", keys.descriptor(true));
    println!("------------------------------------------------------------");
    println!("Initial Address (/0/0): {}", initial_receive_addr);
    println!("============================================================");

    Ok(())
}

fn handle_get_new_address(
    db_path: &std::path::Path,
    network: Network,
    descriptor_type: DescriptorType,
) -> Result<()> {
    let db = WalletDb::open(db_path)?;
    let keys = load_keys_from_db(&db, network, descriptor_type)?;

    let index = db.advance_index(false)?;
    let address = keys.derive_address(false, index)?;
    let script_hex = hex::encode(address.script_pubkey().as_bytes());

    db.insert_address(&address.to_string(), &script_hex, false, index)?;

    println!("New Receive Address (index /0/{}): {}", index, address);
    Ok(())
}

fn handle_get_change_address(
    db_path: &std::path::Path,
    network: Network,
    descriptor_type: DescriptorType,
) -> Result<()> {
    let db = WalletDb::open(db_path)?;
    let keys = load_keys_from_db(&db, network, descriptor_type)?;

    let index = db.advance_index(true)?;
    let address = keys.derive_address(true, index)?;
    let script_hex = hex::encode(address.script_pubkey().as_bytes());

    db.insert_address(&address.to_string(), &script_hex, true, index)?;

    println!("New Change Address (index /1/{}): {}", index, address);
    Ok(())
}

fn handle_get_balance(db_path: &std::path::Path) -> Result<()> {
    let db = WalletDb::open(db_path)?;
    let (confirmed, unconfirmed) = db.get_balance()?;
    let total = confirmed + unconfirmed;

    println!("============================================================");
    println!("Wallet Balance");
    println!("============================================================");
    println!(
        "Confirmed:   {:>12} sats ({:.8} BTC)",
        confirmed,
        confirmed as f64 / 100_000_000.0
    );
    println!(
        "Unconfirmed: {:>12} sats ({:.8} BTC)",
        unconfirmed,
        unconfirmed as f64 / 100_000_000.0
    );
    println!("------------------------------------------------------------");
    println!(
        "Total:       {:>12} sats ({:.8} BTC)",
        total,
        total as f64 / 100_000_000.0
    );
    println!("============================================================");
    Ok(())
}

fn handle_list_utxos(db_path: &std::path::Path) -> Result<()> {
    let db = WalletDb::open(db_path)?;
    let utxos = db.get_unspent_utxos()?;

    println!(
        "=========================================================================================================="
    );
    println!(
        "{:<66} {:<6} {:<12} {:<10} {:<10}",
        "OutPoint (Txid:Vout)", "Vout", "Amount (sats)", "Keychain", "Height"
    );
    println!(
        "=========================================================================================================="
    );

    if utxos.is_empty() {
        println!("No unspent UTXOs tracked in wallet database.");
    } else {
        for u in utxos {
            let keychain_str = if u.is_change { "change" } else { "receive" };
            let height_str = u
                .height
                .map(|h| h.to_string())
                .unwrap_or_else(|| "unconfirmed".to_string());
            println!(
                "{:<66} {:<6} {:<12} {:<10} {:<10}",
                format!("{}:{}", u.txid, u.vout),
                u.vout,
                u.amount_sats,
                keychain_str,
                height_str
            );
        }
    }
    println!(
        "=========================================================================================================="
    );
    Ok(())
}

fn handle_list_addresses(db_path: &std::path::Path) -> Result<()> {
    let db = WalletDb::open(db_path)?;
    let addrs = db.get_all_addresses()?;

    println!(
        "=========================================================================================================="
    );
    println!(
        "{:<64} {:<10} {:<8} {:<6}",
        "Address", "Keychain", "Index", "Used"
    );
    println!(
        "=========================================================================================================="
    );

    for a in addrs {
        let keychain_str = if a.is_change {
            "change (/1)"
        } else {
            "receive (/0)"
        };
        println!(
            "{:<64} {:<10} {:<8} {:<6}",
            a.address, keychain_str, a.index_num, a.used
        );
    }
    println!(
        "=========================================================================================================="
    );
    Ok(())
}

fn handle_show_descriptors(
    db_path: &std::path::Path,
    network: Network,
    descriptor_type: DescriptorType,
) -> Result<()> {
    let db = WalletDb::open(db_path)?;
    let keys = load_keys_from_db(&db, network, descriptor_type)?;

    println!("============================================================");
    println!("Wallet Descriptors");
    println!("============================================================");
    println!("External (Receive): {}", keys.descriptor(false));
    println!("Internal (Change):  {}", keys.descriptor(true));
    println!("============================================================");
    Ok(())
}

fn handle_sync(db_path: &std::path::Path, rpc_config: &RpcConfig) -> Result<()> {
    let db = WalletDb::open(db_path)?;
    let rpc_client = BitcoinRpcClient::new(rpc_config)?;

    let block_count = rpc_client.get_block_count()?;
    println!(
        "Connected to Bitcoin Core RPC. Current block height: {}",
        block_count
    );

    let addresses = db.get_all_addresses()?;
    println!(
        "Scanning UTXO set for {} derived wallet addresses...",
        addresses.len()
    );

    let synced = rpc_client.sync_wallet_utxos(&db, &addresses)?;
    println!("Sync complete! Tracked/updated {} unspent outputs.", synced);

    Ok(())
}

fn handle_send(
    db_path: &std::path::Path,
    network: Network,
    descriptor_type: DescriptorType,
    rpc_config: &RpcConfig,
    args: SendArgs,
) -> Result<()> {
    let db = WalletDb::open(db_path)?;
    let keys = load_keys_from_db(&db, network, descriptor_type)?;

    let recipient_addr = Address::from_str(&args.recipient)
        .context("Invalid recipient address")?
        .require_network(network)
        .context("Recipient address network does not match wallet network")?;

    let strategy = CoinSelectionStrategy::from_str(&args.strategy)?;
    let available_utxos = db.get_unspent_utxos()?;

    let selection = select_coins(
        &available_utxos,
        args.amount_sats,
        args.fee_rate,
        descriptor_type,
        strategy,
    )?;

    let change_addr = if selection.create_change_output {
        let change_idx = db.advance_index(true)?;
        let addr = keys.derive_address(true, change_idx)?;
        let script_hex = hex::encode(addr.script_pubkey().as_bytes());
        db.insert_address(&addr.to_string(), &script_hex, true, change_idx)?;
        Some(addr)
    } else {
        None
    };

    let unsigned_tx = create_unsigned_tx(
        &selection.selected_utxos,
        &recipient_addr,
        args.amount_sats,
        change_addr.as_ref(),
        selection.change_sats,
    )?;

    let built = sign_transaction(unsigned_tx, &selection.selected_utxos, &keys)?;
    let raw_hex = hex::encode(bitcoin::consensus::serialize(&built.transaction));

    println!("============================================================");
    println!("Transaction Constructed & Signed");
    println!("============================================================");
    println!("Txid:             {}", built.txid);
    println!(
        "Selected Inputs:  {} UTXOs ({} sats)",
        selection.selected_utxos.len(),
        selection.total_input_sats
    );
    println!(
        "Recipient:        {} ({} sats)",
        recipient_addr, args.amount_sats
    );
    if let Some(ref ca) = change_addr {
        println!("Change:           {} ({} sats)", ca, selection.change_sats);
    } else {
        println!("Change:           None (dust amount folded into fee)");
    }
    println!("Miner Fee:        {} sats", built.fee_sats);
    println!(
        "Virtual Size:     {} vB (Weight: {} WU)",
        built.vsize,
        built.weight.to_wu()
    );
    println!("------------------------------------------------------------");
    println!("Raw Transaction Hex:\n{}", raw_hex);
    println!("============================================================");

    if args.broadcast {
        println!("Broadcasting transaction to Bitcoin network...");
        let rpc_client = BitcoinRpcClient::new(rpc_config)?;
        let broadcast_txid = rpc_client.broadcast_transaction(&built.transaction)?;
        println!("Broadcast Successful! Txid: {}", broadcast_txid);

        // Mark spent UTXOs in DB
        for u in &selection.selected_utxos {
            db.mark_utxo_spent(&u.txid, u.vout)?;
        }

        // Record transaction in DB
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let tx_record = TxRecord {
            txid: built.txid,
            raw_tx_hex: raw_hex,
            fee_sats: Some(built.fee_sats),
            height: None,
            is_outgoing: true,
            timestamp,
        };
        db.insert_transaction(&tx_record)?;
    }

    Ok(())
}

fn handle_raw_demo(network: Network) -> Result<()> {
    let report = run_raw_script_demo(network)?;

    println!("============================================================");
    println!("Raw rust-bitcoin Script & Spending Demo");
    println!("============================================================");
    println!("Witness Script ASM:    {}", report.script_asm);
    println!("Witness Script Hex:    {}", report.script_hex);
    println!("Funding Outpoint:      {}:0", report.funding_txid);
    println!("Spending Txid:         {}", report.spending_txid);
    println!("Witness Stack Items:");
    for (i, item) in report.witness_items.iter().enumerate() {
        println!("  [{}] {}", i, item);
    }
    println!("------------------------------------------------------------");
    println!("Spending Transaction Raw Hex:\n{}", report.spending_raw_hex);
    println!("------------------------------------------------------------");
    println!("Architectural Comparison:\n{}", report.explanation);
    println!("============================================================");

    Ok(())
}
