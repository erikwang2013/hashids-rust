//! bee-rust（[`bee_router`]）集成。
//!
//! bee 的路由底层就是 axum 0.8：handler 即 axum handler，直接配合 `axum`
//! feature 的提取器使用。bee 原生侧提供 [`HashidsFilter`]——把 [`Guard`] 注入
//! 每次 `Context::dispatch` 的请求扩展，controller 里用
//! [`ContextHashids::hashids`] 取回。
//!
//! ```
//! use std::sync::Arc;
//!
//! use bee_router::Filter;
//! use hashids::integrations::bee::HashidsFilter;
//! use hashids::{Config, ConnectionConfig, Guard, HashidsManager};
//!
//! let manager = Arc::new(HashidsManager::new(
//!     Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
//! ));
//! let guard = Guard::from_manager(manager).unwrap();
//!
//! // controller 内 `ctx.dispatch(cache, ttl, &[&filter], &controller)`；
//! // 然后 `ctx.hashids().unwrap().encode(&[1, 2, 3])`。
//! let filter: &dyn Filter = &HashidsFilter::new(guard);
//! ```

use bee_router::context::RouterError;
use bee_router::{Context, Filter};

use crate::Guard;

/// bee 原生 Filter：把 [`Guard`] 注入每个请求的 `Context`。
pub struct HashidsFilter {
    guard: Guard,
}

impl HashidsFilter {
    /// 用给定守卫创建过滤器。
    pub fn new(guard: Guard) -> Self {
        Self { guard }
    }
}

impl Filter for HashidsFilter {
    fn before(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.request.extensions_mut().insert(self.guard.clone());
        Ok(())
    }
}

/// `Context` 扩展：取回注入的 [`Guard`]。
pub trait ContextHashids {
    /// 取回注入的 [`Guard`]；未挂载过滤器时返回 `None`。
    fn hashids(&self) -> Option<Guard>;
}

impl ContextHashids for Context {
    fn hashids(&self) -> Option<Guard> {
        self.request.extensions().get::<Guard>().cloned()
    }
}
