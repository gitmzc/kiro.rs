//! Admin API 路由配置

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use super::{
    handlers::{
        delete_credential, get_all_credentials, get_credential_balance, reset_failure_count,
        set_credential_disabled, set_credential_priority, upload_credential,
        get_stats_summary, get_stats_timeseries, get_stats_requests, get_admin_health,
        stream_logs, get_config, update_config,
        get_api_keys, create_api_key, update_api_key, delete_api_key, change_password,
    },
    middleware::{admin_auth_middleware, rate_limit_middleware, AdminState},
};

/// 创建 Admin API 路由
///
/// # 端点
/// - `GET /credentials` - 获取所有凭据状态
/// - `POST /credentials/upload` - 上传凭据文件（multipart/form-data）
/// - `POST /credentials/:id/disabled` - 设置凭据禁用状态
/// - `POST /credentials/:id/priority` - 设置凭据优先级
/// - `POST /credentials/:id/reset` - 重置失败计数
/// - `GET /credentials/:id/balance` - 获取凭据余额
/// - `DELETE /credentials/:id` - 删除凭据
///
/// # 认证
/// 需要 Admin API Key 认证，支持：
/// - `x-api-key` header
/// - `Authorization: Bearer <token>` header
pub fn create_admin_router(state: AdminState) -> Router {
    Router::new()
        .route("/credentials", get(get_all_credentials))
        .route("/credentials/upload", post(upload_credential))
        .route("/credentials/{id}/disabled", post(set_credential_disabled))
        .route("/credentials/{id}/priority", post(set_credential_priority))
        .route("/credentials/{id}/reset", post(reset_failure_count))
        .route("/credentials/{id}/balance", get(get_credential_balance))
        .route("/credentials/{id}", delete(delete_credential))
        .route("/stats/summary", get(get_stats_summary))
        .route("/stats/timeseries", get(get_stats_timeseries))
        .route("/stats/requests", get(get_stats_requests))
        .route("/health", get(get_admin_health))
        .route("/logs/stream", get(stream_logs))
        .route("/config", get(get_config).post(update_config))
        .route("/api-keys", get(get_api_keys).post(create_api_key))
        .route("/api-keys/{id}", put(update_api_key).delete(delete_api_key))
        .route("/password", post(change_password))
        // 先应用限流中间件，再应用认证中间件
        .layer(middleware::from_fn_with_state(
            state.clone(),
            admin_auth_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        // 添加 CORS 支持（允许前端开发服务器跨域访问）
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        .with_state(state)
}
