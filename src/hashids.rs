//! Hashids 内核算法：把整数编码为短小、不可猜测的字符串，反之亦然。
//!
//! 实现说明：
//!
//! - 数字范围是 `u64`；超出 u64 的哈希在解码时按非法输入处理（返回空数组）；
//! - 字母表按 Unicode 字符（码点）处理，支持多字节字符；字符不在字母表内按位置 0 处理；
//! - `decode` 末尾做一次往返校验（重新编码并与输入比对），不一致返回空数组；
//! - `shuffle` 结果不缓存（纯函数，重复计算不影响输出，省掉一份可变状态）。

use std::fmt::{self, Write};

use crate::Error;

/// 默认字母表（62 字符）。
pub const DEFAULT_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";

/// 默认分隔符字符集。
pub const DEFAULT_SEPS: &str = "cfhistuCFHISTU";

/// guards 占比除数：`guardCount = ceil(alphabetLen / GUARD_DIV)`。
const GUARD_DIV: usize = 12;

/// seps 占比除数（浮点 3.5）。
const SEP_DIV: f64 = 3.5;

/// 一个已配置好字母表的 Hashids 实例。
#[derive(Debug, Clone)]
pub struct Hashids {
    salt: Salt,
    min_hash_length: usize,
    alphabet: Vec<char>,
    seps: Vec<char>,
    guards: Vec<char>,
}

/// 盐字符串包装：`Debug` 只输出长度，避免日志与断言输出泄漏盐。
#[derive(Clone)]
struct Salt(String);

impl fmt::Debug for Salt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[redacted; {} chars]", self.0.chars().count())
    }
}

impl Hashids {
    /// 由 salt / 最短长度 / 字母表构建实例。
    ///
    /// 字母表先去除重复字符（保留首次出现），再校验：至少 16 个唯一字符、
    /// 不含空格、不含 NUL 字符。`min_hash_length` 由调用方保证取值合理：
    /// padding 循环会按所需长度真实分配并扩展，没有人为上限。
    ///
    /// # Errors
    ///
    /// - [`Error::AlphabetTooShort`]：去重后不足 16 个唯一字符；
    /// - [`Error::AlphabetWithSpace`]：字母表含空格；
    /// - [`Error::AlphabetWithNul`]：字母表含 NUL 字符（内部取模会除零）。
    pub fn new(salt: &str, min_hash_length: usize, alphabet: &str) -> Result<Self, Error> {
        let mut alphabet = dedup_chars(alphabet);
        if alphabet.len() < 16 {
            return Err(Error::AlphabetTooShort);
        }
        if alphabet.contains(&' ') {
            return Err(Error::AlphabetWithSpace);
        }
        if alphabet.contains(&'\0') {
            return Err(Error::AlphabetWithNul);
        }

        // seps 保持其自身顺序；字母表保留其余字符的原顺序。
        let seps_set = dedup_chars(DEFAULT_SEPS);
        let mut seps: Vec<char> = seps_set
            .iter()
            .copied()
            .filter(|c| alphabet.contains(c))
            .collect();
        alphabet.retain(|c| !seps_set.contains(c));

        let salt_chars: Vec<char> = salt.chars().collect();
        seps = shuffle(&seps, &salt_chars);

        if seps.is_empty() || (alphabet.len() as f64 / seps.len() as f64) > SEP_DIV {
            let seps_length = (alphabet.len() as f64 / SEP_DIV).ceil() as usize;
            if seps_length > seps.len() {
                let diff = seps_length - seps.len();
                seps.extend(alphabet.iter().take(diff));
                alphabet.drain(..alphabet.len().min(diff));
            }
        }

        alphabet = shuffle(&alphabet, &salt_chars);
        let guard_count = alphabet.len().div_ceil(GUARD_DIV);

        // 字母表过短（< 3 个字符）时，guards 从 seps 借。
        let (guards, alphabet, seps) = if alphabet.len() < 3 {
            (
                seps.iter().take(guard_count).copied().collect(),
                alphabet,
                seps.iter().skip(guard_count).copied().collect(),
            )
        } else {
            (
                alphabet.iter().take(guard_count).copied().collect(),
                alphabet.iter().skip(guard_count).copied().collect(),
                seps,
            )
        };

        Ok(Self {
            salt: Salt(salt.to_owned()),
            min_hash_length,
            alphabet,
            seps,
            guards,
        })
    }

    /// 用默认字母表构建实例。
    #[must_use]
    pub fn with_default_alphabet(salt: &str, min_hash_length: usize) -> Self {
        Self::new(salt, min_hash_length, DEFAULT_ALPHABET).expect("DEFAULT_ALPHABET 恒为合法字母表")
    }

    /// 编码一组数字。空输入返回空字符串。
    #[must_use]
    pub fn encode(&self, numbers: &[u64]) -> String {
        if numbers.is_empty() {
            return String::new();
        }

        let mut alphabet = self.alphabet.clone();
        let numbers_size = numbers.len();
        let mut numbers_hash_int: u64 = 0;
        for (i, number) in numbers.iter().enumerate() {
            numbers_hash_int += number % (i as u64 + 100);
        }

        let lottery = alphabet[(numbers_hash_int % alphabet.len() as u64) as usize];
        let mut ret = String::new();
        ret.push(lottery);

        for (i, number) in numbers.iter().enumerate() {
            let mut number = *number;
            alphabet = shuffle(&alphabet, &self.encode_salt(lottery, &alphabet));
            // 记录本段哈希的起始偏移，供下方取首字符。
            let start = ret.len();
            hash_into(&mut ret, number, &alphabet);

            if i + 1 < numbers_size {
                number %= ret[start..].chars().next().map_or(0, |c| c as u64) + i as u64;
                let seps_index = (number % self.seps.len() as u64) as usize;
                ret.push(self.seps[seps_index]);
            }
        }

        if char_len(&ret) < self.min_hash_length {
            let guard_index = (numbers_hash_int + ret.chars().next().map_or(0, |c| c as u64))
                % self.guards.len() as u64;
            ret.insert(0, self.guards[guard_index as usize]);

            if char_len(&ret) < self.min_hash_length {
                let guard_index = (numbers_hash_int + ret.chars().nth(2).map_or(0, |c| c as u64))
                    % self.guards.len() as u64;
                ret.push(self.guards[guard_index as usize]);
            }
        }

        // padding：逐轮把字母表后半 + 原串 + 前半拼回，直到达到最短长度。
        let half_length = alphabet.len() / 2;
        while char_len(&ret) < self.min_hash_length {
            // 盐即字母表本身。
            alphabet = shuffle(&alphabet, &alphabet);
            // 单次按字符拼接（容量按字节算：字母表字节数 + 原串字节数）。
            let byte_len = alphabet.iter().map(|c| c.len_utf8()).sum::<usize>();
            let mut next = String::with_capacity(ret.len() + byte_len);
            next.extend(alphabet[half_length..].iter());
            next.push_str(&ret);
            next.extend(alphabet[..half_length].iter());
            ret = next;

            let excess = char_len(&ret).saturating_sub(self.min_hash_length);
            if excess > 0 {
                ret = ret
                    .chars()
                    .skip(excess / 2)
                    .take(self.min_hash_length)
                    .collect();
            }
        }

        ret
    }

    /// 解码哈希串。
    ///
    /// 解码前忽略首尾 Unicode 空白（[`str::trim`]）。数值域为 `u64`：超出该范围的
    /// 哈希按非法输入返回空数组；往返校验（重新编码比对）不通过时同样返回空数组。
    #[must_use]
    pub fn decode(&self, hash: &str) -> Vec<u64> {
        let hash = hash.trim();
        if hash.is_empty() {
            return Vec::new();
        }

        let mut alphabet = self.alphabet.clone();

        let breakdown: String = hash
            .chars()
            .map(|c| if self.guards.contains(&c) { ' ' } else { c })
            .collect();
        let parts: Vec<&str> = breakdown.split(' ').collect();
        let index = usize::from(matches!(parts.len(), 2 | 3));
        let breakdown = parts[index];
        if breakdown.is_empty() {
            return Vec::new();
        }

        let mut chars = breakdown.chars();
        let Some(lottery) = chars.next() else {
            return Vec::new();
        };
        let rest: String = chars.collect();
        let rest: String = rest
            .chars()
            .map(|c| if self.seps.contains(&c) { ' ' } else { c })
            .collect();

        let mut ret: Vec<u64> = Vec::new();
        for sub_hash in rest.split(' ') {
            alphabet = shuffle(&alphabet, &self.encode_salt(lottery, &alphabet));
            let sub: Vec<char> = sub_hash.chars().collect();
            match unhash(&sub, &alphabet) {
                // 超出 u64 的哈希视为非法。
                Some(number) => ret.push(number),
                None => return Vec::new(),
            }
        }

        if self.encode(&ret) != hash {
            ret.clear();
        }

        ret
    }

    /// 编码十六进制字符串。非法输入（为空或含非十六进制字符）返回空字符串。
    #[must_use]
    pub fn encode_hex(&self, hex: &str) -> String {
        if hex.is_empty() || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return String::new();
        }

        let numbers: Vec<u64> = hex
            .as_bytes()
            .chunks(12)
            .map(|chunk| {
                let text = std::str::from_utf8(chunk).expect("十六进制为 ASCII");
                // 等价于按十六进制解析 "1{text}"：补的高位 1 加文本自身的值。
                u64::from_str_radix(text, 16).expect("已验证为十六进制")
                    + 16u64.pow(text.len() as u32)
            })
            .collect();

        self.encode(&numbers)
    }

    /// 解码十六进制。非法哈希返回空字符串。
    #[must_use]
    pub fn decode_hex(&self, hash: &str) -> String {
        let mut ret = String::new();
        for number in self.decode(hash) {
            // 每个数字的十六进制表示去掉首字符（编码时强制补的 '1' 位）。
            let start = ret.len();
            write!(ret, "{number:x}").expect("写入 String 不会失败");
            ret.drain(start..start + 1);
        }
        ret
    }

    /// `lottery . salt . alphabet` 的前 `alphabet.len()` 个字符。
    fn encode_salt(&self, lottery: char, alphabet: &[char]) -> Vec<char> {
        let mut salt: Vec<char> = Vec::with_capacity(alphabet.len());
        salt.push(lottery);
        salt.extend(
            self.salt
                .0
                .chars()
                .chain(alphabet.iter().copied())
                .take(alphabet.len().saturating_sub(1)),
        );
        salt
    }
}

impl Default for Hashids {
    fn default() -> Self {
        Self::with_default_alphabet("", 0)
    }
}

/// 按 Unicode 字符数计长（不是字节数）。
fn char_len(text: &str) -> usize {
    text.chars().count()
}

/// 去除重复字符，保留首次出现的顺序。
fn dedup_chars(input: &str) -> Vec<char> {
    let mut out: Vec<char> = Vec::new();
    for c in input.chars() {
        if !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

/// 按盐打乱字符表（`salt` 为空时原样返回）。
fn shuffle(alphabet: &[char], salt: &[char]) -> Vec<char> {
    if salt.is_empty() || alphabet.is_empty() {
        return alphabet.to_vec();
    }

    let salt_length = salt.len();
    let mut out = alphabet.to_vec();

    let mut i = out.len() - 1;
    let mut v: usize = 0;
    let mut p: u64 = 0;
    while i > 0 {
        let code = salt[v] as u64;
        p += code;
        let j = ((code + v as u64 + p) % i as u64) as usize;
        out.swap(j, i);
        i -= 1;
        // v 每轮只自增 1，达到盐长即归零（等价于取模）。
        v += 1;
        if v == salt_length {
            v = 0;
        }
    }

    out
}

/// 把数字哈希进给定字母表（至少 1 个字符），字符追加写入 `buf`。
fn hash_into(buf: &mut String, mut input: u64, alphabet: &[char]) {
    let length = alphabet.len() as u64;
    // 字母表最短为 2 个字符，u64 最多产生 64 位数字。
    let mut digits = ['\0'; 64];
    let mut n = 0;
    loop {
        digits[n] = alphabet[(input % length) as usize];
        n += 1;
        input /= length;
        if input == 0 {
            break;
        }
    }
    for &c in digits[..n].iter().rev() {
        buf.push(c);
    }
}

/// 从给定字母表还原数字。字符不在字母表内时按位置 0 处理；溢出 u64 返回 `None`。
fn unhash(input: &[char], alphabet: &[char]) -> Option<u64> {
    if input.is_empty() || alphabet.is_empty() {
        return Some(0);
    }

    // ASCII 查表（值域 u32：字母表长度无上限），非 ASCII 字符回退线性扫描。
    let mut ascii = [u32::MAX; 128];
    for (position, &c) in alphabet.iter().enumerate() {
        let c = c as u32;
        if c < 128 && ascii[c as usize] == u32::MAX {
            ascii[c as usize] = position as u32;
        }
    }

    let length = alphabet.len() as u64;
    let mut number: u64 = 0;
    for &ch in input {
        let position = if (ch as u32) < 128 {
            let p = ascii[ch as usize];
            if p == u32::MAX { 0 } else { p as u64 }
        } else {
            alphabet.iter().position(|&c| c == ch).unwrap_or(0) as u64
        };
        number = number.checked_mul(length)?.checked_add(position)?;
    }

    Some(number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_alphabet_is_valid() {
        assert!(Hashids::new("", 0, DEFAULT_ALPHABET).is_ok());
    }

    #[test]
    fn alphabet_must_have_16_unique_chars() {
        assert_eq!(
            Hashids::new("", 0, "1234567890").unwrap_err(),
            Error::AlphabetTooShort
        );
        // 15 个唯一字符 + 重复字符仍是 15。
        assert_eq!(
            Hashids::new("", 0, "1234567890abcdeeeeeeeeeee").unwrap_err(),
            Error::AlphabetTooShort
        );
    }

    #[test]
    fn alphabet_must_not_contain_spaces() {
        assert_eq!(
            Hashids::new("", 0, "a cdefghijklmnopqrstuvwxyz").unwrap_err(),
            Error::AlphabetWithSpace
        );
    }

    #[test]
    fn alphabet_must_not_contain_nul() {
        assert_eq!(
            Hashids::new("", 0, "abcdefghijklmno\0").unwrap_err(),
            Error::AlphabetWithNul
        );
    }

    #[test]
    fn encodes_default_vectors() {
        let hashids = Hashids::default();
        assert_eq!(hashids.encode(&[0]), "gY");
        assert_eq!(hashids.encode(&[1, 2, 3]), "o2fXhV");
        assert_eq!(hashids.decode("o2fXhV"), vec![1, 2, 3]);
        assert_eq!(hashids.decode("f"), Vec::<u64>::new());
        assert_eq!(hashids.decode(""), Vec::<u64>::new());
    }

    #[test]
    fn bad_input_returns_sentinels() {
        let hashids = Hashids::default();
        assert_eq!(hashids.encode(&[]), "");
        assert_eq!(hashids.encode_hex("z"), "");
        assert_eq!(hashids.decode_hex("f"), "");
    }
}
