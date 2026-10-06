//! Hashids 内核算法：把整数编码为短小、不可猜测的字符串，反之亦然。
//!
//! 逐行对齐 [vinkla/hashids](https://github.com/vinkla/hashids)（PHP v5.0.2）
//! 的行为语义，包括多字节字母表、缺失字符按位置 0 处理、`decode` 末尾的
//! 往返校验。与 PHP 版的两处差异：
//!
//! - 数字范围是 `u64`（PHP 经 bcmath/gmp 支持任意精度；官方测试向量全部落在 u64 内）；
//! - `shuffle` 结果不缓存（纯函数，重复计算不影响输出，省掉一份可变状态）。

use crate::Error;

/// 默认字母表（62 字符，与官方实现一致）。
pub const DEFAULT_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";

/// 官方分隔符字符集。
pub const DEFAULT_SEPS: &str = "cfhistuCFHISTU";

/// guards 占比除数：`guardCount = ceil(alphabetLen / GUARD_DIV)`。
const GUARD_DIV: usize = 12;

/// seps 占比除数（PHP 为浮点 3.5）。
const SEP_DIV: f64 = 3.5;

/// 一个已配置好字母表的 Hashids 实例。
#[derive(Debug, Clone)]
pub struct Hashids {
    salt: String,
    min_hash_length: usize,
    alphabet: Vec<char>,
    seps: Vec<char>,
    guards: Vec<char>,
}

impl Hashids {
    /// 由 salt / 最短长度 / 字母表构建实例。
    ///
    /// 字母表先去除重复字符（保留首次出现），再校验：
    /// 至少 16 个唯一字符、不得包含空格——否则返回 [`Error`]。
    pub fn new(salt: &str, min_hash_length: usize, alphabet: &str) -> Result<Self, Error> {
        let mut alphabet = dedup_chars(alphabet);
        if alphabet.len() < 16 {
            return Err(Error::AlphabetTooShort);
        }
        if alphabet.contains(&' ') {
            return Err(Error::AlphabetWithSpace);
        }

        // 对齐 PHP：array_intersect 保持 seps 的顺序；array_diff 保持字母表顺序。
        let seps_set = dedup_chars(DEFAULT_SEPS);
        let mut seps: Vec<char> = seps_set
            .iter()
            .copied()
            .filter(|c| alphabet.contains(c))
            .collect();
        alphabet.retain(|c| !seps_set.contains(c));

        seps = shuffle(&seps, salt);

        if seps.is_empty() || (alphabet.len() as f64 / seps.len() as f64) > SEP_DIV {
            let seps_length = (alphabet.len() as f64 / SEP_DIV).ceil() as usize;
            if seps_length > seps.len() {
                let diff = seps_length - seps.len();
                seps.extend(alphabet.iter().take(diff));
                alphabet.drain(..alphabet.len().min(diff));
            }
        }

        alphabet = shuffle(&alphabet, salt);
        let guard_count = alphabet.len().div_ceil(GUARD_DIV);

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
            salt: salt.to_owned(),
            min_hash_length,
            alphabet,
            seps,
            guards,
        })
    }

    /// 用默认字母表构建实例（等价于 PHP `new Hashids($salt, $length)`）。
    pub fn with_default_alphabet(salt: &str, min_hash_length: usize) -> Self {
        Self::new(salt, min_hash_length, DEFAULT_ALPHABET).expect("DEFAULT_ALPHABET 恒为合法字母表")
    }

    /// 编码一组数字。空输入返回空字符串（对齐 PHP）。
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
            let last = hash(number, &alphabet);
            ret.push_str(&last);

            if i + 1 < numbers_size {
                number %= last.chars().next().map_or(0, |c| c as u64) + i as u64;
                let seps_index = (number % self.seps.len() as u64) as usize;
                ret.push(self.seps[seps_index]);
            }
        }

        if self.char_len(&ret) < self.min_hash_length {
            let guard_index = (numbers_hash_int + ret.chars().next().map_or(0, |c| c as u64))
                % self.guards.len() as u64;
            ret.insert(0, self.guards[guard_index as usize]);

            if self.char_len(&ret) < self.min_hash_length {
                let guard_index = (numbers_hash_int + ret.chars().nth(2).map_or(0, |c| c as u64))
                    % self.guards.len() as u64;
                ret.push(self.guards[guard_index as usize]);
            }
        }

        let half_length = alphabet.len() / 2;
        while self.char_len(&ret) < self.min_hash_length {
            alphabet = shuffle(&alphabet, &alphabet.iter().collect::<String>());
            let tail: String = alphabet[half_length..].iter().collect();
            let head: String = alphabet[..half_length].iter().collect();
            ret = tail + &ret + &head;

            let excess = self.char_len(&ret).saturating_sub(self.min_hash_length);
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

    /// 解码哈希串。非法输入返回空数组（对齐 PHP；供解密端做往返校验）。
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
        let index = if parts.len() == 3 || parts.len() == 2 {
            1
        } else {
            0
        };
        let breakdown = parts[index];
        if breakdown.is_empty() {
            return Vec::new();
        }

        let mut chars = breakdown.chars();
        let lottery = chars.next().expect("breakdown 非空");
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
                // 超出 u64 的哈希视为非法（PHP 走大整数后会被往返校验拒绝）。
                Some(number) => ret.push(number),
                None => return Vec::new(),
            }
        }

        if self.encode(&ret) != hash {
            ret.clear();
        }

        ret
    }

    /// 十六进制字符串编码。非法十六进制输入返回空字符串（对齐 PHP）。
    pub fn encode_hex(&self, hex: &str) -> String {
        if hex.is_empty() || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return String::new();
        }

        let numbers: Vec<u64> = hex
            .as_bytes()
            .chunks(12)
            .map(|chunk| {
                let text = std::str::from_utf8(chunk).expect("十六进制为 ASCII");
                u64::from_str_radix(&format!("1{text}"), 16)
                    .expect("最多 13 位十六进制，必落在 u64")
            })
            .collect();

        self.encode(&numbers)
    }

    /// 十六进制解码。非法哈希返回空字符串（对齐 PHP）。
    pub fn decode_hex(&self, hash: &str) -> String {
        let mut ret = String::new();
        for number in self.decode(hash) {
            let hex = format!("{number:x}");
            // PHP: substr(dechex($number), 1) —— 无条件去掉首字符（编码时强制补的 '1'）。
            ret.push_str(&hex[1..]);
        }
        ret
    }

    /// `lottery . salt . alphabet` 的前 `alphabet.len()` 个字符（PHP 同名逻辑）。
    fn encode_salt(&self, lottery: char, alphabet: &[char]) -> String {
        let mut salt = String::new();
        salt.push(lottery);
        salt.push_str(&self.salt);
        salt.extend(alphabet.iter());
        salt.chars().take(alphabet.len()).collect()
    }

    /// 按 Unicode 字符数计长（对齐 PHP 的 `mb_strlen`）。
    fn char_len(&self, text: &str) -> usize {
        text.chars().count()
    }
}

impl Default for Hashids {
    fn default() -> Self {
        Self::with_default_alphabet("", 0)
    }
}

/// 去除重复字符，保留首次出现的顺序（对齐 PHP `array_unique`）。
fn dedup_chars(input: &str) -> Vec<char> {
    let mut out: Vec<char> = Vec::new();
    for c in input.chars() {
        if !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

/// 按盐打乱字符表（对齐 PHP `shuffle`；`salt` 为空时原样返回）。
fn shuffle(alphabet: &[char], salt: &str) -> Vec<char> {
    if salt.is_empty() || alphabet.is_empty() {
        return alphabet.to_vec();
    }

    let salt_chars: Vec<char> = salt.chars().collect();
    let salt_length = salt_chars.len();
    let mut out = alphabet.to_vec();

    let mut i = out.len() - 1;
    let mut v: usize = 0;
    let mut p: u64 = 0;
    while i > 0 {
        v %= salt_length;
        let code = salt_chars[v] as u64;
        p += code;
        let j = ((code + v as u64 + p) % i as u64) as usize;
        out.swap(j, i);
        i -= 1;
        v += 1;
    }

    out
}

/// 把数字哈希进给定字母表（至少 1 个字符）。
fn hash(mut input: u64, alphabet: &[char]) -> String {
    let length = alphabet.len() as u64;
    let mut out: Vec<char> = Vec::new();
    loop {
        out.push(alphabet[(input % length) as usize]);
        input /= length;
        if input == 0 {
            break;
        }
    }
    out.reverse();
    out.into_iter().collect()
}

/// 从给定字母表还原数字。字符不在字母表内时按位置 0 处理（对齐 PHP
/// `mb_strpos` 返回 false 被隐式转 0 的行为）；溢出 u64 返回 `None`。
fn unhash(input: &[char], alphabet: &[char]) -> Option<u64> {
    if input.is_empty() || alphabet.is_empty() {
        return Some(0);
    }

    let length = alphabet.len() as u64;
    let mut number: u64 = 0;
    for ch in input {
        let position = alphabet.iter().position(|c| c == ch).unwrap_or(0) as u64;
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
