//! Salvo 集成：Depot 注入器。
//!
//! Salvo 没有通用的提取器 trait，惯用做法是 **Depot**：把
//! [`HashidsInjector`] 作为中间件挂到 Router 上（`.hoop(...)`），每个请求把
//! [`Guard`] 注入 Depot；handler 里用 [`DepotHashids::hashids`] 取。
//!
//! ```
//! use std::sync::Arc;
//!
//! use hashids::integrations::salvo::HashidsInjector;
//! use hashids::{Config, ConnectionConfig, HashidsManager};
//! use salvo::Router;
//!
//! let manager = Arc::new(HashidsManager::new(
//!     Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
//! ));
//!
//! let router = Router::new().hoop(HashidsInjector::new(manager));
//! ```

use std::sync::Arc;

use salvo::handler::Handler;
use salvo::http::StatusCode;
use salvo::http::errors::StatusError;
use salvo::routing::FlowCtrl;
use salvo::writing::Text;
use salvo::{Depot, Request, Response, async_trait};

use crate::{Guard, HashidsManager};

/// 请求作用域守卫（Depot 扩展 trait 的返回值）。
#[derive(Debug, Clone)]
pub struct Hashids(pub Guard);

/// 注入器：挂到 Router 上（`.hoop(HashidsInjector::new(manager))`），
/// 每个请求把默认连接的 [`Guard`] 注入 Depot。
pub struct HashidsInjector {
    manager: Arc<HashidsManager>,
}

impl HashidsInjector {
    /// 创建注入器；每请求解析默认连接并注入 Depot。
    pub fn new(manager: Arc<HashidsManager>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl Handler for HashidsInjector {
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        match Guard::from_manager(Arc::clone(&self.manager)) {
            Ok(guard) => {
                depot.insert_typed(guard);
                ctrl.call_next(req, depot, res).await;
            }
            Err(err) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Text::Plain(err.to_string()));
                ctrl.skip_rest();
            }
        }
    }
}

/// Depot 扩展：取默认连接的守卫。
pub trait DepotHashids {
    /// 取默认连接的守卫；未注入时返回 500（internal server error）。
    fn hashids(&self) -> Result<Hashids, StatusError>;
}

impl DepotHashids for Depot {
    fn hashids(&self) -> Result<Hashids, StatusError> {
        self.get_typed::<Guard>()
            .ok()
            .cloned()
            .map(Hashids)
            .ok_or_else(StatusError::internal_server_error)
    }
}
