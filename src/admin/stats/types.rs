//! 统计 API 类型定义

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsSummaryResponse {
    pub range: TimeRange,
    pub requests: RequestSummary,
    pub tokens: TokenSummary,
    pub latency_ms: LatencySummary,
}

#[derive(Debug, Serialize)]
pub struct TimeRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Serialize)]
pub struct RequestSummary {
    pub total: i64,
    pub success: i64,
    pub failed: i64,
    pub error_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct TokenSummary {
    pub input: i64,
    pub output: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct LatencySummary {
    pub avg: f64,
    pub p95: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsTimeseriesResponse {
    pub interval_minutes: i64,
    pub points: Vec<StatsTimeseriesPoint>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsTimeseriesPoint {
    pub ts: String,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub avg_latency_ms: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsRequestItem {
    pub ts: String,
    pub method: String,
    pub path: String,
    pub status: i32,
    pub duration_ms: i64,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub total_tokens: i32,
    pub model: Option<String>,
    pub stream: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsTimeseriesQuery {
    pub hours: Option<i64>,
    pub interval_minutes: Option<i64>,
    pub api_key_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsSummaryQuery {
    pub hours: Option<i64>,
    pub api_key_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestsQuery {
    pub limit: Option<i64>,
    pub api_key_hash: Option<String>,
}
