use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub clone_url: String,
    pub language: Option<String>,
    pub stargazers_count: i32,
    pub watchers_count: i32,
    pub forks_count: i32,
    pub open_issues_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: Option<DateTime<Utc>>,
    pub homepage: Option<String>,
    pub topics: Vec<String>,
    pub visibility: String,
    pub default_branch: String,
    pub license: Option<License>,
    pub size: i32,
    pub archived: bool,
    pub disabled: bool,
    pub fork: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub key: String,
    pub name: String,
    pub spdx_id: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub login: String,
    pub id: i64,
    pub avatar_url: String,
    pub html_url: String,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub email: Option<String>,
    pub blog: Option<String>,
    pub company: Option<String>,
    pub public_repos: i32,
    pub public_gists: i32,
    pub followers: i32,
    pub following: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub commit: CommitDetails,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitDetails {
    pub author: CommitAuthor,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub name: String,
    pub bytes: i32,
    pub percentage: f32,
}

// API响应结构
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
        }
    }
}

// GitHub仓库内容
#[derive(Debug, Clone, Deserialize)]
pub struct RepoContent {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub item_type: String, // "file" or "dir"
    pub size: u32,
    pub download_url: Option<String>,
    pub html_url: String,
}