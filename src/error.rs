//! 错误类型。
//!
//! 全库统一的错误类型；文案直接面向调用方排查。

use std::fmt;

/// hashids 统一错误类型。
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// 连接名为空字符串。
    EmptyConnectionName,
    /// 指定名称的连接未在配置中定义（附连接名）。
    ConnectionNotConfigured(String),
    /// 字母表去重后不足 16 个唯一字符。
    AlphabetTooShort,
    /// 字母表包含空格。
    AlphabetWithSpace,
    /// 字母表包含 NUL 字符（U+0000）。
    AlphabetWithNul,
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
            Self::AlphabetWithNul => {
                write!(f, "The Hashids alphabet can't contain the NUL character.")
            }
        }
    }
}

impl std::error::Error for Error {}
