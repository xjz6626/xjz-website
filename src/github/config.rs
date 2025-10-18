// GitHub API 配置
// 部署时请将你的GitHub Personal Access Token设置在这里

pub struct GitHubConfig;

impl GitHubConfig {
    /// 获取GitHub Personal Access Token
    /// 
    /// 创建Token步骤：
    /// 1. 访问 https://github.com/settings/personal-access-tokens/tokens
    /// 2. 点击 "Generate new token (classic)"
    /// 3. 选择权限: public_repo, read:user
    /// 4. 复制生成的token并替换下面的 "YOUR_TOKEN_HERE"
    pub fn get_token() -> Option<String> {
        // 方法1: 优先使用环境变量（推荐生产环境）
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            if !token.is_empty() {
                return Some(token);
            }
        }
        
        // 方法2: 硬编码token（开发/测试环境）
        // 将 "YOUR_TOKEN_HERE" 替换为你的实际token
        let hardcoded_token = "YOUR_TOKEN_HERE";
        
        if hardcoded_token != "YOUR_TOKEN_HERE" {
            Some(hardcoded_token.to_string())
        } else {
            None
        }
    }
    
    /// 获取GitHub用户名
    pub fn get_username() -> String {
        std::env::var("GITHUB_USERNAME")
            .unwrap_or_else(|_| "xjz6626".to_string())
    }
    
    /// 检查配置状态
    pub fn check_config() {
        println!("=== GitHub配置状态 ===");
        println!("用户名: {}", Self::get_username());
        
        match Self::get_token() {
            Some(_) => {
                println!("Token: 已配置 ✅");
                println!("API限制: 5000次/小时");
            }
            None => {
                println!("Token: 未配置 ⚠️");
                println!("API限制: 60次/小时");
                println!("建议设置Token以提高API限制");
            }
        }
        println!("=====================");
    }
}