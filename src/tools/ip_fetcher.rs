use axum::{extract::ConnectInfo, response::Json};
use serde_json::json;
use std::net::SocketAddr;
use tracing; // 引入 tracing

pub async fn handle_get_ip(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> Json<serde_json::Value> {
    tracing::info!("查询 IP 地址请求来自: {}", addr);
    // 注意：这仍然是直接连接的 IP。处理反向代理 IP 需要额外配置。
    Json(json!({ "ip": addr.ip().to_string() }))
}