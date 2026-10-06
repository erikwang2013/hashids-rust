#![cfg(feature = "salvo")]
//! Salvo 端到端：hoop 注入器把 Guard 放进 Depot，handler 取出后编出官方向量值。

use std::sync::Arc;

use hashids::integrations::salvo::{DepotHashids, HashidsInjector};
use hashids::{Config, ConnectionConfig, HashidsManager};
use salvo::prelude::*;
use salvo::test::{ResponseExt, TestClient};

#[salvo::handler]
async fn user(depot: &mut Depot) -> String {
    depot
        .hashids()
        .expect("guard 应已注入")
        .0
        .encode(&[1, 2, 3])
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
        .send(router(
            Config::new().connection("main", ConnectionConfig::new()),
        ))
        .await;

    assert_eq!(res.status_code, Some(StatusCode::OK));
    assert_eq!(res.take_string().await.unwrap(), "o2fXhV"); // 官方默认向量
}

#[tokio::test]
async fn unconfigured_default_connection_yields_500() {
    let res = TestClient::get("http://127.0.0.1/user")
        .send(router(Config::new())) // 没有 "main" 连接
        .await;

    assert_eq!(res.status_code, Some(StatusCode::INTERNAL_SERVER_ERROR));
}
