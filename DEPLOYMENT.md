# Fedora + Cloudflare 部署指南

## 环境要求

- Fedora Server (推荐 Fedora 38+)
- Rust 1.70+
- Git
- Cloudflare 账户

## 服务器部署步骤

### 1. 系统准备

```bash
# 更新系统
sudo dnf update -y

# 安装必要工具
sudo dnf install -y git curl gcc openssl-devel pkg-config

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. 克隆项目

```bash
git clone https://github.com/xjz6626/xjz-website.git
cd xjz-website
```

### 3. 构建生产版本

```bash
cargo build --release
```

### 4. 创建systemd服务

```bash
sudo tee /etc/systemd/system/xjz-website.service > /dev/null << 'EOF'
[Unit]
Description=XJZ Personal Website
After=network.target

[Service]
Type=simple
User=fedora
WorkingDirectory=/home/fedora/xjz-website
Environment=RUST_LOG=info
Environment=PORT=8181
Environment=HOST=127.0.0.1
ExecStart=/home/fedora/xjz-website/target/release/xjz_website
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

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

### 6. 配置防火墙

```bash
# 开放端口（如果使用防火墙）
sudo firewall-cmd --permanent --add-port=8181/tcp
sudo firewall-cmd --reload

# 或者允许HTTP/HTTPS（推荐通过反向代理）
sudo firewall-cmd --permanent --add-service=http
sudo firewall-cmd --permanent --add-service=https
sudo firewall-cmd --reload
```

## Cloudflare 配置

### 1. 域名配置

1. 登录 [Cloudflare Dashboard](https://dash.cloudflare.com)
2. 添加你的域名
3. 按照提示更改域名服务器到Cloudflare

### 2. DNS 记录配置

添加以下DNS记录：

```
类型: A
名称: @（或你的子域名）
内容: 你的服务器IP地址
代理状态: 已代理（橙色云朵）
TTL: 自动
```

如果使用子域名：
```
类型: A
名称: www
内容: 你的服务器IP地址
代理状态: 已代理（橙色云朵）
```

### 3. SSL/TLS 配置

1. 进入 `SSL/TLS` > `概述`
2. 选择加密模式：`灵活` 或 `完全`
3. 进入 `边缘证书`
4. 开启 `始终使用HTTPS`
5. 开启 `HSTS`

### 4. 性能优化

进入 `速度` 页面：
- 开启 `自动缩小`（CSS、HTML、JavaScript）
- 开启 `Brotli` 压缩
- 开启 `火箭加载器`（可选）

### 5. 页面规则（可选）

创建页面规则来优化缓存：

```
URL: yourdomain.com/public/*
设置:
- 缓存级别: 缓存所有内容
- 边缘缓存TTL: 1个月
```

```
URL: yourdomain.com/api/*
设置:
- 缓存级别: 绕过
```

## 安全配置

### 1. Cloudflare 安全设置

进入 `安全性` 页面：
- 设置安全级别：`中` 或 `高`
- 开启 `机器人斗争模式`（可选）
- 配置 `速率限制`（可选）

### 2. 防火墙规则

创建防火墙规则只允许Cloudflare IP访问：

```bash
# 获取Cloudflare IP列表并配置防火墙（可选）
curl -s https://www.cloudflare.com/ips-v4 | while read ip; do
    sudo firewall-cmd --permanent --add-rich-rule="rule family='ipv4' source address='$ip' port protocol='tcp' port='8181' accept"
done

sudo firewall-cmd --reload
```

## 监控和维护

### 1. 查看日志

```bash
# 查看应用日志
sudo journalctl -u xjz-website -f

# 查看系统日志
sudo journalctl -xe
```

### 2. 更新部署

```bash
cd /home/fedora/xjz-website
git pull origin main
cargo build --release
sudo systemctl restart xjz-website
```

### 3. 监控服务状态

```bash
# 检查服务状态
sudo systemctl status xjz-website

# 检查端口监听
sudo ss -tlnp | grep :8181

# 检查进程
ps aux | grep xjz_website
```

## 故障排除

### 应用无法启动
```bash
# 检查编译错误
cargo build --release

# 检查权限
sudo chown -R fedora:fedora /home/fedora/xjz-website

# 检查日志
sudo journalctl -u xjz-website --no-pager
```

### Cloudflare 问题
- 检查DNS解析：`dig yourdomain.com`
- 验证SSL证书：浏览器访问 `https://yourdomain.com`
- 清除Cloudflare缓存：Dashboard > 缓存 > 清除所有内容

### 性能优化
- 启用 Cloudflare 的 `Argo Smart Routing`
- 配置 `缓存规则` 提高静态资源缓存
- 使用 `Cloudflare Analytics` 监控性能

## 备份策略

```bash
# 创建备份脚本
cat > /home/fedora/backup.sh << 'EOF'
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/home/fedora/backups"
mkdir -p $BACKUP_DIR

# 备份应用目录
tar -czf $BACKUP_DIR/xjz-website_$DATE.tar.gz /home/fedora/xjz-website

# 保留最近7天的备份
find $BACKUP_DIR -name "xjz-website_*.tar.gz" -mtime +7 -delete
EOF

chmod +x /home/fedora/backup.sh

# 添加到定时任务（每天凌晨2点备份）
echo "0 2 * * * /home/fedora/backup.sh" | crontab -
```