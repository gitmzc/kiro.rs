# kiro-rs 仓库上下文报告

## 项目概述
- **项目类型**: Rust API 代理服务
- **版本**: 2025.12.7
- **Rust Edition**: 2024
- **核心功能**: Anthropic Claude API 兼容代理，转换为 Kiro API 请求

## 技术栈
| 依赖 | 版本 | 用途 |
|------|------|------|
| axum | 0.8 | Web 框架 (含 multipart) |
| tokio | 1.0 | 异步运行时 |
| reqwest | 0.12 | HTTP 客户端 |
| serde/serde_json | 1.0 | 序列化 |
| tracing | 0.1 | 日志系统 |
| tower-http | 0.6 | CORS 中间件 |
| parking_lot | 0.12 | 高性能同步原语 |

## 现有 Admin API 端点
| 方法 | 路径 | 功能 |
|------|------|------|
| GET | `/api/admin/credentials` | 获取所有凭据状态 |
| POST | `/api/admin/credentials/upload` | 上传凭据文件 |
| POST | `/api/admin/credentials/{index}/disabled` | 设置禁用状态 |
| POST | `/api/admin/credentials/{index}/priority` | 设置优先级 |
| POST | `/api/admin/credentials/{index}/reset` | 重置失败计数 |
| GET | `/api/admin/credentials/{index}/balance` | 获取余额 |
| DELETE | `/api/admin/credentials/{index}` | 删除凭据 |

## 代码模式
- 模块结构: `router.rs` + `handlers.rs` + `service.rs` + `types.rs` + `error.rs`
- 状态管理: `AdminState` 包含 `Arc<AdminService>`
- 认证: `admin_auth_middleware` 使用常量时间比较

## 集成点
- 新路由在 `src/admin/router.rs` 注册
- 业务逻辑在 `src/admin/service.rs` 实现
- 类型定义在 `src/admin/types.rs`
- 通过 `MultiTokenManager` 访问凭据数据
