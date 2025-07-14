#![doc = include_str!("../../README.md")]

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
