//! 多连接管理器：连接解析、懒构建缓存、默认连接与代理方法。

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::Error;
use crate::config::Config;
use crate::factory::HashidsFactory;
use crate::hashids::Hashids;

/// 多连接管理器（对齐 PHP 版 `HashidsManager`）。
///
/// - 连接**懒构建**：首次取用时经 [`HashidsFactory`] 构建，之后复用缓存
///   （同一连接 `connection()` 每次返回同一个 `Arc`）。
/// - **配置在构造时快照**：对齐 PHP 版注意事项——改配置需要重建 Manager。
/// - `Send + Sync`：可直接包进 `Arc` 放进任意 Web 框架的应用状态。
#[derive(Debug)]
pub struct HashidsManager {
    config: Config,
    factory: HashidsFactory,
    default_connection: String,
    connections: RwLock<HashMap<String, Arc<Hashids>>>,
}

impl HashidsManager {
    /// 由配置构建。`default` 缺省/为空时按 `"main"` 处理（对齐 vinkla/hashids）。
    pub fn new(config: Config) -> Self {
        let default_connection = match config.default.as_deref() {
            Some(name) if !name.is_empty() => name.to_owned(),
            _ => "main".to_owned(),
        };

        Self {
            config,
            factory: HashidsFactory::new(),
            default_connection,
            connections: RwLock::new(HashMap::new()),
        }
    }

    /// 当前默认连接名。
    pub fn default_connection(&self) -> &str {
        &self.default_connection
    }

    /// 切换默认连接名。注意：`HashidsManager` 放进 `Arc`（各框架适配器都如此）
    /// 之后无法再改，请在包装前调用，或用 [`Self::with_default_connection`] 链式设置。
    pub fn set_default_connection(&mut self, name: impl Into<String>) {
        self.default_connection = name.into();
    }

    /// 链式设置默认连接名（构造期使用）。
    pub fn with_default_connection(mut self, name: impl Into<String>) -> Self {
        self.set_default_connection(name);
        self
    }

    /// 工厂引用。
    pub fn factory(&self) -> &HashidsFactory {
        &self.factory
    }

    /// 取命名连接（`None` 取默认连接）；懒构建 + 缓存。
    pub fn connection(&self, name: Option<&str>) -> Result<Arc<Hashids>, Error> {
        let name = name.unwrap_or_else(|| self.default_connection());
        if name.is_empty() {
            return Err(Error::EmptyConnectionName);
        }

        // 命中缓存直接返回；锁中毒无害（缓存值不可变），恢复后继续。
        {
            let cache = self.connections.read().unwrap_or_else(|e| e.into_inner());
            if let Some(hashids) = cache.get(name) {
                return Ok(Arc::clone(hashids));
            }
        }

        let connection_config = self
            .config
            .connections
            .get(name)
            .ok_or_else(|| Error::ConnectionNotConfigured(name.to_owned()))?;
        let built = Arc::new(self.factory.make(connection_config)?);

        // 不在持写锁时做构建；并发下两个线程构建出的是相同不可变值，先到先得。
        let mut cache = self.connections.write().unwrap_or_else(|e| e.into_inner());
        Ok(Arc::clone(cache.entry(name.to_owned()).or_insert(built)))
    }

    /// 编码（代理到默认连接，对齐 PHP `__call` 转发）。
    pub fn encode(&self, numbers: &[u64]) -> Result<String, Error> {
        Ok(self.connection(None)?.encode(numbers))
    }

    /// 解码（代理到默认连接）。
    pub fn decode(&self, hash: &str) -> Result<Vec<u64>, Error> {
        Ok(self.connection(None)?.decode(hash))
    }

    /// 十六进制编码（代理到默认连接）。
    pub fn encode_hex(&self, hex: &str) -> Result<String, Error> {
        Ok(self.connection(None)?.encode_hex(hex))
    }

    /// 十六进制解码（代理到默认连接）。
    pub fn decode_hex(&self, hash: &str) -> Result<String, Error> {
        Ok(self.connection(None)?.decode_hex(hash))
    }
}
