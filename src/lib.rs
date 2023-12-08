//! # 阿里云云服务sdk
//!
//! # Example
//!
//! 使用示例请查看项目根目录下`/tests/`文件夹，在每一个功能模块的文件夹下，在`test_config`目录下，创建`test_config.toml`文件并参考`config.sample.toml`配置好相关选项。
//!
//! 然后选择`./tests/main.rs`中的一个测试方法，如`get_object_meta_test()`，输入`cargo test --all-features get_object_meta_test --show-output --exact`来运行相关示例。
//!
//! # 说明
//!
//! - `blocking`为同步API，同步的API都在此模块下。
//! - `error`模块包含所有错误类型，大部分错误类型直接来自使用的`crates`的`Error`。
//! - 所有sdk均使用`https`进行请求，无法配置为`http`进行请求
//! - 其它模块为对应sdk的异步实现
//! - 请结合阿里云对应服务的API文档使用，相关字段说明参阅阿里云文档
//!
//! ## OSS
//!
//! - `oss` sdk关于版本控制的功能没有做，当开启版本控制时，发出请求的`Host`值和未开启版本控制时有所差别，代码未实现版本控制的请求
//!
//! # 注意
//!
//! - `blocking`模块暂时没有开发，等到异步API开发完成稳定后，将会删除该模块，更改为`email-sync`, `oss-sync`模块。
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
#[cfg(feature = "gen-trans")]
pub mod translate;
