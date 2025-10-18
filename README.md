# XJZ Personal Website

一个基于 Rust + Axum 构建的个人网站，具有动态GitHub集成功能。

## 功能特性

- 🚀 **高性能**: 基于 Rust + Axum 框架
- 📱 **响应式设计**: 支持多设备访问
- 🔗 **GitHub集成**: 动态获取项目和文章数据
- 📊 **实时统计**: 展示GitHub统计信息
- 📝 **动态博客**: 从GitHub仓库自动获取Markdown文章
- 🎨 **现代UI**: 简洁优雅的界面设计

## 技术栈

- **后端**: Rust, Axum, Tokio
- **前端**: HTML5, CSS3, JavaScript (ES6+)
- **模板引擎**: Askama
- **API集成**: GitHub REST API
- **部署**: Linux服务器

## 快速开始

### 环境要求

- Rust 1.70+
- Git

### 本地开发

1. 克隆项目
```bash
git clone <repository-url>
cd xjz_website
```

2. 配置GitHub Token (可选，用于API访问)
```bash
export GITHUB_TOKEN=your_github_token
```

3. 运行项目
```bash
cargo run
```

4. 访问网站
```
http://localhost:3000
```

## API端点

- `GET /` - 主页
- `GET /projects` - 项目页面
- `GET /blog` - 博客页面
- `GET /about` - 关于页面
- `GET /resume` - 简历页面
- `GET /contact` - 联系页面
- `GET /api/projects` - 获取GitHub项目数据
- `GET /api/articles` - 获取博客文章数据
- `GET /api/stats` - 获取GitHub统计数据

## 部署

### 生产环境构建

```bash
cargo build --release
```

### 环境变量

- `GITHUB_TOKEN`: GitHub API访问令牌（可选）
- `PORT`: 服务器端口（默认3000）
- `HOST`: 绑定地址（默认0.0.0.0）

### 服务器部署

1. 上传构建产物到服务器
2. 配置反向代理（如Nginx）
3. 设置systemd服务（可选）
4. 启动应用

## 项目结构

```
xjz_website/
├── src/
│   ├── main.rs              # 应用入口
│   ├── handlers/            # 路由处理器
│   ├── github/              # GitHub API集成
│   └── utils/               # 工具函数
├── templates/               # HTML模板
├── public/                  # 静态资源
├── data/                    # 数据文件
└── Cargo.toml              # 项目配置
```

## 贡献

欢迎提交Issue和Pull Request！

## 许可证

MIT License