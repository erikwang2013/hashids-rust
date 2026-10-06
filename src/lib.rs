//! **Hashids for Rust** —— 把数据库自增 ID 换成短小、不可猜测的字符串。
//!
//! 内核原生实现 hashids 算法（对齐 [vinkla/hashids](https://github.com/vinkla/hashids)
//! 官方测试向量），配置与用法对齐其「多连接 + 默认连接 + Manager + Factory」风格；
//! 默认**零依赖**，八个 Web 框架适配（Axum / Actix Web / Rocket / Poem / Salvo /
//! Warp / bee-rust / e-cat）按需开启 feature。
//!
//! ```
//! use hashids::{Config, ConnectionConfig, HashidsManager};
//!
//! let config = Config::new()
//!     .default_connection("main")
//!     .connection(
//!         "main",
//!         ConnectionConfig::new().salt("this is my salt").min_hash_length(8),
//!     );
//!
//! let manager = HashidsManager::new(config);
//! let hash = manager.encode(&[1, 2, 3]).unwrap();
//! assert_eq!(manager.decode(&hash).unwrap(), vec![1, 2, 3]);
//! ```
//!
//! 未适配的框架/场景可以直接用 [`Guard`]（原生请求守卫）：把它放进任意框架的
//! 应用状态，每请求克隆即可。
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod error;
pub mod factory;
pub mod guard;
pub mod hashids;
pub mod manager;
pub mod mascot;

#[cfg(any(
    feature = "axum",
    feature = "actix",
    feature = "rocket",
    feature = "poem",
    feature = "salvo",
    feature = "warp",
    feature = "bee",
    feature = "ecat"
))]
pub mod integrations;

pub use config::{Config, ConnectionConfig};
pub use error::Error;
pub use factory::HashidsFactory;
pub use guard::Guard;
pub use hashids::Hashids;
pub use manager::HashidsManager;
