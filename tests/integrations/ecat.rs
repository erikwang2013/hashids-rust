#![cfg(feature = "ecat")]
//! e-cat 端到端：Extension 层携带 Guard（e-cat 的 HTTP 传输就是 axum Router）。

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::Extension;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use hashids::integrations::ecat::hashids_layer;
use hashids::{Config, ConnectionConfig, Guard, HashidsManager};
use tower::ServiceExt;

fn app(guard: Guard) -> Router {
    Router::new()
        .route(
            "/user",
            get(|Extension(guard): Extension<Guard>| async move { guard.encode(&[1, 2, 3]) }),
        )
        .layer(hashids_layer(guard))
}

#[tokio::test]
async fn extension_layer_provides_guard() {
    let guard = Guard::from_manager(Arc::new(HashidsManager::new(
        Config::new().connection("main", ConnectionConfig::new()),
    )))
    .unwrap();

    let res = app(guard)
        .oneshot(Request::builder().uri("/user").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    assert_eq!(&body[..], b"o2fXhV"); // 官方默认向量
}

#[test]
fn unconfigured_default_connection_fails_at_wiring() {
    let manager = Arc::new(HashidsManager::new(Config::new())); // 没有 "main" 连接
    assert!(Guard::from_manager(manager).is_err());
}
