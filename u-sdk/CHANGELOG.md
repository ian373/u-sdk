# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

[lib]

- 添加`credentials`模块，定义`Credentials` trait和相关实现供阿里云各个sdk模块使用

[oss]

- 添加`GetObject`预签名URL的方法
- 添加`PutObject`预签名URL的方法
- 添加`PostObject`获取签名信息的方法
- 添加`PutObject`的callback的功能
- 添加`PostObject`获取签名信息时携带callback的功能
- 添加`PutObject`生成预签名时携带callback的功能
- 添加oss callback服务器端验证的axum Layer
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

[oss]

- 更新和完善了代码/测试/用户的文档说明
- 重构/简化了签名模块的函数参数结构

[email]

- email sdk使用OpenAPI V3签名版本

### Fixed

[oss]

- 修复了`GetObject`header和query没有区分的问题
