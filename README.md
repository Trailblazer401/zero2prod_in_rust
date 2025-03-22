# Zero2Prod Newsletter System

一个基于 Rust 构建的邮件简报订阅系统。

## 🌟 功能特性

- 📧 邮件订阅管理
- 👤 用户认证系统
- 📬 通讯发布功能
- 🔒 安全的密码管理
- ⚡ 异步任务处理
- 🔄 幂等性保证
- 📊 管理员仪表盘

## 🛠 技术栈

### 后端框架
- Actix-web: 高性能 Web 框架
- SQLx: 异步数据库操作
- Tokio: 异步运行时

### 数据存储
- PostgreSQL: 主数据库
- Redis: 会话管理和消息存储

### 安全性
- HMAC签名验证
- Argon2哈希
- 会话认证

### 监控和日志
- Tracing跨度追踪
- 结构化日志记录

## 🚀 快速开始

### 环境要求
- Rust 1.70+
- PostgreSQL 12+
- Redis 6+
- 支持 OpenSSL 的环境

### 安装步骤

1. 克隆仓库
```bash
git clone https://github.com/Trailblazer401/zero2prod.git
cd zero2prod
```

2. 配置环境变量
```bash
cp .env.example .env
# 编辑 .env 文件配置必要的环境变量
```

3. 数据库设置
```bash
sqlx database create
sqlx migrate run
```

4. 运行应用
```bash
cargo run
```

## 🏗 项目结构

```
src/
├── main.rs              # 应用入口
├── startup.rs           # 应用初始化
├── configurations/      # 配置管理
├── routes/             # HTTP 路由处理
├── domain/             # 业务领域模型
├── authentication/     # 认证相关
├── email_client/      # 邮件客户端
└── idempotency/       # 幂等性处理
```

## 📝 API 文档

### 公开接口
- `POST /subscriptions` - 订阅newsletter
- `GET /subscriptions/confirm` - 确认订阅
- `GET /health_check` - 健康检查

### 认证接口
- `GET /login` - 登录页面
- `POST /login` - 提交登录

### 管理接口
- `GET /admin/dashboard` - 管理仪表盘
- `POST /admin/newsletters` - 发布newsletter
- `GET /admin/password` - 密码修改页面

## 🔐 安全特性

- 密码强度要求
- 登录尝试限制
- 会话超时控制
- 安全的密码重置流程

## 💡 最佳实践

- 异步任务处理
- 数据库连接池
- 优雅的错误处理
- 完整的测试覆盖

## 📊 性能特性

- 异步处理
- 连接池管理
- Redis 缓存
- 任务队列

## 🤝 贡献指南

欢迎贡献代码！请：

1. Fork 本仓库
2. 创建特性分支
3. 提交变更
4. 推送到分支
5. 创建 Pull Request

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 👥 作者

- Trailblazer401 - [@Trailblazer401](https://github.com/Trailblazer401)

## 🙏 致谢

感谢Luca Palmieri带来的精彩项目与指引！

---
⭐️ 如果这个项目对您有帮助，请给一个星标支持！
