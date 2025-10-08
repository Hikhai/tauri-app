use anyhow::Result;
use sqlx::{SqlitePool, Row};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct OrderRow {
    pub order_number: String,
    pub trade_type: String,
    pub fiat: String,
    pub asset: String,
    pub amount_asset: String,
    pub total_fiat: String,
    pub price: String,
    pub status_code: i64,
    pub status_label: String,
    pub buyer_nickname: String,
    pub seller_nickname: String,
    pub has_payment_detail: bool,
    pub last_api_sync_ts: i64
}

pub struct OrderRepo { pool: SqlitePool }
impl OrderRepo { pub fn new(pool: SqlitePool) -> Self { Self { pool } } }

impl OrderRepo {
    pub async fn upsert_from_api(&self, order: &serde_json::Value, now: i64) -> Result<()> {
        let order_number = order.get("orderNumber").and_then(|x| x.as_str()).unwrap_or("");
        if order_number.is_empty() { return Ok(()); }
        let trade_type = order.get("tradeType").and_then(|x| x.as_str()).unwrap_or("");
        let asset = order.get("asset").and_then(|x| x.as_str()).unwrap_or("");
        let fiat = order.get("fiat").and_then(|x| x.as_str()).unwrap_or("");
        let amount_asset = order.get("amount").and_then(|x| x.as_str()).unwrap_or("");
        let total_fiat = order.get("totalPrice").and_then(|x| x.as_str()).unwrap_or("");
        let price = order.get("price").and_then(|x| x.as_str()).unwrap_or("");
        let status_code = order.get("orderStatus").and_then(|x| x.as_i64()).unwrap_or(-1);
        let create_time = order.get("createTime").and_then(|x| x.as_i64()).unwrap_or(0);
        let buyer_nick = order.get("buyerNickname").and_then(|x| x.as_str()).unwrap_or("");
        let seller_nick = order.get("sellerNickname").and_then(|x| x.as_str()).unwrap_or("");
        sqlx::query(r#"INSERT INTO orders (order_number, trade_type, asset, fiat, price, amount_asset, total_fiat, order_status_code, create_time_ms, update_time_ms, buyer_nickname, seller_nickname, last_api_sync_ts, source_flags) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,1) ON CONFLICT(order_number) DO UPDATE SET trade_type=excluded.trade_type, asset=excluded.asset, fiat=excluded.fiat, price=excluded.price, amount_asset=excluded.amount_asset, total_fiat=excluded.total_fiat, order_status_code=excluded.order_status_code, update_time_ms=excluded.update_time_ms, buyer_nickname=excluded.buyer_nickname, seller_nickname=excluded.seller_nickname, last_api_sync_ts=excluded.last_api_sync_ts, source_flags = orders.source_flags | 1"#)
            .bind(order_number).bind(trade_type).bind(asset).bind(fiat).bind(price).bind(amount_asset).bind(total_fiat).bind(status_code).bind(create_time).bind(now).bind(buyer_nick).bind(seller_nick).bind(now)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_orders(&self, limit: i64) -> Result<Vec<OrderRow>> {
        let rows = sqlx::query(r#"SELECT order_number, trade_type, fiat, asset, amount_asset, total_fiat, price, order_status_code, buyer_nickname, seller_nickname, has_payment_detail, last_api_sync_ts FROM orders ORDER BY create_time_ms DESC LIMIT ?"#)
            .bind(limit).fetch_all(&self.pool).await?;
        let mut out = Vec::new();
        for r in rows {
            let code: i64 = r.get("order_status_code");
            out.push(OrderRow {
                order_number: r.get("order_number"),
                trade_type: r.get("trade_type"),
                fiat: r.get("fiat"),
                asset: r.get("asset"),
                amount_asset: r.get("amount_asset"),
                total_fiat: r.get("total_fiat"),
                price: r.get("price"),
                status_code: code,
                status_label: map_status(code),
                buyer_nickname: r.get("buyer_nickname"),
                seller_nickname: r.get("seller_nickname"),
                has_payment_detail: r.get::<i64,_>("has_payment_detail") == 1,
                last_api_sync_ts: r.get("last_api_sync_ts")
            });
        }
        Ok(out)
    }
}

fn map_status(code: i64) -> String {
    match code {
        1 => "Đang chờ xử lý".into(),
        2 => "Người mua đã thanh toán".into(),
        4 | 6 => "Đã hoàn thành".into(),
        5 => "Đã hủy".into(),
        _ => format!("Code{}", code)
    }
}
