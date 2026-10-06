#![cfg(feature = "bee")]
//! Bee 端到端：原生 Filter 把 Guard 注入 Context，controller 取出后编出官方向量值。

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::Request;
use bee_cache::MemoryCache;
use bee_router::context::RouterError;
use bee_router::{Context, Controller};
use bee_session::Session;
use bee_template::TemplateEngine;
use hashids::integrations::bee::{ContextHashids, HashidsFilter};
use hashids::{Config, ConnectionConfig, Guard, HashidsManager};

struct ShowHashids;

#[async_trait::async_trait]
impl Controller for ShowHashids {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let hash = ctx.hashids().expect("guard 应已注入").encode(&[1, 2, 3]);
        ctx.text(&hash)
    }
}

fn context() -> (Context, Arc<dyn bee_cache::Cache>) {
    let cache: Arc<dyn bee_cache::Cache> = Arc::new(MemoryCache::new());
    let ctx = Context::new(
        Request::builder().body(Body::empty()).unwrap(),
        Session::new(Arc::clone(&cache), Duration::from_secs(3600)),
        Arc::new(TemplateEngine::new(Path::new(".")).unwrap()),
    );
    (ctx, cache)
}

#[tokio::test]
async fn filter_injects_guard_into_context() {
    let manager = Arc::new(HashidsManager::new(
        Config::new().connection("main", ConnectionConfig::new()),
    ));
    let guard = Guard::from_manager(manager).unwrap();
    let filter = HashidsFilter::new(guard);

    let (mut ctx, cache) = context();
    ctx.dispatch(cache, Duration::from_secs(3600), &[&filter], &ShowHashids)
        .await
        .unwrap();

    let response = ctx.into_response();
    let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    assert_eq!(&body[..], b"o2fXhV"); // 官方默认向量
}

#[tokio::test]
async fn without_filter_the_guard_is_absent() {
    let (ctx, _cache) = context();
    assert!(ctx.hashids().is_none());
}
