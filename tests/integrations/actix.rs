#![cfg(feature = "actix")]
//! Actix Web 端到端：提取器拿到默认连接的 Guard，handler 里编出官方向量值。

use actix_web::{App, test, web};
use hashids::integrations::actix::Hashids;
use hashids::{Config, ConnectionConfig, HashidsManager};

fn manager(config: Config) -> HashidsManager {
    HashidsManager::new(config)
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

const CUSTOM_VECTOR: &str = "18apy3wlqkjvd5h1id7mn5ore2d06b";

#[actix_web::test]
async fn extractor_encodes_with_default_connection() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(manager(custom_config())))
            .route(
                "/user",
                web::get().to(|ids: Hashids| async move { ids.0.encode(&[1, 2, 3]) }),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/user").to_request();
    let body = test::call_and_read_body(&app, req).await;
    assert_eq!(&body[..], CUSTOM_VECTOR.as_bytes());
}

#[actix_web::test]
async fn unconfigured_default_connection_yields_500() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(manager(Config::new()))) // 没有 "main" 连接
            .route(
                "/user",
                web::get().to(|ids: Hashids| async move { ids.0.encode(&[1, 2, 3]) }),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/user").to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(
        res.status(),
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    );
    let body = test::read_body(res).await;
    assert!(
        String::from_utf8_lossy(&body).contains("hashids: default connection not configured"),
        "body: {}",
        String::from_utf8_lossy(&body)
    );
}

#[actix_web::test]
async fn missing_manager_yields_500() {
    // 未注册 web::Data：提取器拒绝请求（500），绝不 panic。
    let app = test::init_service(App::new().route(
        "/user",
        web::get().to(|ids: Hashids| async move { ids.0.encode(&[1, 2, 3]) }),
    ))
    .await;

    let req = test::TestRequest::get().uri("/user").to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(
        res.status(),
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    );
    let body = test::read_body(res).await;
    assert!(
        String::from_utf8_lossy(&body).contains("hashids: HashidsManager not registered"),
        "body: {}",
        String::from_utf8_lossy(&body)
    );
}
