//! Poem 集成：`FromRequest` 提取器。
//!
//! 用 `EndpointExt::data(Arc::new(manager))` 注册，handler 参数直接写
//! [`Hashids`]。
//!
//! ```
//! use std::sync::Arc;
//!
//! use hashids::integrations::poem::Hashids;
//! use hashids::{Config, ConnectionConfig, HashidsManager};
//! use poem::{EndpointExt, Route, get};
//!
//! let manager = Arc::new(HashidsManager::new(
//!     Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
//! ));
//!
//! #[poem::handler]
//! async fn index(hashids: Hashids) -> String {
//!     hashids.0.encode(&[1, 2, 3])
//! }
//!
//! let app = Route::new().at("/", get(index)).data(manager);
//! ```

use std::sync::Arc;

use poem::http::StatusCode;
use poem::{FromRequest, Request, RequestBody, Result};

use crate::{Guard, HashidsManager};

/// 提取器：默认连接的请求守卫。
#[derive(Debug, Clone)]
pub struct Hashids(pub Guard);

impl<'a> FromRequest<'a> for Hashids {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        let manager = req.data::<Arc<HashidsManager>>().ok_or_else(|| {
            poem::Error::from_string(
                "hashids: HashidsManager not registered",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        Guard::from_manager(Arc::clone(manager))
            .map(Self)
            .map_err(|err| {
                poem::Error::from_string(err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })
    }
}
