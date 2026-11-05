//! 配置结构。
//!
//! 根级 `default` 选择默认连接，`connections` 表按名字登记各连接；
//! 连接项含 `salt` / `min_hash_length` / `alphabet` 三个字段。

use std::collections::BTreeMap;
use std::fmt;

/// 单个连接的配置（[`Config::connections`] 中的一项）。
#[derive(Clone, Default, PartialEq, Eq)]
pub struct ConnectionConfig {
    /// 盐值；留空等于没有保护（编码结果可枚举），上线前务必设置。
    pub salt: String,
    /// 最短哈希长度。
    pub min_hash_length: usize,
    /// 可选自定义字母表；`None` 使用默认字母表。
    pub alphabet: Option<String>,
}

// 与 `Hashids` 的 `Salt` 一致：Debug 不输出盐值，避免日志泄漏。
impl fmt::Debug for ConnectionConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionConfig")
            .field(
                "salt",
                &format_args!("[redacted; {} chars]", self.salt.chars().count()),
            )
            .field("min_hash_length", &self.min_hash_length)
            .field("alphabet", &self.alphabet)
            .finish()
    }
}

impl ConnectionConfig {
    /// 创建空配置（等价 `Default`）。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置盐值。
    #[must_use]
    pub fn salt(mut self, salt: impl Into<String>) -> Self {
        self.salt = salt.into();
        self
    }

    /// 设置最短哈希长度。
    #[must_use]
    pub fn min_hash_length(mut self, min_hash_length: usize) -> Self {
        self.min_hash_length = min_hash_length;
        self
    }

    /// 设置自定义字母表。
    #[must_use]
    pub fn alphabet(mut self, alphabet: impl Into<String>) -> Self {
        self.alphabet = Some(alphabet.into());
        self
    }
}

/// 多连接配置：根级 `default` + `connections`。
///
/// `default` 缺省或为空时按 `"main"` 处理。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    /// 默认连接名；`None`/空串回落 `"main"`。
    pub default: Option<String>,
    /// 连接名 => 连接配置。`BTreeMap` 保证迭代顺序确定。
    pub connections: BTreeMap<String, ConnectionConfig>,
}

impl Config {
    /// 创建空配置（等价 `Default`）。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置默认连接名。
    #[must_use]
    pub fn default_connection(mut self, name: impl Into<String>) -> Self {
        self.default = Some(name.into());
        self
    }

    /// 注册一个命名连接（可链式调用多次）。
    #[must_use]
    pub fn connection(mut self, name: impl Into<String>, config: ConnectionConfig) -> Self {
        self.connections.insert(name.into(), config);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_redacts_salt() {
        let config =
            Config::new().connection("main", ConnectionConfig::new().salt("super-secret-salt"));
        let debug = format!("{config:?}");
        assert!(!debug.contains("super-secret-salt"));
        assert!(debug.contains("redacted"));
    }
}
