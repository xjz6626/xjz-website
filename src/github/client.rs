use reqwest::Client;
use std::env;
use super::models::*;
use super::config::GitHubConfig;

pub struct GitHubClient {
    client: Client,
    base_url: String,
    token: Option<String>,
    username: String,
}

impl GitHubClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::new();
        let base_url = "https://api.github.com".to_string();
        
        // 使用配置模块获取token和用户名
        let token = GitHubConfig::get_token();
        let username = GitHubConfig::get_username();
        
        // 显示配置状态
        GitHubConfig::check_config();
        
        Ok(Self {
            client,
            base_url,
            token,
            username,
        })
    }

    // 获取用户的所有公开仓库
    pub async fn get_user_repos(&self) -> Result<Vec<Repository>, Box<dyn std::error::Error>> {
        let url = format!("{}/users/{}/repos", self.base_url, self.username);
        
        let mut request = self.client.get(&url)
            .header("User-Agent", "xjz-website/1.0")
            .query(&[
                ("sort", "updated"),
                ("direction", "desc"),
                ("type", "owner"),
                ("per_page", "100")
            ]);

        // 如果有token，添加认证头
        if let Some(ref token) = self.token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let headers = response.headers().clone();
            
            // 检查是否是速率限制问题
            if status.as_u16() == 403 {
                if let Some(remaining) = headers.get("x-ratelimit-remaining") {
                    if remaining == "0" {
                        if let Some(reset_time) = headers.get("x-ratelimit-reset") {
                            return Err(format!(
                                "GitHub API速率限制已达上限。重置时间: {}。建议设置GITHUB_TOKEN环境变量以提高限制到5000次/小时。", 
                                reset_time.to_str().unwrap_or("unknown")
                            ).into());
                        }
                    }
                }
            }
            
            return Err(format!("GitHub API error: {} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")).into());
        }

        let repos: Vec<Repository> = response.json().await?;
        Ok(repos)
    }

    // 获取精选仓库（过滤和排序）
    pub async fn get_featured_repos(&self) -> Result<Vec<Repository>, Box<dyn std::error::Error>> {
        let all_repos = self.get_user_repos().await?;
        
        // 过滤和排序逻辑
        let featured_repos = all_repos
            .into_iter()
            .filter(|repo| {
                // 排除fork的仓库
                !repo.fork &&
                // 排除archived的仓库
                !repo.archived &&
                // 可以添加更多过滤条件，比如star数量、最近活跃度等
                true
            })
            .collect::<Vec<_>>();
        
        Ok(featured_repos)
    }

    // 获取用户信息
    pub async fn get_user_info(&self) -> Result<User, Box<dyn std::error::Error>> {
        let url = format!("{}/users/{}", self.base_url, self.username);
        
        let mut request = self.client.get(&url)
            .header("User-Agent", "xjz-website/1.0");

        if let Some(ref token) = self.token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let headers = response.headers().clone();
            
            // 检查是否是速率限制问题
            if status.as_u16() == 403 {
                if let Some(remaining) = headers.get("x-ratelimit-remaining") {
                    if remaining == "0" {
                        if let Some(reset_time) = headers.get("x-ratelimit-reset") {
                            return Err(format!(
                                "GitHub API速率限制已达上限。重置时间: {}。建议设置GITHUB_TOKEN环境变量以提高限制到5000次/小时。", 
                                reset_time.to_str().unwrap_or("unknown")
                            ).into());
                        }
                    }
                }
            }
            
            return Err(format!("GitHub API error: {} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")).into());
        }

        let user: User = response.json().await?;
        Ok(user)
    }

    // 获取仓库内容
    pub async fn get_repo_contents(&self, repo_name: &str, path: &str) -> Result<Vec<RepoContent>, Box<dyn std::error::Error>> {
        let url = format!("{}/repos/{}/{}/contents/{}", self.base_url, self.username, repo_name, path);
        
        let mut request = self.client.get(&url)
            .header("User-Agent", "xjz-website/1.0");

        if let Some(ref token) = self.token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            return Err(format!("GitHub API error: {} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")).into());
        }

        let contents: Vec<RepoContent> = response.json().await?;
        Ok(contents)
    }

    // 获取文件内容
    pub async fn get_file_content(&self, download_url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut request = self.client.get(download_url)
            .header("User-Agent", "xjz-website/1.0");

        if let Some(ref token) = self.token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            return Err(format!("GitHub API error: {} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")).into());
        }

        let content = response.text().await?;
        Ok(content)
    }
}