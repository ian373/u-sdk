# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- [oss]
- 添加`GetObject`预签名URL的方法
- 添加`PutObject`预签名URL的方法
- 添加`PostObject`获取签名信息的方法
- 添加`PutObject`的callback的功能
- 添加`PostObject`获取签名信息时携带callback的功能
- 添加`PutObject`生成预签名时携带callback的功能
- 添加oss callback服务器端验证的axum Layer

### Changed

- [oss]
- 更新和完善了代码/测试/用户的文档说明

### Fixed

- [oss]
- 修复了`GetObject`header和query没有区分的问题
