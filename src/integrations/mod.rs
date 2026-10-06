//! Web 框架适配层：只做接线。
//!
//! 每个适配器都是薄封装——把 [`crate::Guard`]（原生请求守卫）接到各框架自己的
//! 状态 / 提取器机制上，不重复核心逻辑。默认零开销：各框架按 feature 开启。
//!
//! | feature | crate | 接线机制 |
//! |---------|-------|----------|
//! | `axum`  | axum 0.8 | `FromRequestParts` 提取器（`FromRef` 模式） |
//! | `actix` | actix-web 4 | `FromRequest` 提取器（`web::Data`） |
//! | `rocket`| rocket 0.5 | `FromRequest` 请求守卫（managed state） |
//! | `poem`  | poem 3 | `FromRequest` 提取器（`EndpointExt::data`） |
//! | `salvo` | salvo 1 | Depot 注入器（`.hoop()`）+ `DepotHashids` 扩展 |
//! | `warp`  | warp 0.4 | Filter 组合（接线期解析、快速失败） |
//! | `bee`   | bee_router 1 | 原生 `Filter` 钩子注入 `Context`（底层即 axum） |
//! | `ecat`  | ecat 4 | axum `Extension` 层（e-cat 的 HTTP 传输就是 axum Router） |

#[cfg(feature = "actix")]
pub mod actix;
#[cfg(feature = "axum")]
pub mod axum;
#[cfg(feature = "bee")]
pub mod bee;
#[cfg(feature = "ecat")]
pub mod ecat;
#[cfg(feature = "poem")]
pub mod poem;
#[cfg(feature = "rocket")]
pub mod rocket;
#[cfg(feature = "salvo")]
pub mod salvo;
#[cfg(feature = "warp")]
pub mod warp;
