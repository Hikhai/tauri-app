use anyhow::Result;
use chrono::{Utc, Duration};
use crate::api::c2c_api_client::C2CApiClient;
use crate::orders::repo::OrderRepo;

pub struct SyncEngine<'a> {
    pub client: &'a C2CApiClient,
    pub repo: &'a OrderRepo
}

impl<'a> SyncEngine<'a> {
    pub fn new(client: &'a C2CApiClient, repo: &'a OrderRepo) -> Self { Self { client, repo } }

    pub async fn force_initial_sync(&self, days: i64) -> Result<()> {
        let now = Utc::now().timestamp_millis();
        let start = (Utc::now() - Duration::days(days)).timestamp_millis();
        for trade in ["BUY", "SELL"] {
            println!("[SYNC] Initial sync {trade} days={days}");
            let mut page = 1;
            loop {
                let res = self.client.list_user_order_history(trade, start, now, page, 100).await?;
                let data_arr = res.get("data")
                    .and_then(|d| d.get("data"))
                    .and_then(|x| x.as_array())
                    .unwrap_or(&vec![]);
                if data_arr.is_empty() && page == 1 { println!("[SYNC] No orders for {trade}"); }
                for o in data_arr { self.repo.upsert_from_api(o, now).await?; }
                println!("[SYNC] {trade} page {page} -> {} orders", data_arr.len());
                if data_arr.len() < 100 { break; }
                page += 1;
            }
        }
        Ok(())
    }
}
