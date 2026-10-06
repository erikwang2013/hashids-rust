//! 配置结构。
//!
//! 与 PHP 版的配置数组同构（`default` + `connections`，连接项含
//! `salt` / `length` / `alphabet`），只是从数组变成类型化结构体。

use std::collections::BTreeMap;

/// 单个连接的配置（对应 PHP 配置里的 `connections.<name>` 一项）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectionConfig {
    /// 盐值；留空等于没有保护（编码结果可枚举），上线前务必设置。
    pub salt: String,
    /// 最短哈希长度（PHP 配置键为 `length`）。
    pub min_hash_length: usize,
    /// 可选自定义字母表；`None` 使用默认字母表。
    pub alphabet: Option<String>,
}

impl ConnectionConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置盐值。
    pub fn salt(mut self, salt: impl Into<String>) -> Self {
        self.salt = salt.into();
        self
    }

    /// 设置最短哈希长度。
    pub fn min_hash_length(mut self, min_hash_length: usize) -> Self {
        self.min_hash_length = min_hash_length;
        self
    }

    /// 设置自定义字母表。
    pub fn alphabet(mut self, alphabet: impl Into<String>) -> Self {
        self.alphabet = Some(alphabet.into());
        self
    }
}

/// 多连接配置（对应 PHP 的扁平配置数组：根级 `default` + `connections`）。
///
/// `default` 缺省或为空时按 `"main"` 处理（对齐 vinkla/hashids 的行为）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    /// 默认连接名；`None`/空串回落 `"main"`。
    pub default: Option<String>,
    /// 连接名 => 连接配置。`BTreeMap` 保证迭代顺序确定。
    pub connections: BTreeMap<String, ConnectionConfig>,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置默认连接名。
    pub fn default_connection(mut self, name: impl Into<String>) -> Self {
        self.default = Some(name.into());
        self
    }

    /// 注册一个命名连接（可链式调用多次）。
    pub fn connection(mut self, name: impl Into<String>, config: ConnectionConfig) -> Self {
        self.connections.insert(name.into(), config);
        self
    }
}
