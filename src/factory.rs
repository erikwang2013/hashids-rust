//! 连接工厂：由连接配置构建 [`Hashids`] 实例。

use crate::Error;
use crate::config::ConnectionConfig;
use crate::hashids::Hashids;

/// 工厂：唯一构建 `Hashids` 的地方。
#[derive(Debug, Clone, Copy, Default)]
pub struct HashidsFactory;

impl HashidsFactory {
    /// 创建工厂（零大小类型）。
    pub fn new() -> Self {
        Self
    }

    /// 由连接配置构建实例；`alphabet` 为 `None` 或空串时回落默认字母表
    /// （空串视作未设置）。
    ///
    /// # Errors
    ///
    /// 仅当配置了自定义字母表且其非法时返回 [`Error`]：
    /// 去重后不足 16 个唯一字符（[`Error::AlphabetTooShort`]）
    /// 或包含空格（[`Error::AlphabetWithSpace`]）。
    pub fn make(&self, config: &ConnectionConfig) -> Result<Hashids, Error> {
        match config.alphabet.as_deref() {
            Some(alphabet) if !alphabet.is_empty() => {
                Hashids::new(&config.salt, config.min_hash_length, alphabet)
            }
            _ => Ok(Hashids::with_default_alphabet(
                &config.salt,
                config.min_hash_length,
            )),
        }
    }
}
