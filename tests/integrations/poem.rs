#![cfg(feature = "poem")]
//! Poem 端到端：提取器拿到默认连接的 Guard，handler 里编出官方向量值。

use std::sync::Arc;

use hashids::integrations::poem::Hashids;
use hashids::{Config, ConnectionConfig, HashidsManager};
use poem::test::TestClient;
use poem::{EndpointExt, Route, get};

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

#[poem::handler]
async fn index(hashids: Hashids) -> String {
    hashids.0.encode(&[1, 2, 3])
}

fn route(config: Config) -> impl poem::Endpoint {
    Route::new()
        .at("/user", get(index))
        .data(Arc::new(HashidsManager::new(config)))
}

#[tokio::test]
async fn extractor_encodes_with_default_connection() {
    let app = route(custom_config());
    let resp = TestClient::new(app).get("/user").send().await;

    resp.assert_status_is_ok();
    resp.assert_text(CUSTOM_VECTOR).await;
}

#[tokio::test]
async fn unconfigured_default_connection_yields_500() {
    let app = route(Config::new()); // 没有 "main" 连接
    let resp = TestClient::new(app).get("/user").send().await;

    resp.assert_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR);
    let body = resp.0.into_body().into_string().await.unwrap();
    assert!(
        body.contains("hashids: default connection not configured"),
        "body: {body}"
    );
}

#[tokio::test]
async fn missing_manager_yields_500() {
    // 未注册 Data：提取器拒绝请求（500），绝不 panic。
    let app = Route::new().at("/user", get(index));
    let resp = TestClient::new(app).get("/user").send().await;

    resp.assert_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR);
    let body = resp.0.into_body().into_string().await.unwrap();
    assert!(
        body.contains("hashids: HashidsManager not registered"),
        "body: {body}"
    );
}
