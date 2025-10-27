use axum::{
    extract::{Multipart, Query},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use base64::{engine::general_purpose::STANDARD as BASE64Engine, Engine as _};
use image::{GenericImageView, ImageFormat, Rgba};
use serde::Deserialize;
use serde_json::json;
use std::io::Cursor;
use tracing; // 引入 tracing
use image::GenericImage; // 添加这一行

#[derive(Deserialize)]
pub struct BgChangeParams {
    original_color: String,
    target_color: String,
    tolerance: Option<u8>,
}

pub async fn handle_change_background(
    Query(params): Query<BgChangeParams>,
    mut multipart: Multipart,
) -> Result<Response, (StatusCode, String)> {
    tracing::info!("接收到背景更换请求: 原色={}, 新色={}, 容差={:?}",
        params.original_color, params.target_color, params.tolerance);

    let original_color_rgb = hex_to_rgb(&params.original_color)?;
    let target_color_rgb = hex_to_rgb(&params.target_color)?;
    let tolerance_sq = (params.tolerance.unwrap_or(20) as u32).pow(2); // 使用距离平方进行比较

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("unknown").to_string();
        if name == "image" {
            let data = field.bytes().await.map_err(|e| {
                tracing::error!("读取图片数据失败: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "读取图片失败".to_string())
            })?;
            let format = image::guess_format(&data).map_err(|e| {
                tracing::error!("无法识别图片格式: {}", e);
                (StatusCode::BAD_REQUEST, "无法识别格式".to_string())
            })?;
            let mut img = image::load_from_memory(&data).map_err(|e| {
                tracing::error!("加载图片失败: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "加载图片失败".to_string())
            })?;
            let (width, height) = img.dimensions();
            tracing::info!("图片尺寸: {}x{}", width, height);

            for y in 0..height {
                for x in 0..width {
                    let pixel = img.get_pixel(x, y);
                    if color_distance_sq(pixel.0, original_color_rgb) <= tolerance_sq {
                        img.put_pixel(x, y, Rgba(target_color_rgb));
                    }
                }
            }
            tracing::info!("背景色替换完成");

            let mut output_data = Cursor::new(Vec::new());
            img.write_to(&mut output_data, format).map_err(|e| {
                tracing::error!("图片编码失败: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("编码失败: {}", e))
            })?;

            let mime_type = match format {
                ImageFormat::Jpeg => "image/jpeg",
                ImageFormat::Png => "image/png",
                _ => "application/octet-stream", // Fallback, consider erroring instead
            };

            let base64_image = BASE64Engine.encode(output_data.into_inner());
            let data_url = format!("data:{};base64,{}", mime_type, base64_image);

            return Ok(Json(json!({ "success": true, "imageData": data_url })).into_response());
        }
    }
    Err((StatusCode::BAD_REQUEST, "缺少图片".to_string()))
}

// --- 辅助函数 ---
fn hex_to_rgb(hex: &str) -> Result<[u8; 4], (StatusCode, String)> {
     let hex = hex.trim_start_matches('#');
     if hex.len() != 6 {
         return Err((StatusCode::BAD_REQUEST, "无效的颜色格式 (应为 #RRGGBB)".to_string()));
     }
     let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| (StatusCode::BAD_REQUEST, "无效的红色值".to_string()))?;
     let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| (StatusCode::BAD_REQUEST, "无效的绿色值".to_string()))?;
     let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| (StatusCode::BAD_REQUEST, "无效的蓝色值".to_string()))?;
     Ok([r, g, b, 255])
}

fn color_distance_sq(c1: [u8; 4], c2: [u8; 4]) -> u32 {
     let r_diff = (c1[0] as i32 - c2[0] as i32).pow(2) as u32;
     let g_diff = (c1[1] as i32 - c2[1] as i32).pow(2) as u32;
     let b_diff = (c1[2] as i32 - c2[2] as i32).pow(2) as u32;
     r_diff + g_diff + b_diff
}