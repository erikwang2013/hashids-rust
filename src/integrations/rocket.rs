//! Rocket 集成：`FromRequest` 请求守卫。
//!
//! 用 `rocket::build().manage(Arc::new(manager))` 注册托管状态，路由函数参数
//! 直接写 [`Hashids`]。
//!
//! ```
//! use std::sync::Arc;
//!
//! use hashids::integrations::rocket::Hashids;
//! use hashids::{Config, ConnectionConfig, HashidsManager};
//!
//! let manager = Arc::new(HashidsManager::new(
//!     Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
//! ));
//!
//! #[rocket::get("/user")]
//! fn user(hashids: Hashids) -> String {
//!     hashids.0.encode(&[1, 2, 3])
//! }
//!
//! let rocket = rocket::build().manage(manager).mount("/", rocket::routes![user]);
//! ```

use std::sync::Arc;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use crate::{Guard, HashidsManager};

/// 请求守卫：默认连接的守卫生成器。
#[derive(Debug, Clone)]
pub struct Hashids(pub Guard);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Hashids {
    type Error = Status;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.rocket().state::<Arc<HashidsManager>>() {
            Some(manager) => match Guard::from_manager(Arc::clone(manager)) {
                Ok(guard) => Outcome::Success(Self(guard)),
                Err(_) => {
                    Outcome::Error((Status::InternalServerError, Status::InternalServerError))
                }
            },
            None => Outcome::Error((Status::InternalServerError, Status::InternalServerError)),
        }
    }
}
