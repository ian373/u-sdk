# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.1] - 2025-12-17

### Fixed

[oss]

- `PostObject`生成Policy部分，在返回中添加必要的`sts security token`，前端才能顺利使用临时凭证进行请求

## [0.5.0] - 2025-12-13

### Changed

[lib]

- 更改了credentials模块中的`CredentialsProvider` trait的方法签名

## [0.4.0] - 2025-12-05

### Added

[lib]

- 添加`credentials`模块，定义`Credentials` trait和相关实现供阿里云各个sdk模块使用
- 添加`oss_callback_verify_layer`模块，实现oss callback服务器端验证的axum Layer

[oss]

- 添加`GetObject`预签名URL的方法
- 添加`PutObject`预签名URL的方法
- 添加`PostObject`获取签名信息的方法
- 添加`PutObject`的callback的功能
- 添加`PostObject`获取签名信息时携带callback的功能
- 添加`PutObject`生成预签名时携带callback的功能
- 为现有方法添加sts临时凭证支持

[sts]

- 添加sts模块，实现`AssumeRole`功能
- 添加构建policy相关的方法
- 添加临时凭证的Credentials功能

[translate]

- 支持sts临时凭证进行相关api调用

[email]

- 支持sts临时凭证进行相关api调用

### Changed

[lib]

- 更新了文档

[oss]

- 更新和完善了代码/测试/用户的文档说明
- 重构/简化了签名模块的函数参数结构

[email]

- email sdk使用OpenAPI V3签名版本

### Fixed

[oss]

- 修复了`GetObject`header和query没有区分的问题
