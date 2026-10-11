#![cfg(feature = "axum")]
//! Axum 端到端：提取器拿到默认连接的 Guard，handler 里编出官方向量值；
//! 另覆盖 `FromRef` 自定义应用状态路径。

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::FromRef;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use hashids::integrations::axum::Hashids;
use hashids::{Config, ConnectionConfig, HashidsManager};
use tower::ServiceExt;

/// 有区分度的官方向量（tests/vectors.rs 的 custom_params）：
/// salt + 最短长度 30 + 自定义字母表，encode([1, 2, 3]) 即本向量；
/// 默认参数会编出 "o2fXhV"，测不出配置链上的退化。
fn custom_config() -> Config {
    Config::new().connection(
        "main",
        ConnectionConfig::new()
            .salt("this is my salt")
            .min_hash_length(30)
            .alphabet("xzal86grmb4jhysfoqp3we7291kuct5iv0nd"),
    )
}

const CUSTOM_VECTOR: &str = "18apy3wlqkjvd5h1id7mn5ore2d06b";

fn app(config: Config) -> Router {
    let manager = Arc::new(HashidsManager::new(config));
    Router::new()
        .route(
            "/user",
            get(|hashids: Hashids| async move { hashids.0.encode(&[1, 2, 3]) }),
        )
        .with_state(manager)
}

async fn body_text(res: axum::response::Response) -> String {
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

#[tokio::test]
async fn extractor_encodes_with_default_connection() {
    let res = app(custom_config())
        .oneshot(Request::builder().uri("/user").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(body_text(res).await, CUSTOM_VECTOR);
}

#[tokio::test]
async fn unconfigured_default_connection_yields_500() {
    let res = app(Config::new()) // 没有 "main" 连接
        .oneshot(Request::builder().uri("/user").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

/// 自定义应用状态：manager 嵌在自己的 `AppState` 里，经 `FromRef` 取出。
#[derive(Clone)]
struct AppState {
    hashids: Arc<HashidsManager>,
}

impl FromRef<AppState> for Arc<HashidsManager> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.hashids)
    }
}

#[tokio::test]
async fn extractor_works_with_custom_from_ref_state() {
    let state = AppState {
        hashids: Arc::new(HashidsManager::new(custom_config())),
    };
    let app = Router::new()
        .route(
            "/user",
            get(|hashids: Hashids| async move { hashids.0.encode(&[1, 2, 3]) }),
        )
        .with_state(state);

    let res = app
        .oneshot(Request::builder().uri("/user").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(body_text(res).await, CUSTOM_VECTOR);
}
