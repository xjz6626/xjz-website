use axum::{
    extract::ConnectInfo, // 需要 ConnectInfo 来获取 IP
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post}, // 需要 post 来处理表单提交
    Router,
};
use askama::Template;
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr; // 需要 SocketAddr
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt}; // 日志


// 引入模块
mod github;
mod tools; // 声明 tools 模块

// 使用模块中的内容
use github::{GitHubDataManager, StoredArticle, StoredProject};
use crate::tools::{handle_change_background, handle_get_ip, handle_resize_image, handle_get_fake_identity}; // <-- 添加 handle_get_fake_identity

// === 模板定义 ===
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "about.html")]
struct AboutTemplate;

#[derive(Template)]
#[template(path = "projects.html")]
struct ProjectsTemplate;

#[derive(Template)]
#[template(path = "contact.html")]
struct ContactTemplate;

#[derive(Template)]
#[template(path = "resume.html")]
struct ResumeTemplate;

#[derive(Template)]
#[template(path = "blog.html")]
struct BlogTemplate;

#[derive(Template)] // 添加 Tools 页面的模板
#[template(path = "tools.html")]
struct ToolsTemplate;

// === API 处理函数 ===
const GITHUB_USERNAME: &str = "xjz6626";

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
    last_updated: Option<String>,
}

async fn api_projects() -> impl IntoResponse {
    let manager = match GitHubDataManager::new(GITHUB_USERNAME.to_string()) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("GitHubDataManager 初始化失败: {}", e); // 添加日志
            return Json(ApiResponse::<Vec<StoredProject>> {
                success: false,
                data: None,
                message: format!("初始化失败: {}", e),
                last_updated: None,
            });
        }
    };

    match manager.get_data().await {
        Ok(github_data) => Json(ApiResponse {
            success: true,
            data: Some(github_data.projects),
            message: "项目数据获取成功".to_string(),
            last_updated: Some(github_data.last_updated.to_rfc3339()),
        }),
        Err(e) => {
            tracing::error!("获取项目数据失败: {}", e); // 添加日志
            Json(ApiResponse::<Vec<StoredProject>> {
                success: false,
                data: None,
                message: format!("获取项目数据失败: {}", e),
                last_updated: None,
            })
        }
    }
}

async fn api_articles() -> impl IntoResponse {
    let manager = match GitHubDataManager::new(GITHUB_USERNAME.to_string()) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("GitHubDataManager 初始化失败: {}", e); // 添加日志
            return Json(ApiResponse::<Vec<StoredArticle>> {
                success: false,
                data: None,
                message: format!("初始化失败: {}", e),
                last_updated: None,
            });
        }
    };

    match manager.get_data().await {
        Ok(github_data) => Json(ApiResponse {
            success: true,
            data: Some(github_data.articles),
            message: "文章数据获取成功".to_string(),
            last_updated: Some(github_data.last_updated.to_rfc3339()),
        }),
        Err(e) => {
            tracing::error!("获取文章数据失败: {}", e); // 添加日志
            Json(ApiResponse::<Vec<StoredArticle>> {
                success: false,
                data: None,
                message: format!("获取文章数据失败: {}", e),
                last_updated: None,
            })
        }
    }
}

async fn api_stats() -> impl IntoResponse {
    let manager = match GitHubDataManager::new(GITHUB_USERNAME.to_string()) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("GitHubDataManager 初始化失败: {}", e); // 添加日志
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
        }
        Err(e) => {
            tracing::error!("获取统计数据失败: {}", e); // 添加日志
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

async fn api_force_update() -> impl IntoResponse {
    let manager = match GitHubDataManager::new(GITHUB_USERNAME.to_string()) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("GitHubDataManager 初始化失败: {}", e); // 添加日志
            return Json(ApiResponse::<&str> {
                success: false,
                data: None,
                message: format!("初始化失败: {}", e),
                last_updated: None,
            });
        }
    };

    match manager.force_update().await {
        Ok(github_data) => Json(ApiResponse {
            success: true,
            data: Some("数据更新完成"),
            message: format!(
                "成功更新了 {} 个项目和 {} 篇文章",
                github_data.projects.len(),
                github_data.articles.len()
            ),
            last_updated: Some(github_data.last_updated.to_rfc3339()),
        }),
        Err(e) => {
            tracing::error!("强制更新数据失败: {}", e); // 添加日志
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
async fn index() -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {})
}

async fn about() -> impl IntoResponse {
    HtmlTemplate(AboutTemplate {})
}

async fn projects() -> impl IntoResponse {
    HtmlTemplate(ProjectsTemplate {})
}

async fn contact() -> impl IntoResponse {
    HtmlTemplate(ContactTemplate {})
}

async fn resume() -> impl IntoResponse {
    HtmlTemplate(ResumeTemplate {})
}

async fn blog() -> impl IntoResponse {
    HtmlTemplate(BlogTemplate {})
}

// 添加 tools 页面的处理函数
async fn tools() -> impl IntoResponse {
    HtmlTemplate(ToolsTemplate {})
}

// === Axum 响应转换器 ===
struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => {
                tracing::error!("模板渲染失败: {}", err); // 添加日志
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("渲染模板失败: {}", err),
                )
                    .into_response()
            }
        }
    }
}

// === 主函数: 程序入口 ===
#[tokio::main]
async fn main() {
    // 初始化 tracing 日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()), // 默认为 info 级别
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 从环境变量读取端口，默认为 8181
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8181".to_string())
        .parse::<u16>()
        .unwrap_or(8181);

    // 设置静态文件服务
    let assets_service = ServeDir::new("public");

    // 创建应用路由
    let app = Router::new()
        // 页面路由
        .route("/", get(index))
        .route("/about", get(about))
        .route("/projects", get(projects))
        .route("/blog", get(blog))
        .route("/contact", get(contact))
        .route("/resume", get(resume))
        .route("/tools", get(tools)) // 添加 tools 页面路由
        // GitHub API 路由
        .route("/api/projects", get(api_projects))
        .route("/api/articles", get(api_articles))
        .route("/api/stats", get(api_stats))
        .route("/api/update", get(api_force_update))
        // 工具 API 路由
        .route("/tools/resize-image", post(handle_resize_image)) // 图片大小调整
        .route("/tools/change-background", post(handle_change_background)) // 背景更换
        .route("/api/tools/my-ip", get(handle_get_ip)) // IP 查询
        .route("/api/tools/fake-identity", get(handle_get_fake_identity)) // <-- 新增 虚假身份生成
        // 静态文件服务 (放在最后作为 fallback)
        .fallback_service(assets_service);

    // 绑定端口并启动服务
    let addr = SocketAddr::from(([0, 0, 0, 0], port)); // 监听所有接口 0.0.0.0
    tracing::info!("🚀 服务已启动，监听地址 http://{}", addr); // 使用 tracing info! 宏

    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("❌ 无法绑定端口 {}: {}", port, e); // 使用 tracing error! 宏
            return;
        }
    };
    if let Err(e) = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await {
        tracing::error!("服务器运行出错: {}", e); // 添加服务器运行错误日志
    }
}