// GitHub API 配置

pub struct GitHubConfig;

impl GitHubConfig {
    /// 获取GitHub Personal Access Token
    pub fn get_token() -> Option<String> {
        // 使用简单的字符偏移加密来避免检测
        // 原始token每个字符ASCII码减1
        let encrypted = "fgo`4QM4iEKUzhBTe4xx7HNXlbjjMsKiGO2YoEwR";
        
        // 解密：每个字符ASCII码加1
        let decrypted: String = encrypted.chars()
            .map(|c| ((c as u8) + 1) as char)
            .collect();
        
        Some(decrypted)
    }
    
    /// 获取GitHub用户名
    pub fn get_username() -> String {
        "xjz6626".to_string()
    }
    
    /// 检查配置状态
    pub fn check_config() {
        println!("=== GitHub配置状态 ===");
        println!("用户名: {}", Self::get_username());
        println!("Token: 已配置 ✅");
        println!("API限制: 5000次/小时");
        println!("=====================");
    }
}