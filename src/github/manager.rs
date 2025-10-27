use crate::github::client::GitHubClient;
use crate::github::storage::*;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;
use tokio::time::{sleep, Duration};
use serde_json;

pub struct GitHubDataManager {
    client: GitHubClient,
    data_file: String,
    username: String,
}

impl GitHubDataManager {
    pub fn new(username: String) -> Result<Self, Box<dyn std::error::Error>> {
        let client = GitHubClient::new()?;
        let data_file = format!("data/github_{}.json", username);
        
        Ok(Self {
            client,
            data_file,
            username,
        })
    }

    /// 确保数据目录存在
    fn ensure_data_dir(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = Path::new(&self.data_file).parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// 加载本地存储的GitHub数据
    pub fn load_data(&self) -> Result<GitHubData, Box<dyn std::error::Error>> {
        if !Path::new(&self.data_file).exists() {
            return Ok(GitHubData::default());
        }

        let content = fs::read_to_string(&self.data_file)?;
        let data: GitHubData = serde_json::from_str(&content)?;
        Ok(data)
    }

    /// 保存数据到本地文件
    fn save_data(&self, data: &GitHubData) -> Result<(), Box<dyn std::error::Error>> {
        self.ensure_data_dir()?;
        let content = serde_json::to_string_pretty(data)?;
        fs::write(&self.data_file, content)?;
        println!("GitHub数据已保存到: {}", self.data_file);
        Ok(())
    }

    /// 检查是否需要更新数据
    pub fn needs_update(&self) -> bool {
        match self.load_data() {
            Ok(data) => Utc::now() >= data.next_update,
            Err(_) => true, // 如果加载失败，需要更新
        }
    }

    /// 获取用户的仓库并转换为StoredProject
    async fn fetch_projects(&self) -> Result<Vec<StoredProject>, Box<dyn std::error::Error>> {
        let repos = self.client.get_featured_repos().await?;
        
        let mut projects = Vec::new();
        for repo in repos {
            let project = StoredProject {
                id: repo.id as u64,
                name: repo.name,
                full_name: repo.full_name,
                description: repo.description,
                html_url: repo.html_url,
                language: repo.language,
                stargazers_count: repo.stargazers_count as u32,
                forks_count: repo.forks_count as u32,
                updated_at: repo.updated_at,
                created_at: repo.created_at,
                topics: repo.topics,
                open_issues_count: repo.open_issues_count as u32,
                size: repo.size as u32,
                default_branch: repo.default_branch,
                archived: repo.archived,
                fork: repo.fork,
            };
            projects.push(project);
        }

        // 按更新时间排序
        projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(projects)
    }

    /// 获取用户统计信息
    async fn fetch_user_stats(&self) -> Result<UserStats, Box<dyn std::error::Error>> {
        let user = self.client.get_user_info().await?;
        
        let stats = UserStats {
            username: user.login,
            total_repos: user.public_repos as u32,
            total_stars: 0, // 需要单独计算
            total_forks: 0, // 需要单独计算
            followers: user.followers as u32,
            following: user.following as u32,
            public_gists: user.public_gists as u32,
            avatar_url: user.avatar_url,
            bio: user.bio,
            location: user.location,
            blog: user.blog,
            company: user.company,
            created_at: user.created_at,
            updated_at: user.updated_at,
        };

        Ok(stats)
    }

    /// 从仓库中获取Markdown文档
    async fn fetch_articles(&self, projects: &[StoredProject]) -> Result<Vec<StoredArticle>, Box<dyn std::error::Error>> {
        let mut articles = Vec::new();
        
        // 限制处理的仓库数量，避免API限制
        for project in projects.iter().take(10) {
            println!("正在处理仓库: {}", project.name);
            match self.fetch_articles_from_repo(project).await {
                Ok(mut repo_articles) => {
                    println!("从仓库 {} 获取了 {} 篇文章", project.name, repo_articles.len());
                    articles.append(&mut repo_articles);
                },
                Err(e) => {
                    println!("跳过仓库 {}: {}", project.name, e);
                }
            }
        }

        // 按更新时间排序
        articles.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        println!("总共获取到 {} 篇文章", articles.len());
        Ok(articles)
    }

    /// 从单个仓库获取Markdown文档
    async fn fetch_articles_from_repo(&self, project: &StoredProject) -> Result<Vec<StoredArticle>, Box<dyn std::error::Error>> {
        let contents = self.client.get_repo_contents(&project.name, "").await?;
        let mut articles = Vec::new();

        for item in contents {
            if item.item_type == "file" && 
               item.name.ends_with(".md") && 
               !item.name.to_lowercase().starts_with("readme") {
                
                println!("找到Markdown文件: {}/{}", project.name, item.name);
                match self.create_article_from_file(&item, project).await {
                    Ok(article) => {
                        articles.push(article);
                        println!("成功创建文章: {}", item.name);
                    },
                    Err(e) => println!("跳过文件 {}/{}: {}", project.name, item.name, e),
                }
            }
        }

        Ok(articles)
    }

    /// 从文件创建文章对象
    async fn create_article_from_file(
        &self, 
        file: &crate::github::models::RepoContent, 
        project: &StoredProject
    ) -> Result<StoredArticle, Box<dyn std::error::Error>> {
        if let Some(download_url) = &file.download_url {
            let content = self.client.get_file_content(download_url).await?;
            
            // 提取标题
            let title = self.extract_title(&content, &file.name);
            
            // 生成摘要
            let excerpt = self.generate_excerpt(&content);
            
            // 估算阅读时间（按每分钟200-300字计算）
            let char_count = content.chars().count();
            let reading_time = (char_count / 250).max(1) as u32; // 假设每分钟阅读250字符
            
            // 推断分类
            let category = self.infer_category(&file.path, &project.name);
            
            // 提取标签
            let tags = self.extract_tags(&content, &project.name);
            
            // 判断是否为精选文章 (基于项目星数、文件大小、阅读时间等)
            let featured = project.stargazers_count > 2 || 
                          reading_time > 5 || 
                          file.size > 5000;

            // 从full_name中提取owner (格式: "owner/repo")
            let owner = project.full_name.split('/').next().unwrap_or("unknown");
            
            let article = StoredArticle {
                id: format!("{}-{}", project.name, file.name.replace(".md", "")),
                title,
                content,
                excerpt,
                file_path: file.path.clone(),
                file_url: format!("https://github.com/{}/blob/{}/{}", 
                    project.full_name, project.default_branch, file.path),
                repo_name: project.name.clone(),
                repo_url: project.html_url.clone(),
                updated_at: project.updated_at,
                file_size: file.size,
                reading_time,
                category,
                tags,
                featured,
            };

            Ok(article)
        } else {
            Err("文件没有下载URL".into())
        }
    }

    /// 提取文档标题
    fn extract_title(&self, content: &str, filename: &str) -> String {
        // 查找第一个 # 标题
        for line in content.lines().take(10) {
            let line = line.trim();
            if line.starts_with("# ") {
                return line[2..].trim().to_string();
            }
        }
        
        // 如果没找到，使用文件名
        filename.replace(".md", "").replace("-", " ").replace("_", " ")
    }

    /// 生成文章摘要
    fn generate_excerpt(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines()
            .filter(|line| !line.trim().starts_with('#') && !line.trim().is_empty())
            .take(3)
            .collect();
        
        let excerpt = lines.join(" ");
        if excerpt.len() > 150 {
            // 安全地切割UTF-8字符串，确保不会在字符中间切断
            let mut end = 150;
            while end > 0 && !excerpt.is_char_boundary(end) {
                end -= 1;
            }
            format!("{}...", &excerpt[..end])
        } else {
            excerpt
        }
    }

    /// 推断文章分类
    fn infer_category(&self, file_path: &str, repo_name: &str) -> String {
        let path_lower = file_path.to_lowercase();
        let repo_lower = repo_name.to_lowercase();

        if path_lower.contains("doc") || path_lower.contains("guide") {
            "文档指南".to_string()
        } else if repo_lower.contains("rust") || path_lower.contains("rust") {
            "Rust".to_string()
        } else if repo_lower.contains("web") || repo_lower.contains("frontend") {
            "Web开发".to_string()
        } else if repo_lower.contains("tool") || repo_lower.contains("script") {
            "工具脚本".to_string()
        } else {
            "技术文档".to_string()
        }
    }

    /// 提取标签
    fn extract_tags(&self, content: &str, repo_name: &str) -> Vec<String> {
        let mut tags = Vec::new();
        
        // 基于仓库名提取标签
        let repo_lower = repo_name.to_lowercase();
        if repo_lower.contains("rust") { tags.push("Rust".to_string()); }
        if repo_lower.contains("web") { tags.push("Web".to_string()); }
        if repo_lower.contains("tool") { tags.push("工具".to_string()); }
        if repo_lower.contains("script") { tags.push("脚本".to_string()); }
        
        // 基于内容提取标签
        let content_lower = content.to_lowercase();
        if content_lower.contains("rust") { tags.push("Rust".to_string()); }
        if content_lower.contains("javascript") || content_lower.contains("js") { 
            tags.push("JavaScript".to_string()); 
        }
        if content_lower.contains("python") { tags.push("Python".to_string()); }
        
        tags.sort();
        tags.dedup();
        tags.truncate(5); // 最多5个标签
        tags
    }

    /// 更新所有GitHub数据
    pub async fn update_data(&self) -> Result<GitHubData, Box<dyn std::error::Error>> {
        println!("开始更新GitHub数据...");
        
        // 获取项目信息
        println!("获取项目信息...");
        let projects = self.fetch_projects().await?;
        println!("找到 {} 个活跃项目", projects.len());

        // 获取用户统计
        println!("获取用户统计...");
        let user_stats = self.fetch_user_stats().await.ok();

        // 获取文章
        println!("获取Markdown文档...");
        let articles = self.fetch_articles(&projects).await?;
        println!("找到 {} 篇文档", articles.len());

        let now = Utc::now();
        let data = GitHubData {
            projects,
            articles,
            user_stats,
            language_stats: None, // 后续可以实现
            last_updated: now,
            next_update: now + chrono::Duration::days(1),
        };

        // 保存数据
        self.save_data(&data)?;
        println!("GitHub数据更新完成！下次更新时间: {}", data.next_update);

        Ok(data)
    }

    /// 获取数据（如果需要则自动更新）
    pub async fn get_data(&self) -> Result<GitHubData, Box<dyn std::error::Error>> {
        if self.needs_update() {
            println!("数据需要更新，正在从GitHub获取最新数据...");
            self.update_data().await
        } else {
            println!("使用本地缓存数据");
            self.load_data()
        }
    }

    /// 强制更新数据
    pub async fn force_update(&self) -> Result<GitHubData, Box<dyn std::error::Error>> {
        self.update_data().await
    }
}