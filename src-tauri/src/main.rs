#![cfg_attr(all(not(debug_assertions), target_os="windows"), windows_subsystem="windows")]
mod orders;

use orders::store::OrderStore;
use orders::parser_list::parse_order_list;
use orders::parser_detail::parse_order_detail;
use orders::dedup::Deduper;

use std::sync::Arc;
use tauri::State;
use futures::StreamExt;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use serde::Deserialize;
use serde_json::Value;
use anyhow::Result;

#[derive(Clone)]
struct AppState {
    store: Arc<OrderStore>
}

#[tauri::command]
async fn list_orders(state: State<'_, AppState>) -> Result<Vec<orders::store::OrderView>, String> {
    Ok(state.store.list().await)
}

#[tauri::command]
async fn set_my_nickname(state: State<'_, AppState>, nickname: String) -> Result<(), String> {
    state.store.set_my_nickname(nickname).await;
    Ok(())
}

fn main() {
    let store = Arc::new(OrderStore::default());

    let ws_store = store.clone();
    tauri::Builder::default()
        .setup(move |_app| {
            tauri::async_runtime::spawn(async move {
                if let Err(e) = ws_server("127.0.0.1:8123", ws_store).await {
                    eprintln!("[WS] crashed: {e:?}");
                }
            });
            Ok(())
        })
        .manage(AppState { store })
        .invoke_handler(tauri::generate_handler![list_orders, set_my_nickname])
        .run(tauri::generate_context!())
        .expect("run failed");
}

#[derive(Deserialize)]
struct NetWrapper {
    kind: String,
    payload: Value
}

async fn ws_server(addr: &str, store: Arc<OrderStore>) -> Result<()> {
    println!("[WS] Listening {addr}");
    let listener = TcpListener::bind(addr).await?;
    let deduper = Arc::new(tokio::sync::Mutex::new(Deduper::new(1500, 500))); // 1.5s window

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let store_peer = store.clone();
        let dedup_peer = deduper.clone();
        tokio::spawn(async move {
            let ws_stream = match accept_async(stream).await {
                Ok(s) => {
                    println!("[WS] Client connected from {peer_addr}");
                    s
                },
                Err(e) => { eprintln!("[WS] handshake error from {peer_addr}: {e}"); return; }
            };
            let (_w, mut r) = ws_stream.split();
            while let Some(msg) = r.next().await {
                match msg {
                    Ok(m) if m.is_text() => {
                        let raw = m.to_text().unwrap();
                        if let Ok(wrapper) = serde_json::from_str::<NetWrapper>(raw) {
                            if wrapper.kind != "NET_CAPTURE" { continue; }
                            let req_url = wrapper.payload.get("url").and_then(|x| x.as_str()).unwrap_or("");
                            let status = wrapper.payload.get("status").and_then(|x| x.as_i64()).unwrap_or(0);
                            let ts = wrapper.payload.get("ts").and_then(|x| x.as_i64()).unwrap_or(0);
                            // Lightweight trace (first 60 chars url)
                            println!("[WS] Capture {} status={} ts={}", &req_url.chars().take(60).collect::<String>(), status, ts);

                            let body = wrapper.payload.get("data").unwrap_or(&wrapper.payload);
                            let body_len = body.to_string().len();
                            let fp = orders::dedup::Deduper::make_fp(req_url, status, body_len);

                            {
                                let mut d = dedup_peer.lock().await;
                                if !d.allow(&fp) && !req_url.contains("order-list") {
                                    // println!("[WS] Dedup skip {}", req_url);
                                    continue;
                                }
                            }

                            if req_url.contains("order-list") {
                                let list = parse_order_list(body);
                                if !list.is_empty() {
                                    let sample: Vec<String> = list.iter().take(3).map(|o| o.order_number.clone()).collect();
                                    println!("[WS] Parsed order-list: {} orders sample={:?}", list.len(), sample);
                                    store_peer.upsert_summaries(list, ts).await;
                                }
                            } else if req_url.contains("order-detail") || req_url.contains("getOrderDetail") {
                                if let Some(detail) = parse_order_detail(body) {
                                    println!("[WS] Parsed order-detail: {}", detail.order_number);
                                    store_peer.upsert_detail(detail, ts).await;
                                } else {
                                    println!("[WS] order-detail parse failed");
                                }
                            } else if req_url.contains("get-order-status") {
                                if let Some(code_val) = body.get("data")
                                    .and_then(|d| d.get("orderStatus"))
                                    .and_then(|x| x.as_i64()) {
                                        println!("[WS] quick status code={}", code_val);
                                }
                            } else {
                                // println!("[WS] Other url {}", req_url);
                            }
                        } else {
                            eprintln!("[WS] JSON parse failed (ignored)");
                        }
                    }
                    Ok(_) => {},
                    Err(e) => {
                        eprintln!("[WS] Read error: {e}");
                        break;
                    }
                }
            }
            println!("[WS] Client disconnected {peer_addr}");
        });
    }
}