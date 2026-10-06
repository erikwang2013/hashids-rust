//! Axum 集成：`FromRequestParts` 提取器。
//!
//! 把 `Arc<HashidsManager>` 放进应用状态（`Router::with_state`），handler 里
//! 直接提取 [`Hashids`]；应用状态是自己的 `AppState` 时，补一个
//! `impl FromRef<AppState> for Arc<HashidsManager>` 即可。
//!
//! ```
//! use std::sync::Arc;
//!
//! use axum::{routing::get, Router};
//! use hashids::integrations::axum::Hashids;
//! use hashids::{Config, ConnectionConfig, HashidsManager};
//!
//! let manager = Arc::new(HashidsManager::new(
//!     Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
//! ));
//!
//! let app: Router = Router::new()
//!     .route(
//!         "/user",
//!         get(|hashids: Hashids| async move { hashids.0.encode(&[1, 2, 3]) }),
//!     )
//!     .with_state(manager);
//! ```

use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::StatusCode;
use axum::http::request::Parts;

use crate::{Guard, HashidsManager};

/// 提取器：默认连接的请求守卫。
#[derive(Debug, Clone)]
pub struct Hashids(pub Guard);

impl<S> FromRequestParts<S> for Hashids
where
    Arc<HashidsManager>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let manager = Arc::<HashidsManager>::from_ref(state);
        Guard::from_manager(manager).map(Self).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "hashids: default connection not configured",
            )
        })
    }
}
