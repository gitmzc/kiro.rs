//! 运行时可热更新配置

use std::collections::HashMap;
use std::sync::OnceLock;

use parking_lot::RwLock;

#[derive(Clone, Debug, Default)]
pub struct RuntimeConfig {
    pub thinking_budget_tokens: i32,
    pub model_mapping: HashMap<String, String>,
}

static RUNTIME_CONFIG: OnceLock<RwLock<RuntimeConfig>> = OnceLock::new();

pub fn init_runtime_config(config: RuntimeConfig) {
    let _ = RUNTIME_CONFIG.set(RwLock::new(config));
}

pub fn update_thinking_budget_tokens(value: i32) {
    if let Some(lock) = RUNTIME_CONFIG.get() {
        lock.write().thinking_budget_tokens = value;
    }
}

pub fn update_model_mapping(mapping: HashMap<String, String>) {
    if let Some(lock) = RUNTIME_CONFIG.get() {
        lock.write().model_mapping = mapping;
    }
}

pub fn thinking_budget_tokens() -> i32 {
    RUNTIME_CONFIG
        .get()
        .map(|lock| lock.read().thinking_budget_tokens)
        .unwrap_or(20000)
}

pub fn model_mapping() -> HashMap<String, String> {
    RUNTIME_CONFIG
        .get()
        .map(|lock| lock.read().model_mapping.clone())
        .unwrap_or_default()
}
