//! # 阿里云云服务功能sdk
//!
//! 目前实现以下功能：
//! - 邮件推送(`email`)部分功能
//! - OSS(`oss`)部分功能
//!
//! # 说明
//!
//! - `blocking`为同步API，所有同步的API都在此模块下。
//! - `error`模块包含所有错误类型，大部分错误类型直接来自使用的`crates`的`Error`。
//! - 其它模块为对应sdk的异步实现
//! - 请结合阿里云对应服务的API文档使用，相关字段说明参阅阿里云文档
//!
//! # 注意
//!
//! - `blocking`模块暂时没有开发，等到异步API开发完成稳定后，再实现该模块。
//! - 理论上`blocking`API和异步API基本相同
//! - 调用相应API时请注意你的RAM用户是否有相应得权限
//!

#[cfg(feature = "blocking")]
pub mod blocking;

#[cfg(feature = "email")]
pub mod email;
#[cfg(feature = "oss")]
pub mod oss;

pub mod error;
