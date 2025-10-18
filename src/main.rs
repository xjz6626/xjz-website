use axum::{
    routing::get,
    Router,
    response::{Html, IntoResponse, Response, Json},
    http::StatusCode,
};
use tower_http::services::ServeDir;
use askama::Template;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use serde::Serialize;
use std::collections::HashMap;

mod github;
use github::{GitHubDataManager, StoredProject, StoredArticle};

// === 模板定义 ===
// 定义主页模板结构体，并关联到 `index.html` 文件
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

// 定义关于页模板结构体，并关联到 `about.html` 文件
#[derive(Template)]
#[template(path = "about.html")]
struct AboutTemplate;

// 定义项目页模板结构体，并关联到 `projects.html` 文件
#[derive(Template)]
#[template(path = "projects.html")]
struct ProjectsTemplate;

// 定义联系页模板结构体，并关联到 `contact.html` 文件
#[derive(Template)]
#[template(path = "contact.html")]
struct ContactTemplate;

// 定义简历页模板结构体，并关联到 `resume.html` 文件
#[derive(Template)]
#[template(path = "resume.html")]
struct ResumeTemplate;

// 定义博客页模板结构体，并关联到 `blog.html` 文件
#[derive(Template)]
#[template(path = "blog.html")]
struct BlogTemplate;


// === API 处理函数 ===
// GitHub用户名配置
const GITHUB_USERNAME: &str = "xjz6626";

// API响应数据结构
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
    last_updated: Option<String>,
}

// 获取项目数据的API
async fn api_projects() -> impl IntoResponse {
    let manager = match GitHubDataManager::new(GITHUB_USERNAME.to_string()) {
        Ok(m) => m,
        Err(e) => {
            return Json(ApiResponse::<Vec<StoredProject>> {
                success: false,
                data: None,
                message: format!("初始化失败: {}", e),
                last_updated: None,
            });
        }
    };
    
    match manager.get_data().await {
        Ok(github_data) => {
            Json(ApiResponse {
                success: true,
                data: Some(github_data.projects),
                message: "项目数据获取成功".to_string(),
                last_updated: Some(github_data.last_updated.to_rfc3339()),
            })
        },
        Err(e) => {
            Json(ApiResponse::<Vec<StoredProject>> {
                success: false,
                data: None,
                message: format!("获取项目数据失败: {}", e),
                last_updated: None,
            })
        }
    }
}

// 获取文章数据的API
async fn api_articles() -> impl IntoResponse {
    let manager = match GitHubDataManager::new(GITHUB_USERNAME.to_string()) {
        Ok(m) => m,
        Err(e) => {
            return Json(ApiResponse::<Vec<StoredArticle>> {
                success: false,
                data: None,
                message: format!("初始化失败: {}", e),
                last_updated: None,
            });
        }
    };
    
    match manager.get_data().await {
        Ok(github_data) => {
            Json(ApiResponse {
                success: true,
                data: Some(github_data.articles),
                message: "文章数据获取成功".to_string(),
                last_updated: Some(github_data.last_updated.to_rfc3339()),
            })
        },
        Err(e) => {
            Json(ApiResponse::<Vec<StoredArticle>> {
                success: false,
                data: None,
                message: format!("获取文章数据失败: {}", e),
                last_updated: None,
            })
        }
    }
}

// 获取统计数据的API
async fn api_stats() -> impl IntoResponse {
    let manager = match GitHubDataManager::new(GITHUB_USERNAME.to_string()) {
        Ok(m) => m,
        Err(e) => {
            let empty_stats: HashMap<&str, u32> = HashMap::new();
            return Json(ApiResponse {
                success: false,
                data: Some(empty_stats),
                message: format!("初始化失败: {}", e),
                last_updated: None,
            });
        }
    };
    
    match manager.get_data().await {
        Ok(github_data) => {
            // 计算项目统计
            let total_stars: u32 = github_data.projects.iter().map(|p| p.stargazers_count).sum();
            let total_forks: u32 = github_data.projects.iter().map(|p| p.forks_count).sum();
            
            let mut stats = HashMap::new();
            stats.insert("total_projects", github_data.projects.len() as u32);
            stats.insert("total_articles", github_data.articles.len() as u32);
            stats.insert("total_stars", total_stars);
            stats.insert("total_forks", total_forks);
            
            if let Some(user_stats) = &github_data.user_stats {
                stats.insert("followers", user_stats.followers);
                stats.insert("public_repos", user_stats.total_repos);
            }
            
            Json(ApiResponse {
                success: true,
                data: Some(stats),
                message: "统计数据获取成功".to_string(),
                last_updated: Some(github_data.last_updated.to_rfc3339()),
            })
        },
        Err(e) => {
            let empty_stats: HashMap<&str, u32> = HashMap::new();
            Json(ApiResponse {
                success: false,
                data: Some(empty_stats),
                message: format!("获取统计数据失败: {}", e),
                last_updated: None,
            })
        }
    }
}

// 强制更新数据的API
async fn api_force_update() -> impl IntoResponse {
    let manager = match GitHubDataManager::new(GITHUB_USERNAME.to_string()) {
        Ok(m) => m,
        Err(e) => {
            return Json(ApiResponse::<&str> {
                success: false,
                data: None,
                message: format!("初始化失败: {}", e),
                last_updated: None,
            });
        }
    };
    
    match manager.force_update().await {
        Ok(github_data) => {
            Json(ApiResponse {
                success: true,
                data: Some("数据更新完成"),
                message: format!("成功更新了 {} 个项目和 {} 篇文章", 
                               github_data.projects.len(), 
                               github_data.articles.len()),
                last_updated: Some(github_data.last_updated.to_rfc3339()),
            })
        },
        Err(e) => {
            Json(ApiResponse::<&str> {
                success: false,
                data: None,
                message: format!("强制更新失败: {}", e),
                last_updated: None,
            })
        }
    }
}

// === 页面处理函数 ===
// 处理根路径 `/` 的请求
async fn index() -> impl IntoResponse {
    let template = IndexTemplate {};
    HtmlTemplate(template)
}

// 处理 `/about` 路径的请求
async fn about() -> impl IntoResponse {
    let template = AboutTemplate {};
    HtmlTemplate(template)
}

// 处理 `/projects` 路径的请求
async fn projects() -> impl IntoResponse {
    let template = ProjectsTemplate {};
    HtmlTemplate(template)
}

// 处理 `/contact` 路径的请求
async fn contact() -> impl IntoResponse {
    let template = ContactTemplate {};
    HtmlTemplate(template)
}

// 处理 `/resume` 路径的请求
async fn resume() -> impl IntoResponse {
    let template = ResumeTemplate {};
    HtmlTemplate(template)
}

// 处理 `/blog` 路径的请求
async fn blog() -> impl IntoResponse {
    let template = BlogTemplate {};
    HtmlTemplate(template)
}

// === Axum 响应转换器 ===
// 一个辅助工具，用于将 Askama 模板安全地转换为 Axum 能理解的 HTML 响应
struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("渲染模板失败: {}", err),
            ).into_response(),
        }
    }
}


// === 主函数: 程序入口 ===
#[tokio::main]
async fn main() {
    // 设置静态文件服务，它会托管 `public` 文件夹下的所有内容
    let assets_service = ServeDir::new("public");

    // 创建应用路由
    let app = Router::new()
        // 注册动态页面路由
        .route("/", get(index))
        .route("/about", get(about))
        .route("/projects", get(projects))
        .route("/blog", get(blog))
        .route("/contact", get(contact))
        .route("/resume", get(resume))
        // 注册API路由
        .route("/api/projects", get(api_projects))
        .route("/api/articles", get(api_articles))
        .route("/api/stats", get(api_stats))
        .route("/api/update", get(api_force_update))
        // 注册静态文件服务（使用 fallback_service 替代 nest_service）
        .fallback_service(assets_service);

    // 绑定端口并启动服务
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 服务已启动，请访问 http://{}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}