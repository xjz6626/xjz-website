use axum::{
    extract::{Multipart, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use image::ImageFormat;
use serde::Deserialize;
use std::io::Cursor;
use tracing;

// JPEG 编码器
use image::codecs::jpeg::JpegEncoder;

#[derive(Deserialize)]
pub struct ResizeParams {
    target_size_kb: usize,
}

pub async fn handle_resize_image(
    Query(params): Query<ResizeParams>,
    mut multipart: Multipart,
) -> Result<Response, (StatusCode, String)> {
    tracing::info!("接收到图片调整大小请求, 目标大小: {} KB", params.target_size_kb);
    let target_bytes = params.target_size_kb * 1024;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("unknown").to_string();
        if name == "image" {
            let data = field.bytes().await.map_err(|e| {
                tracing::error!("读取图片数据失败: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "无法读取图片数据".to_string())
            })?;
            let format = image::guess_format(&data).map_err(|e| {
                tracing::error!("无法识别图片格式: {}", e);
                (StatusCode::BAD_REQUEST, "无法识别图片格式".to_string())
            })?;
            tracing::info!("图片格式: {:?}", format);

            if format != ImageFormat::Jpeg && format != ImageFormat::Png {
                 return Err((StatusCode::BAD_REQUEST, "仅支持JPEG/PNG格式".to_string()));
            }

            let img = image::load_from_memory_with_format(&data, format).map_err(|e| {
                tracing::error!("加载图片失败: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "无法加载图片".to_string())
            })?;

            let mut quality: u8 = 90;
            let mut output_data: Vec<u8> = Vec::new();
            let max_iterations = 10;

            for i in 0..max_iterations {
                output_data.clear();
                let mut cursor = Cursor::new(&mut output_data);

                let encode_result = if format == ImageFormat::Jpeg {
                    let mut encoder = JpegEncoder::new_with_quality(&mut cursor, quality);
                    // 应用修复：添加 .into()
                    encoder.encode(img.as_bytes(), img.width(), img.height(), img.color().into())
                } else {
                    img.write_to(&mut cursor, ImageFormat::Png)
                };

                if let Err(e) = encode_result {
                     tracing::error!("图片编码失败 (尝试 {}): {}", i + 1, e);
                     if i == 0 {
                         return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("图片编码失败: {}", e)));
                     }
                     break;
                }

                tracing::info!(
                    "尝试 {}: 质量={}, 文件大小={} bytes",
                    i + 1,
                    quality,
                    output_data.len()
                );

                if output_data.len() <= target_bytes || quality <= 5 || format == ImageFormat::Png {
                    break;
                }

                quality = quality.saturating_sub(10);
                if quality < 5 {
                    quality = 5;
                }
            }

            if output_data.is_empty() {
                tracing::error!("未能成功编码图片");
                return Err((StatusCode::INTERNAL_SERVER_ERROR, "未能成功编码图片".to_string()));
            }

            tracing::info!("最终文件大小: {} bytes", output_data.len());

            let mime_type = match format {
                ImageFormat::Jpeg => "image/jpeg",
                ImageFormat::Png => "image/png",
                _ => unreachable!(),
            };

            return Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime_type)],
                output_data,
            )
                .into_response());
        }
    }
    Err((StatusCode::BAD_REQUEST, "缺少图片文件".to_string()))
}