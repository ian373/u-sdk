# AGENTS.md

## Crate 概述

`u-sdk` 是 workspace 中的主要 SDK crate。`src/` 下第一层目录或文件通常代表一个独立 SDK 模块，各 SDK 主要通过 Cargo feature
按需启用。

当前主要模块包括：

- `email`
- `oss`
- `oss_callback_verify_layer`
- `translate`
- `server_chan`
- `deep_seek`
- `sts`
- `esa`
- `ecs`

处理任务时，应将一个 SDK 模块视为主要修改边界，只关注该模块直接相关的源码、测试、文档和 changelog。不要顺手修改或统一其他 SDK
的实现结构。

## 模块与 Feature

- SDK 模块应由 `Cargo.toml` 中对应的 feature 控制。
- SDK 的 crate-level 导出应在 `src/lib.rs` 中使用 `#[cfg(feature = "...")]` 控制。
- 新增或调整模块时，应确保 feature、`src/lib.rs` 导出和模块内部使用的可选依赖保持一致。
- 不得因为当前任务而改变无关 feature 的依赖关系或启用范围。
- 修改 `Cargo.toml` 仍受根目录 `AGENTS.md` 的限制，必须先说明并等待用户确认。

## 共享模块

### `credentials.rs`

`src/credentials.rs` 提供阿里云 SDK 共用的 `Credentials` 和 `CredentialsProvider`。它不是单一 SDK 的内部实现。

当前可能使用该模块的 feature 包括：

- `email`
- `oss`
- `translate`
- `sts`
- `esa`
- `ecs`

修改凭证结构、trait 或行为时，必须先说明对上述模块的影响，不得只按当前 SDK 的局部需求直接修改。仅使用固定 API key 或自身鉴权方式的
SDK 不应无理由依赖该模块。

### 其他跨模块关系

- `oss_callback_verify_layer` 是独立 feature 控制的单文件模块，与 `oss` feature 不应被默认视为同一个启用单元。
- ESA 和 ECS 的集成测试通过 STS 获取临时凭证，因此其测试同时要求目标 SDK feature 和 `sts` feature。
- 如果一个改动会影响多个 SDK，应明确列出影响范围，并按根目录规则分步处理。

## SDK 源码组织

- 每个 SDK 保留自己的 `Client`、错误类型、请求与响应类型、辅助函数和公开导出结构。
- 新增 API 时，优先放入目标 SDK 已有且职责匹配的文件。
- 只有在职责清楚、现有文件不适合承载时才新增源码文件。
- builder、错误转换、响应解析和 re-export 应沿用目标 SDK 当前的写法。
- 不跨 SDK 做命名统一、公共抽象提取或风格重构，除非用户明确要求。
- 修改公开 API 时，应检查对应的 README、Rust 文档注释和 changelog 是否需要同步更新。

## 测试组织

`tests/` 按 SDK 模块组织：

```text
tests/<sdk>/main.rs
tests/<sdk>/config.sample.toml
```

遵循以下约定：

- `tests/<sdk>/main.rs` 应使用 crate-level `#![cfg(...)]` 绑定对应 feature。
- 原则上，一个 SDK API 方法对应一个独立测试函数。
- 异步 API 测试沿用 `#[tokio::test]`。
- 调用真实外部服务、依赖真实账号或会产生外部副作用的测试必须标记 `#[ignore]`。
- 测试通常从 `tests/<sdk>/config.toml` 读取本地配置。
- 只提交 `config.sample.toml`，不得提交、记录或输出真实密钥、token、账号和资源标识。
- 新增或修改测试配置字段时，应同步更新对应的 `config.sample.toml`。
- 测试代码可以保持重复和模板化；不要为了减少少量重复而跨 SDK 提取测试抽象。
- 测试失败时，应保留能直接定位目标 API 的上下文，但不得打印敏感凭据。

ESA 和 ECS 测试的 feature 条件应包含对应 SDK 和 STS，例如：

```rust
#![cfg(all(feature = "esa", feature = "sts"))]
```
