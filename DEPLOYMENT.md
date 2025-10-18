# 部署指南

## 环境要求

- Linux 服务器
- Rust 1.70+
- Nginx (推荐)
- Git

## 服务器部署步骤

### 1. 克隆项目

```bash
git clone <repository-url>
cd xjz_website
```

### 2. 构建生产版本

```bash
cargo build --release
```

### 3. 配置环境变量 (可选)

```bash
# 创建环境变量文件
cat > .env << EOF
GITHUB_TOKEN=your_github_token_here
PORT=3000
HOST=0.0.0.0
EOF
```

### 4. 创建systemd服务

```bash
sudo tee /etc/systemd/system/xjz-website.service > /dev/null << EOF
[Unit]
Description=XJZ Personal Website
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/path/to/xjz_website
Environment=RUST_LOG=info
EnvironmentFile=-/path/to/xjz_website/.env
ExecStart=/path/to/xjz_website/target/release/xjz_website
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF
```

### 5. 启动服务

```bash
sudo systemctl daemon-reload
sudo systemctl enable xjz-website
sudo systemctl start xjz-website
sudo systemctl status xjz-website
```

### 6. 配置Nginx反向代理

```bash
sudo tee /etc/nginx/sites-available/xjz-website > /dev/null << EOF
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }

    location /public/ {
        alias /path/to/xjz_website/public/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
EOF

sudo ln -s /etc/nginx/sites-available/xjz-website /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### 7. 配置SSL (推荐)

```bash
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d your-domain.com
```

## 更新部署

```bash
cd /path/to/xjz_website
git pull origin main
cargo build --release
sudo systemctl restart xjz-website
```

## 日志查看

```bash
# 查看应用日志
sudo journalctl -u xjz-website -f

# 查看Nginx日志
sudo tail -f /var/log/nginx/access.log
sudo tail -f /var/log/nginx/error.log
```

## 故障排除

### 检查服务状态
```bash
sudo systemctl status xjz-website
```

### 检查端口占用
```bash
sudo netstat -tlnp | grep :3000
```

### 检查防火墙
```bash
sudo ufw status
sudo ufw allow 80
sudo ufw allow 443
```