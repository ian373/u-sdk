# 介绍

sdk包含有以下应用部分功能：

- [阿里云邮件服务(email)](https://help.aliyun.com/zh/direct-mail/api-dm-2015-11-23-overview)
- [阿里云对象存储(oss)](https://help.aliyun.com/zh/oss/developer-reference/list-of-operations-by-function)
- [阿里云机器翻译(translate)](https://help.aliyun.com/zh/machine-translation/developer-reference/api-alimt-2018-10-12-dir/)
- [阿里云sts](https://help.aliyun.com/zh/ram/developer-reference/api-reference-5/)
- [server酱(server_chan)](https://doc.sc3.ft07.com/zh/serverchan3)
- [deepseek](https://api-docs.deepseek.com/zh-cn/api/deepseek-api/)

# 了解如何使用

在项目的[u-sdk/u-sdk/tests/](https://github.com/ian373/u-sdk/tree/main/u-sdk/tests)下有各个模块的测试用例，同时也是各个模块的使用示例。

各个请求的参数内容和定义，请查看对应模块的官方API文档。

## server酱举例

在`u-sdk/u-sdk/tests/server_chan/`目录下创建`config.toml`文件，内容参考同目录下的`config.sample.toml`，并填上真实的内容：

```toml
# config.toml
uid = 1234
key = "server chan send key"
```

在`u-sdk/tests/server_chan/main.rs`中，选择一个测试用例并尝试修改：

```rust
#[tokio::test]
async fn server_chan_test() {
    let conf = Config::get_conf();
    let client = Client::builder().uid(conf.uid).key(&conf.key).build();

    let resp = client
        .send_msg()
        .title("test--title")
        .description("this is a description")
        .short("short")
        .tag("123")
        .tags(["tag1", "tag2"])
        .build()
        .send()
        .await;
    if let Err(e) = resp {
        eprintln!("Error sending message: {}", e);
    } else {
        println!("Message sent successfully");
    }
}
```

运行测试并查看结果：

```bash
cargo nextest run --all-features --no-capture --run-ignored=all server_chan_test
```

结果：

```text
------------
 Nextest run ID 99a5c68a-d89c-4f56-b4fe-0fd454f1cb5e with nextest profile: default
    Starting 1 test across 8 binaries (36 tests skipped)
     Running [ 00:00:00] 0/1: 0 running, 0 passed, 0 skipped
       START (1/1) u-sdk::server_chan server_chan_test

running 1 test
Message sent successfully
test server_chan_test ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.70s

        PASS [   1.734s] (1/1) u-sdk::server_chan server_chan_test
------------
```
