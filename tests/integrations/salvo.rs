#![cfg(feature = "salvo")]
//! Salvo 端到端：hoop 注入器把 Guard 放进 Depot，handler 取出后编出官方向量值。

use std::sync::Arc;

use hashids::integrations::salvo::{DepotHashids, HashidsInjector};
use hashids::{Config, ConnectionConfig, HashidsManager};
use salvo::http::errors::StatusError;
use salvo::prelude::*;
use salvo::test::{ResponseExt, TestClient};

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

#[salvo::handler]
async fn user(depot: &mut Depot) -> String {
    depot
        .hashids()
        .expect("guard 应已注入")
        .0
        .encode(&[1, 2, 3])
}

/// 不预设 guard 的 handler：取不到时返回 500，绝不 panic。
#[salvo::handler]
async fn user_optional(depot: &mut Depot) -> Result<String, StatusError> {
    Ok(depot.hashids()?.0.encode(&[1, 2, 3]))
}

fn router(config: Config) -> Router {
    let manager = Arc::new(HashidsManager::new(config));
    Router::new()
        .hoop(HashidsInjector::new(manager))
        .push(Router::with_path("/user").get(user))
}

#[tokio::test]
async fn injector_provides_guard_to_handlers() {
    let mut res = TestClient::get("http://127.0.0.1/user")
        .send(router(custom_config()))
        .await;

    assert_eq!(res.status_code, Some(StatusCode::OK));
    assert_eq!(res.take_string().await.unwrap(), CUSTOM_VECTOR);
}

#[tokio::test]
async fn unconfigured_default_connection_yields_500() {
    let mut res = TestClient::get("http://127.0.0.1/user")
        .send(router(Config::new())) // 没有 "main" 连接
        .await;

    assert_eq!(res.status_code, Some(StatusCode::INTERNAL_SERVER_ERROR));
    let body = res.take_string().await.unwrap();
    assert!(
        body.contains("hashids: default connection not configured"),
        "body: {body}"
    );
}

#[tokio::test]
async fn missing_injector_yields_500() {
    // 不挂 hoop：Depot 里没有 guard，handler 走 Result 路径返回 500，绝不 panic。
    let router = Router::with_path("/user").get(user_optional);
    let mut res = TestClient::get("http://127.0.0.1/user").send(router).await;

    assert_eq!(res.status_code, Some(StatusCode::INTERNAL_SERVER_ERROR));
    let body = res.take_string().await.unwrap();
    // 注入器未挂载时统一文案不会出现，此处回退为 salvo 标准 StatusError 文案。
    assert!(body.contains("Internal Server Error"), "body: {body}");
}
