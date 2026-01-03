//! SQLite 数据库操作

use anyhow::Result;
use tokio_rusqlite::Connection;
use super::types::{RequestStats, StatsSummary, TimeSeriesPoint, TimeRange, RequestMetrics, TokenMetrics, LatencyMetrics};

/// 统计数据库
pub struct StatsDb {
    conn: Connection,
}

impl StatsDb {
    /// 创建新的统计数据库实例
    pub async fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path).await?;

        // 创建表结构
        conn.call(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS request_stats (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp INTEGER NOT NULL,
                    method TEXT NOT NULL,
                    path TEXT NOT NULL,
                    status INTEGER NOT NULL,
                    duration_ms INTEGER NOT NULL,
                    input_tokens INTEGER,
                    output_tokens INTEGER,
                    total_tokens INTEGER,
                    model TEXT,
                    stream INTEGER NOT NULL
                )",
                [],
            )?;

            // 创建索引以加速查询
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_timestamp ON request_stats(timestamp)",
                [],
            )?;

            Ok(())
        }).await?;

        Ok(Self { conn })
    }

    /// 插入请求统计记录
    pub async fn insert_request(&self, stats: RequestStats) -> Result<()> {
        self.conn.call(move |conn| {
            conn.execute(
                "INSERT INTO request_stats (timestamp, method, path, status, duration_ms, input_tokens, output_tokens, total_tokens, model, stream)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    stats.timestamp,
                    stats.method,
                    stats.path,
                    stats.status,
                    stats.duration_ms,
                    stats.input_tokens,
                    stats.output_tokens,
                    stats.total_tokens,
                    stats.model,
                    if stats.stream { 1 } else { 0 }
                ],
            )?;
            Ok(())
        }).await?;

        Ok(())
    }

    /// 获取统计汇总（过去 24 小时）
    pub async fn get_summary(&self) -> Result<StatsSummary> {
        let now = chrono::Utc::now().timestamp();
        let start = now - 86400; // 24 小时前

        self.conn.call(move |conn| {
            // 查询请求统计
            let mut stmt = conn.prepare(
                "SELECT COUNT(*),
                        SUM(CASE WHEN status < 400 THEN 1 ELSE 0 END),
                        SUM(CASE WHEN status >= 400 THEN 1 ELSE 0 END)
                 FROM request_stats WHERE timestamp >= ?1"
            )?;

            let (total, success, failed): (i64, i64, i64) = stmt.query_row([start], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?;

            let error_rate = if total > 0 {
                failed as f64 / total as f64
            } else {
                0.0
            };

            // 查询 Token 统计
            let mut stmt = conn.prepare(
                "SELECT COALESCE(SUM(input_tokens), 0),
                        COALESCE(SUM(output_tokens), 0),
                        COALESCE(SUM(total_tokens), 0)
                 FROM request_stats WHERE timestamp >= ?1"
            )?;

            let (input, output, total_tokens): (i64, i64, i64) = stmt.query_row([start], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?;

            // 查询延迟统计
            let mut stmt = conn.prepare(
                "SELECT AVG(duration_ms) FROM request_stats WHERE timestamp >= ?1"
            )?;
            let avg: f64 = stmt.query_row([start], |row| row.get(0)).unwrap_or(0.0);

            // 计算 P95
            let mut stmt = conn.prepare(
                "SELECT duration_ms FROM request_stats WHERE timestamp >= ?1 ORDER BY duration_ms"
            )?;
            let durations: Vec<u64> = stmt.query_map([start], |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?;

            let p95 = if !durations.is_empty() {
                let idx = (durations.len() as f64 * 0.95) as usize;
                durations.get(idx).copied().unwrap_or(0) as f64
            } else {
                0.0
            };

            Ok(StatsSummary {
                range: TimeRange { start, end: now },
                requests: RequestMetrics {
                    total,
                    success,
                    failed,
                    error_rate,
                },
                tokens: TokenMetrics {
                    input,
                    output,
                    total: total_tokens,
                },
                latency_ms: LatencyMetrics { avg, p95 },
            })
        }).await
    }

    /// 获取时间序列数据（按小时聚合）
    pub async fn get_timeseries(&self, hours: i64) -> Result<Vec<TimeSeriesPoint>> {
        let now = chrono::Utc::now().timestamp();
        let start = now - (hours * 3600);

        self.conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT (timestamp / 3600) * 3600 as hour,
                        COUNT(*) as requests,
                        COALESCE(SUM(input_tokens), 0) as input_tokens,
                        COALESCE(SUM(output_tokens), 0) as output_tokens,
                        COALESCE(SUM(total_tokens), 0) as total_tokens,
                        AVG(duration_ms) as avg_latency
                 FROM request_stats
                 WHERE timestamp >= ?1
                 GROUP BY hour
                 ORDER BY hour"
            )?;

            let points = stmt.query_map([start], |row| {
                Ok(TimeSeriesPoint {
                    timestamp: row.get(0)?,
                    requests: row.get(1)?,
                    input_tokens: row.get(2)?,
                    output_tokens: row.get(3)?,
                    total_tokens: row.get(4)?,
                    avg_latency_ms: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

            Ok(points)
        }).await
    }

    /// 获取最近的请求列表
    pub async fn get_recent_requests(&self, limit: i64) -> Result<Vec<RequestStats>> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT id, timestamp, method, path, status, duration_ms,
                        input_tokens, output_tokens, total_tokens, model, stream
                 FROM request_stats
                 ORDER BY timestamp DESC
                 LIMIT ?1"
            )?;

            let requests = stmt.query_map([limit], |row| {
                Ok(RequestStats {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    method: row.get(2)?,
                    path: row.get(3)?,
                    status: row.get(4)?,
                    duration_ms: row.get(5)?,
                    input_tokens: row.get(6)?,
                    output_tokens: row.get(7)?,
                    total_tokens: row.get(8)?,
                    model: row.get(9)?,
                    stream: row.get::<_, i32>(10)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

            Ok(requests)
        }).await
    }

    /// 清理过期数据（保留指定天数）
    pub async fn cleanup_old_data(&self, days: i64) -> Result<usize> {
        let cutoff = chrono::Utc::now().timestamp() - (days * 86400);

        self.conn.call(move |conn| {
            let deleted = conn.execute(
                "DELETE FROM request_stats WHERE timestamp < ?1",
                [cutoff],
            )?;
            Ok(deleted)
        }).await
    }
}
