// src/tools/background_changer.rs

use axum::{
    extract::{Query, Multipart},
    http::StatusCode,
    response::{IntoResponse, Response, Json},
};
use base64::{engine::general_purpose::STANDARD as BASE64Engine, Engine as _};
use image; // 导入 image crate
use reqwest::multipart;
use serde::Deserialize;
use serde_json::json;
// use std::env; // 不再需要直接读取 env
use tracing;

// 导入同级目录下的 ToolsConfig
use super::config::ToolsConfig;

#[derive(Deserialize)]
pub struct BgChangeParams {
    target_color: Option<String>,
}

pub async fn handle_change_background(
    Query(params): Query<BgChangeParams>,
    mut multipart: Multipart,
) -> Result<Response, (StatusCode, String)> {
    tracing::info!(target_color = ?params.target_color, "开始处理背景移除请求");

    let api_key = match ToolsConfig::get_removebg_api_key() {
        Some(key) => key,
        None => {
            tracing::error!("未能获取 remove.bg API 密钥。请检查配置。");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "服务器配置错误：缺少 API 密钥".to_string(),
            ));
        }
    };

    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap_or("unknown").to_string();

        if field_name == "image" {
            let file_name = field.file_name().unwrap_or("uploaded_image").to_string();
            let data = field.bytes().await.map_err(|e| {
                tracing::error!("读取图片数据失败: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "无法读取图片数据".to_string())
            })?;
            let _input_format = image::guess_format(&data).ok();
            
            tracing::info!(file_name = %file_name, size = %data.len(), "已读取上传的图片文件");

            let client = reqwest::Client::new();
            let image_part = multipart::Part::bytes(data.to_vec())
                .file_name(file_name.clone())
                .mime_str("application/octet-stream")
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("创建 image part 失败: {}", e)))?;

            let mut form = multipart::Form::new().part("image_file", image_part);

            if let Some(ref color) = params.target_color {
                let bg_color = color.trim_start_matches('#');
                if bg_color.len() == 6 && bg_color.chars().all(|c| c.is_ascii_hexdigit()) {
                     form = form.text("bg_color", bg_color.to_string());
                     tracing::info!("API 请求中包含背景色: {}", bg_color);
                } else {
                    tracing::warn!("提供的背景色 '{}' 不是有效的6位十六进制值，将忽略", color);
                }
            } else {
                tracing::info!("未提供背景色，将移除为透明背景");
            }
            form = form.text("size", "auto");

            let response = client
                .post("https://api.remove.bg/v1.0/removebg")
                .header("X-Api-Key", api_key) 
                .multipart(form)
                .send()
                .await
                .map_err(|e| {
                    tracing::error!("发送到 remove.bg API 失败: {}", e);
                    (StatusCode::SERVICE_UNAVAILABLE, format!("调用背景移除服务失败: {}", e))
                })?;

            if response.status().is_success() {
                // --- 修复点 ---
                // 必须在 response.bytes() 之前调用 .headers()，因为 .bytes() 会消耗 response
                let mime_type = response.headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("image/png").to_string();
                // --- 修复点结束 ---

                 let image_bytes = response.bytes().await.map_err(|e| {
                     tracing::error!("读取 remove.bg 响应体失败: {}", e);
                     (StatusCode::INTERNAL_SERVER_ERROR, "读取API响应失败".to_string())
                 })?;
                 
                 tracing::info!("成功从 remove.bg API 获取到 {} bytes 的图片", image_bytes.len());
                 
                 let base64_image = BASE64Engine.encode(&image_bytes);
                 let data_url = format!("data:{};base64,{}", mime_type, base64_image);
                 return Ok(Json(json!({ "success": true, "imageData": data_url })).into_response());
            
            } else {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "无法读取错误详情".to_string());
                
                tracing::error!(status = %status, error_details = %error_text, "remove.bg API 请求失败");
                
                return Err((
                    status, 
                    format!("背景移除服务失败: {}", error_text),
                ));
            }
        }
    }

    Err((StatusCode::BAD_REQUEST, "缺少图片文件".to_string()))
}