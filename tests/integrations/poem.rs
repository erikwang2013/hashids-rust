#![cfg(feature = "poem")]
//! Poem 端到端：提取器拿到默认连接的 Guard，handler 里编出官方向量值。

use std::sync::Arc;

use hashids::integrations::poem::Hashids;
use hashids::{Config, ConnectionConfig, HashidsManager};
use poem::test::TestClient;
use poem::{EndpointExt, Route, get};

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
    let app = route(Config::new().connection("main", ConnectionConfig::new()));
    let resp = TestClient::new(app).get("/user").send().await;

    resp.assert_status_is_ok();
    resp.assert_text("o2fXhV").await; // 官方默认向量
}

#[tokio::test]
async fn unconfigured_default_connection_yields_500() {
    let app = route(Config::new()); // 没有 "main" 连接
    let resp = TestClient::new(app).get("/user").send().await;

    resp.assert_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR);
}
