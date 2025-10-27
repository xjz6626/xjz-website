// 声明并导出子模块
pub mod image_resizer;
pub mod background_changer;
pub mod ip_fetcher;

// 重新导出处理函数，方便 main.rs 导入
pub use image_resizer::handle_resize_image;
pub use background_changer::handle_change_background;
pub use ip_fetcher::handle_get_ip;