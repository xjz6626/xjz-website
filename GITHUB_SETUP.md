# GitHub API 配置说明

## 配置GitHub Personal Access Token

### 步骤1: 创建GitHub Token

1. 访问 https://github.com/settings/personal-access-tokens/tokens
2. 点击 "Generate new token (classic)"
3. 设置名称：比如 "xjz_website_api"
4. 选择权限（scopes）：
   - ✅ `public_repo` - 访问公开仓库
   - ✅ `read:user` - 读取用户信息
5. 点击 "Generate token"
6. **重要**：复制生成的token（只显示一次）

### 步骤2: 配置Token

在 `src/github/config.rs` 文件中，找到这一行：
```rust
let hardcoded_token = "YOUR_TOKEN_HERE";
```

将 `"YOUR_TOKEN_HERE"` 替换为你刚才复制的token：
```rust
let hardcoded_token = "ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
```

### 步骤3: 重启服务器

```bash
cargo run
```

启动时你会看到配置状态：
```
=== GitHub配置状态 ===
用户名: xjz6626
Token: 已配置 ✅
API限制: 5000次/小时
=====================
```

## API 限制对比

- **无token**: 60次/小时
- **有token**: 5000次/小时

## 验证配置

配置完成后，测试API：
```bash
curl http://127.0.0.1:3000/api/update
curl http://127.0.0.1:3000/api/stats
```

应该能看到正确的项目数据而不是错误信息。