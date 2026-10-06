#![cfg(feature = "axum")]
//! Axum 端到端：提取器拿到默认连接的 Guard，handler 里编出官方向量值。

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::routing::get;
use hashids::integrations::axum::Hashids;
use hashids::{Config, ConnectionConfig, HashidsManager};
use tower::ServiceExt;

fn app(config: Config) -> Router {
    let manager = Arc::new(HashidsManager::new(config));
    Router::new()
        .route(
            "/user",
            get(|hashids: Hashids| async move { hashids.0.encode(&[1, 2, 3]) }),
        )
        .with_state(manager)
}

fn default_config() -> Config {
    Config::new().connection("main", ConnectionConfig::new())
}

#[tokio::test]
async fn extractor_encodes_with_default_connection() {
    let res = app(default_config())
        .oneshot(Request::builder().uri("/user").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    assert_eq!(&body[..], b"o2fXhV"); // 官方默认向量 encode([1,2,3])
}

#[tokio::test]
async fn unconfigured_default_connection_yields_500() {
    let res = app(Config::new()) // 没有 "main" 连接
        .oneshot(Request::builder().uri("/user").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
