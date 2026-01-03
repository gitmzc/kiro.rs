use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use chrono::Utc;

/// API Key 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyConfig {
    /// 唯一标识符
    pub id: String,
    /// API Key 值
    pub key: String,
    /// 显示名称
    pub name: String,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间戳（秒）
    pub created_at: i64,
}

impl ApiKeyConfig {
    pub fn new(key: String, name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key,
            name,
            enabled: true,
            created_at: Utc::now().timestamp(),
        }
    }
}

/// KNA 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_region")]
    pub region: String,

    #[serde(default = "default_kiro_version")]
    pub kiro_version: String,

    #[serde(default)]
    pub machine_id: Option<String>,

    /// 单个 API Key（已废弃，保留用于向后兼容）
    #[serde(default)]
    pub api_key: Option<String>,

    /// 多个 API Keys（新版本）
    #[serde(default)]
    pub api_keys: Vec<ApiKeyConfig>,

    #[serde(default = "default_system_version")]
    pub system_version: String,

    #[serde(default = "default_node_version")]
    pub node_version: String,

    /// 外部 count_tokens API 地址（可选）
    #[serde(default)]
    pub count_tokens_api_url: Option<String>,

    /// count_tokens API 密钥（可选）
    #[serde(default)]
    pub count_tokens_api_key: Option<String>,

    /// count_tokens API 认证类型（可选，"x-api-key" 或 "bearer"，默认 "x-api-key"）
    #[serde(default = "default_count_tokens_auth_type")]
    pub count_tokens_auth_type: String,

    /// HTTP 代理地址（可选）
    /// 支持格式: http://host:port, https://host:port, socks5://host:port
    #[serde(default)]
    pub proxy_url: Option<String>,

    /// 代理认证用户名（可选）
    #[serde(default)]
    pub proxy_username: Option<String>,

    /// 代理认证密码（可选）
    #[serde(default)]
    pub proxy_password: Option<String>,

    /// Admin API 密钥（可选，启用 Admin API 功能）
    #[serde(default)]
    pub admin_api_key: Option<String>,

    /// thinking 默认预算（可选，热重载）
    #[serde(default)]
    pub thinking_budget_tokens: Option<i32>,

    /// 模型映射（可选，热重载）
    #[serde(default)]
    pub model_mapping: Option<HashMap<String, String>>,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_region() -> String {
    "us-east-1".to_string()
}

fn default_kiro_version() -> String {
    "0.8.0".to_string()
}

fn default_system_version() -> String {
    const SYSTEM_VERSIONS: &[&str] = &["darwin#24.6.0", "win32#10.0.22631"];
    SYSTEM_VERSIONS[fastrand::usize(..SYSTEM_VERSIONS.len())].to_string()
}

fn default_node_version() -> String {
    "22.21.1".to_string()
}

fn default_count_tokens_auth_type() -> String {
    "x-api-key".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            region: default_region(),
            kiro_version: default_kiro_version(),
            machine_id: None,
            api_key: None,
            api_keys: Vec::new(),
            system_version: default_system_version(),
            node_version: default_node_version(),
            count_tokens_api_url: None,
            count_tokens_api_key: None,
            count_tokens_auth_type: default_count_tokens_auth_type(),
            proxy_url: None,
            proxy_username: None,
            proxy_password: None,
            admin_api_key: None,
            thinking_budget_tokens: None,
            model_mapping: None,
        }
    }
}

impl Config {
    /// 获取默认配置文件路径
    pub fn default_config_path() -> &'static str {
        "config.json"
    }

    /// 从文件加载配置
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            // 配置文件不存在，返回默认配置
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let mut config: Config = serde_json::from_str(&content)?;

        // 向后兼容：如果有旧的 api_key 但没有 api_keys，自动迁移
        if config.api_keys.is_empty() {
            if let Some(old_key) = config.api_key.take() {
                config.api_keys.push(ApiKeyConfig::new(old_key, "Default".to_string()));
            }
        }

        Ok(config)
    }

    /// 获取所有启用的 API Keys
    pub fn get_enabled_api_keys(&self) -> Vec<&ApiKeyConfig> {
        self.api_keys.iter().filter(|k| k.enabled).collect()
    }

    /// 验证 API Key 是否有效
    pub fn validate_api_key(&self, key: &str) -> Option<&ApiKeyConfig> {
        self.api_keys.iter().find(|k| k.enabled && k.key == key)
    }

    /// 添加新的 API Key
    pub fn add_api_key(&mut self, key: String, name: String) -> String {
        let config = ApiKeyConfig::new(key, name);
        let id = config.id.clone();
        self.api_keys.push(config);
        id
    }

    /// 删除 API Key
    pub fn remove_api_key(&mut self, id: &str) -> bool {
        if let Some(pos) = self.api_keys.iter().position(|k| k.id == id) {
            self.api_keys.remove(pos);
            true
        } else {
            false
        }
    }

    /// 更新 API Key 状态
    pub fn update_api_key_status(&mut self, id: &str, enabled: bool) -> bool {
        if let Some(key) = self.api_keys.iter_mut().find(|k| k.id == id) {
            key.enabled = enabled;
            true
        } else {
            false
        }
    }
}
