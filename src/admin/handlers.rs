//! Admin API HTTP 处理器

use axum::{
    extract::{Multipart, Path, State},
    response::IntoResponse,
    Json,
};

use super::{
    middleware::AdminState,
    types::{SetDisabledRequest, SetPriorityRequest, SuccessResponse, UploadedCredential},
};

/// GET /api/admin/credentials
/// 获取所有凭据状态
pub async fn get_all_credentials(State(state): State<AdminState>) -> impl IntoResponse {
    let response = state.service.get_all_credentials();
    Json(response)
}

/// POST /api/admin/credentials/:index/disabled
/// 设置凭据禁用状态
pub async fn set_credential_disabled(
    State(state): State<AdminState>,
    Path(index): Path<usize>,
    Json(payload): Json<SetDisabledRequest>,
) -> impl IntoResponse {
    match state.service.set_disabled(index, payload.disabled) {
        Ok(_) => {
            let action = if payload.disabled { "禁用" } else { "启用" };
            Json(SuccessResponse::new(format!("凭据 #{} 已{}", index, action))).into_response()
        }
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}

/// POST /api/admin/credentials/:index/priority
/// 设置凭据优先级
pub async fn set_credential_priority(
    State(state): State<AdminState>,
    Path(index): Path<usize>,
    Json(payload): Json<SetPriorityRequest>,
) -> impl IntoResponse {
    match state.service.set_priority(index, payload.priority) {
        Ok(_) => Json(SuccessResponse::new(format!(
            "凭据 #{} 优先级已设置为 {}",
            index, payload.priority
        )))
        .into_response(),
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}

/// POST /api/admin/credentials/:index/reset
/// 重置失败计数并重新启用
pub async fn reset_failure_count(
    State(state): State<AdminState>,
    Path(index): Path<usize>,
) -> impl IntoResponse {
    match state.service.reset_and_enable(index) {
        Ok(_) => Json(SuccessResponse::new(format!(
            "凭据 #{} 失败计数已重置并重新启用",
            index
        )))
        .into_response(),
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}

/// GET /api/admin/credentials/:index/balance
/// 获取指定凭据的余额
pub async fn get_credential_balance(
    State(state): State<AdminState>,
    Path(index): Path<usize>,
) -> impl IntoResponse {
    match state.service.get_balance(index).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}

/// POST /api/admin/credentials/upload
/// 上传凭据文件（multipart/form-data）
///
/// 兼容格式：
/// ```json
/// {
///   "access_token": "...",
///   "refresh_token": "...",
///   "auth_method": "builder-id",
///   "client_id": "...",
///   "client_secret": "...",
///   "expires_at": "2026-01-03T04:46:11.521+08:00",
///   "email": "user@example.com"
/// }
/// ```
pub async fn upload_credential(
    State(state): State<AdminState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // 查找 file 字段
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            // 读取文件内容
            let data = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => {
                    return (
                        axum::http::StatusCode::BAD_REQUEST,
                        Json(super::types::AdminErrorResponse::invalid_request(
                            format!("读取文件失败: {}", e)
                        ))
                    ).into_response();
                }
            };

            // 解析 JSON
            let uploaded: UploadedCredential = match serde_json::from_slice(&data) {
                Ok(cred) => cred,
                Err(e) => {
                    return (
                        axum::http::StatusCode::BAD_REQUEST,
                        Json(super::types::AdminErrorResponse::invalid_request(
                            format!("JSON 解析失败: {}", e)
                        ))
                    ).into_response();
                }
            };

            // 调用服务上传
            return match state.service.upload_credential(uploaded) {
                Ok(response) => Json(response).into_response(),
                Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
            };
        }
    }

    // 没有找到 file 字段
    (
        axum::http::StatusCode::BAD_REQUEST,
        Json(super::types::AdminErrorResponse::invalid_request(
            "缺少 file 字段"
        ))
    ).into_response()
}

/// DELETE /api/admin/credentials/:index
/// 删除凭据
pub async fn delete_credential(
    State(state): State<AdminState>,
    Path(index): Path<usize>,
) -> impl IntoResponse {
    match state.service.delete_credential(index) {
        Ok(total) => Json(SuccessResponse::new(format!(
            "凭据 #{} 已删除，剩余 {} 个凭据",
            index, total
        ))).into_response(),
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}
