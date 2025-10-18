#!/bin/bash
# XJZ Website 系统服务安装脚本

echo "🚀 开始配置 XJZ Website 系统服务..."

# 检查是否以root权限运行
if [ "$EUID" -ne 0 ]; then
    echo "请使用 sudo 运行此脚本"
    exit 1
fi

# 服务配置
SERVICE_NAME="xjz-website"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"
WORK_DIR="/home/xjz/workplace/xjz-website"
EXECUTABLE="${WORK_DIR}/target/release/xjz_website"

# 检查可执行文件是否存在
if [ ! -f "$EXECUTABLE" ]; then
    echo "❌ 可执行文件不存在: $EXECUTABLE"
    echo "请先运行 'cargo build --release' 构建项目"
    exit 1
fi

# 创建systemd服务文件
echo "📝 创建systemd服务文件..."
cat > "$SERVICE_FILE" << EOF
[Unit]
Description=XJZ Personal Website Server
Documentation=https://github.com/xjz6626/xjz-website
After=network.target

[Service]
Type=simple
User=xjz
Group=xjz
WorkingDirectory=$WORK_DIR
ExecStart=$EXECUTABLE
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
ReadWritePaths=$WORK_DIR/data
PrivateTmp=true

# Network
PrivateNetwork=false

[Install]
WantedBy=multi-user.target
EOF

echo "✅ 服务文件已创建: $SERVICE_FILE"

# 重新加载systemd配置
echo "🔄 重新加载systemd配置..."
systemctl daemon-reload

# 启用服务（开机自启）
echo "⚡ 启用服务（开机自启）..."
systemctl enable $SERVICE_NAME

# 启动服务
echo "🚀 启动服务..."
systemctl start $SERVICE_NAME

# 检查服务状态
echo "📊 检查服务状态..."
sleep 2
systemctl status $SERVICE_NAME --no-pager -l

echo ""
echo "🎉 XJZ Website 服务配置完成！"
echo ""
echo "常用命令："
echo "  查看状态: sudo systemctl status $SERVICE_NAME"
echo "  查看日志: sudo journalctl -u $SERVICE_NAME -f"
echo "  重启服务: sudo systemctl restart $SERVICE_NAME"
echo "  停止服务: sudo systemctl stop $SERVICE_NAME"
echo "  禁用服务: sudo systemctl disable $SERVICE_NAME"
echo ""
echo "网站地址: http://localhost:8181"