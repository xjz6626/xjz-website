use axum::{
    extract::{Multipart, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use image::{
    codecs::jpeg::JpegEncoder,
    codecs::png::{CompressionType, FilterType as PngFilterType, PngEncoder},
    ColorType, DynamicImage, ImageBuffer,
    ImageEncoder, // 需要 ImageEncoder trait 来调用 write_image
    ImageFormat,
    Rgba,
    GenericImageView, // 需要 GenericImageView 来获取尺寸等
};
use imagequant;
use serde::Deserialize;
use std::io::Cursor;
use tracing;

#[derive(Deserialize)]
pub struct ResizeParams {
    target_size_kb: usize,
    output_format: Option<String>,
}

pub async fn handle_resize_image(
    Query(params): Query<ResizeParams>,
    mut multipart: Multipart,
) -> Result<Response, (StatusCode, String)> {
    let target_bytes = params.target_size_kb * 1024;
    let requested_format_str = params.output_format.unwrap_or_else(|| "auto".to_string());

    tracing::info!(
        "接收到图片调整大小请求, 目标大小: {} KB, 输出格式: {}",
        params.target_size_kb,
        requested_format_str
    );

    while let Some(field) = multipart.next_field().await.unwrap() {
        if field.name().unwrap_or("") == "image" {
            let data = field.bytes().await.map_err(|e| {
                tracing::error!("读取图片数据失败: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "无法读取图片数据".to_string())
            })?;

            let input_format = image::guess_format(&data).map_err(|e| {
                tracing::error!("无法识别图片格式: {}", e);
                (StatusCode::BAD_REQUEST, "无法识别图片格式".to_string())
            })?;
            tracing::info!("输入图片格式: {:?}", input_format);

            if input_format != ImageFormat::Jpeg && input_format != ImageFormat::Png {
                return Err((StatusCode::BAD_REQUEST, "仅支持JPEG/PNG输入格式".to_string()));
            }

            // --- 修改点：不再指定格式加载，让 image 库自动推断 ---
            let img = image::load_from_memory(&data).map_err(|e| {
                tracing::error!("加载图片失败: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "无法加载图片".to_string())
            })?;
            // --- 修改结束 ---

            let output_format = match requested_format_str.to_lowercase().as_str() {
                "jpeg" | "jpg" => ImageFormat::Jpeg,
                "png" => ImageFormat::Png,
                _ => input_format,
            };
            tracing::info!("目标输出格式: {:?}", output_format);

            let mut final_output_data: Vec<u8> = Vec::new();

            // --- JPEG 处理逻辑 ---
            if output_format == ImageFormat::Jpeg {
                let mut best_jpeg_data: Vec<u8> = Vec::new();
                let mut last_over_target_data: Option<Vec<u8>> = None;
                let min_quality: u8 = 10;
                let max_iterations = 15;

                // --- 修改点：提前进行颜色转换 ---
                // 检查是否需要从 RGBA 转换为 RGB
                let img_to_encode = if img.color() == ColorType::Rgba8 || img.color() == ColorType::La8 {
                    tracing::warn!("输入图片包含 Alpha 通道，将转换为 RGB8 以编码为 JPEG");
                    DynamicImage::ImageRgb8(img.to_rgb8()) // 转换为 RGB8
                } else {
                    img.clone() // 已经是 RGB 或 Luma，直接克隆使用
                                // 注意：这里简单克隆，对于大图可能消耗内存，可优化
                };
                // --- 修改结束 ---


                for i in 0..=max_iterations {
                    let quality = calculate_jpeg_quality_v2(i, max_iterations + 1, 100, min_quality);
                    let mut current_output = Vec::new();
                    let mut cursor = Cursor::new(&mut current_output);

                    let encode_result = {
                        let mut encoder = JpegEncoder::new_with_quality(&mut cursor, quality);
                        // --- 修改点：使用转换后的 img_to_encode ---
                        // 现在可以确信 color type 是 JpegEncoder 支持的 (RGB8 或 Luma8)
                        encoder.encode(
                            img_to_encode.as_bytes(),
                            img_to_encode.width(),
                            img_to_encode.height(),
                            img_to_encode.color().into(), // 颜色类型现在应该是 RGB8 或 Luma8
                        )
                        // --- 修改结束 ---
                    };

                    if let Err(e) = encode_result {
                        tracing::error!("JPEG 编码失败 (尝试 {}, 质量 {}): {}", i, quality, e);
                        if best_jpeg_data.is_empty() && last_over_target_data.is_none() {
                             return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("首次JPEG编码失败: {}", e)));
                        }
                        break;
                    }

                    tracing::info!(
                        "JPEG 尝试 {}: 质量={}, 文件大小={} bytes (目标: {} bytes)",
                        i, quality, current_output.len(), target_bytes
                    );

                    if current_output.len() <= target_bytes {
                        best_jpeg_data = current_output;
                        tracing::info!("找到满足条件的 JPEG 大小");
                        break;
                    } else {
                        last_over_target_data = Some(current_output);
                    }

                    if quality == min_quality {
                         tracing::warn!("已达最低 JPEG 质量");
                        break;
                    }
                }
                // ... (JPEG 结果选择逻辑不变) ...
                 if !best_jpeg_data.is_empty() {
                    final_output_data = best_jpeg_data;
                } else if let Some(last_data) = last_over_target_data {
                     tracing::warn!("所有 JPEG 压缩尝试都大于目标，使用质量最低的结果");
                    final_output_data = last_data;
                } else {
                     tracing::error!("未能生成有效的 JPEG 数据");
                     return Err((StatusCode::INTERNAL_SERVER_ERROR, "未能生成 JPEG 数据".to_string()));
                }

            // --- PNG 处理逻辑 (不变) ---
            } else {
                 let mut initial_png_data = Vec::new();
                {
                    let mut cursor = Cursor::new(&mut initial_png_data);
                    let encoder = PngEncoder::new_with_quality(
                        &mut cursor,
                        CompressionType::Best,
                        PngFilterType::Adaptive,
                    );
                     match encoder.write_image(
                        img.as_bytes(),
                        img.width(),
                        img.height(),
                        img.color().into(),
                    ) {
                        Ok(_) => {},
                        Err(e) => {
                             tracing::error!("初始 PNG 编码失败: {}", e);
                             return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("初始 PNG 编码失败: {}", e)));
                        }
                    }
                     tracing::info!("初始 PNG (最佳无损压缩) 编码大小: {} bytes", initial_png_data.len());
                }

                if initial_png_data.len() <= target_bytes {
                    tracing::info!("初始 PNG 大小已满足要求");
                    final_output_data = initial_png_data;
                } else {
                    tracing::info!("PNG 大小超过目标，尝试使用 imagequant 进行有损压缩...");
                     let rgba_img = match img {
                        DynamicImage::ImageRgba8(buffer) => buffer,
                        _ => {
                             tracing::warn!("输入图片非 RGBA8 ({:?})，转换为 RGBA8 以进行量化", img.color());
                            img.to_rgba8()
                        }
                    };

                    match create_quantized_png_rgba(&rgba_img) {
                        Ok(quantized_data) => {
                            tracing::info!("imagequant 压缩后大小: {} bytes", quantized_data.len());
                            if quantized_data.len() < initial_png_data.len() {
                                final_output_data = quantized_data;
                                tracing::info!("使用 imagequant 压缩结果");
                            } else {
                                final_output_data = initial_png_data;
                                tracing::warn!("imagequant 压缩效果不佳，使用原始 PNG (最佳无损压缩) 结果");
                            }
                        }
                        Err(e) => {
                            tracing::error!("imagequant 压缩失败: {}", e);
                            final_output_data = initial_png_data;
                            tracing::warn!("imagequant 失败，回退到原始 PNG (最佳无损压缩) 结果");
                        }
                    }
                }
            }

            // --- 填充逻辑 (不变) ---
            let current_size = final_output_data.len();
            if current_size < target_bytes {
                let padding_needed = target_bytes - current_size;
                tracing::info!(
                    "当前大小 {} bytes 小于目标 {} bytes，需要填充 {} bytes",
                    current_size, target_bytes, padding_needed
                );
                final_output_data.resize(target_bytes, 0);

                if final_output_data.len() == target_bytes {
                    tracing::info!("填充完成，最终大小: {} bytes", final_output_data.len());
                } else {
                    tracing::error!(
                        "填充后大小不匹配！预期 {}, 实际 {}",
                        target_bytes, final_output_data.len()
                    );
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "填充文件大小失败".to_string(),
                    ));
                }
            } else {
                tracing::info!(
                    "处理后大小 {} bytes 不小于目标 {} bytes，无需填充",
                    current_size, target_bytes
                );
            }
            // --- 填充逻辑结束 ---

            if final_output_data.is_empty() {
                tracing::error!("未能成功生成最终图片数据");
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "未能成功处理图片".to_string(),
                ));
            }

            tracing::info!("最终返回文件大小: {} bytes", final_output_data.len());

            let mime_type = match output_format {
                ImageFormat::Jpeg => "image/jpeg",
                ImageFormat::Png => "image/png",
                _ => unreachable!(),
            };

            return Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime_type)],
                final_output_data,
            )
                .into_response());
        }
    }
    Err((StatusCode::BAD_REQUEST, "缺少图片文件".to_string()))
}

// 辅助函数：计算 JPEG 质量 (不变)
fn calculate_jpeg_quality_v2(iteration: usize, max_iterations: usize, start_quality: u8, min_quality: u8) -> u8 {
     if max_iterations <= 1 || iteration == 0 {
        return start_quality.max(min_quality);
    }
    let adjusted_iteration = iteration.max(1);
    let quality_range = start_quality.saturating_sub(min_quality) as f32;
    let progress = (adjusted_iteration -1) as f32 / (max_iterations - 1).max(1) as f32;
    let current_quality = start_quality as f32 - quality_range * progress.sqrt();
    (current_quality.round() as u8).max(min_quality)
}


// 辅助函数：使用 imagequant 进行 PNG 有损压缩，输出 RGBA PNG (不变)
fn create_quantized_png_rgba(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> Result<Vec<u8>, String> { // 返回 String 错误
    // 1. 创建和配置 Attributes
    let mut liq = imagequant::new();
    liq.set_speed(5).map_err(|e| e.to_string())?;
    liq.set_quality(65, 85).map_err(|e| e.to_string())?; // 质量范围

    // 2. 创建 imagequant::Image
    let liq_pixels: Vec<imagequant::RGBA> = img
        .pixels()
        .map(|p| imagequant::RGBA { r: p[0], g: p[1], b: p[2], a: p[3] })
        .collect();

    let mut liq_img = liq.new_image(
            &liq_pixels[..],
            img.width() as usize,
            img.height() as usize,
            0.0, // gamma
        ).map_err(|e| e.to_string())?;

    // 3. 量化
    let mut quantization_result = liq.quantize(&mut liq_img).map_err(|e| e.to_string())?;

    // 4. 设置抖动
    quantization_result.set_dithering_level(1.0).map_err(|e| e.to_string())?;

    // 5. 重新映射获取调色板和像素索引
    let (palette, pixels_indices) = quantization_result.remapped(&mut liq_img).map_err(|e| e.to_string())?;

    // --- 创建新的 RGBA ImageBuffer ---
    let width = img.width();
    let height = img.height();
    let mut quantized_rgba_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);

    for (idx, pixel_index) in pixels_indices.iter().enumerate() {
        if let Some(color) = palette.get(*pixel_index as usize) {
            let x = (idx % width as usize) as u32;
            let y = (idx / width as usize) as u32;
            quantized_rgba_buffer.put_pixel(x, y, Rgba([color.r, color.g, color.b, color.a]));
        } else {
             tracing::error!("Invalid pixel index {} found during remapping", pixel_index);
             return Err(format!("Invalid pixel index {} found", pixel_index));
        }
    }

    // --- 编码新的 RGBA ImageBuffer 为 PNG ---
    let mut buffer = Vec::new();
    {
        let mut cursor = Cursor::new(&mut buffer);
        let encoder = PngEncoder::new_with_quality(
            &mut cursor,
            CompressionType::Best,
            PngFilterType::Adaptive,
        );
         encoder
            .write_image(
                quantized_rgba_buffer.as_raw(), // 使用新 buffer 的原始数据
                width,
                height,
                ColorType::Rgba8.into(), // 明确编码为 RGBA8
            )
            .map_err(|e| e.to_string())?;
    }

    if buffer.is_empty() {
        return Err("PNG encoding resulted in empty buffer".to_string());
    }

    Ok(buffer)
}