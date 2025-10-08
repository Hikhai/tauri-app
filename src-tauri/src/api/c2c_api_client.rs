use anyhow::{Result, anyhow};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use reqwest::Client;
use chrono::Utc;
use serde_json::Value;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct C2CApiClient {
    api_key: String,
    api_secret: String,
    http: Client,
    base: String
}

impl C2CApiClient {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self { api_key, api_secret, http: Client::new(), base: "https://api.binance.com".into() }
    }

    pub async fn list_user_order_history(&self, trade_type: &str, start_ts: i64, end_ts: i64, page: u32, rows: u32) -> Result<Value> {
        let timestamp = Utc::now().timestamp_millis();
        let query = format!(
            "tradeType={}&startTimestamp={}&endTimestamp={}&page={}&rows={}&timestamp={}",
            trade_type, start_ts, end_ts, page, rows, timestamp
        );
        let signature = self.sign(&query)?;
        let url = format!("{}/sapi/v1/c2c/orderMatch/listUserOrderHistory?{}&signature={}", self.base, query, signature);
        let res = self.http.get(&url).header("X-MBX-APIKEY", &self.api_key).send().await?;
        let text = res.text().await?;
        let json: Value = serde_json::from_str(&text).map_err(|e| anyhow!("JSON parse error: {e} body={text}"))?;
        if json.get("code").and_then(|x| x.as_str()) == Some("000000") { Ok(json) } else { Err(anyhow!("API error: {text}")) }
    }

    fn sign(&self, query: &str) -> Result<String> {
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes())?;
        mac.update(query.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }
}
