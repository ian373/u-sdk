//! 权限策略语言
//!
//! [官方文档](https://help.aliyun.com/zh/ram/policy-language/)

use bon::Builder;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;

/// Version 目前只有 "1"
#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
pub enum PolicyVersion {
    #[serde(rename = "1")]
    #[default]
    V1,
}

/// Effect = "Allow" | "Deny"
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum Effect {
    Allow,
    Deny,
}

/// JSON 中“可以是单值或数组”的通用包装：
/// 比如 Action 可以是 "ecs:*" 或 ["ecs:*", "oss:*"]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> From<T> for OneOrMany<T> {
    fn from(value: T) -> Self {
        OneOrMany::One(value)
    }
}

impl<T> From<Vec<T>> for OneOrMany<T> {
    fn from(values: Vec<T>) -> Self {
        OneOrMany::Many(values)
    }
}

// Policy / Statement
/// 条件值：文档要求 Number/Boolean/Date/IP 都用字符串包起来
///（例如 `"10"`、`"true"`、`"2019-08-12T17:00:00+08:00"`）
/// 用新类型方便做 From<bool/number> 等转换。
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ConditionValue(pub String);

impl From<String> for ConditionValue {
    fn from(s: String) -> Self {
        ConditionValue(s)
    }
}

impl From<&str> for ConditionValue {
    fn from(s: &str) -> Self {
        ConditionValue(s.to_owned())
    }
}

impl From<bool> for ConditionValue {
    fn from(b: bool) -> Self {
        ConditionValue(if b { "true".into() } else { "false".into() })
    }
}

impl From<i64> for ConditionValue {
    fn from(n: i64) -> Self {
        ConditionValue(n.to_string())
    }
}

impl From<u64> for ConditionValue {
    fn from(n: u64) -> Self {
        ConditionValue(n.to_string())
    }
}
/// Condition 相关别名，对应：
/// <condition_map> = { <condition_type_string> : { <condition_key_string> : <condition_value_list>, ... }, ... }
pub type ConditionOperator = String; // 比如 "StringEquals" / "IpAddress"
pub type ConditionKey = String; // 比如 "acs:SourceIp"
pub type ConditionMap =
    HashMap<ConditionOperator, HashMap<ConditionKey, OneOrMany<ConditionValue>>>;

/// 条件块 Condition Block
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct ConditionBlock(pub ConditionMap);

impl ConditionBlock {
    pub fn new() -> Self {
        ConditionBlock(HashMap::new())
    }

    /// 通用插入方式：
    /// op: "StringEquals" / "IpAddress" / ...
    /// key: 条件键，例如 "acs:SourceIp"
    /// values: 单值或多值
    pub fn insert(
        &mut self,
        op: impl Into<String>,
        key: impl Into<String>,
        values: impl Into<OneOrMany<ConditionValue>>,
    ) {
        let op = op.into();
        let key = key.into();
        let entry = self.0.entry(op).or_default();
        entry.insert(key, values.into());
    }
}

impl Default for ConditionBlock {
    fn default() -> Self {
        Self::new()
    }
}

/// Condition 运算符名字常量（避免魔法字符串）
pub mod condition_ops {
    // String
    pub const STRING_EQUALS: &str = "StringEquals";
    pub const STRING_NOT_EQUALS: &str = "StringNotEquals";
    pub const STRING_EQUALS_IGNORE_CASE: &str = "StringEqualsIgnoreCase";
    pub const STRING_NOT_EQUALS_IGNORE_CASE: &str = "StringNotEqualsIgnoreCase";
    pub const STRING_LIKE: &str = "StringLike";
    pub const STRING_NOT_LIKE: &str = "StringNotLike";

    // Number
    pub const NUMERIC_EQUALS: &str = "NumericEquals";
    pub const NUMERIC_NOT_EQUALS: &str = "NumericNotEquals";
    pub const NUMERIC_LESS_THAN: &str = "NumericLessThan";
    pub const NUMERIC_LESS_THAN_EQUALS: &str = "NumericLessThanEquals";
    pub const NUMERIC_GREATER_THAN: &str = "NumericGreaterThan";
    pub const NUMERIC_GREATER_THAN_EQUALS: &str = "NumericGreaterThanEquals";

    // Date and time
    pub const DATE_EQUALS: &str = "DateEquals";
    pub const DATE_NOT_EQUALS: &str = "DateNotEquals";
    pub const DATE_LESS_THAN: &str = "DateLessThan";
    pub const DATE_LESS_THAN_EQUALS: &str = "DateLessThanEquals";
    pub const DATE_GREATER_THAN: &str = "DateGreaterThan";
    pub const DATE_GREATER_THAN_EQUALS: &str = "DateGreaterThanEquals";

    // Boolean
    pub const BOOL: &str = "Bool";

    // IP address
    pub const IP_ADDRESS: &str = "IpAddress";
    pub const NOT_IP_ADDRESS: &str = "NotIpAddress";
    pub const IP_ADDRESS_INCLUDE_BORDER: &str = "IpAddressIncludeBorder";
    pub const NOT_IP_ADDRESS_INCLUDE_BORDER: &str = "NotIpAddressIncludeBorder";
}

/// 单条授权语句 Statement，
/// 对应语法：
/// ```txt
/// <statement> = {
///   <effect_block>,
///   <action_block>,
///   <resource_block>,
///   <condition_block?>
/// }
/// ```
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Statement {
    pub effect: Effect,
    /// Action / NotAction 二选一（语义层面），用 validate() 做约束。
    pub action: Option<OneOrMany<String>>,
    pub not_action: Option<OneOrMany<String>>,
    pub resource: OneOrMany<String>,
    pub condition: Option<ConditionBlock>,
}

/// 整个 Policy
///
/// ```txt
/// policy = {
///   "Version": "1",
///   "Statement": [ <statement>, ... ]
/// }
/// ```
#[derive(Builder, Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Policy {
    #[builder(field)]
    pub statement: Vec<Statement>,
    #[builder(default)]
    pub version: PolicyVersion,
}

// 校验 Action / NotAction 约束
/// Policy 结构校验错误，只关注语法约束（Action/NotAction）
#[derive(thiserror::Error, Debug)]
pub enum PolicyValidationError {
    /// 既没有 Action 也没有 NotAction
    #[error("statement must contain either Action or NotAction")]
    MissingActionAndNotAction,
    /// 同时包含 Action 与 NotAction
    #[error("statement cannot contain both Action and NotAction")]
    BothActionAndNotActionPresent,
}

impl Statement {
    /// 校验当前语句是否满足文档要求：
    /// - Action / NotAction 必须二选一
    fn validate(&self) -> Result<(), PolicyValidationError> {
        match (&self.action, &self.not_action) {
            (None, None) => Err(PolicyValidationError::MissingActionAndNotAction),
            (Some(_), Some(_)) => Err(PolicyValidationError::BothActionAndNotActionPresent),
            _ => Ok(()),
        }
    }
}

impl<S: policy_builder::State> PolicyBuilder<S> {
    pub fn statement(mut self, stmt: Statement) -> Result<Self, PolicyValidationError> {
        stmt.validate()?;
        self.statement.push(stmt);
        Ok(self)
    }

    pub fn statements(
        mut self,
        stmts: impl IntoIterator<Item = Statement>,
    ) -> Result<Self, PolicyValidationError> {
        for stmt in stmts.into_iter() {
            stmt.validate()?;
            self.statement.push(stmt);
        }
        Ok(self)
    }
}

impl Policy {
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn to_json_string_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[test]
fn build_policy_test() {
    // Condition
    let mut cond = ConditionBlock::new();
    cond.insert(
        condition_ops::STRING_EQUALS,
        "acs:ResourceTag/team",
        ConditionValue::from("dev"),
    );
    // Statement
    let stmt = Statement {
        effect: Effect::Allow,
        action: Some(OneOrMany::One("ecs:*".to_string())),
        not_action: None,
        resource: OneOrMany::One("*".to_string()),
        condition: Some(cond),
    };
    // Policy
    let policy = Policy::builder().statement(stmt).unwrap().build();
    println!("policy json:\n{}", policy.to_json_string_pretty().unwrap());

    let mut cond = ConditionBlock::new();
    cond.insert(
        condition_ops::NUMERIC_LESS_THAN_EQUALS,
        "kms:RecoveryWindowInDays",
        ConditionValue::from(10_i64),
    );
    let stmt = Statement {
        effect: Effect::Deny,
        action: Some(OneOrMany::One("kms:DeleteSecret".to_string())),
        not_action: None,
        resource: OneOrMany::One("*".to_string()),
        condition: Some(cond),
    };
    let policy = Policy::builder().statement(stmt).unwrap().build();
    println!("policy json:\n{}", policy.to_json_string_pretty().unwrap());

    let mut cond = ConditionBlock::new();
    cond.insert(
        condition_ops::DATE_LESS_THAN,
        "acs:CurrentTime",
        ConditionValue::from("2019-08-12T17:00:00+08:00"),
    );
    let stmt = Statement {
        effect: Effect::Deny,
        action: Some(OneOrMany::One("oss:DeleteObject".to_string())),
        not_action: None,
        resource: OneOrMany::One("acs:oss:*:*:mybucket/myobject".to_string()),
        condition: Some(cond),
    };
    let s = Policy::builder().statement(stmt).unwrap().build();
    println!("policy json:\n{}", s.to_json_string_pretty().unwrap());
}
