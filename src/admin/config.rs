//! 配置管理（热重载）

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::model::config::Config;
use crate::runtime_config;

#[derive(Clone)]
pub struct ConfigManager {
    path: PathBuf,
    config: Arc<RwLock<Config>>,
}

impl ConfigManager {
    pub fn new(path: impl AsRef<Path>, config: Config) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub fn get(&self) -> Config {
        self.config.read().clone()
    }

    pub fn update_runtime(&self, thinking_budget_tokens: Option<i32>, model_mapping: Option<HashMap<String, String>>) {
        if let Some(tokens) = thinking_budget_tokens {
            runtime_config::update_thinking_budget_tokens(tokens.max(1));
        }
        if let Some(mapping) = model_mapping {
            runtime_config::update_model_mapping(mapping);
        }
    }

    pub fn update_config(&self, patch: ConfigPatch) -> anyhow::Result<Config> {
        let mut current = self.config.write();
        if let Some(tokens) = patch.thinking_budget_tokens {
            current.thinking_budget_tokens = Some(tokens.max(1));
        }
        if let Some(mapping) = patch.model_mapping {
            current.model_mapping = Some(mapping);
        }

        let serialized = serde_json::to_string_pretty(&*current)?;
        std::fs::write(&self.path, serialized)?;

        self.update_runtime(current.thinking_budget_tokens, current.model_mapping.clone());
        Ok(current.clone())
    }

    // ============ API Key 管理 ============

    /// 添加新的 API Key
    pub fn add_api_key(&self, key: String, name: String) -> anyhow::Result<(String, i64)> {
        let mut current = self.config.write();
        let id = current.add_api_key(key, name);
        let created_at = current.api_keys.iter()
            .find(|k| k.id == id)
            .map(|k| k.created_at)
            .unwrap_or(0);

        let serialized = serde_json::to_string_pretty(&*current)?;
        std::fs::write(&self.path, serialized)?;

        Ok((id, created_at))
    }

    /// 更新 API Key
    pub fn update_api_key(&self, id: &str, name: Option<String>, enabled: Option<bool>) -> anyhow::Result<()> {
        let mut current = self.config.write();

        let key = current.api_keys.iter_mut()
            .find(|k| k.id == id)
            .ok_or_else(|| anyhow::anyhow!("API Key 不存在"))?;

        if let Some(n) = name {
            key.name = n;
        }
        if let Some(e) = enabled {
            key.enabled = e;
        }

        let serialized = serde_json::to_string_pretty(&*current)?;
        std::fs::write(&self.path, serialized)?;

        Ok(())
    }

    /// 删除 API Key
    pub fn delete_api_key(&self, id: &str) -> anyhow::Result<()> {
        let mut current = self.config.write();

        if !current.remove_api_key(id) {
            return Err(anyhow::anyhow!("API Key 不存在"));
        }

        let serialized = serde_json::to_string_pretty(&*current)?;
        std::fs::write(&self.path, serialized)?;

        Ok(())
    }

    // ============ 密码管理 ============

    /// 修改管理员密码
    pub fn change_admin_password(&self, old_password: &str, new_password: &str) -> anyhow::Result<()> {
        let mut current = self.config.write();

        // 验证旧密码
        if let Some(current_password) = &current.admin_api_key {
            if current_password != old_password {
                return Err(anyhow::anyhow!("旧密码不正确"));
            }
        } else {
            return Err(anyhow::anyhow!("未设置管理员密码"));
        }

        // 验证新密码长度
        if new_password.len() < 8 {
            return Err(anyhow::anyhow!("新密码长度至少为 8 个字符"));
        }

        // 更新密码
        current.admin_api_key = Some(new_password.to_string());

        let serialized = serde_json::to_string_pretty(&*current)?;
        std::fs::write(&self.path, serialized)?;

        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPatch {
    pub thinking_budget_tokens: Option<i32>,
    pub model_mapping: Option<HashMap<String, String>>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigView {
    pub host: String,
    pub port: u16,
    pub region: String,
    pub kiro_version: String,
    pub system_version: String,
    pub node_version: String,
    pub count_tokens_api_url: Option<String>,
    pub count_tokens_auth_type: String,
    pub proxy_url: Option<String>,
    pub admin_api_key: Option<String>,
    pub api_key: Option<String>,
    pub proxy_username: Option<String>,
    pub proxy_password: Option<String>,
    pub count_tokens_api_key: Option<String>,
    pub thinking_budget_tokens: Option<i32>,
    pub model_mapping: Option<HashMap<String, String>>,
}

impl ConfigView {
    pub fn from_config(config: Config) -> Self {
        Self {
            host: config.host,
            port: config.port,
            region: config.region,
            kiro_version: config.kiro_version,
            system_version: config.system_version,
            node_version: config.node_version,
            count_tokens_api_url: config.count_tokens_api_url,
            count_tokens_auth_type: config.count_tokens_auth_type,
            proxy_url: config.proxy_url,
            admin_api_key: mask(config.admin_api_key),
            api_key: mask(config.api_key),
            proxy_username: mask(config.proxy_username),
            proxy_password: mask(config.proxy_password),
            count_tokens_api_key: mask(config.count_tokens_api_key),
            thinking_budget_tokens: config.thinking_budget_tokens,
            model_mapping: config.model_mapping,
        }
    }
}

fn mask(value: Option<String>) -> Option<String> {
    value.map(|v| {
        if v.len() <= 6 {
            "***".to_string()
        } else {
            format!("{}***", &v[..4])
        }
    })
}
