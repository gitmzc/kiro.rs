//! Web 管理界面静态文件服务
//!
//! 使用 rust-embed 嵌入前端构建产物，提供 SPA 路由支持

use axum::{
    body::Body,
    extract::Request,
    http::{header, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "web-admin/dist"]
struct WebAssets;

/// 服务静态文件的处理器
///
/// 实现 SPA fallback：
/// - 只处理 GET 请求
/// - 如果请求的文件存在，返回该文件
/// - 如果文件不存在，返回 index.html（让前端路由处理）
pub async fn serve_web_assets(req: Request) -> Response {
    // 只处理 GET 请求，其他请求返回 404
    if req.method() != Method::GET {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap();
    }

    let uri = req.uri();
    let path = uri.path().trim_start_matches('/');

    // 如果是根路径，返回 index.html
    if path.is_empty() {
        return serve_index_html();
    }

    // 尝试获取请求的文件
    match WebAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap()
        }
        None => {
            // 文件不存在，返回 index.html（SPA fallback）
            serve_index_html()
        }
    }
}

/// 返回 index.html
fn serve_index_html() -> Response {
    match WebAssets::get("index.html") {
        Some(content) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from(content.data))
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Web admin interface not found"))
            .unwrap(),
    }
}
