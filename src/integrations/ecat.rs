//! e-cat（[`ecat`]）集成。
//!
//! e-cat 的 HTTP 传输就是 axum `Router`（`HttpServer::new(addr).router(router)`），
//! 因此适配器只需把 [`Guard`] 包成 axum `Extension` 层挂上去，handler 提取
//! `Extension<Guard>`。
//!
//! ```
//! use std::sync::Arc;
//!
//! use axum::extract::Extension;
//! use axum::{Router, routing::get};
//! use hashids::integrations::ecat::hashids_layer;
//! use hashids::{Config, ConnectionConfig, Guard, HashidsManager};
//!
//! let manager = Arc::new(HashidsManager::new(
//!     Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
//! ));
//! let guard = Guard::from_manager(manager).unwrap();
//!
//! let router: Router = Router::new()
//!     .route(
//!         "/user",
//!         get(|Extension(guard): Extension<Guard>| async move { guard.encode(&[1, 2, 3]) }),
//!     )
//!     .layer(hashids_layer(guard));
//! ```

use axum::Extension;

use crate::Guard;

/// 把守卫包成 axum 扩展层：`router.layer(hashids_layer(guard))`。
///
/// 建议接线期就 [`Guard::from_manager`] 解析好（快速失败），而不是每请求解析。
pub fn hashids_layer(guard: Guard) -> Extension<Guard> {
    Extension(guard)
}
