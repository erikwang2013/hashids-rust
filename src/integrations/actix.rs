//! Actix Web 集成：`FromRequest` 提取器。
//!
//! 用 `App::app_data(web::Data::new(manager))` 注册，handler 参数直接写
//! [`Hashids`]。
//!
//! ```
//! use actix_web::web;
//! use hashids::integrations::actix::Hashids;
//! use hashids::{Config, ConnectionConfig, HashidsManager};
//!
//! let manager = HashidsManager::new(
//!     Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
//! );
//!
//! async fn show(ids: Hashids) -> String {
//!     ids.0.encode(&[1, 2, 3])
//! }
//!
//! let app = actix_web::App::new().app_data(web::Data::new(manager));
//! ```

use std::future::{Ready, ready};

use actix_web::dev::Payload;
use actix_web::error::ErrorInternalServerError;
use actix_web::{FromRequest, HttpRequest, web};

use crate::{Guard, HashidsManager};

/// 提取器：默认连接的请求守卫。
#[derive(Debug, Clone)]
pub struct Hashids(pub Guard);

impl FromRequest for Hashids {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        match req.app_data::<web::Data<HashidsManager>>().cloned() {
            // 固定文案：不回显内部连接名等细节（与 axum 适配器一致）。
            Some(data) => ready(
                Guard::from_manager(data.into_inner())
                    .map(Self)
                    .map_err(|_| {
                        ErrorInternalServerError("hashids: default connection not configured")
                    }),
            ),
            None => ready(Err(ErrorInternalServerError(
                "hashids: HashidsManager not registered",
            ))),
        }
    }
}
