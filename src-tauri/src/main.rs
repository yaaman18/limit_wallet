// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod coingecko;

use env_logger;
use dotenv::dotenv;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::str::FromStr;
use tauri::{Manager, State};
use base64::{engine::general_purpose::STANDARD, Engine};
use bdk::bitcoin::{Network, Address};
use bdk::database::MemoryDatabase;
use bdk::keys::{DerivableKey, ExtendedKey, bip39::Mnemonic};
use bdk::template::Bip84;
use bdk::{Wallet, SignOptions};
use bdk::blockchain::{electrum::ElectrumBlockchain, Blockchain};
use bdk::electrum_client::Client;
use bdk::wallet::{AddressIndex, SyncOptions};
use bdk::bitcoin::{
    consensus::{serialize, deserialize},
    util::{
        bip32::{ExtendedPubKey, ExtendedPrivKey},
        psbt::PartiallySignedTransaction,
    },
};
use coingecko::get_bitcoin_price;

struct BalanceState(Arc<Mutex<BalanceStateInner>>);
struct BalanceStateInner {
    balance: u64,
    changed: bool,
}
struct WalletState(Arc<Mutex<Option<Wallet<MemoryDatabase>>>>);


#[derive(serde::Serialize)]
struct SerializableWallet {
    address: String,
    network: String,
}
#[derive(serde::Serialize)]
struct WalletInfo {
    network: String,
    descriptor: String,
    balance: u64,
    address: String,
}





#[tauri::command]
async fn fetch_bitcoin_price() -> Result<f64, String> {
    match get_bitcoin_price().await {
        Ok(price) => Ok(price),
        Err(e) => Err(format!("Error fetching Bitcoin price: {}", e)),
    }
}


#[tauri::command]
fn get_mnemonic_from_env() -> Result<Mnemonic, String> {
    dotenv().ok();
    let mnemonic_words = env::var("MNEMONIC").map_err(|e| e.to_string())?;
    let mnemonic = Mnemonic::parse(&mnemonic_words).map_err(|e| e.to_string())?;
    Ok(mnemonic)
}


#[tauri::command]
fn get_xprv_from_mnemonic(mnemonic: Mnemonic, network: Network) -> Result<ExtendedPrivKey, String> {
    let xkey: ExtendedKey<bdk::miniscript::Segwitv0> = mnemonic
        .into_extended_key()
        .map_err(|e| e.to_string())?;
    let xprv = xkey.into_xprv(network)
        .ok_or_else(|| "拡張秘密鍵を導出できません".to_string())?;
    Ok(xprv)
}


#[tauri::command]
fn get_xpub_from_xprv(xprv: ExtendedPrivKey) -> Result<String, String> {
    let xpub = ExtendedPubKey::from_priv(&bdk::bitcoin::secp256k1::Secp256k1::new(), &xprv);
    Ok(xpub.to_string())
}


#[tauri::command]
fn create_wallet_from_env(wallet_state: State<'_, WalletState>) -> Result<SerializableWallet, String> {
    let network = Network::Testnet;
    let mnemonic = get_mnemonic_from_env().map_err(|e| format!("Mnemonic Error: {}", e))?;
    let xprv = get_xprv_from_mnemonic(mnemonic, network)?;
    let wallet = Wallet::new(
        Bip84(xprv.clone(), bdk::KeychainKind::External),
        Some(Bip84(xprv, bdk::KeychainKind::Internal)),
        network,
        MemoryDatabase::default(),
    ).map_err(|e| e.to_string())?;
    {
        let mut state = wallet_state.0.lock().map_err(|e| e.to_string())?;
        *state = Some(wallet);
    }
    let wallet_guard = wallet_state.0.lock().map_err(|e| e.to_string())?;
    let wallet_ref = wallet_guard.as_ref().ok_or_else(|| "ウォレットが初期化されていません".to_string())?;
    let address_info = wallet_ref
        .get_address(AddressIndex::New)
        .map_err(|e| e.to_string())?;

    Ok(SerializableWallet {
        address: address_info.address.to_string(),
        network: network.to_string(),
    })
}


#[tauri::command]
fn initialize_wallet(mnemonic_str: String, wallet_state: State<'_, WalletState>) -> Result<String, String> {
    let network = Network::Testnet;
    let mnemonic = Mnemonic::parse(&mnemonic_str).map_err(|e| e.to_string())?;
    let xkey: ExtendedKey<bdk::miniscript::Segwitv0> = mnemonic.into_extended_key().map_err(|e| e.to_string())?;
    let xprv = xkey.into_xprv(network).ok_or_else(|| "拡張秘密鍵を生成できません".to_string())?;
    let wallet = Wallet::new(
        Bip84(xprv.clone(), bdk::KeychainKind::External),
        Some(Bip84(xprv, bdk::KeychainKind::Internal)),
        network,
        MemoryDatabase::default(),
    ).map_err(|e| e.to_string())?;
    {
        let mut wallet_guard = wallet_state.0.lock().map_err(|e| e.to_string())?;
        *wallet_guard = Some(wallet);
    }

    Ok("Wallet created and saved successfully!".to_string())
}


#[tauri::command]
fn connect_to_testnet(wallet_state: State<'_, WalletState>, app_handle: tauri::AppHandle) -> Result<String, String> {
    let wallet_state = wallet_state.0.clone();
    let app_handle_clone = app_handle.clone();
    std::thread::spawn(move || {
        let electrum_url = "ssl://electrum.blockstream.info:60002";
        println!("Electrum URL: {}", electrum_url);
        let client = match Client::new(electrum_url) {
            Ok(client) => client,
            Err(e) => {
                eprintln!("Failed to connect to Electrum: {}", e);
                return;
            }
        };
        let blockchain = ElectrumBlockchain::from(client);
        let mut wallet_guard = wallet_state.lock().unwrap();
        let wallet = match wallet_guard.as_mut() {
            Some(wallet) => wallet,
            None => {
                eprintln!("Wallet is not initialized");
                return;
            }
        };
        match wallet.sync(&blockchain, SyncOptions::default()) {
            Ok(_) => {
                println!("Wallet synced successfully!");
                app_handle_clone.emit_all("sync_completed", "Wallet synced with testnet").unwrap();
            }
            Err(e) => {
                eprintln!("Wallet sync failed: {}", e);
            }
        }
    });

    Ok("Sync started in background!".to_string())
}


#[tauri::command]
fn print_wallet_info(wallet_state: State<'_, WalletState>) -> Result<WalletInfo, String> {
    let wallet_guard = wallet_state.0.lock().map_err(|e| e.to_string())?;
    let wallet = wallet_guard.as_ref().ok_or("ウォレットが初期化されていません".to_string())?;
    let network = wallet.network().to_string();
    let descriptor = wallet.public_descriptor(bdk::KeychainKind::External)
        .map(|d| d.map(|d| d.to_string()))
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let balance = wallet.get_balance().map_err(|e| e.to_string())?;
    let address = wallet.get_address(AddressIndex::New).map_err(|e| e.to_string())?.to_string();

    Ok(WalletInfo {
        network,
        descriptor,
        balance: balance.confirmed,
        address,
    })
}


#[tauri::command]
fn list_utxos(wallet_state: State<'_, WalletState>) -> Result<(), String> {
    let wallet_guard = wallet_state.0.lock().map_err(|e| e.to_string())?;
    let wallet = wallet_guard.as_ref().ok_or_else(|| "ウォレットが初期化されていません".to_string())?;
    let utxos = wallet.list_unspent().map_err(|e| e.to_string())?;
    for utxo in utxos {
        println!(
            "UTXO: txid = {}, vout = {}, value = {} satoshis",
            utxo.outpoint.txid, utxo.outpoint.vout, utxo.txout.value
        );
    }

    Ok(())
}


#[tauri::command]
fn get_wallet_balance(wallet_state: State<'_, WalletState>) -> Result<u64, String> {
    let wallet_guard = wallet_state.0.lock().map_err(|e| e.to_string())?;
    let wallet = wallet_guard.as_ref().ok_or_else(|| "ウォレットが初期化されていません".to_string())?;
    let balance = wallet.get_balance().map_err(|e| e.to_string())?;

    Ok(balance.confirmed)
}


#[tauri::command]
fn generate_receive_address(wallet_state: State<'_, WalletState>) -> Result<String, String> {
    let wallet_guard = wallet_state.0.lock().map_err(|e| e.to_string())?;
    let wallet = wallet_guard.as_ref().ok_or_else(|| "ウォレットが初期化されていません".to_string())?;
    let address = wallet.get_address(AddressIndex::New).map_err(|e| e.to_string())?;

    Ok(address.to_string())
}


#[tauri::command]
fn create_transaction(wallet_state: State<'_, WalletState>, to_address: &str, amount: u64) -> Result<PartiallySignedTransaction, String> {
    let wallet_guard = wallet_state.0.lock().map_err(|e| e.to_string())?;
    let wallet = wallet_guard.as_ref().ok_or_else(|| "ウォレット初期化されていません".to_string())?;
    let to_address = Address::from_str(to_address).map_err(|e| e.to_string())?;
    let mut builder = wallet.build_tx();
    builder.add_recipient(to_address.script_pubkey(), amount);
    let (psbt, _details) = builder.finish().map_err(|e| e.to_string())?;
    Ok(psbt)
}


#[tauri::command]
fn sign_and_broadcast_transaction(wallet_state: State<'_, WalletState>, mut psbt: PartiallySignedTransaction) -> Result<(), String> {
    let wallet_guard = wallet_state.0.lock().map_err(|e| e.to_string())?;
    let wallet = wallet_guard.as_ref().ok_or_else(|| "ウォレットが初期化されていません".to_string())?;
    wallet.sign(&mut psbt, SignOptions::default()).map_err(|e| e.to_string())?;
    let client = Client::new("ssl://electrum.blockstream.info:60002").map_err(|e| e.to_string())?;
    let blockchain = ElectrumBlockchain::from(client);
    let final_tx = psbt.clone().extract_tx();
    blockchain.broadcast(&final_tx).map_err(|e| e.to_string())?;
    println!("Transaction broadcasted! Txid: {}", final_tx.txid());

    Ok(())
}


#[tauri::command]
fn save_psbt_to_env(psbt: PartiallySignedTransaction) -> Result<(), String> {
    let serialized_psbt = serialize(&psbt);
    let encoded_psbt = STANDARD.encode(&serialized_psbt);
    let mut file = OpenOptions::new().append(true).open(".env").map_err(|e| e.to_string())?;
    writeln!(file, "PSBT={}", encoded_psbt).map_err(|e| e.to_string())?;

    Ok(())
}


#[tauri::command]
fn load_psbt_from_env() -> Result<PartiallySignedTransaction, String> {
    dotenv().ok();
    let encoded_psbt = env::var("PSBT").map_err(|e| e.to_string())?;
    let decoded_psbt = STANDARD.decode(&encoded_psbt).map_err(|e| e.to_string())?;
    let psbt: PartiallySignedTransaction = deserialize(&decoded_psbt).map_err(|e| e.to_string())?;

    Ok(psbt)
}


#[tauri::command]
fn save_rate_to_env(rate: f64) -> Result<(), String> {
    let mut file = OpenOptions::new().append(true).open(".env").map_err(|e| e.to_string())?;
    writeln!(file, "TARGET_RATE={}", rate).map_err(|e| e.to_string())?;
    Ok(())
}


#[tauri::command]
fn load_rate_from_env() -> Result<f64, String> {
    dotenv().ok();
    let rate_str = env::var("TARGET_RATE").map_err(|e: env::VarError| e.to_string())?;
    let rate: f64 = rate_str.parse().map_err(|e: std::num::ParseFloatError| e.to_string())?;
    Ok(rate)
}


fn get_wallet_balance_internal(wallet: &Option<Wallet<MemoryDatabase>>) -> Result<u64, String> {
    let wallet = wallet.as_ref().ok_or_else(|| "ウォレットが初期化されていません".to_string())?;
    let balance = wallet.get_balance().map_err(|e| e.to_string())?;
    Ok(balance.confirmed)
}


#[tauri::command]
fn start_balance_monitor(
    app_handle: tauri::AppHandle,
    balance_state: State<'_, BalanceState>,
    wallet_state: State<'_, WalletState>,
) {
    let balance_state_clone = Arc::clone(&balance_state.0);
    let app_handle_clone = app_handle.clone();
    let wallet_state_clone = Arc::clone(&wallet_state.0);
    thread::spawn(move || {
        loop {
            let wallet_guard = wallet_state_clone.lock().unwrap();
            match get_wallet_balance_internal(&wallet_guard) {
                Ok(new_balance) => {
                    let mut state = balance_state_clone.lock().unwrap();
                    if new_balance != state.balance {
                        state.balance = new_balance;
                        state.changed = true;
                        if let Err(e) = app_handle_clone.emit_all("balance_changed", new_balance) {
                            eprintln!("Failed to emit balance_changed event: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get wallet balance: {}", e);
                }
            }
            thread::sleep(Duration::from_secs(10));
        }
    });
}


#[tauri::command]
fn check_balance_changed(balance_state: State<'_, BalanceState>) -> (bool, u64) {
    let mut state = balance_state.0.lock().unwrap();
    let changed = state.changed;
    state.changed = false;
    (changed, state.balance)
}


fn main() {
    env_logger::init();
    let balance_state = BalanceState(Arc::new(Mutex::new(BalanceStateInner {
        balance: 0,
        changed: false,
    })));
    let wallet_state = WalletState(Arc::new(Mutex::new(None)));

    tauri::Builder::default()
        .manage(balance_state)
        .manage(wallet_state)
        .invoke_handler(tauri::generate_handler![
            fetch_bitcoin_price,
            get_mnemonic_from_env,
            get_xprv_from_mnemonic,
            get_xpub_from_xprv,
            create_wallet_from_env,
            initialize_wallet,
            connect_to_testnet,
            print_wallet_info,
            list_utxos,
            get_wallet_balance,
            generate_receive_address,
            create_transaction,
            sign_and_broadcast_transaction,
            save_psbt_to_env,
            load_psbt_from_env,
            save_rate_to_env,
            load_rate_from_env,
            start_balance_monitor,
            check_balance_changed,
        ])

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
