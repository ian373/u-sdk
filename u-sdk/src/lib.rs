#![doc = include_str!("../README.md")]

#[cfg(feature = "oss_callback_verify_layer")]
pub mod oss_callback_verify_layer;

#[cfg(feature = "email")]
pub mod email;
#[cfg(feature = "oss")]
pub mod oss;
#[cfg(feature = "translate")]
pub mod translate;

#[cfg(feature = "server_chan")]
pub mod server_chan;

#[cfg(feature = "deep_seek")]
pub mod deep_seek;

#[cfg(feature = "sts")]
pub mod sts;

/// Credentials related implementations for Aliyun SDKs
#[cfg(any(
    feature = "email",
    feature = "oss",
    feature = "translate",
    feature = "sts"
))]
pub mod credentials;
