#![cfg(feature = "rocket")]
//! Rocket 端到端：请求守卫拿到默认连接的 Guard，路由里编出官方向量值。

use std::sync::Arc;

use hashids::integrations::rocket::Hashids;
use hashids::{Config, ConnectionConfig, HashidsManager};

#[rocket::get("/user")]
fn user(hashids: Hashids) -> String {
    hashids.0.encode(&[1, 2, 3])
}

fn default_config() -> Config {
    Config::new().connection("main", ConnectionConfig::new())
}

#[test]
fn guard_encodes_with_default_connection() {
    let rocket = rocket::build()
        .manage(Arc::new(HashidsManager::new(default_config())))
        .mount("/", rocket::routes![user]);
    let client = rocket::local::blocking::Client::tracked(rocket).unwrap();

    let response = client.get("/user").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert_eq!(response.into_string().unwrap(), "o2fXhV"); // 官方默认向量
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
