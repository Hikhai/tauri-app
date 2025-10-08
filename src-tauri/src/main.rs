#![cfg_attr(all(not(debug_assertions), target_os="windows"), windows_subsystem="windows")]

mod crypto;
mod db;
mod api { pub mod c2c_api_client; pub mod credentials; pub mod sync_engine; }
mod orders { pub mod repo; }

use std::sync::{Arc, Mutex};
use tauri::State;
use anyhow::Result;

use crypto::CryptoCtx;
use db::Db;
use api::credentials::CredentialsRepo;
use api::c2c_api_client::C2CApiClient;
use api::sync_engine::SyncEngine;
use orders::repo::OrderRepo;

// App context for Phase 4 (WS extension capture will be reintegrated with DB in Phase 6)
struct AppCtx {
    db: Arc<Db>,
    order_repo: Arc<OrderRepo>,
    creds_repo: Arc<CredentialsRepo>,
    api_client: Mutex<Option<C2CApiClient>>,
}

#[tauri::command]
async fn store_api_credentials(state: State<'_, AppCtx>, label: String, api_key: String, api_secret: String) -> Result<(), String> {
    state.creds_repo.store(&label, &api_key, &api_secret).await.map_err(|e| e.to_string())?;
    {
        let mut guard = state.api_client.lock().unwrap();
        *guard = Some(C2CApiClient::new(api_key, api_secret));
    }
    Ok(())
}

#[tauri::command]
async fn test_api_credentials(state: State<'_, AppCtx>) -> Result<String, String> {
    let guard = state.api_client.lock().unwrap();
    if guard.is_none() { return Err("Chưa có API credentials".into()); }
    let client = guard.as_ref().unwrap();
    let now = chrono::Utc::now().timestamp_millis();
    let start = now - 5 * 60 * 1000;
    let res = client.list_user_order_history("BUY", start, now, 1, 1).await.map_err(|e| e.to_string())?;
    Ok(res.to_string())
}

#[tauri::command]
async fn force_initial_sync(state: State<'_, AppCtx>, days: i64) -> Result<String, String> {
    let (client, repo) = {
        let guard = state.api_client.lock().unwrap();
        if guard.is_none() { return Err("Chưa cấu hình API client".into()); }
        (guard.as_ref().unwrap().clone(), state.order_repo.clone())
    };
    let engine = SyncEngine::new(&client, &repo);
    engine.force_initial_sync(days).await.map_err(|e| e.to_string())?;
    Ok("SYNC_OK".into())
}

#[tauri::command]
async fn list_orders_from_db(state: State<'_, AppCtx>, limit: i64) -> Result<Vec<orders::repo::OrderRow>, String> {
    state.order_repo.list_orders(limit).await.map_err(|e| e.to_string())
}

#[tokio::main]
async fn main() {
    let db = Arc::new(Db::init("p2p_app.db").await.expect("DB init failed"));
    let crypto = CryptoCtx::new_dummy();
    let creds_repo = Arc::new(CredentialsRepo::new(db.pool().clone(), crypto));
    let order_repo = Arc::new(OrderRepo::new(db.pool().clone()));

    let api_client = {
        let mut opt = None;
        if let Ok(Some((k,s))) = creds_repo.latest().await { opt = Some(C2CApiClient::new(k, s)); }
        Mutex::new(opt)
    };

    let app_ctx = AppCtx { db, order_repo, creds_repo, api_client };

    tauri::Builder::default()
        .manage(app_ctx)
        .invoke_handler(tauri::generate_handler![
            store_api_credentials,
            test_api_credentials,
            force_initial_sync,
            list_orders_from_db
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}