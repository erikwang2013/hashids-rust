#![cfg(feature = "warp")]
//! Warp 端到端：过滤器携带 Guard，handler 里编出官方向量值。

use std::sync::Arc;

use hashids::integrations::warp::{Hashids, hashids};
use hashids::{Config, ConnectionConfig, HashidsManager};
use warp::Filter;

#[tokio::test]
async fn filter_provides_guard() {
    let manager = Arc::new(HashidsManager::new(
        Config::new().connection("main", ConnectionConfig::new()),
    ));
    let route = warp::get()
        .and(warp::path("user"))
        .and(hashids(manager).unwrap())
        .map(|ids: Hashids| ids.0.encode(&[1, 2, 3]));

    let res = warp::test::request()
        .method("GET")
        .path("/user")
        .reply(&route)
        .await;

    assert_eq!(res.status(), 200);
    assert_eq!(&res.body()[..], b"o2fXhV"); // 官方默认向量
}

#[test]
fn unconfigured_default_connection_fails_at_wiring() {
    let manager = Arc::new(HashidsManager::new(Config::new())); // 没有 "main" 连接
    assert!(hashids(manager).is_err());
}
