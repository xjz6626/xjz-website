use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredProject {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub language: Option<String>,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub topics: Vec<String>,
    pub open_issues_count: u32,
    pub size: u32,
    pub default_branch: String,
    pub archived: bool,
    pub fork: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredArticle {
    pub id: String,
    pub title: String,
    pub content: String,
    pub excerpt: String,
    pub file_path: String,
    pub file_url: String,  // GitHub文件查看URL
    pub repo_name: String,
    pub repo_url: String,
    pub updated_at: DateTime<Utc>,
    pub file_size: u32,
    pub reading_time: u32, // 预估阅读时间（分钟）
    pub category: String,
    pub tags: Vec<String>,
    pub featured: bool,    // 是否为精选文章
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub username: String,
    pub total_repos: u32,
    pub total_stars: u32,
    pub total_forks: u32,
    pub followers: u32,
    pub following: u32,
    pub public_gists: u32,
    pub avatar_url: String,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub blog: Option<String>,
    pub company: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub languages: HashMap<String, u32>, // 语言名称 -> 字节数
    pub total_bytes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubData {
    pub projects: Vec<StoredProject>,
    pub articles: Vec<StoredArticle>,
    pub user_stats: Option<UserStats>,
    pub language_stats: Option<LanguageStats>,
    pub last_updated: DateTime<Utc>,
    pub next_update: DateTime<Utc>,
}

impl Default for GitHubData {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            projects: Vec::new(),
            articles: Vec::new(),
            user_stats: None,
            language_stats: None,
            last_updated: now,
            next_update: now + chrono::Duration::days(3),
        }
    }
}