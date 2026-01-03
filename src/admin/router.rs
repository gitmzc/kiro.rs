//! Admin API 路由配置

use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};

use super::{
    handlers::{
        delete_credential, get_all_credentials, get_credential_balance, reset_failure_count,
        set_credential_disabled, set_credential_priority, upload_credential,
    },
    middleware::{admin_auth_middleware, AdminState},
};

/// 创建 Admin API 路由
///
/// # 端点
/// - `GET /credentials` - 获取所有凭据状态
/// - `POST /credentials/upload` - 上传凭据文件（multipart/form-data）
/// - `POST /credentials/:index/disabled` - 设置凭据禁用状态
/// - `POST /credentials/:index/priority` - 设置凭据优先级
/// - `POST /credentials/:index/reset` - 重置失败计数
/// - `GET /credentials/:index/balance` - 获取凭据余额
/// - `DELETE /credentials/:index` - 删除凭据
///
/// # 认证
/// 需要 Admin API Key 认证，支持：
/// - `x-api-key` header
/// - `Authorization: Bearer <token>` header
pub fn create_admin_router(state: AdminState) -> Router {
    Router::new()
        .route("/credentials", get(get_all_credentials))
        .route("/credentials/upload", post(upload_credential))
        .route("/credentials/{index}/disabled", post(set_credential_disabled))
        .route("/credentials/{index}/priority", post(set_credential_priority))
        .route("/credentials/{index}/reset", post(reset_failure_count))
        .route("/credentials/{index}/balance", get(get_credential_balance))
        .route("/credentials/{index}", delete(delete_credential))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            admin_auth_middleware,
        ))
        .with_state(state)
}
