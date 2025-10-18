use axum::{
    response::Json,
    extract::Path,
};
use serde_json::json;
use crate::github::GitHubClient;

// 获取所有仓库
pub async fn get_repos() -> Json<serde_json::Value> {
    match GitHubClient::new() {
        Ok(client) => {
            match client.get_featured_repos().await {
                Ok(repos) => Json(json!({
                    "success": true,
                    "data": repos,
                    "timestamp": chrono::Utc::now()
                })),
                Err(e) => Json(json!({
                    "success": false,
                    "error": format!("获取仓库失败: {}", e),
                    "timestamp": chrono::Utc::now()
                })),
            }
        }
        Err(e) => Json(json!({
            "success": false,
            "error": format!("GitHub客户端初始化失败: {}", e),
            "timestamp": chrono::Utc::now()
        })),
    }
}

// 获取用户信息
pub async fn get_user() -> Json<serde_json::Value> {
    match GitHubClient::new() {
        Ok(client) => {
            match client.get_user_info().await {
                Ok(user) => Json(json!({
                    "success": true,
                    "data": user,
                    "timestamp": chrono::Utc::now()
                })),
                Err(e) => Json(json!({
                    "success": false,
                    "error": format!("获取用户信息失败: {}", e),
                    "timestamp": chrono::Utc::now()
                })),
            }
        }
        Err(e) => Json(json!({
            "success": false,
            "error": format!("GitHub客户端初始化失败: {}", e),
            "timestamp": chrono::Utc::now()
        })),
    }
}

// 获取指定仓库详情
pub async fn get_repo_details(Path(repo_name): Path<String>) -> Json<serde_json::Value> {
    match GitHubClient::new() {
        Ok(client) => {
            match client.get_repo_details(&repo_name).await {
                Ok(repo) => Json(json!({
                    "success": true,
                    "data": repo,
                    "timestamp": chrono::Utc::now()
                })),
                Err(e) => Json(json!({
                    "success": false,
                    "error": format!("获取仓库详情失败: {}", e),
                    "timestamp": chrono::Utc::now()
                })),
            }
        }
        Err(e) => Json(json!({
            "success": false,
            "error": format!("GitHub客户端初始化失败: {}", e),
            "timestamp": chrono::Utc::now()
        })),
    }
}

// 获取指定仓库的语言统计
pub async fn get_repo_languages(Path(repo_name): Path<String>) -> Json<serde_json::Value> {
    match GitHubClient::new() {
        Ok(client) => {
            match client.get_repo_languages(&repo_name).await {
                Ok(languages) => Json(json!({
                    "success": true,
                    "data": languages,
                    "timestamp": chrono::Utc::now()
                })),
                Err(e) => Json(json!({
                    "success": false,
                    "error": format!("获取语言统计失败: {}", e),
                    "timestamp": chrono::Utc::now()
                })),
            }
        }
        Err(e) => Json(json!({
            "success": false,
            "error": format!("GitHub客户端初始化失败: {}", e),
            "timestamp": chrono::Utc::now()
        })),
    }
}

// 获取指定仓库的最新提交
pub async fn get_repo_commits(Path(repo_name): Path<String>) -> Json<serde_json::Value> {
    match GitHubClient::new() {
        Ok(client) => {
            match client.get_repo_commits(&repo_name, 10).await {
                Ok(commits) => Json(json!({
                    "success": true,
                    "data": commits,
                    "timestamp": chrono::Utc::now()
                })),
                Err(e) => Json(json!({
                    "success": false,
                    "error": format!("获取提交记录失败: {}", e),
                    "timestamp": chrono::Utc::now()
                })),
            }
        }
        Err(e) => Json(json!({
            "success": false,
            "error": format!("GitHub客户端初始化失败: {}", e),
            "timestamp": chrono::Utc::now()
        })),
    }
}