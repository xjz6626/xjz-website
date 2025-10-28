
use tracing; // 保留 tracing

pub struct ToolsConfig;

impl ToolsConfig {
    /// 获取 remove.bg API 密钥
    /// 直接从硬编码（偏移加密）值解密。
    pub fn get_removebg_api_key() -> Option<String> {
        
        // 直接使用硬编码（偏移加密）值解密
        tracing::info!("尝试从硬编码值获取 remove.bg API 密钥");

        // !!! --- 重要：确保这里是你自己加密后的 API Key --- !!!
        let encrypted_key = "5ccRiIp7u50q3UqaO0wG5Sb7"; // <--- 确保替换成你加密后的 Key

        // 简单的解密逻辑（每个字符 ASCII + 1）
        let decrypted_key: String = encrypted_key
            .chars()
            .map(|c| ((c as u8).wrapping_add(1)) as char)
            .collect();

        // 基本验证密钥看起来不像占位符或示例值
        if decrypted_key.is_empty()
            || encrypted_key == "INSERT_YOUR_API_KEY_HERE" // 检查原始占位符
            || encrypted_key == "XntqB`st`k@ohJ`z012" // 检查示例加密值
        {
            tracing::error!(
                "硬编码的 remove.bg API 密钥无效或仍为示例值，请在 src/tools/config.rs 中设置！"
            );
            None // 返回 None 表示密钥无效
        } else {
            tracing::info!("成功从硬编码值获取 remove.bg API 密钥");
            Some(decrypted_key)
        }
    }

    /// 检查并打印配置状态 (可选)
    pub fn check_config() {
        println!("=== 工具配置状态 ===");
        match Self::get_removebg_api_key() {
            Some(_) => println!("remove.bg API Key: 已从代码配置 ✅"), // 修改提示
            None => {
                println!("remove.bg API Key: 未配置或无效 ❌"); // 修改提示
                println!("  请确保已在 src/tools/config.rs 中设置正确的加密密钥！");
            }
        }
        println!("=====================");
    }
}

// --- 加密/解密辅助 (保持不变，用于生成加密密钥) ---
/*
// examples/encrypt_key.rs
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("用法: cargo run --example encrypt_key -- <你的API Key>");
        return;
    }
    let key = &args[1];
    let encrypted: String = key.chars().map(|c| ((c as u8).wrapping_sub(1)) as char).collect();
    println!("加密后的 Key: {}", encrypted);
}
*/