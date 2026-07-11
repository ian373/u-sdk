# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0] - 2026-07-11

### Changed

[server_chan]

- `Client` 构建参数由 `uid` 和 `key` 调整为 `send_key`，并新增了错误类型。

## [0.6.3] - 2026-03-02

### Added

- ESA 模块添加 DNS 相关接口：`CreateRecord`、`DeleteRecord`、`UpdateRecord`、`ListRecords`、`GetRecord`。

## [0.6.2] - 2026-01-15

### Added

- 添加 ESA 模块，实现 `ListSites`、`GetOriginProtection`、`UpdateOriginProtectionIpWhiteList` 功能。
- 添加 ECS 模块，实现 `DescribePrefixListAttributes`、`ModifyPrefixList` 功能。

### Changed

- 更新了一些方法的文档。

## [0.6.1] - 2025-12-17

### Fixed

[oss]

- `PostObject` 生成 Policy 部分，在返回中添加必要的 `sts security token`，前端才能顺利使用临时凭证进行请求。

## [0.5.0] - 2025-12-13

### Changed

[lib]

- 更改了 credentials 模块中的 `CredentialsProvider` trait 的方法签名。

## [0.4.0] - 2025-12-05

### Added

[lib]

- 添加 `credentials` 模块，定义 `Credentials` trait 和相关实现供阿里云各个 sdk 模块使用。
- 添加 `oss_callback_verify_layer` 模块，实现 oss callback 服务器端验证的 axum Layer。

[oss]

- 添加 `GetObject` 预签名 URL 的方法。
- 添加 `PutObject` 预签名 URL 的方法。
- 添加 `PostObject` 获取签名信息的方法。
- 添加 `PutObject` 的 callback 的功能。
- 添加 `PostObject` 获取签名信息时携带 callback 的功能。
- 添加 `PutObject` 生成预签名时携带 callback 的功能。
- 为现有方法添加 sts 临时凭证支持。

[sts]

- 添加 sts 模块，实现 `AssumeRole` 功能。
- 添加构建 policy 相关的方法。
- 添加临时凭证的 Credentials 功能。

[translate]

- 支持 sts 临时凭证进行相关 api 调用。

[email]

- 支持 sts 临时凭证进行相关 api 调用。

### Changed

[lib]

- 更新了文档。

[oss]

- 更新和完善了代码、测试、用户的文档说明。
- 重构、简化了签名模块的函数参数结构。

[email]

- email sdk 使用 OpenAPI V3 签名版本。

### Fixed

[oss]

- 修复了 `GetObject` header 和 query 没有区分的问题。
