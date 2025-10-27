# XJZ Website 系统服务管理

## 🎉 部署完成！

XJZ个人网站已成功配置为Linux系统服务，现在可以在后台自动运行，并在系统重启后自动启动。

## 📊 服务状态

- **服务名称**: `xjz-website`
- **运行端口**: `8181`
- **工作目录**: `/home/xjz/workplace/xjz-website`
- **可执行文件**: `/home/xjz/workplace/xjz-website/target/release/xjz_website`
- **配置文件**: `/etc/systemd/system/xjz-website.service`

## 🔧 常用管理命令

### 查看服务状态
```bash
sudo systemctl status xjz-website
```

### 启动服务
```bash
sudo systemctl start xjz-website
```

### 停止服务
```bash
sudo systemctl stop xjz-website
```

### 重启服务
```bash
sudo systemctl restart xjz-website
```

### 启用开机自启（已配置）
```bash
sudo systemctl enable xjz-website
```

### 禁用开机自启
```bash
sudo systemctl disable xjz-website
```

### 查看实时日志
```bash
sudo journalctl -u xjz-website -f
```

### 查看最近日志
```bash
sudo journalctl -u xjz-website -n 50
```

## 🌐 访问地址

- **本地访问**: http://localhost:8181
- **局域网访问**: http://[你的内网IP]:8181

## 📝 自动更新机制

网站会自动检查和更新GitHub数据：
- **更新频率**: 每1天自动检查一次
- **触发方式**: 当访问API时自动检查数据是否过期
- **数据内容**: 项目信息、README文档、用户统计等
- **缓存策略**: 未过期时使用本地缓存，提高响应速度

服务配置文件位于 `/etc/systemd/system/xjz-website.service`：

```ini
[Unit]
Description=XJZ Personal Website Server
Documentation=https://github.com/xjz6626/xjz-website
After=network.target

[Service]
Type=simple
User=xjz
Group=xjz
WorkingDirectory=/home/xjz/workplace/xjz-website
ExecStart=/home/xjz/workplace/xjz-website/target/release/xjz_website
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

# Environment
Environment=RUST_LOG=info

# Security settings
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=false
ReadWritePaths=/home/xjz/workplace/xjz-website/data
PrivateTmp=true

# Network
PrivateNetwork=false

[Install]
WantedBy=multi-user.target
```

## 🔄 更新网站代码

当你修改代码后，需要重新构建和重启服务：

```bash
cd /home/xjz/workplace/xjz-website

# 构建新版本
cargo build --release

# 重启服务
sudo systemctl restart xjz-website

# 检查状态
sudo systemctl status xjz-website
```

## 🔧 API测试

### 查看统计数据
```bash
curl http://localhost:8181/api/stats | jq .
```

### 强制更新GitHub数据
```bash
curl http://localhost:8181/api/update
```

### 检查网站响应
```bash
curl -I http://localhost:8181
```

## 🛡️ 防火墙配置（可选）

如果需要外网访问，需要开放8181端口：

### Fedora/CentOS (firewalld)
```bash
sudo firewall-cmd --permanent --add-port=8181/tcp
sudo firewall-cmd --reload
```

### Ubuntu (ufw)
```bash
sudo ufw allow 8181/tcp
```

## 📋 故障排除

### 服务无法启动
1. 检查可执行文件权限：`ls -la /home/xjz/workplace/xjz-website/target/release/xjz_website`
2. 查看详细错误日志：`sudo journalctl -u xjz-website -n 100`
3. 检查端口是否被占用：`sudo netstat -tlnp | grep 8181`

### SELinux权限问题 ⚠️
如果看到 "Permission denied" 错误，可能是SELinux问题：

**症状**: 
- 服务状态显示 `activating (auto-restart)`
- 日志显示 `Permission denied` 和 `exit-code 203/EXEC`

**解决方法**:
```bash
# 方法1: 使用修复脚本
./fix_selinux.sh

# 方法2: 手动修复
sudo chcon -t bin_t /home/xjz/workplace/xjz-website/target/release/xjz_website
sudo systemctl restart xjz-website
```

**验证修复**:
```bash
# 检查SELinux上下文 (应该显示 bin_t)
ls -Z /home/xjz/workplace/xjz-website/target/release/xjz_website

# 检查服务状态 (应该显示 active (running))
sudo systemctl status xjz-website
```

### GitHub API 401错误
1. 检查Token是否过期
2. 更新 `src/github/config.rs` 中的Token
3. 重新构建并重启服务

### 数据更新失败
1. 检查网络连接：`curl -I https://api.github.com`
2. 验证Token权限：需要 `public_repo` 和 `read:user` 权限
3. 查看详细日志了解具体错误

## ✅ 成功指标

- ✅ 服务状态：`Active: active (running)`
- ✅ 网站响应：返回200状态码
- ✅ API正常：`/api/stats` 返回正确数据
- ✅ GitHub集成：能正常获取仓库和文章数据
- ✅ 开机自启：系统重启后自动运行

## 🎯 后续优化建议

1. 配置反向代理（Nginx）支持HTTPS
2. 设置定期备份GitHub数据
3. 添加监控和告警
4. 优化API缓存策略
5. 配置CDN加速静态资源

恭喜！你的个人网站现在已经是一个专业的系统服务了！🚀