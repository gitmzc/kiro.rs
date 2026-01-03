//! Admin API 中间件

use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use parking_lot::RwLock;

use crate::common::auth;
use super::service::AdminService;
use super::types::AdminErrorResponse;

/// 请求限流器（基于 IP 的滑动窗口）
#[derive(Clone)]
pub struct RateLimiter {
    /// 存储每个 IP 的请求时间戳
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    /// 时间窗口（秒）
    window_secs: u64,
    /// 窗口内最大请求数
    max_requests: usize,
}

impl RateLimiter {
    pub fn new(window_secs: u64, max_requests: usize) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            window_secs,
            max_requests,
        }
    }

    /// 检查 IP 是否超过限流
    pub fn check_rate_limit(&self, ip: &str) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);

        let mut requests = self.requests.write();
        let timestamps = requests.entry(ip.to_string()).or_insert_with(Vec::new);

        // 移除过期的时间戳
        timestamps.retain(|&t| now.duration_since(t) < window);

        // 检查是否超过限制
        if timestamps.len() >= self.max_requests {
            return false;
        }

        // 记录新请求
        timestamps.push(now);
        true
    }

    /// 定期清理过期数据（可选，用于防止内存泄漏）
    pub fn cleanup(&self) {
        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);

        let mut requests = self.requests.write();
        requests.retain(|_, timestamps| {
            timestamps.retain(|&t| now.duration_since(t) < window);
            !timestamps.is_empty()
        });
    }
}

/// Admin API 共享状态
#[derive(Clone)]
pub struct AdminState {
    /// Admin API 密钥
    pub admin_api_key: String,
    /// Admin 服务
    pub service: Arc<AdminService>,
    /// 请求限流器
    pub rate_limiter: RateLimiter,
}

impl AdminState {
    pub fn new(admin_api_key: impl Into<String>, service: AdminService) -> Self {
        Self {
            admin_api_key: admin_api_key.into(),
            service: Arc::new(service),
            // 默认：每分钟最多 60 个请求
            rate_limiter: RateLimiter::new(60, 60),
        }
    }
}

/// Admin API 认证中间件
pub async fn admin_auth_middleware(
    State(state): State<AdminState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let api_key = auth::extract_api_key(&request);

    match api_key {
        Some(key) if auth::constant_time_eq(&key, &state.admin_api_key) => next.run(request).await,
        _ => {
            let error = AdminErrorResponse::authentication_error();
            (StatusCode::UNAUTHORIZED, Json(error)).into_response()
        }
    }
}

/// Admin API 请求限流中间件
pub async fn rate_limit_middleware(
    State(state): State<AdminState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // 尝试从请求中提取 IP 地址
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    // 检查限流
    if !state.rate_limiter.check_rate_limit(&ip) {
        let error = AdminErrorResponse::new("rate_limit_exceeded", "请求过于频繁，请稍后再试");
        return (StatusCode::TOO_MANY_REQUESTS, Json(error)).into_response();
    }

    next.run(request).await
}
