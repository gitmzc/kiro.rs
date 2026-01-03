mod admin;
mod anthropic;
mod common;
mod http_client;
mod kiro;
mod model;
pub mod token;
mod runtime_config;
mod web;

use std::sync::Arc;
use std::io::Write;

use clap::Parser;
use kiro::model::credentials::{CredentialsConfig, KiroCredentials};
use kiro::provider::KiroProvider;
use kiro::token_manager::MultiTokenManager;
use model::config::Config;
use model::arg::Args;

// 组合 Writer：同时写入文件和广播
#[derive(Clone)]
struct CombinedWriter<F, B> {
    file: F,
    broadcast: B,
}

impl<F: Write, B: Write> Write for CombinedWriter<F, B> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // 写入文件
        let _ = self.file.write(buf);
        // 写入广播（忽略错误，因为可能没有订阅者）
        let _ = self.broadcast.write(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let _ = self.file.flush();
        let _ = self.broadcast.flush();
        Ok(())
    }
}

// 实现 MakeWriter trait 以支持 tracing_subscriber
impl<'a, F, B> tracing_subscriber::fmt::MakeWriter<'a> for CombinedWriter<F, B>
where
    F: Write + Clone + 'a,
    B: Write + Clone + 'a,
{
    type Writer = CombinedWriter<F, B>;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[tokio::main]
async fn main() {
    // 解析命令行参数
    let args = Args::parse();

    // 加载配置
    let config_path = args.config.unwrap_or_else(|| Config::default_config_path().to_string());
    let config = Config::load(&config_path).unwrap_or_else(|e| {
        tracing::error!("加载配置失败: {}", e);
        std::process::exit(1);
    });

    // 初始化运行时可热更新配置
    runtime_config::init_runtime_config(runtime_config::RuntimeConfig {
        thinking_budget_tokens: config.thinking_budget_tokens.unwrap_or(20000),
        model_mapping: config.model_mapping.clone().unwrap_or_default(),
    });

    // 初始化日志（stdout + 轮转文件 + SSE 广播）
    let log_broadcaster = admin::LogBroadcaster::new(1000);

    // 创建组合 writer：同时写入 stdout 和广播
    let broadcast_writer = log_broadcaster.writer();
    let combined_writer = CombinedWriter {
        file: std::io::stdout(),
        broadcast: broadcast_writer,
    };

    // 配置日志输出到 stdout 和 SSE 广播
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_ansi(false) // 禁用 ANSI 颜色代码
        .with_target(false) // 隐藏模块路径
        .compact() // 使用紧凑格式
        .with_writer(combined_writer)
        .init();

    // 加载凭证（支持单对象或数组格式）
    let credentials_path = args.credentials.unwrap_or_else(|| KiroCredentials::default_credentials_path().to_string());
    let credentials_config = CredentialsConfig::load(&credentials_path).unwrap_or_else(|e| {
        tracing::error!("加载凭证失败: {}", e);
        std::process::exit(1);
    });

    // 判断是否为多凭据格式（用于刷新后回写）
    let is_multiple_format = credentials_config.is_multiple();

    // 转换为按优先级排序的凭据列表
    let credentials_list = credentials_config.into_sorted_credentials();
    tracing::info!("已加载 {} 个凭据配置", credentials_list.len());

    // 获取第一个凭据用于日志显示
    let first_credentials = credentials_list.first().cloned().unwrap_or_default();
    tracing::debug!("主凭证: {:?}", first_credentials);

    // 获取第一个启用的 API Key（用于向后兼容的单 key 认证）
    let api_key = config.get_enabled_api_keys()
        .first()
        .map(|k| k.key.clone())
        .unwrap_or_else(|| {
            tracing::error!("配置文件中未设置任何启用的 API Key");
            std::process::exit(1);
        });

    // 构建代理配置
    let proxy_config = config.proxy_url.as_ref().map(|url| {
        let mut proxy = http_client::ProxyConfig::new(url);
        if let (Some(username), Some(password)) = (&config.proxy_username, &config.proxy_password) {
            proxy = proxy.with_auth(username, password);
        }
        proxy
    });

    if proxy_config.is_some() {
        tracing::info!("已配置 HTTP 代理: {}", config.proxy_url.as_ref().unwrap());
    }

    // 创建 MultiTokenManager 和 KiroProvider
    let token_manager = MultiTokenManager::new(
        config.clone(),
        credentials_list,
        proxy_config.clone(),
        Some(credentials_path.into()),
        is_multiple_format,
    )
    .unwrap_or_else(|e| {
        tracing::error!("创建 Token 管理器失败: {}", e);
        std::process::exit(1);
    });
    let token_manager = Arc::new(token_manager);
    let kiro_provider = KiroProvider::with_proxy(token_manager.clone(), proxy_config.clone());

    // 初始化 count_tokens 配置
    token::init_config(token::CountTokensConfig {
        api_url: config.count_tokens_api_url.clone(),
        api_key: config.count_tokens_api_key.clone(),
        auth_type: config.count_tokens_auth_type.clone(),
        proxy: proxy_config,
    });

    // 初始化统计服务
    let stats_service = admin::StatsService::new("./data/admin_stats.db")
        .await
        .unwrap_or_else(|e| {
            tracing::error!("初始化统计数据库失败: {}", e);
            std::process::exit(1);
        });
    let stats_service = std::sync::Arc::new(stats_service);
    if let Err(e) = stats_service.cleanup_older_than(7).await {
        tracing::warn!("统计数据清理失败: {}", e);
    }
    let stats_cleanup = stats_service.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(24 * 60 * 60));
        loop {
            interval.tick().await;
            if let Err(e) = stats_cleanup.cleanup_older_than(7).await {
                tracing::warn!("统计数据定时清理失败: {}", e);
            }
        }
    });

    // 构建 Anthropic API 路由（从第一个凭据获取 profile_arn）
    let anthropic_app = anthropic::create_router_with_provider(
        &api_key,
        Some(kiro_provider),
        first_credentials.profile_arn.clone(),
    );

    // 构建 Admin API 路由（如果配置了非空的 admin_api_key）
    // 安全检查：空字符串被视为未配置，防止空 key 绕过认证
    let admin_key_valid = config
        .admin_api_key
        .as_ref()
        .map(|k| !k.trim().is_empty())
        .unwrap_or(false);

    let mut app = if let Some(admin_key) = &config.admin_api_key {
        if admin_key.trim().is_empty() {
            tracing::warn!("admin_api_key 配置为空，Admin API 未启用");
            anthropic_app
        } else {
            let config_manager = admin::ConfigManager::new(&config_path, config.clone());
            let admin_service = admin::AdminService::new(
                token_manager.clone(),
                stats_service.clone(),
                config_manager,
                log_broadcaster.clone(),
            );
            let admin_state = admin::AdminState::new(admin_key, admin_service);
            let admin_app = admin::create_admin_router(admin_state);

            tracing::info!("Admin API 已启用");
            anthropic_app.nest("/api/admin", admin_app)
        }
    } else {
        anthropic_app
    };

    app = app.layer(axum::middleware::from_fn_with_state(
        stats_service.clone(),
        admin::stats_middleware,
    ));

    // 启动服务器
    let addr = format!("{}:{}", config.host, config.port);

    // 添加 Web 管理界面路由（SPA fallback）
    app = app.fallback(web::serve_web_assets);
    tracing::info!("Web 管理界面已启用: http://{}/", addr);
    tracing::info!("启动 Anthropic API 端点: {}", addr);
    tracing::info!("API Key: {}***", &api_key[..(api_key.len() / 2)]);
    tracing::info!("可用 API:");
    tracing::info!("  GET  /v1/models");
    tracing::info!("  POST /v1/messages");
    tracing::info!("  POST /v1/messages/count_tokens");
    if admin_key_valid {
        tracing::info!("Admin API:");
        tracing::info!("  GET  /api/admin/credentials");
        tracing::info!("  POST /api/admin/credentials/:index/disabled");
        tracing::info!("  POST /api/admin/credentials/:index/priority");
        tracing::info!("  POST /api/admin/credentials/:index/reset");
        tracing::info!("  GET  /api/admin/credentials/:index/balance");
        tracing::info!("  GET  /api/admin/stats/summary");
        tracing::info!("  GET  /api/admin/stats/timeseries");
        tracing::info!("  GET  /api/admin/stats/requests");
        tracing::info!("  GET  /api/admin/health");
        tracing::info!("  GET  /api/admin/logs/stream");
        tracing::info!("  GET  /api/admin/config");
        tracing::info!("  POST /api/admin/config");
    }

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
