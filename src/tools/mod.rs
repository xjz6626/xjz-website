// src/tools/mod.rs

// 声明并导出子模块
pub mod image_resizer;
pub mod background_changer;
pub mod ip_fetcher;
pub mod config;
pub mod fake_identity; // <-- 新增

// 重新导出处理函数
pub use image_resizer::handle_resize_image;
pub use background_changer::handle_change_background;
pub use ip_fetcher::handle_get_ip;
pub use fake_identity::handle_get_fake_identity; // <-- 新增
// 重新导出配置结构体
pub use config::ToolsConfig;