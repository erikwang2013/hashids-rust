//! 加固测试：随机环回属性、Send/Sync 承诺、并发访问。
//!
//! 官方向量覆盖已知行为；这里补的是「任意输入不炸、编解码互逆」的覆盖面。

use std::sync::Arc;

use hashids::hashids::DEFAULT_ALPHABET;
use hashids::{Config, ConnectionConfig, Error, Guard, Hashids, HashidsManager};

/// 零依赖确定性伪随机数（LCG），保证失败可复现。
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound.max(1)
    }
}

const SALTS: &[&str] = &["", "this is my salt", "   ", "世界", "!áàã eéè", "0"];
const ALPHABETS: &[&str] = &[
    DEFAULT_ALPHABET,
    "cCsSfFhHuUiItT01",
    "abdegjklmnopqrvwxyzABDEGJKLMNOPQRVWXYZ1234567890",
    "áàãăâeéèêiíìĩoóòõôơuúùũưyýỳđ",
];

#[test]
fn random_round_trips() {
    let mut rng = Rng(0x00C0_FFEE);
    for case in 0..3000 {
        let salt = SALTS[rng.below(SALTS.len() as u64) as usize];
        let alphabet = ALPHABETS[rng.below(ALPHABETS.len() as u64) as usize];
        let min_length = rng.below(48) as usize;
        let count = 1 + rng.below(8) as usize;

        let mut numbers = Vec::with_capacity(count);
        for _ in 0..count {
            numbers.push(match rng.below(4) {
                0 => rng.next(),
                1 => 0,
                2 => u64::MAX,
                _ => rng.below(1_000_000_000_000),
            });
        }

        let hashids = Hashids::new(salt, min_length, alphabet).unwrap();
        let encoded = hashids.encode(&numbers);
        assert_eq!(
            hashids.decode(&encoded),
            numbers,
            "case {case}: salt={salt:?} min={min_length} numbers={numbers:?} hash={encoded:?}"
        );
        if min_length > 0 {
            assert!(encoded.chars().count() >= min_length, "case {case}");
        }
    }
}

#[test]
fn random_hex_round_trips() {
    let mut rng = Rng(0xDEAD_BEEF);
    let hashids = Hashids::new("this is my salt", 0, DEFAULT_ALPHABET).unwrap();
    const HEX: &[u8] = b"0123456789abcdef";

    for case in 0..500 {
        let length = 1 + rng.below(128) as usize;
        let hex: String = (0..length)
            .map(|_| HEX[rng.below(16) as usize] as char)
            .collect();

        let encoded = hashids.encode_hex(&hex);
        assert_eq!(
            hashids.decode_hex(&encoded),
            hex,
            "case {case}: hex={hex:?}"
        );
    }
}

#[test]
fn core_types_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Hashids>();
    assert_send_sync::<HashidsManager>();
    assert_send_sync::<Guard>();
    assert_send_sync::<Config>();
    assert_send_sync::<Error>();
}

#[test]
fn concurrent_access_is_safe() {
    let manager = Arc::new(HashidsManager::new(
        Config::new()
            .connection("main", ConnectionConfig::new().salt("this is my salt"))
            .connection("alt", ConnectionConfig::new().salt("other salt")),
    ));

    // 8 个线程并发首取连接，覆盖 RwLock 懒构建的竞争路径。
    let handles: Vec<_> = (0..8u64)
        .map(|thread| {
            let manager = Arc::clone(&manager);
            std::thread::spawn(move || {
                for number in 0..200 {
                    let hash = manager.encode(&[number]).unwrap();
                    assert_eq!(manager.decode(&hash).unwrap(), vec![number]);
                    if thread % 2 == 0 {
                        let alt = manager.connection(Some("alt")).unwrap();
                        assert_eq!(alt.decode(&alt.encode(&[number])), vec![number]);
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
