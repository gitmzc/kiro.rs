//! Admin API 业务逻辑服务

use std::sync::Arc;
use std::time::Instant;

use crate::kiro::token_manager::MultiTokenManager;

use super::config::ConfigManager;
use super::error::AdminServiceError;
use super::logs::LogBroadcaster;
use super::stats::StatsService;
use super::types::{BalanceResponse, CredentialStatusItem, CredentialsStatusResponse, UploadCredentialResponse, UploadedCredential};

/// Admin 服务
///
/// 封装所有 Admin API 的业务逻辑
pub struct AdminService {
    token_manager: Arc<MultiTokenManager>,
    stats: Arc<StatsService>,
    config_manager: ConfigManager,
    log_broadcaster: LogBroadcaster,
    started_at: Instant,
}

impl AdminService {
    pub fn new(
        token_manager: Arc<MultiTokenManager>,
        stats: Arc<StatsService>,
        config_manager: ConfigManager,
        log_broadcaster: LogBroadcaster,
    ) -> Self {
        Self {
            token_manager,
            stats,
            config_manager,
            log_broadcaster,
            started_at: Instant::now(),
        }
    }

    pub fn stats(&self) -> Arc<StatsService> {
        self.stats.clone()
    }

    pub fn config_manager(&self) -> &ConfigManager {
        &self.config_manager
    }

    pub fn log_broadcaster(&self) -> LogBroadcaster {
        self.log_broadcaster.clone()
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    /// 获取所有凭据状态
    pub fn get_all_credentials(&self) -> CredentialsStatusResponse {
        let snapshot = self.token_manager.snapshot();

        let credentials: Vec<CredentialStatusItem> = snapshot
            .entries
            .into_iter()
            .map(|entry| CredentialStatusItem {
                id: entry.id,
                priority: entry.priority,
                disabled: entry.disabled,
                failure_count: entry.failure_count,
                is_current: entry.id == snapshot.current_id,
                expires_at: entry.expires_at,
                auth_method: entry.auth_method,
                has_profile_arn: entry.has_profile_arn,
            })
            .collect();

        CredentialsStatusResponse {
            total: snapshot.total,
            available: snapshot.available,
            current_id: snapshot.current_id,
            credentials,
        }
    }

    /// 上传凭据（从文件上传的 JSON 格式）
    pub fn upload_credential(&self, uploaded: UploadedCredential) -> Result<UploadCredentialResponse, AdminServiceError> {
        let email = uploaded.email.clone();

        // 获取当前凭据数量作为新凭据的优先级
        let priority = self.token_manager.total_count() as u32;

        // 转换为 KiroCredentials
        let credentials = uploaded.into_kiro_credentials(priority)
            .map_err(|e| AdminServiceError::InvalidRequest(e))?;

        // 添加到 token manager
        let (index, total) = self.token_manager.add_credential(credentials)
            .map_err(|e| AdminServiceError::InternalError(e.to_string()))?;

        Ok(UploadCredentialResponse {
            success: true,
            message: format!("凭据已添加，索引: {}", index),
            index,
            total,
            email,
        })
    }

    /// 删除凭据
    pub fn delete_credential(&self, index: usize) -> Result<usize, AdminServiceError> {
        self.token_manager.remove_credential(index)
            .map_err(|e| AdminServiceError::InternalError(e.to_string()))
    }

    /// 设置凭据禁用状态
    pub fn set_disabled(&self, id: u64, disabled: bool) -> Result<(), AdminServiceError> {
        // 先获取当前凭据 ID，用于判断是否需要切换
        let snapshot = self.token_manager.snapshot();
        let current_id = snapshot.current_id;

        self.token_manager
            .set_disabled(id, disabled)
            .map_err(|e| self.classify_error(e, id))?;

        // 只有禁用的是当前凭据时才尝试切换到下一个
        if disabled && id == current_id {
            let _ = self.token_manager.switch_to_next();
        }
        Ok(())
    }

    /// 设置凭据优先级
    pub fn set_priority(&self, id: u64, priority: u32) -> Result<(), AdminServiceError> {
        self.token_manager
            .set_priority(id, priority)
            .map_err(|e| self.classify_error(e, id))
    }

    /// 重置失败计数并重新启用
    pub fn reset_and_enable(&self, id: u64) -> Result<(), AdminServiceError> {
        self.token_manager
            .reset_and_enable(id)
            .map_err(|e| self.classify_error(e, id))
    }

    /// 获取凭据余额
    pub async fn get_balance(&self, id: u64) -> Result<BalanceResponse, AdminServiceError> {
        let usage = self
            .token_manager
            .get_usage_limits_for(id)
            .await
            .map_err(|e| self.classify_balance_error(e, id))?;

        let current_usage = usage.current_usage();
        let usage_limit = usage.usage_limit();
        let remaining = (usage_limit - current_usage).max(0.0);
        let usage_percentage = if usage_limit > 0.0 {
            (current_usage / usage_limit * 100.0).min(100.0)
        } else {
            0.0
        };

        // 检查余额是否用完，如果用完则自动禁用
        if remaining <= 0.0 {
            if let Err(e) = self.token_manager.check_and_disable_if_exhausted(id).await {
                tracing::warn!("检查并禁用凭据 #{} 失败: {}", id, e);
            }
        }

        Ok(BalanceResponse {
            id,
            subscription_title: usage.subscription_title().map(|s| s.to_string()),
            current_usage,
            usage_limit,
            remaining,
            usage_percentage,
            next_reset_at: usage.next_date_reset,
        })
    }

    /// 分类简单操作错误（set_disabled, set_priority, reset_and_enable）
    fn classify_error(
        &self,
        e: anyhow::Error,
        id: u64,
    ) -> AdminServiceError {
        let msg = e.to_string();
        if msg.contains("不存在") {
            AdminServiceError::NotFound { id }
        } else {
            AdminServiceError::InternalError(msg)
        }
    }

    /// 分类余额查询错误（可能涉及上游 API 调用）
    fn classify_balance_error(
        &self,
        e: anyhow::Error,
        id: u64,
    ) -> AdminServiceError {
        let msg = e.to_string();

        // 1. 凭据不存在
        if msg.contains("不存在") {
            return AdminServiceError::NotFound { id };
        }

        // 2. 上游服务错误特征：HTTP 响应错误或网络错误
        let is_upstream_error =
            // HTTP 响应错误（来自 refresh_*_token 的错误消息）
            msg.contains("凭证已过期或无效") ||
            msg.contains("权限不足") ||
            msg.contains("已被限流") ||
            msg.contains("服务器错误") ||
            msg.contains("Token 刷新失败") ||
            msg.contains("暂时不可用") ||
            // 网络错误（reqwest 错误）
            msg.contains("error trying to connect") ||
            msg.contains("connection") ||
            msg.contains("timeout") ||
            msg.contains("timed out");

        if is_upstream_error {
            AdminServiceError::UpstreamError(msg)
        } else {
            // 3. 默认归类为内部错误（本地验证失败、配置错误等）
            // 包括：缺少 refreshToken、refreshToken 已被截断、无法生成 machineId 等
            AdminServiceError::InternalError(msg)
        }
    }

    /// 批量设置禁用状态
    pub fn batch_set_disabled(&self, ids: &[u64], disabled: bool) -> (usize, usize) {
        let mut succeeded = 0;
        let mut failed = 0;
        let snapshot = self.token_manager.snapshot();
        let current_id = snapshot.current_id;

        for &id in ids {
            match self.token_manager.set_disabled(id, disabled) {
                Ok(_) => {
                    succeeded += 1;
                    if disabled && id == current_id {
                        let _ = self.token_manager.switch_to_next();
                    }
                }
                Err(_) => failed += 1,
            }
        }
        (succeeded, failed)
    }

    /// 批量重置失败计数
    pub fn batch_reset(&self, ids: &[u64]) -> (usize, usize) {
        let mut succeeded = 0;
        let mut failed = 0;
        for &id in ids {
            match self.token_manager.reset_and_enable(id) {
                Ok(_) => succeeded += 1,
                Err(_) => failed += 1,
            }
        }
        (succeeded, failed)
    }

    /// 批量删除凭据
    pub fn batch_delete(&self, ids: &[u64]) -> (usize, usize) {
        let mut succeeded = 0;
        let mut failed = 0;
        for &id in ids {
            match self.token_manager.remove_credential_by_id(id) {
                Ok(_) => succeeded += 1,
                Err(_) => failed += 1,
            }
        }
        (succeeded, failed)
    }
}
