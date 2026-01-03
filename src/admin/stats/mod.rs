//! 统计数据存储与查询

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

use anyhow::Context;
use chrono::{DateTime, Utc};
use tokio_rusqlite::Connection;

use crate::admin::stats::types::{
    LatencySummary, RequestSummary, RequestsQuery, StatsRequestItem, StatsSummaryResponse,
    StatsTimeseriesPoint, StatsTimeseriesResponse, TimeRange, TokenSummary,
};

pub mod middleware;
pub mod types;

#[derive(Clone)]
pub struct StatsService {
    conn: Arc<Connection>,
}

#[derive(Clone, Debug)]
pub struct RequestStat {
    pub request_id: String,
    pub ts: i64,
    pub method: String,
    pub path: String,
    pub status: i32,
    pub duration_ms: i64,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub total_tokens: i32,
    pub model: Option<String>,
    pub stream: bool,
    pub api_key_hash: Option<String>,
}

/// 计算延迟所属的桶编号（用于 P95 统计）
/// 桶划分：0-10ms, 10-20ms, 20-50ms, 50-100ms, 100-200ms, 200-500ms, 500-1000ms, 1000-2000ms, 2000-5000ms, 5000+ms
fn latency_bucket(duration_ms: i64) -> i32 {
    match duration_ms {
        0..=10 => 0,
        11..=20 => 1,
        21..=50 => 2,
        51..=100 => 3,
        101..=200 => 4,
        201..=500 => 5,
        501..=1000 => 6,
        1001..=2000 => 7,
        2001..=5000 => 8,
        _ => 9,
    }
}

/// 获取桶的上界值（用于 P95 估算）
fn bucket_upper_bound(bucket: i32) -> f64 {
    match bucket {
        0 => 10.0,
        1 => 20.0,
        2 => 50.0,
        3 => 100.0,
        4 => 200.0,
        5 => 500.0,
        6 => 1000.0,
        7 => 2000.0,
        8 => 5000.0,
        _ => 10000.0,
    }
}

impl StatsService {
    pub async fn new(db_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let db_path = db_path.as_ref();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("创建数据目录失败: {:?}", parent))?;
        }

        let conn = Connection::open(db_path).await?;

        // 启用 WAL 模式以提升并发性能
        conn.call(|conn| {
            conn.pragma_update(None, "journal_mode", "WAL")?;
            Ok(())
        })
        .await?;

        let service = Self {
            conn: Arc::new(conn),
        };
        service.init_schema().await?;
        Ok(service)
    }

    async fn init_schema(&self) -> anyhow::Result<()> {
        let conn = self.conn.clone();
        conn.call(|conn| {
            // 先创建表（不包含 latency_bucket）
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS request_stats (
                    request_id TEXT PRIMARY KEY,
                    ts INTEGER NOT NULL,
                    method TEXT NOT NULL,
                    path TEXT NOT NULL,
                    status INTEGER NOT NULL,
                    duration_ms INTEGER NOT NULL,
                    input_tokens INTEGER NOT NULL,
                    output_tokens INTEGER NOT NULL,
                    total_tokens INTEGER NOT NULL,
                    model TEXT,
                    stream INTEGER NOT NULL
                );
                "#,
            )?;

            // 迁移：为已有数据库添加 latency_bucket 列
            let has_latency_bucket: bool = conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('request_stats') WHERE name='latency_bucket'",
                [],
                |row| row.get(0)
            ).unwrap_or(0) > 0;

            if !has_latency_bucket {
                conn.execute("ALTER TABLE request_stats ADD COLUMN latency_bucket INTEGER NOT NULL DEFAULT 0", [])?;
                // 为已有数据计算并更新 latency_bucket
                conn.execute(
                    r#"
                    UPDATE request_stats SET latency_bucket =
                        CASE
                            WHEN duration_ms <= 10 THEN 0
                            WHEN duration_ms <= 20 THEN 1
                            WHEN duration_ms <= 50 THEN 2
                            WHEN duration_ms <= 100 THEN 3
                            WHEN duration_ms <= 200 THEN 4
                            WHEN duration_ms <= 500 THEN 5
                            WHEN duration_ms <= 1000 THEN 6
                            WHEN duration_ms <= 2000 THEN 7
                            WHEN duration_ms <= 5000 THEN 8
                            ELSE 9
                        END
                    "#,
                    []
                )?;
            }

            // 迁移：为已有数据库添加 api_key_hash 列
            let has_api_key_hash: bool = conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('request_stats') WHERE name='api_key_hash'",
                [],
                |row| row.get(0)
            ).unwrap_or(0) > 0;

            if !has_api_key_hash {
                conn.execute("ALTER TABLE request_stats ADD COLUMN api_key_hash TEXT", [])?;
            }

            // 创建索引（在列存在之后）
            conn.execute_batch(
                r#"
                CREATE INDEX IF NOT EXISTS idx_request_stats_ts ON request_stats (ts);
                CREATE INDEX IF NOT EXISTS idx_request_stats_path ON request_stats (path);
                CREATE INDEX IF NOT EXISTS idx_request_stats_latency_bucket ON request_stats (latency_bucket, ts);
                CREATE INDEX IF NOT EXISTS idx_request_stats_api_key_hash ON request_stats (api_key_hash, ts);
                "#,
            )?;

            Ok(())
        })
        .await?;
        Ok(())
    }

    pub async fn record_request(&self, stat: RequestStat) -> anyhow::Result<()> {
        let bucket = latency_bucket(stat.duration_ms);
        let conn = self.conn.clone();
        conn.call(move |conn| {
            conn.execute(
                r#"
                INSERT OR REPLACE INTO request_stats
                (request_id, ts, method, path, status, duration_ms, input_tokens, output_tokens, total_tokens, model, stream, latency_bucket, api_key_hash)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                "#,
                rusqlite::params![
                    stat.request_id,
                    stat.ts,
                    stat.method,
                    stat.path,
                    stat.status,
                    stat.duration_ms,
                    stat.input_tokens,
                    stat.output_tokens,
                    stat.total_tokens,
                    stat.model,
                    if stat.stream { 1 } else { 0 },
                    bucket,
                    stat.api_key_hash
                ],
            )?;
            Ok(())
        })
        .await?;
        Ok(())
    }

    pub async fn cleanup_older_than(&self, keep_days: i64) -> anyhow::Result<u64> {
        let cutoff = Utc::now().timestamp() - keep_days * 86400;
        let conn = self.conn.clone();
        let deleted = conn
            .call(move |conn| {
                let rows = conn.execute(
                    "DELETE FROM request_stats WHERE ts < ?1",
                    rusqlite::params![cutoff],
                )?;
                Ok(rows as u64)
            })
            .await?;
        Ok(deleted)
    }

    pub async fn summary(&self, hours: i64, api_key_hash: Option<String>) -> anyhow::Result<StatsSummaryResponse> {
        let now = Utc::now();
        let start = now - chrono::Duration::hours(hours);
        let start_ts = start.timestamp();

        let conn = self.conn.clone();
        let api_key_hash_clone = api_key_hash.clone();
        let (total, success, failure, input_tokens, output_tokens, total_tokens, avg_latency) =
            conn.call(move |conn| {
                let (sql, params): (String, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(ref hash) = api_key_hash_clone {
                    (
                        r#"
                        SELECT
                            COUNT(*),
                            SUM(CASE WHEN status >= 200 AND status < 400 THEN 1 ELSE 0 END),
                            SUM(CASE WHEN status >= 400 THEN 1 ELSE 0 END),
                            COALESCE(SUM(input_tokens), 0),
                            COALESCE(SUM(output_tokens), 0),
                            COALESCE(SUM(total_tokens), 0),
                            COALESCE(AVG(duration_ms), 0)
                        FROM request_stats
                        WHERE ts >= ?1 AND api_key_hash = ?2
                        "#.to_string(),
                        vec![Box::new(start_ts), Box::new(hash.clone())]
                    )
                } else {
                    (
                        r#"
                        SELECT
                            COUNT(*),
                            SUM(CASE WHEN status >= 200 AND status < 400 THEN 1 ELSE 0 END),
                            SUM(CASE WHEN status >= 400 THEN 1 ELSE 0 END),
                            COALESCE(SUM(input_tokens), 0),
                            COALESCE(SUM(output_tokens), 0),
                            COALESCE(SUM(total_tokens), 0),
                            COALESCE(AVG(duration_ms), 0)
                        FROM request_stats
                        WHERE ts >= ?1
                        "#.to_string(),
                        vec![Box::new(start_ts)]
                    )
                };

                let mut stmt = conn.prepare(&sql)?;
                let row = stmt.query_row(rusqlite::params_from_iter(params.iter()), |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, f64>(6)?,
                    ))
                })?;
                Ok(row)
            })
            .await?;

        // 使用分桶统计计算 P95
        let p95 = self.calculate_p95_from_buckets(start_ts, api_key_hash).await?;

        let error_rate = if total > 0 {
            (failure as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Ok(StatsSummaryResponse {
            range: TimeRange {
                start: start.to_rfc3339(),
                end: now.to_rfc3339(),
            },
            requests: RequestSummary {
                total,
                success,
                failed: failure,
                error_rate,
            },
            tokens: TokenSummary {
                input: input_tokens,
                output: output_tokens,
                total: total_tokens,
            },
            latency_ms: LatencySummary {
                avg: avg_latency,
                p95,
            },
        })
    }

    /// 使用分桶统计高效计算 P95 延迟
    async fn calculate_p95_from_buckets(&self, start_ts: i64, api_key_hash: Option<String>) -> anyhow::Result<f64> {
        let conn = self.conn.clone();
        let bucket_counts = conn
            .call(move |conn| {
                let (sql, params): (String, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(ref hash) = api_key_hash {
                    (
                        r#"
                        SELECT latency_bucket, COUNT(*) as count
                        FROM request_stats
                        WHERE ts >= ?1 AND api_key_hash = ?2
                        GROUP BY latency_bucket
                        ORDER BY latency_bucket
                        "#.to_string(),
                        vec![Box::new(start_ts), Box::new(hash.clone())]
                    )
                } else {
                    (
                        r#"
                        SELECT latency_bucket, COUNT(*) as count
                        FROM request_stats
                        WHERE ts >= ?1
                        GROUP BY latency_bucket
                        ORDER BY latency_bucket
                        "#.to_string(),
                        vec![Box::new(start_ts)]
                    )
                };

                let mut stmt = conn.prepare(&sql)?;
                let mut rows = stmt.query(rusqlite::params_from_iter(params.iter()))?;
                let mut counts = Vec::new();
                while let Some(row) = rows.next()? {
                    counts.push((row.get::<_, i32>(0)?, row.get::<_, i64>(1)?));
                }
                Ok(counts)
            })
            .await?;

        if bucket_counts.is_empty() {
            return Ok(0.0);
        }

        // 计算总请求数和 P95 阈值
        let total: i64 = bucket_counts.iter().map(|(_, count)| count).sum();
        let p95_threshold = (total as f64 * 0.95).ceil() as i64;

        // 累加桶计数，找到包含 P95 的桶
        let mut cumulative = 0i64;
        for (bucket, count) in bucket_counts {
            cumulative += count;
            if cumulative >= p95_threshold {
                return Ok(bucket_upper_bound(bucket));
            }
        }

        // 如果没有找到（理论上不应该发生），返回最大桶的上界
        Ok(bucket_upper_bound(9))
    }

    pub async fn timeseries(
        &self,
        hours: i64,
        interval_minutes: i64,
        api_key_hash: Option<String>,
    ) -> anyhow::Result<StatsTimeseriesResponse> {
        let now = Utc::now();
        let start = now - chrono::Duration::hours(hours);
        let start_ts = start.timestamp();
        let bucket_seconds = interval_minutes * 60;

        let conn = self.conn.clone();
        let points = conn
            .call(move |conn| {
                let (sql, params): (String, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(ref hash) = api_key_hash {
                    (
                        r#"
                        SELECT
                            (ts / ?1) * ?1 AS bucket,
                            COUNT(*),
                            COALESCE(SUM(input_tokens), 0),
                            COALESCE(SUM(output_tokens), 0),
                            COALESCE(SUM(total_tokens), 0),
                            COALESCE(AVG(duration_ms), 0)
                        FROM request_stats
                        WHERE ts >= ?2 AND api_key_hash = ?3
                        GROUP BY bucket
                        ORDER BY bucket
                        "#.to_string(),
                        vec![Box::new(bucket_seconds), Box::new(start_ts), Box::new(hash.clone())]
                    )
                } else {
                    (
                        r#"
                        SELECT
                            (ts / ?1) * ?1 AS bucket,
                            COUNT(*),
                            COALESCE(SUM(input_tokens), 0),
                            COALESCE(SUM(output_tokens), 0),
                            COALESCE(SUM(total_tokens), 0),
                            COALESCE(AVG(duration_ms), 0)
                        FROM request_stats
                        WHERE ts >= ?2
                        GROUP BY bucket
                        ORDER BY bucket
                        "#.to_string(),
                        vec![Box::new(bucket_seconds), Box::new(start_ts)]
                    )
                };

                let mut stmt = conn.prepare(&sql)?;
                let mut rows = stmt.query(rusqlite::params_from_iter(params.iter()))?;
                let mut points = Vec::new();
                while let Some(row) = rows.next()? {
                    let bucket_ts: i64 = row.get(0)?;
                    points.push(StatsTimeseriesPoint {
                        ts: DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(bucket_ts as u64))
                            .to_rfc3339(),
                        requests: row.get::<_, i64>(1)?,
                        input_tokens: row.get::<_, i64>(2)?,
                        output_tokens: row.get::<_, i64>(3)?,
                        total_tokens: row.get::<_, i64>(4)?,
                        avg_latency_ms: row.get::<_, f64>(5)?,
                    });
                }
                Ok(points)
            })
            .await?;

        Ok(StatsTimeseriesResponse {
            interval_minutes,
            points,
        })
    }

    pub async fn recent_requests(&self, query: RequestsQuery) -> anyhow::Result<Vec<StatsRequestItem>> {
        let limit = query.limit.unwrap_or(20).clamp(1, 200);
        let api_key_hash = query.api_key_hash;
        let conn = self.conn.clone();
        let items = conn
            .call(move |conn| {
                let (sql, params): (String, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(ref hash) = api_key_hash {
                    (
                        r#"
                        SELECT ts, method, path, status, duration_ms, input_tokens, output_tokens, total_tokens, model, stream
                        FROM request_stats
                        WHERE api_key_hash = ?1
                        ORDER BY ts DESC
                        LIMIT ?2
                        "#.to_string(),
                        vec![Box::new(hash.clone()), Box::new(limit)]
                    )
                } else {
                    (
                        r#"
                        SELECT ts, method, path, status, duration_ms, input_tokens, output_tokens, total_tokens, model, stream
                        FROM request_stats
                        ORDER BY ts DESC
                        LIMIT ?1
                        "#.to_string(),
                        vec![Box::new(limit)]
                    )
                };

                let mut stmt = conn.prepare(&sql)?;
                let mut rows = stmt.query(rusqlite::params_from_iter(params.iter()))?;
                let mut items = Vec::new();
                while let Some(row) = rows.next()? {
                    let ts: i64 = row.get(0)?;
                    items.push(StatsRequestItem {
                        ts: DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(ts as u64))
                            .to_rfc3339(),
                        method: row.get(1)?,
                        path: row.get(2)?,
                        status: row.get(3)?,
                        duration_ms: row.get(4)?,
                        input_tokens: row.get(5)?,
                        output_tokens: row.get(6)?,
                        total_tokens: row.get(7)?,
                        model: row.get(8)?,
                        stream: row.get::<_, i32>(9)? == 1,
                    });
                }
                Ok(items)
            })
            .await?;
        Ok(items)
    }
}
