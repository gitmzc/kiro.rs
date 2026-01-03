//! Admin API 类型定义

use serde::{Deserialize, Serialize};

use crate::kiro::model::credentials::KiroCredentials;

// ============ 凭据上传 ============

/// 上传的凭据文件格式（兼容你之前项目的格式）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UploadedCredential {
    /// 访问令牌（可选，系统会自动刷新）
    pub access_token: Option<String>,
    /// 刷新令牌（必需）
    pub refresh_token: Option<String>,
    /// 认证方式：builder-id / social / idc
    pub auth_method: Option<String>,
    /// OIDC Client ID（IDC/builder-id 必需）
    pub client_id: Option<String>,
    /// OIDC Client Secret（IDC/builder-id 必需）
    pub client_secret: Option<String>,
    /// 过期时间
    pub expires_at: Option<String>,
    /// 邮箱（用于显示，可选）
    pub email: Option<String>,
    /// 上次刷新时间（忽略）
    #[serde(default)]
    pub last_refresh: Option<String>,
    /// 提供者（忽略）
    #[serde(default)]
    pub provider: Option<String>,
    /// 类型（忽略）
    #[serde(rename = "type")]
    #[serde(default)]
    pub credential_type: Option<String>,
}

impl UploadedCredential {
    /// 转换为 KiroCredentials
    pub fn into_kiro_credentials(self, priority: u32) -> Result<KiroCredentials, String> {
        // 验证必需字段
        let refresh_token = self.refresh_token
            .filter(|s| !s.is_empty())
            .ok_or("缺少 refresh_token 字段")?;

        // 验证 refresh_token 长度
        if refresh_token.len() < 100 {
            return Err(format!(
                "refresh_token 长度不足（{}字符），可能已被截断",
                refresh_token.len()
            ));
        }

        // 获取认证方式，默认为 builder-id
        let auth_method = self.auth_method
            .map(|m| m.to_lowercase())
            .unwrap_or_else(|| "builder-id".to_string());

        // IDC/builder-id 需要 client_id 和 client_secret
        if auth_method == "idc" || auth_method == "builder-id" {
            if self.client_id.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                return Err("IDC/builder-id 认证需要 client_id".to_string());
            }
            if self.client_secret.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                return Err("IDC/builder-id 认证需要 client_secret".to_string());
            }
        }

        Ok(KiroCredentials {
            access_token: self.access_token.filter(|s| !s.is_empty()),
            refresh_token: Some(refresh_token),
            profile_arn: None,
            expires_at: self.expires_at,
            auth_method: Some(auth_method),
            client_id: self.client_id,
            client_secret: self.client_secret,
            priority,
        })
    }
}

/// 上传凭据响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadCredentialResponse {
    pub success: bool,
    pub message: String,
    /// 新凭据的索引
    pub index: usize,
    /// 凭据总数
    pub total: usize,
    /// 识别的邮箱（如果有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

// ============ 凭据状态 ============

/// 所有凭据状态响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialsStatusResponse {
    /// 凭据总数
    pub total: usize,
    /// 可用凭据数量（未禁用）
    pub available: usize,
    /// 当前活跃凭据索引
    pub current_index: usize,
    /// 各凭据状态列表
    pub credentials: Vec<CredentialStatusItem>,
}

/// 单个凭据的状态信息
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialStatusItem {
    /// 凭据索引（唯一标识符）
    pub index: usize,
    /// 优先级（数字越小优先级越高）
    pub priority: u32,
    /// 是否被禁用
    pub disabled: bool,
    /// 连续失败次数
    pub failure_count: u32,
    /// 是否为当前活跃凭据
    pub is_current: bool,
    /// Token 过期时间（RFC3339 格式）
    pub expires_at: Option<String>,
    /// 认证方式
    pub auth_method: Option<String>,
    /// 是否有 Profile ARN
    pub has_profile_arn: bool,
}

// ============ 操作请求 ============

/// 启用/禁用凭据请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetDisabledRequest {
    /// 是否禁用
    pub disabled: bool,
}

/// 修改优先级请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPriorityRequest {
    /// 新优先级值
    pub priority: u32,
}

// ============ 余额查询 ============

/// 余额查询响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    /// 凭据索引
    pub index: usize,
    /// 订阅类型
    pub subscription_title: Option<String>,
    /// 当前使用量
    pub current_usage: f64,
    /// 使用限额
    pub usage_limit: f64,
    /// 剩余额度
    pub remaining: f64,
    /// 使用百分比
    pub usage_percentage: f64,
    /// 下次重置时间（Unix 时间戳）
    pub next_reset_at: Option<f64>,
}

// ============ 通用响应 ============

/// 操作成功响应
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

impl SuccessResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }
}

/// 错误响应
#[derive(Debug, Serialize)]
pub struct AdminErrorResponse {
    pub error: AdminError,
}

#[derive(Debug, Serialize)]
pub struct AdminError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

impl AdminErrorResponse {
    pub fn new(error_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: AdminError {
                error_type: error_type.into(),
                message: message.into(),
            },
        }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new("invalid_request", message)
    }

    pub fn authentication_error() -> Self {
        Self::new("authentication_error", "Invalid or missing admin API key")
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("not_found", message)
    }

    pub fn api_error(message: impl Into<String>) -> Self {
        Self::new("api_error", message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new("internal_error", message)
    }
}
