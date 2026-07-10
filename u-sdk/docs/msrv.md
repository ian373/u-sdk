# MSRV 维护提醒

本项目将开发工具链和最低支持 Rust 版本（MSRV）分开管理：

- 根目录的 `rust-toolchain.toml` 指定日常开发、lint、测试和文档构建使用的 Rust 版本。
- `u-sdk/Cargo.toml` 和 `u-sdk-common/Cargo.toml` 中的 `package.rust-version` 声明两个 crate 的 MSRV。
- `.github/workflows/ci.yml` 的 `msrv` job 显式安装并检查该 MSRV，不跟随 `rust-toolchain.toml`。

升级开发工具链时，只需修改 `rust-toolchain.toml`，不需要同步提高 MSRV。CI 会继续使用声明的 MSRV 检查兼容性。

决定提高 MSRV 时，必须同步修改以下三个位置：

1. `u-sdk/Cargo.toml` 中的 `package.rust-version`。
2. `u-sdk-common/Cargo.toml` 中的 `package.rust-version`。
3. `.github/workflows/ci.yml` 中 `msrv` job 的工具链版本和名称。

提交前应使用新的 MSRV 执行以下检查：

```bash
cargo +<MSRV> check --workspace --all-targets --all-features --locked
```

提高 MSRV 属于面向使用者的兼容性变化，应在对应 crate 的 changelog 中说明。
