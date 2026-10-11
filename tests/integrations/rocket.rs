#![cfg(feature = "rocket")]
//! Rocket 端到端：请求守卫拿到默认连接的 Guard，路由里编出官方向量值。

use std::sync::Arc;

use hashids::integrations::rocket::Hashids;
use hashids::{Config, ConnectionConfig, HashidsManager};

#[rocket::get("/user")]
fn user(hashids: Hashids) -> String {
    hashids.0.encode(&[1, 2, 3])
}

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

#[test]
fn guard_encodes_with_default_connection() {
    let rocket = rocket::build()
        .manage(Arc::new(HashidsManager::new(custom_config())))
        .mount("/", rocket::routes![user]);
    let client = rocket::local::blocking::Client::tracked(rocket).unwrap();

    let response = client.get("/user").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert_eq!(
        response.into_string().unwrap(),
        "18apy3wlqkjvd5h1id7mn5ore2d06b" // custom_params 向量 encode([1,2,3])
    );
}

#[test]
fn unconfigured_default_connection_yields_500() {
    let rocket = rocket::build()
        .manage(Arc::new(HashidsManager::new(Config::new()))) // 没有 "main" 连接
        .mount("/", rocket::routes![user]);
    let client = rocket::local::blocking::Client::tracked(rocket).unwrap();

    assert_eq!(
        client.get("/user").dispatch().status(),
        rocket::http::Status::InternalServerError
    );
}

#[test]
fn missing_manager_yields_500() {
    // 未 manage 状态：守卫拒绝请求（500），绝不 panic。rocket 只回状态码，无自定义文案。
    let rocket = rocket::build().mount("/", rocket::routes![user]);
    let client = rocket::local::blocking::Client::tracked(rocket).unwrap();

    assert_eq!(
        client.get("/user").dispatch().status(),
        rocket::http::Status::InternalServerError
    );
}
