//! Admin API HTTP 处理器

use axum::{
    extract::{Multipart, Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use bytes::Bytes;
use std::time::Duration;
use tokio::time::interval;

use super::{
    middleware::AdminState,
    stats::types::{RequestsQuery, StatsSummaryQuery, StatsTimeseriesQuery},
    config::{ConfigPatch, ConfigView},
    types::{
        SetDisabledRequest, SetPriorityRequest, SuccessResponse, UploadedCredential,
        ApiKeysResponse, ApiKeyItem, CreateApiKeyRequest, CreateApiKeyResponse,
        UpdateApiKeyRequest, ChangePasswordRequest,
        BatchIdsRequest, BatchDisabledRequest, BatchResponse,
    },
};
use super::types::HealthResponse;

/// GET /api/admin/credentials
/// 获取所有凭据状态
pub async fn get_all_credentials(State(state): State<AdminState>) -> impl IntoResponse {
    let response = state.service.get_all_credentials();
    Json(response)
}

/// POST /api/admin/credentials/:id/disabled
/// 设置凭据禁用状态
pub async fn set_credential_disabled(
    State(state): State<AdminState>,
    Path(id): Path<u64>,
    Json(payload): Json<SetDisabledRequest>,
) -> impl IntoResponse {
    match state.service.set_disabled(id, payload.disabled) {
        Ok(_) => {
            let action = if payload.disabled { "禁用" } else { "启用" };
            Json(SuccessResponse::new(format!("凭据 #{} 已{}", id, action))).into_response()
        }
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}

/// POST /api/admin/credentials/:id/priority
/// 设置凭据优先级
pub async fn set_credential_priority(
    State(state): State<AdminState>,
    Path(id): Path<u64>,
    Json(payload): Json<SetPriorityRequest>,
) -> impl IntoResponse {
    match state.service.set_priority(id, payload.priority) {
        Ok(_) => Json(SuccessResponse::new(format!(
            "凭据 #{} 优先级已设置为 {}",
            id, payload.priority
        )))
        .into_response(),
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}

/// POST /api/admin/credentials/:id/reset
/// 重置失败计数并重新启用
pub async fn reset_failure_count(
    State(state): State<AdminState>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    match state.service.reset_and_enable(id) {
        Ok(_) => Json(SuccessResponse::new(format!(
            "凭据 #{} 失败计数已重置并重新启用",
            id
        )))
        .into_response(),
        Err(e) => (e.status_code(), Json(e.into_response())).into_response(),
    }
}

/// GET /api/admin/credentials/:id/balance
/// 获取指定凭据的余额
pub async fn get_credential_balance(
    State(state): State<AdminState>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    match state.service.get_balance(id).await {
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

/// GET /api/admin/stats/summary
pub async fn get_stats_summary(
    State(state): State<AdminState>,
    Query(query): Query<StatsSummaryQuery>,
) -> impl IntoResponse {
    let hours = query.hours.unwrap_or(24).clamp(1, 168);
    match state.service.stats().summary(hours, query.api_key_hash).await {
        Ok(resp) => Json(resp).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(super::types::AdminErrorResponse::internal_error(format!(
                "统计汇总查询失败: {}",
                e
            ))),
        )
            .into_response(),
    }
}

/// GET /api/admin/stats/timeseries
pub async fn get_stats_timeseries(
    State(state): State<AdminState>,
    Query(query): Query<StatsTimeseriesQuery>,
) -> impl IntoResponse {
    let hours = query.hours.unwrap_or(24).clamp(1, 168);
    let interval_minutes = query.interval_minutes.unwrap_or(60).clamp(5, 1440);
    match state.service.stats().timeseries(hours, interval_minutes, query.api_key_hash).await {
        Ok(resp) => Json(resp).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(super::types::AdminErrorResponse::internal_error(format!(
                "时间序列查询失败: {}",
                e
            ))),
        )
            .into_response(),
    }
}

/// GET /api/admin/stats/requests
pub async fn get_stats_requests(
    State(state): State<AdminState>,
    Query(query): Query<RequestsQuery>,
) -> impl IntoResponse {
    match state.service.stats().recent_requests(query).await {
        Ok(items) => Json(serde_json::json!({ "items": items })).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(super::types::AdminErrorResponse::internal_error(format!(
                "请求列表查询失败: {}",
                e
            ))),
        )
            .into_response(),
    }
}

/// GET /api/admin/health
pub async fn get_admin_health(State(state): State<AdminState>) -> impl IntoResponse {
    let db_status = match state.service.stats().summary(1, None).await {
        Ok(_) => "ok",
        Err(_) => "error",
    };
    let response = HealthResponse {
        status: "ok".to_string(),
        db: db_status.to_string(),
        now: chrono::Utc::now().to_rfc3339(),
        uptime_seconds: state.service.uptime_seconds(),
    };
    Json(response)
}

/// GET /api/admin/logs/stream
pub async fn stream_logs(
    State(state): State<AdminState>,
    Query(query): Query<LogsQuery>,
) -> Response {
    let mut receiver = state.service.log_broadcaster().subscribe();
    let level_filter = query.level.map(|v| v.to_lowercase());

    let stream = futures::stream::unfold(
        (receiver, interval(Duration::from_millis(100))),
        move |(mut receiver, mut limiter)| {
            let level_filter = level_filter.clone();
            async move {
                loop {
                    match receiver.recv().await {
                        Ok(line) => {
                            if !filter_log_line(&line, level_filter.as_deref()) {
                                continue;
                            }
                            limiter.tick().await;
                            let payload = format!("data: {}\n\n", line);
                            let bytes = Bytes::from(payload);
                            return Some((Ok::<_, std::io::Error>(bytes), (receiver, limiter)));
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                    }
                }
            }
        },
    );

    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "text/event-stream")
        .header(axum::http::header::CACHE_CONTROL, "no-cache")
        .header(axum::http::header::CONNECTION, "keep-alive")
        .body(axum::body::Body::from_stream(stream))
        .unwrap()
}

/// GET /api/admin/config
pub async fn get_config(State(state): State<AdminState>) -> impl IntoResponse {
    let config = state.service.config_manager().get();
    Json(ConfigView::from_config(config))
}

/// POST /api/admin/config
pub async fn update_config(
    State(state): State<AdminState>,
    Json(payload): Json<ConfigPatch>,
) -> impl IntoResponse {
    match state.service.config_manager().update_config(payload) {
        Ok(updated) => Json(ConfigView::from_config(updated)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(super::types::AdminErrorResponse::internal_error(format!(
                "配置更新失败: {}",
                e
            ))),
        )
            .into_response(),
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogsQuery {
    pub level: Option<String>,
}

fn filter_log_line(line: &str, level: Option<&str>) -> bool {
    match level {
        Some("error") => line.contains(" ERROR "),
        Some("warn") => line.contains(" WARN "),
        Some("info") => line.contains(" INFO "),
        Some("debug") => line.contains(" DEBUG "),
        Some("trace") => line.contains(" TRACE "),
        Some(_) => true,
        None => true,
    }
}

// ============ API Key 管理 ============

/// GET /api/admin/api-keys
/// 获取所有 API Keys
pub async fn get_api_keys(State(state): State<AdminState>) -> impl IntoResponse {
    let config = state.service.config_manager().get();
    let api_keys: Vec<ApiKeyItem> = config
        .api_keys
        .iter()
        .map(|k| ApiKeyItem {
            id: k.id.clone(),
            name: k.name.clone(),
            key_preview: mask_api_key(&k.key),
            enabled: k.enabled,
            created_at: k.created_at,
        })
        .collect();

    Json(ApiKeysResponse { api_keys })
}

/// POST /api/admin/api-keys
/// 创建新的 API Key
pub async fn create_api_key(
    State(state): State<AdminState>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> impl IntoResponse {
    // 生成新的 API Key
    let key = generate_api_key();
    let name = payload.name.clone();

    match state.service.config_manager().add_api_key(key.clone(), payload.name) {
        Ok((id, created_at)) => {
            Json(CreateApiKeyResponse {
                id,
                key,
                name,
                created_at,
            }).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(super::types::AdminErrorResponse::internal_error(format!(
                "创建 API Key 失败: {}",
                e
            ))),
        ).into_response(),
    }
}

/// PUT /api/admin/api-keys/:id
/// 更新 API Key
pub async fn update_api_key(
    State(state): State<AdminState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateApiKeyRequest>,
) -> impl IntoResponse {
    match state.service.config_manager().update_api_key(&id, payload.name, payload.enabled) {
        Ok(_) => Json(SuccessResponse::new("API Key 已更新")).into_response(),
        Err(e) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(super::types::AdminErrorResponse::not_found(format!(
                "API Key 不存在: {}",
                e
            ))),
        ).into_response(),
    }
}

/// DELETE /api/admin/api-keys/:id
/// 删除 API Key
pub async fn delete_api_key(
    State(state): State<AdminState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.service.config_manager().delete_api_key(&id) {
        Ok(_) => Json(SuccessResponse::new("API Key 已删除")).into_response(),
        Err(e) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(super::types::AdminErrorResponse::not_found(format!(
                "API Key 不存在: {}",
                e
            ))),
        ).into_response(),
    }
}

// ============ 密码管理 ============

/// POST /api/admin/password
/// 修改管理员密码
pub async fn change_password(
    State(state): State<AdminState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    match state.service.config_manager().change_admin_password(&payload.old_password, &payload.new_password) {
        Ok(_) => Json(SuccessResponse::new("密码已修改")).into_response(),
        Err(e) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(super::types::AdminErrorResponse::invalid_request(format!(
                "密码修改失败: {}",
                e
            ))),
        ).into_response(),
    }
}

// ============ 辅助函数 ============

/// 生成随机 API Key (sk-kiro-rs-xxx 格式)
fn generate_api_key() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let random_part: String = (0..32)
        .map(|_| {
            let idx = fastrand::usize(..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("sk-kiro-rs-{}", random_part)
}

/// 遮蔽 API Key，只显示前8位和后4位
fn mask_api_key(key: &str) -> String {
    if key.len() <= 12 {
        return "*".repeat(key.len());
    }
    let prefix = &key[..8];
    let suffix = &key[key.len() - 4..];
    format!("{}...{}", prefix, suffix)
}

// ============ 批量操作 ============

/// POST /api/admin/credentials/batch/disabled
/// 批量启用/禁用凭据
pub async fn batch_set_disabled(
    State(state): State<AdminState>,
    Json(payload): Json<BatchDisabledRequest>,
) -> impl IntoResponse {
    if payload.ids.is_empty() {
        return Json(BatchResponse {
            success: false,
            message: "未选择任何凭据".to_string(),
            succeeded: 0,
            failed: 0,
        }).into_response();
    }

    let (succeeded, failed) = state.service.batch_set_disabled(&payload.ids, payload.disabled);
    let action = if payload.disabled { "禁用" } else { "启用" };
    Json(BatchResponse {
        success: failed == 0,
        message: format!("批量{}: {} 成功, {} 失败", action, succeeded, failed),
        succeeded,
        failed,
    }).into_response()
}

/// POST /api/admin/credentials/batch/reset
/// 批量重置失败计数
pub async fn batch_reset(
    State(state): State<AdminState>,
    Json(payload): Json<BatchIdsRequest>,
) -> impl IntoResponse {
    if payload.ids.is_empty() {
        return Json(BatchResponse {
            success: false,
            message: "未选择任何凭据".to_string(),
            succeeded: 0,
            failed: 0,
        }).into_response();
    }

    let (succeeded, failed) = state.service.batch_reset(&payload.ids);
    Json(BatchResponse {
        success: failed == 0,
        message: format!("批量重置: {} 成功, {} 失败", succeeded, failed),
        succeeded,
        failed,
    }).into_response()
}

/// POST /api/admin/credentials/batch/delete
/// 批量删除凭据
pub async fn batch_delete(
    State(state): State<AdminState>,
    Json(payload): Json<BatchIdsRequest>,
) -> impl IntoResponse {
    if payload.ids.is_empty() {
        return Json(BatchResponse {
            success: false,
            message: "未选择任何凭据".to_string(),
            succeeded: 0,
            failed: 0,
        }).into_response();
    }

    let (succeeded, failed) = state.service.batch_delete(&payload.ids);
    Json(BatchResponse {
        success: failed == 0,
        message: format!("批量删除: {} 成功, {} 失败", succeeded, failed),
        succeeded,
        failed,
    }).into_response()
}
