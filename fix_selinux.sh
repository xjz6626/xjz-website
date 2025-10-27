#!/bin/bash
# SELinux修复脚本 - 解决xjz-website服务权限问题

echo "🔧 修复 XJZ Website 服务的SELinux权限问题..."

EXECUTABLE="/home/xjz/workplace/xjz-website/target/release/xjz_website"

# 检查文件是否存在
if [ ! -f "$EXECUTABLE" ]; then
    echo "❌ 可执行文件不存在: $EXECUTABLE"
    echo "请先运行 'cargo build --release' 构建项目"
    exit 1
fi

# 检查当前SELinux上下文
echo "📋 当前SELinux上下文:"
ls -Z "$EXECUTABLE"

# 设置正确的SELinux上下文
echo "🔨 设置SELinux上下文为bin_t..."
sudo chcon -t bin_t "$EXECUTABLE"

# 验证更改
echo "✅ 更改后的SELinux上下文:"
ls -Z "$EXECUTABLE"

# 重启服务
echo "🔄 重启xjz-website服务..."
sudo systemctl restart xjz-website

# 检查服务状态
echo "📊 服务状态:"
sudo systemctl status xjz-website --no-pager -l

echo ""
echo "🎉 SELinux权限修复完成！"
echo "如果问题仍然存在，请查看日志: sudo journalctl -u xjz-website -n 20"