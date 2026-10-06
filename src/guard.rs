//! 原生 Guard（请求守卫）：不依赖任何框架的通用入口。
//!
//! 八个框架适配器（Axum / Actix Web / Rocket / Poem / Salvo / Warp / Bee / e-cat）
//! 全部产出 `Guard`；未适配的框架也可以自己从应用状态里克隆一份。

use std::sync::Arc;

use crate::Error;
use crate::hashids::Hashids;
use crate::manager::HashidsManager;

/// 请求守卫：**接线/提取时**解析默认连接、快速失败；此后编解码不可失败。
///
/// `Clone` 只是两次原子计数，`Send + Sync`，适合每请求克隆。
#[derive(Debug, Clone)]
pub struct Guard {
    manager: Arc<HashidsManager>,
    default: Arc<Hashids>,
}

impl Guard {
    /// 解析默认连接并守卫之；默认连接未配置时返回错误（接线期快速失败）。
    pub fn from_manager(manager: Arc<HashidsManager>) -> Result<Self, Error> {
        let default = manager.connection(None)?;
        Ok(Self { manager, default })
    }

    /// 底层管理器。
    pub fn manager(&self) -> &HashidsManager {
        &self.manager
    }

    /// 默认连接实例。
    pub fn hashids(&self) -> &Arc<Hashids> {
        &self.default
    }

    /// 切换到命名连接（默认连接之外的连接按需构建并缓存）。
    pub fn connection(&self, name: &str) -> Result<Arc<Hashids>, Error> {
        self.manager.connection(Some(name))
    }

    /// 编码（默认连接）。
    pub fn encode(&self, numbers: &[u64]) -> String {
        self.default.encode(numbers)
    }

    /// 解码（默认连接）。
    pub fn decode(&self, hash: &str) -> Vec<u64> {
        self.default.decode(hash)
    }

    /// 十六进制编码（默认连接）。
    pub fn encode_hex(&self, hex: &str) -> String {
        self.default.encode_hex(hex)
    }

    /// 十六进制解码（默认连接）。
    pub fn decode_hex(&self, hash: &str) -> String {
        self.default.decode_hex(hash)
    }
}
