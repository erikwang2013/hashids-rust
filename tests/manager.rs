//! HashidsManager 行为：默认连接回落、连接解析与缓存、代理方法、错误路径。

use std::sync::Arc;

use hashids::{Config, ConnectionConfig, Error, HashidsManager};

fn manager() -> HashidsManager {
    HashidsManager::new(
        Config::new()
            .default_connection("main")
            .connection("main", ConnectionConfig::new().salt("this is my salt"))
            .connection(
                "alternative",
                ConnectionConfig::new()
                    .salt("another salt")
                    .min_hash_length(10),
            ),
    )
}

#[test]
fn resolves_named_and_default_connections() {
    let manager = manager();

    let main = manager.connection(None).unwrap();
    assert_eq!(main.decode(&main.encode(&[1, 2, 3])), vec![1, 2, 3]);

    let alt = manager.connection(Some("alternative")).unwrap();
    assert_ne!(alt.encode(&[1, 2, 3]), main.encode(&[1, 2, 3]));
}

#[test]
fn default_falls_back_to_main() {
    // 未设置 default。
    let manager = HashidsManager::new(Config::new().connection("main", ConnectionConfig::new()));
    assert_eq!(manager.default_connection(), "main");
    assert!(manager.encode(&[1, 2, 3]).is_ok());

    // default 为空串。
    let manager = HashidsManager::new(
        Config::new()
            .default_connection("")
            .connection("main", ConnectionConfig::new()),
    );
    assert_eq!(manager.default_connection(), "main");
}

#[test]
fn unknown_connection_is_rejected() {
    let err = manager().connection(Some("missing")).unwrap_err();
    assert_eq!(err, Error::ConnectionNotConfigured("missing".to_owned()));
}

#[test]
fn empty_connection_name_is_rejected() {
    let err = manager().connection(Some("")).unwrap_err();
    assert_eq!(err, Error::EmptyConnectionName);
}

#[test]
fn unconfigured_default_is_rejected_on_use() {
    let manager = HashidsManager::new(Config::new());
    assert_eq!(
        manager.encode(&[1]).unwrap_err(),
        Error::ConnectionNotConfigured("main".to_owned())
    );
}

#[test]
fn connections_are_cached() {
    let manager = manager();
    let first = manager.connection(None).unwrap();
    let second = manager.connection(Some("main")).unwrap();
    assert!(Arc::ptr_eq(&first, &second), "同一连接应复用缓存实例");
}

#[test]
fn set_default_connection_redirects() {
    let mut manager = manager();
    let before = manager.connection(None).unwrap();

    manager.set_default_connection("alternative");
    assert_eq!(manager.default_connection(), "alternative");
    let after = manager.connection(None).unwrap();
    assert!(!Arc::ptr_eq(&before, &after));
}

#[test]
fn with_default_connection_chains() {
    let manager = manager().with_default_connection("alternative");
    assert_eq!(manager.default_connection(), "alternative");
}

#[test]
fn proxies_match_direct_calls() {
    let manager = manager();
    let direct = manager.connection(None).unwrap();

    assert_eq!(
        manager.encode(&[1, 2, 3]).unwrap(),
        direct.encode(&[1, 2, 3])
    );
    assert_eq!(manager.decode(&direct.encode(&[4, 5])).unwrap(), vec![4, 5]);
    assert_eq!(
        manager.encode_hex("deadbeef").unwrap(),
        direct.encode_hex("deadbeef")
    );
    let hex = manager.encode_hex("deadbeef").unwrap();
    assert_eq!(manager.decode_hex(&hex).unwrap(), "deadbeef");
}

#[test]
fn factory_is_exposed() {
    let manager = manager();
    let built = manager
        .factory()
        .make(&ConnectionConfig::new().salt("this is my salt"))
        .unwrap();
    // 与连接 "main"（同 salt/长度）行为一致。
    assert_eq!(
        built.encode(&[1, 2, 3]),
        manager.encode(&[1, 2, 3]).unwrap()
    );
}
