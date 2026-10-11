#![cfg(feature = "warp")]
//! Warp 端到端：过滤器携带 Guard，handler 里编出官方向量值。

use std::sync::Arc;

use hashids::integrations::warp::{Hashids, hashids};
use hashids::{Config, ConnectionConfig, HashidsManager};
use warp::Filter;

#[tokio::test]
async fn filter_provides_guard() {
    // custom_params 向量（tests/vectors.rs）：salt + 最短长度 30 + 自定义字母表。
    let manager = Arc::new(HashidsManager::new(
        Config::new().connection(
            "main",
            ConnectionConfig::new()
                .salt("this is my salt")
                .min_hash_length(30)
                .alphabet("xzal86grmb4jhysfoqp3we7291kuct5iv0nd"),
        ),
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
    assert_eq!(&res.body()[..], b"18apy3wlqkjvd5h1id7mn5ore2d06b");
}

#[test]
fn unconfigured_default_connection_fails_at_wiring() {
    let manager = Arc::new(HashidsManager::new(Config::new())); // 没有 "main" 连接
    assert!(hashids(manager).is_err());
}
