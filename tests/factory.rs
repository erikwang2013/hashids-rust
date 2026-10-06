//! HashidsFactory：由连接配置构建实例。

use hashids::{ConnectionConfig, Error, Hashids, HashidsFactory};

#[test]
fn builds_with_custom_alphabet() {
    let factory = HashidsFactory::new();
    let config = ConnectionConfig::new()
        .salt("this is my salt")
        .min_hash_length(30)
        .alphabet("xzal86grmb4jhysfoqp3we7291kuct5iv0nd");

    let hashids = factory.make(&config).unwrap();
    let id = hashids.encode(&[1, 2, 3]);
    assert_eq!(id, "18apy3wlqkjvd5h1id7mn5ore2d06b"); // 官方 custom 向量
}

#[test]
fn empty_alphabet_falls_back_to_default() {
    let factory = HashidsFactory::new();
    let hashids = factory.make(&ConnectionConfig::new().alphabet("")).unwrap();
    assert_eq!(
        hashids.encode(&[1, 2, 3]),
        Hashids::default().encode(&[1, 2, 3])
    );
}

#[test]
fn absent_alphabet_uses_default() {
    let factory = HashidsFactory::new();
    let hashids = factory.make(&ConnectionConfig::new()).unwrap();
    assert_eq!(hashids.encode(&[1]), "jR"); // 官方 default 向量
}

#[test]
fn invalid_alphabet_propagates_error() {
    let err = HashidsFactory::new()
        .make(&ConnectionConfig::new().alphabet("1234567890"))
        .unwrap_err();
    assert_eq!(err, Error::AlphabetTooShort);
}
