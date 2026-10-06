//! 原生 Guard（请求守卫）：接线期快速失败、此后编解码不可失败。

use std::sync::Arc;

use hashids::{Config, ConnectionConfig, Guard, HashidsManager};

fn guard() -> Guard {
    let manager = Arc::new(HashidsManager::new(
        Config::new()
            .default_connection("main")
            .connection("main", ConnectionConfig::new().salt("this is my salt"))
            .connection("alt", ConnectionConfig::new().salt("another salt")),
    ));
    Guard::from_manager(manager).unwrap()
}

#[test]
fn encode_decode_are_infallible() {
    let guard = guard();
    assert_eq!(guard.decode(&guard.encode(&[1, 2, 3])), vec![1, 2, 3]);
    assert_eq!(guard.decode_hex(&guard.encode_hex("deadbeef")), "deadbeef");
    assert_eq!(guard.encode(&[]), "");
    assert_eq!(guard.decode("f"), Vec::<u64>::new());
}

#[test]
fn fails_fast_when_default_connection_missing() {
    let manager = Arc::new(HashidsManager::new(Config::new()));
    assert!(Guard::from_manager(manager).is_err());
}

#[test]
fn switches_to_named_connections() {
    let guard = guard();
    let alt = guard.connection("alt").unwrap();
    assert_ne!(alt.encode(&[1, 2, 3]), guard.encode(&[1, 2, 3]));
    assert!(guard.connection("missing").is_err());
}

#[test]
fn clone_shares_the_same_state() {
    let guard = guard();
    let cloned = guard.clone();
    assert!(Arc::ptr_eq(guard.hashids(), cloned.hashids()));
    assert!(std::ptr::eq(guard.manager(), cloned.manager()));
    assert_eq!(cloned.encode(&[1, 2, 3]), guard.encode(&[1, 2, 3]));
}
