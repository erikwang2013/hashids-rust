#![cfg(feature = "actix")]
//! Actix Web 端到端：提取器拿到默认连接的 Guard，handler 里编出官方向量值。

use actix_web::{App, test, web};
use hashids::integrations::actix::Hashids;
use hashids::{Config, ConnectionConfig, HashidsManager};

fn manager(config: Config) -> HashidsManager {
    HashidsManager::new(config)
}

#[actix_web::test]
async fn extractor_encodes_with_default_connection() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(manager(
                Config::new().connection("main", ConnectionConfig::new()),
            )))
            .route(
                "/user",
                web::get().to(|ids: Hashids| async move { ids.0.encode(&[1, 2, 3]) }),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/user").to_request();
    let body = test::call_and_read_body(&app, req).await;
    assert_eq!(&body[..], b"o2fXhV"); // 官方默认向量 encode([1,2,3])
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
}
