//! 连接工厂：由连接配置构建 [`Hashids`] 实例。

use crate::Error;
use crate::config::ConnectionConfig;
use crate::hashids::{DEFAULT_ALPHABET, Hashids};

/// 工厂：唯一构建 `Hashids` 的地方（对齐 PHP 版 `HashidsFactory`）。
#[derive(Debug, Clone, Copy, Default)]
pub struct HashidsFactory;

impl HashidsFactory {
    pub fn new() -> Self {
        Self
    }

    /// 由连接配置构建实例；`alphabet` 为 `None` 或空串时回落默认字母表
    /// （对齐 PHP：空串视作未设置）。
    pub fn make(&self, config: &ConnectionConfig) -> Result<Hashids, Error> {
        match config.alphabet.as_deref() {
            Some(alphabet) if !alphabet.is_empty() => {
                Hashids::new(&config.salt, config.min_hash_length, alphabet)
            }
            _ => Ok(
                Hashids::new(&config.salt, config.min_hash_length, DEFAULT_ALPHABET)
                    .expect("DEFAULT_ALPHABET 恒为合法字母表"),
            ),
        }
    }
}
