# 阿里云 OpenAPI 通用（非 SLS, OSS) API 注意事项记录

- 对于 API 文档的请求参数，比如：

| 名称                                            | 类型              | 必填 | 描述             | 示例值  |
|:----------------------------------------------|:----------------|----|----------------|------|
| TagFilter                                     | `array<object>` | 否  | 标签过滤规则。        |      |
| <span style="padding-left:20px"> - Key</span> | `string`        | 否  | 标签键，用于查询的过滤条件。 | tag1 |
| <span style="padding-left:20px"> - Key</span> | `string`        | 否  | 标签值，用于查询的过滤条件。 | aaa  |

签名的时候对于这种复杂对象的转化为url query参数的时候有两种方法：

1. 扁平化参数：将复杂对象展开为多个简单参数。例如，上述的 TagFilter 可以展开为 TagFilter.1.Key=tag1, TagFilter.1.Value=aaa
2. JSON 序列化：将复杂对象序列化为 JSON 字符串，然后作为单个参数传递。例如，上述的 TagFilter 可以序列化为
   TagFilter=[{"Key":"tag1","Value":"aaa"}]

具体使用的是哪种，直接看API的文档可能不太清楚，需要参考API元数据，如果下面该字段API的元数据：

```json
{
    "name": "TagFilter",
    "in": "query",
    "style": "json",
    "schema": {
        "description": "标签过滤规则。",
        "type": "array",
        "items": {
            "type": "object",
            "properties": {
                "Key": {
                    "description": "标签键，用于查询的过滤条件。",
                    "type": "string",
                    "required": false,
                    "example": "tag1"
                },
                "Value": {
                    "description": "标签值，用于查询的过滤条件。",
                    "type": "string",
                    "required": false,
                    "example": "aaa"
                }
            },
            "required": false
        },
        "required": false
    }
}
```

[OpenAPI签名文档](https://help.aliyun.com/zh/sdk/product-overview/v3-request-structure-and-signature)说到：

在OpenAPI元数据中，如果API的请求参数信息包含了"in":"query"时，需要将这些请求参数按照如下构造方法拼接起来：

1. 将请求参数按照参数名称的字符顺序升序排列。
2. 使用UTF-8字符集按照RFC3986的规则对每个参数的参数名称和参数值分别进行URI编码，具体规则与上一节中的CanonicalURI编码规则相同。
3. 使用等号（=）连接编码后的请求参数名称和参数值。当参数没有值时，应使用空字符串作为该参数的值。
4. 多个请求参数之间使用与号（&）连接。

> ### 重要
>
> 当请求参数类型是array、object时，需要将参数值转化为带索引的键值对。
>
> 当请求参数是JSON字符串类型时，JSON字符串中的参数顺序不会影响签名计算。
>
> 当无查询字符串时，使用空字符串作为规范化查询字符串。

所以这个字段包含在url的query中，但是需要使用哪种方式，需要参考API元数据中的style字段。如果为json，则把这个复杂字段序列化为JSON字
符串传递； 如果style字段不是`json`，遇见过的有`repeatList`或其他，则使用扁平化参数传递。

在usdk中，传入的给签名的struct如果序列化的时候这个字段是struct类型，那么会按照扁平化方式处理，如果需要最终结果是json字符串，
那么需要自定义这个字段的serialize方法，手动把它序列化为json字符串。
