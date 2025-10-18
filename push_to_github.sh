#!/bin/bash

# GitHub仓库推送脚本
# 请先在GitHub网站创建仓库，然后运行此脚本

echo "请输入您的GitHub用户名:"
read GITHUB_USERNAME

echo "请输入您的仓库名(例如: xjz-website):"
read REPO_NAME

# 添加远程仓库
git remote add origin https://github.com/xjz6626/xjz-website.git

# 推送到主分支
git branch -M main
git push -u origin main

echo "✅ 项目已成功推送到 GitHub!"
echo "🌐 访问: https://github.com/$GITHUB_USERNAME/$REPO_NAME"