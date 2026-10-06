//! 错误类型。
//!
//! 与 PHP 版 `InvalidArgumentException` 的报错语义对齐；消息文案沿用
//! [vinkla/hashids](https://github.com/vinkla/hashids) 原文，便于对照排查。

use std::fmt;

/// hashids 统一错误类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// 连接名为空字符串。
    EmptyConnectionName,
    /// 指定名称的连接未在配置中定义（附连接名）。
    ConnectionNotConfigured(String),
    /// 字母表去重后不足 16 个唯一字符。
    AlphabetTooShort,
    /// 字母表包含空格。
    AlphabetWithSpace,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyConnectionName => write!(f, "Hashids connection name cannot be empty."),
            Self::ConnectionNotConfigured(name) => {
                write!(f, "Hashids connection [{name}] is not configured.")
            }
            Self::AlphabetTooShort => write!(
                f,
                "The Hashids alphabet must contain at least 16 unique characters."
            ),
            Self::AlphabetWithSpace => {
                write!(f, "The Hashids alphabet can't contain spaces.")
            }
        }
    }
}

impl std::error::Error for Error {}
