//! 统计中间件

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};
use sha2::{Sha256, Digest};
use uuid::Uuid;

use super::RequestStat;
use super::StatsService;
use crate::common::auth::extract_api_key;

#[derive(Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub model: Option<String>,
    pub stream: bool,
}

pub struct StatsRecorder {
    request_id: String,
    started_at: Instant,
    method: String,
    path: String,
    status: Arc<parking_lot::Mutex<Option<u16>>>,
    tokens: Arc<parking_lot::Mutex<Option<TokenUsage>>>,
    stats: Arc<StatsService>,
    completed: Arc<AtomicBool>,
    api_key_hash: Option<String>,
}

impl Clone for StatsRecorder {
    fn clone(&self) -> Self {
        Self {
            request_id: self.request_id.clone(),
            started_at: self.started_at,
            method: self.method.clone(),
            path: self.path.clone(),
            status: Arc::clone(&self.status),
            tokens: Arc::clone(&self.tokens),
            stats: Arc::clone(&self.stats),
            completed: Arc::clone(&self.completed),
            api_key_hash: self.api_key_hash.clone(),
        }
    }
}

impl StatsRecorder {
    pub fn new(method: String, path: String, stats: Arc<StatsService>, api_key_hash: Option<String>) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            started_at: Instant::now(),
            method,
            path,
            status: Arc::new(parking_lot::Mutex::new(None)),
            tokens: Arc::new(parking_lot::Mutex::new(None)),
            stats,
            completed: Arc::new(AtomicBool::new(false)),
            api_key_hash,
        }
    }

    pub fn set_status(&self, status: u16) {
        *self.status.lock() = Some(status);
    }

    pub fn record_tokens(&self, input: i32, output: i32, model: Option<String>, stream: bool) {
        *self.tokens.lock() = Some(TokenUsage {
            input_tokens: input,
            output_tokens: output,
            model,
            stream,
        });
    }

    pub async fn complete(&self) {
        if self.completed.swap(true, Ordering::SeqCst) {
            return;
        }
        let status = self.status.lock().as_ref().copied().unwrap_or(200) as i32;
        let duration_ms = self.started_at.elapsed().as_millis() as i64;
        let tokens = self.tokens.lock().clone().unwrap_or_default();
        let total_tokens = tokens.input_tokens + tokens.output_tokens;

        // 记录请求日志
        let api_key_display = self.api_key_hash.as_ref()
            .map(|h| format!("{}...", &h[..8]))
            .unwrap_or_else(|| "none".to_string());

        tracing::info!(
            "{} {} - {} - {}ms - tokens: {}/{}/{} - model: {} - key: {}",
            self.method,
            self.path,
            status,
            duration_ms,
            tokens.input_tokens,
            tokens.output_tokens,
            total_tokens,
            tokens.model.as_deref().unwrap_or("none"),
            api_key_display
        );

        let stat = RequestStat {
            request_id: self.request_id.clone(),
            ts: chrono::Utc::now().timestamp(),
            method: self.method.clone(),
            path: self.path.clone(),
            status,
            duration_ms,
            input_tokens: tokens.input_tokens,
            output_tokens: tokens.output_tokens,
            total_tokens,
            model: tokens.model,
            stream: tokens.stream,
            api_key_hash: self.api_key_hash.clone(),
        };
        if let Err(e) = self.stats.record_request(stat).await {
            tracing::warn!("记录统计数据失败: {}", e);
        }
    }
}

pub async fn stats_middleware(
    State(stats): State<Arc<StatsService>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // 提取 API key 并计算哈希值
    let api_key_hash = extract_api_key(&request).map(|key| {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    });

    let recorder = StatsRecorder::new(method, path, stats, api_key_hash);
    request.extensions_mut().insert(recorder.clone());

    let response = next.run(request).await;
    let status = response.status().as_u16();
    recorder.set_status(status);
    if recorder.tokens.lock().is_none() {
        recorder.record_tokens(0, 0, None, false);
    }
    if !recorder.tokens.lock().as_ref().map(|t| t.stream).unwrap_or(false) {
        recorder.complete().await;
    }
    response
}
