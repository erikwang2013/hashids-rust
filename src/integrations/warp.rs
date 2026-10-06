//! Warp 集成：Filter 组合。
//!
//! warp 没有提取器 trait，惯用做法是过滤器链。[`hashids`] 产出一个携带
//! [`Hashids`] 的 Filter，**接线期**解析默认连接（未配置时立即返回
//! [`crate::Error`]，快速失败）。
//!
//! ```
//! use std::sync::Arc;
//!
//! use hashids::integrations::warp::{Hashids, hashids};
//! use hashids::{Config, ConnectionConfig, HashidsManager};
//! use warp::Filter;
//!
//! let manager = Arc::new(HashidsManager::new(
//!     Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
//! ));
//!
//! let route = warp::get()
//!     .and(hashids(manager).unwrap())
//!     .map(|ids: Hashids| ids.0.encode(&[1, 2, 3]));
//! ```

use std::convert::Infallible;
use std::sync::Arc;

use warp::Filter;

use crate::{Error, Guard, HashidsManager};

/// 过滤器中携带的守卫。
#[derive(Debug, Clone)]
pub struct Hashids(pub Guard);

/// 构造产出 [`Hashids`] 的 warp Filter；默认连接未配置时返回错误。
pub fn hashids(
    manager: Arc<HashidsManager>,
) -> Result<impl Filter<Extract = (Hashids,), Error = Infallible> + Clone, Error> {
    let guard = Guard::from_manager(manager)?;
    Ok(warp::any().map(move || Hashids(guard.clone())))
}
