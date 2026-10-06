# erikwang2013/hashids-rust

[![Test](https://github.com/erikwang2013/hashids-rust/actions/workflows/test.yml/badge.svg)](https://github.com/erikwang2013/hashids-rust/actions/workflows/test.yml)
[![Release](https://img.shields.io/github/v/release/erikwang2013/hashids-rust)](https://github.com/erikwang2013/hashids-rust/releases)
![MSRV](https://img.shields.io/badge/MSRV-1.85-blue)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

<p align="center">
  <img src="./docs/mascot.svg" alt="哈希迪 Hashy — erikwang2013/hashids-rust 项目宠物" width="200" />
</p>

<p align="center"><strong>哈希迪 Hashy</strong> — 项目宠物，胸口的 <code>#</code> 是它的招牌</p>

把数据库自增 ID 换成短小、不可猜测的字符串，**一套 API 同时跑在 Axum、Actix Web、Rocket、Poem、Salvo、Warp、Bee、e-cat 上**。

内核原生实现 hashids 算法（对齐 [vinkla/hashids](https://github.com/vinkla/hashids) 的官方测试向量）；配置与用法对齐 [**vinkla/hashids**](https://github.com/vinkla/laravel-hashids)（多连接、默认连接、`HashidsManager` + 工厂），迁移成本低。默认 feature 下**零第三方依赖**。

## 项目说明

**Hashids** 是一款短 ID 生成器，可将数字 ID（如数据库主键）编码为短小、唯一且不可猜测的字符串。它不同于 UUID 或雪花 ID——Hashids 更适合用于面向用户的场景（URL、分享码、订单号等），在保持短小可读的同时隐藏原始数字。

本包 `erikwang2013/hashids-rust` 是 Hashids 的 **Rust 原生实现与多框架集成层**，在设计上参考并对齐了 [vinkla/hashids](https://github.com/vinkla/laravel-hashids) 的 API 风格（多连接、默认连接、Manager + Factory 模式），并适配了 Rust 生态常用的 Web 框架。

**核心特性：**

- **多框架兼容**：同一套 API 同时支持 Axum、Actix Web、Rocket、Poem、Salvo、Warp、Bee、e-cat；未适配的框架用原生 `Guard` 即可接入，迁移成本极低。
- **多连接支持**：一个应用可同时配置多套 salt/length 组合（如用户 ID 与订单 ID 使用不同盐值），通过 `connection(Some("xxx"))` 切换。
- **无框架依赖**：不依赖任何特定框架即可独立使用，直接 `HashidsManager::new(config)` 即可工作；默认 feature 下零第三方依赖。
- **对齐 vinkla/hashids**：`Config`、连接配置（`salt` / `min_hash_length` / `alphabet`）与 `HashidsManager` + `HashidsFactory` 的结构与 vinkla/hashids 一致，语义可平滑迁移。
- **框架原生风格**：各框架集成遵循各自的惯用法——Axum 用提取器、Actix Web 用 `FromRequest`、Rocket 用请求守卫、Poem 用提取器、Salvo 用 Depot 注入器、Warp 用 Filter、Bee 用原生 Filter 钩子、e-cat 用 Extension 层。

**适用场景：**

| 场景 | 说明 |
|------|------|
| 隐藏数据库自增 ID | 将 `user_id=100` 映射为 `/user/3kTMd`，避免暴露业务规模 |
| 生成短链接/分享码 | 比 UUID 更短，比随机字符串可控 |
| 订单号/流水号 | 可读性好，便于客服沟通与日志排查 |
| 多租户/多模块隔离 | 不同连接使用不同 Salt，确保编码空间相互独立 |

**注意事项：**

- Hashids 是 **编码（encode/decode）而非加密**。Salt 仅增加猜测难度，不可用于安全敏感场景（如 token、密码）。
- 一旦上线后修改 Salt 或 Length，所有已编码的 ID 将变为无效，请提前规划并固定配置。
- **Salt 留空等于没有保护**：空 salt 下编码结果可枚举（`encode(1)`、`encode(2)`… 顺序可预测）。空 salt 既不报错也不告警，只是静默降级 —— 上线前请确认盐已设置。
- **配置在构造时快照**：`HashidsManager` 构造后配置固定（`set_default_connection` 也须在包进 `Arc` 之前调用），改配置需要重建 Manager —— 常驻服务同理，重建后生效。
- **数字范围为 `u64`**：不复刻 PHP 版 bcmath/gmp 的任意精度；官方测试向量（含 `u64::MAX`）全部落在 `u64` 内，无损。

## 项目结构

```
hashids-rust/
├── src/
│   ├── lib.rs                       # crate 文档与再导出
│   ├── hashids.rs                   # 内核算法：encode/decode/encode_hex/decode_hex
│   ├── config.rs                    # Config / ConnectionConfig（default + connections）
│   ├── error.rs                     # Error（对齐 PHP 版报错语义与文案）
│   ├── factory.rs                   # HashidsFactory：唯一构造点
│   ├── manager.rs                   # 核心：多连接解析、实例缓存、默认连接代理
│   ├── guard.rs                     # 原生 Guard（请求守卫）：不依赖任何框架的入口
│   ├── mascot.rs                    # 项目宠物「哈希迪 Hashy」的 ASCII 版
│   └── integrations/                # 八框架适配（均 feature 门控、只做接线）
│       ├── axum.rs · actix.rs · rocket.rs · poem.rs
│       ├── salvo.rs · warp.rs
│       └── bee.rs（bee_router）· ecat.rs（e-cat，底层即 axum）
├── tests/
│   ├── vectors.rs                   # 官方测试向量（vinkla/hashids 全量转写）
│   ├── manager.rs · guard.rs · factory.rs · mascot.rs
│   └── integrations/                # 八个框架各一个端到端用例
├── docs/
│   ├── mascot.svg                   # 项目宠物「哈希迪 Hashy」
│   ├── weixinpay.png · alipay.png   # 赞助二维码
├── .github/workflows/               # test / release
├── Cargo.toml
└── LICENSE
```

## 架构设计

**四层单向依赖**，上层依赖下层，反向不成立：

| 层 | 职责 | 位置 |
|----|------|------|
| 应用调用层 | 提取器、`Extension`、`Depot`、`Guard` 克隆等多种入口 | 业务代码 |
| 框架适配层 | 只做接线：状态注册、请求守卫、注入 | `src/integrations/` |
| 核心层 | 多连接管理与实例构建，**不依赖任何框架** | `src/manager.rs`、`src/factory.rs`、`src/guard.rs` |
| 内核算法 | 实际编解码实现，**零依赖** | `src/hashids.rs` |

核心层是整个包的重心：`HashidsManager` 持有配置，`connection()` 按需构建并缓存连接（`Arc` 复用），`encode()/decode()` 等代理方法转发到默认连接；`HashidsFactory::make()` 是唯一构造 `Hashids` 的地方。`Guard` 在接线期一次性解析默认连接、快速失败，此后编解码不可失败。八个框架适配加起来约 400 行——它们只负责把内核接进各自的状态机制。

## 功能设计

八个能力分组，全部围绕同一个内核：

- **编解码 API**：`encode()` / `decode()` / `encode_hex()` / `decode_hex()`，经代理方法落到默认连接，无需显式 `connection()`。
- **多连接管理**：`connection(Some("alternative"))` 切换；懒加载构建，同一连接只构建一次。
- **原生 Guard**：不依赖任何框架的请求守卫，放进任意框架的应用状态、每请求克隆即可用；八个适配器全部产出 `Guard`。
- **八框架适配**：Axum 用 `FromRequestParts`、Actix Web 用 `FromRequest`、Rocket 用 `FromRequest` 守卫、Poem 用 `FromRequest`、Salvo 用 Depot 注入器（`.hoop()`）、Warp 用 Filter、Bee 用原生 `Filter` 钩子、e-cat 用 `Extension` 层，各自遵循框架惯用法。
- **快速失败**：默认连接未配置时，Warp 在接线期报错、e-cat/Guard 在接线期报错、其余框架在提取时给出 500——配置错误不会静默降级。
- **零依赖内核**：`HashidsManager::new(config)` 即可工作，默认 feature 不引入任何第三方 crate。

## 生命周期

| 阶段 | 发生了什么 |
|------|-----------|
| **构建期** | 安装 `hashids-rust` 并开启所需框架 feature → 启动时 `HashidsManager::new(config)` → 包进 `Arc` 放进框架状态；Warp / e-cat / Guard 在此解析默认连接、快速失败 |
| **运行期** | 请求 → 提取器/守卫/过滤器取出 `Guard` → `encode()` / `decode()` → **连接命中缓存直接复用，未命中才 `HashidsFactory::make()` 构建并写入缓存** → 返回短 ID 或原数字 |
| **释放期** | 随进程退出回收；无外部资源需要清理 |

连接是**按需构建**的：只调用默认连接的进程，不会为 `alternative` 之类未使用的连接付出任何构建成本。

## 安装

尚未发布到 crates.io；当前从 GitHub 安装，锁定 `v1.0.0`：

```bash
cargo add hashids-rust --git https://github.com/erikwang2013/hashids-rust --tag v1.0.0
```

或直接写进 `Cargo.toml`：

```toml
[dependencies]
hashids-rust = { git = "https://github.com/erikwang2013/hashids-rust", tag = "v1.0.0" }
```

按需开启框架适配（默认不编译任何框架）：

| feature | 框架 | 注册方式 |
|---------|------|----------|
| `axum` | Axum 0.8 | `Router::with_state(Arc::new(manager))` |
| `actix` | Actix Web 4 | `App::app_data(web::Data::new(manager))` |
| `rocket` | Rocket 0.5 | `rocket::build().manage(Arc::new(manager))` |
| `poem` | Poem 3 | `Route::data(Arc::new(manager))` |
| `salvo` | Salvo 1 | `Router::hoop(HashidsInjector::new(manager))` |
| `warp` | Warp 0.4 | `hashids(manager)` → Filter |
| `bee` | Bee（`bee_router` 1） | `HashidsFilter` 或直接配合 axum 提取器 |
| `ecat` | e-cat（`ecat` 4） | `router.layer(hashids_layer(guard))` |

> **运行环境**：核心（默认 feature）需要 Rust **1.85+**。框架适配的 MSRV 以各框架 crate 自身为准——Actix Web 4.15 需 **1.88+**、Salvo 1.0 需 **1.94+**，`--all-features` 实际按最高的 **1.94** 计。
>
> **包名说明**：crates.io 上另有一个 2015 年停更的同名 `hashids` crate。本包的实际 crate 名为 `hashids-rust`，库名为 `hashids`（代码里写 `use hashids::...`）；同时依赖两者时请注意区分。

## 配置结构

与 PHP 版配置**同构**（根级 `default` + `connections`，连接项含 `salt` / `length` / `alphabet`），只是从数组变成类型化结构体：

```rust
use hashids::{Config, ConnectionConfig};

let config = Config::new()
    .default_connection("main")
    .connection("main", ConnectionConfig::new().salt("").min_hash_length(0))
    // 可选自定义字母表：
    .connection(
        "alternative",
        ConnectionConfig::new()
            .salt("your-salt-string")
            .min_hash_length(6)
            .alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890"),
    );
```

- `default`：默认连接名（如 `main`）；缺省或为空时按 `main` 处理（对齐 vinkla/hashids）。
- `connections`：连接名 => `salt`、`min_hash_length`（PHP 配置键为 `length`）、可选 `alphabet`。

## 原生用法（Guard）

不依赖任何框架，直接用管理器或原生守卫：

```rust
use std::sync::Arc;

use hashids::{Config, ConnectionConfig, Guard, HashidsManager};

let manager = Arc::new(HashidsManager::new(
    Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
));

// 方式一：直接用 Manager（每次调用解析默认连接，可能失败）
let hash = manager.encode(&[1, 2, 3])?;
let ids = manager.decode(&hash)?;

// 方式二：Guard —— 接线期解析默认连接、快速失败；此后编解码不可失败
let hashids = Guard::from_manager(manager)?;
let hash = hashids.encode(&[1, 2, 3]);
let ids = hashids.decode(&hash);
```

---

## Axum

提取器：把 `Arc<HashidsManager>` 放进应用状态，handler 参数直接写 `Hashids`。

```rust
use std::sync::Arc;

use axum::{routing::get, Router};
use hashids::integrations::axum::Hashids;
use hashids::{Config, ConnectionConfig, HashidsManager};

let manager = Arc::new(HashidsManager::new(
    Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
));

let app = Router::new()
    .route(
        "/user",
        get(|hashids: Hashids| async move { hashids.0.encode(&[1, 2, 3]) }),
    )
    .with_state(manager);
```

应用状态是自己的 `AppState` 时，补一个 `impl FromRef<AppState> for Arc<HashidsManager>` 即可（axum 标准模式）。

---

## Actix Web

用 `App::app_data(web::Data::new(manager))` 注册，handler 参数直接写 `Hashids`。

```rust
use actix_web::{App, web};
use hashids::integrations::actix::Hashids;
use hashids::{Config, ConnectionConfig, HashidsManager};

let manager = HashidsManager::new(
    Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
);

async fn show(ids: Hashids) -> String {
    ids.0.encode(&[1, 2, 3])
}

let app = App::new()
    .app_data(web::Data::new(manager))
    .route("/user", web::get().to(show));
```

---

## Rocket

用 `rocket::build().manage(Arc::new(manager))` 注册托管状态，路由函数参数直接写 `Hashids`。

```rust
use std::sync::Arc;

use hashids::integrations::rocket::Hashids;
use hashids::{Config, ConnectionConfig, HashidsManager};

let manager = Arc::new(HashidsManager::new(
    Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
));

#[rocket::get("/user")]
fn user(hashids: Hashids) -> String {
    hashids.0.encode(&[1, 2, 3])
}

let rocket = rocket::build().manage(manager).mount("/", rocket::routes![user]);
```

---

## Poem

用 `EndpointExt::data(Arc::new(manager))` 注册，handler 参数直接写 `Hashids`。

```rust
use std::sync::Arc;

use hashids::integrations::poem::Hashids;
use hashids::{Config, ConnectionConfig, HashidsManager};
use poem::{EndpointExt, Route, get};

let manager = Arc::new(HashidsManager::new(
    Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
));

async fn index(hashids: Hashids) -> String {
    hashids.0.encode(&[1, 2, 3])
}

let app = Route::new().at("/user", get(index)).data(manager);
```

---

## Salvo

Salvo 没有通用的提取器 trait，惯用做法是 **Depot**：把 `HashidsInjector` 作为中间件挂到 Router 上，每个请求把 `Guard` 注入 Depot。

```rust
use std::sync::Arc;

use hashids::integrations::salvo::{DepotHashids, HashidsInjector};
use hashids::{Config, ConnectionConfig, HashidsManager};
use salvo::prelude::*;

let manager = Arc::new(HashidsManager::new(
    Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
));

#[salvo::handler]
async fn user(depot: &mut Depot) -> String {
    depot.hashids().expect("guard 已注入").0.encode(&[1, 2, 3])
}

let router = Router::new()
    .hoop(HashidsInjector::new(manager))
    .push(Router::with_path("/user").get(user));
```

默认连接未配置时，注入器会直接以 500 短路请求，不会静默放行。

---

## Warp

warp 没有提取器 trait，惯用做法是过滤器链。`hashids()` 产出一个携带 `Hashids` 的 Filter，**接线期**解析默认连接（未配置时立即返回错误，快速失败）。

```rust
use std::sync::Arc;

use hashids::integrations::warp::{Hashids, hashids};
use hashids::{Config, ConnectionConfig, HashidsManager};
use warp::Filter;

let manager = Arc::new(HashidsManager::new(
    Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
));

let route = warp::get()
    .and(warp::path("user"))
    .and(hashids(manager)?)
    .map(|ids: Hashids| ids.0.encode(&[1, 2, 3]));
```

---

## Bee

[`bee_router`](https://github.com/erikwang2013/bee-rust) 的路由底层就是 axum 0.8：handler 即 axum handler，直接配合上面的 `axum` 提取器使用；bee 原生侧提供 `HashidsFilter`——把 `Guard` 注入每次 `Context::dispatch` 的请求扩展。

```rust
use std::sync::Arc;

use bee_router::Filter;
use hashids::integrations::bee::{ContextHashids, HashidsFilter};
use hashids::{Config, ConnectionConfig, Guard, HashidsManager};

let manager = Arc::new(HashidsManager::new(
    Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
));
let guard = Guard::from_manager(manager)?;
let filter = HashidsFilter::new(guard);

// controller 内：
// ctx.dispatch(cache, ttl, &[&filter], &controller).await?;
// let hash = ctx.hashids().expect("guard 已注入").encode(&[1, 2, 3]);
```

---

## e-cat

[`ecat`](https://github.com/erikwang2013/e-cat) 的 HTTP 传输就是 axum `Router`（`HttpServer::new(addr).router(router)`）：把 `Guard` 挂成 `Extension` 层，handler 提取 `Extension<Guard>` 即可。

```rust
use std::sync::Arc;

use axum::extract::Extension;
use axum::{Router, routing::get};
use hashids::integrations::ecat::hashids_layer;
use hashids::{Config, ConnectionConfig, Guard, HashidsManager};

let manager = Arc::new(HashidsManager::new(
    Config::new().connection("main", ConnectionConfig::new().salt("this is my salt")),
));
let guard = Guard::from_manager(manager)?;

let router: Router = Router::new()
    .route(
        "/user",
        get(|Extension(guard): Extension<Guard>| async move { guard.encode(&[1, 2, 3]) }),
    )
    .layer(hashids_layer(guard));
```

---

## 项目宠物

项目宠物是 **哈希迪 Hashy**——一只头顶天线、胸口印着 `#` 的圆角方块小宠物，矢量图见 [`docs/mascot.svg`](docs/mascot.svg)。

终端里没有 SVG，所以代码里留了一份等价的 ASCII 版本（`hashids::mascot`）：

```rust
use hashids::mascot;

println!("{}", mascot::greet());
```

```
           ●
           │
      ╭─────────╮
      │ ◉     ◉ │
      │    ‿    │
      │    #    │
      ╰──┬───┬──╯
         ╵   ╵
哈希迪 Hashy · 把数据库自增 ID 换成短小、不可猜测的字符串
```

---

## 开源不易，欢迎支持 / Open Source is Not Easy, Your Support is Welcome

<p align="center">
  <img src="./docs/weixinpay.png" alt="微信 WeChat Pay" width="130" height="130" />
  &nbsp;&nbsp;&nbsp;&nbsp;
  <img src="./docs/alipay.png" alt="支付宝 Alipay" width="130" height="130" />
</p>

<p align="center"><strong>微信 WeChat Pay</strong>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<strong>支付宝 Alipay</strong></p>

---

## License

MIT. See [LICENSE](LICENSE).
