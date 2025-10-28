use axum::{
    extract::ConnectInfo, 
    http::HeaderMap, // 引入 HeaderMap
    response::Json
};
use serde_json::json;
use std::net::SocketAddr;
use tracing;

/// 尝试从 HTTP 标头中提取真实 IP
fn get_real_ip(headers: &HeaderMap, fallback_addr: &SocketAddr) -> String {
    
    // 1. 检查 Cloudflare 标头 (最高优先级)
    if let Some(cf_ip) = headers
        .get("CF-Connecting-IP")
        .and_then(|val| val.to_str().ok())
    {
        tracing::info!("IP source: CF-Connecting-IP ({})", cf_ip);
        return cf_ip.to_string();
    }

    // 2. 检查 X-Forwarded-For 标头 (通用标准)
    if let Some(xforward_ip) = headers
        .get("X-Forwarded-For")
        .and_then(|val| val.to_str().ok())
    {
        // X-Forwarded-For 可能是一个列表: "client, proxy1, proxy2"
        // 我们需要的是第一个 (最左边的)
        if let Some(client_ip) = xforward_ip.split(',').next() {
            let trimmed_ip = client_ip.trim().to_string();
            tracing::info!("IP source: X-Forwarded-For ({})", trimmed_ip);
            return trimmed_ip;
        }
    }
    
    // 3. 检查 X-Real-IP (一些反向代理使用)
    if let Some(xreal_ip) = headers
        .get("X-Real-IP")
        .and_then(|val| val.to_str().ok())
    {
        tracing::info!("IP source: X-Real-IP ({})", xreal_ip);
        return xreal_ip.to_string();
    }

    // 4. 回退到直接连接的 Socket 地址
    // (在 Cloudflare 场景下，这会是 Cloudflare 的 IP)
    let fallback_ip = fallback_addr.ip().to_string();
    tracing::warn!("IP source: Fallback/SocketAddr ({})", fallback_ip);
    fallback_ip
}

pub async fn handle_get_ip(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap, // Axum 会自动注入
) -> Json<serde_json::Value> {
    
    tracing::info!("开始查询 IP 地址，原始连接来自: {}", addr);
    
    // 调用新函数来获取 IP
    let real_ip = get_real_ip(&headers, &addr);

    Json(json!({ "ip": real_ip }))
}