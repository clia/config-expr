# Config Expression

一个灵活的配置表达式评估器，支持JSON schema定义的规则系统。

## 特性

- 🚀 **高性能**: 基于Rust实现，零拷贝解析
- 📝 **JSON Schema**: 结构清晰、语义明确的JSON规则定义
- 🔧 **可扩展**: 支持多种操作符和复合条件
- 🎯 **类型安全**: 完整的Rust类型系统支持
- ✅ **验证器**: 内置规则合法性验证
- 🌐 **前后端友好**: 易于前端配置和后端执行

## 支持的操作符

| 操作符 | 说明 | 示例 | 备注 |
|--------|------|------|------|
| `equals` | 完全等于 | `"platform" equals "RTD"` | 字符串比较 |
| `contains` | 包含 | `"platform" contains "RTD"` | 字符串比较 |
| `prefix` | 前缀匹配 | `"platform" prefix "Hi"` | 字符串比较 |
| `suffix` | 后缀匹配 | `"platform" suffix "Pro"` | 字符串比较 |
| `regex` | 正则匹配 | `"version" regex "^v\\d+\\.\\d+\\.\\d+$"` | 字符串比较 |
| `gt` | 大于 | `"score" gt "80"` | 数值比较 |
| `lt` | 小于 | `"age" lt "18"` | 数值比较 |
| `ge` | 大于等于 | `"level" ge "5"` | 数值比较 |
| `le` | 小于等于 | `"temperature" le "25.5"` | 数值比较 |

## 支持的条件类型

- **简单条件**: 单个字段比较
- **AND条件**: 所有子条件都必须满足
- **OR条件**: 至少一个子条件满足
- **嵌套条件**: 支持任意层级的条件嵌套

## 快速开始

### 添加依赖

```toml
[dependencies]
clia-config-expr = "0.1.0"
```

### 基本用法

```rust
use clia_config_expr::{evaluate_json, validate_json};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 定义规则
    let rules = r#"
    {
        "rules": [
            {
                "if": {
                    "and": [
                        { "field": "platform", "op": "contains", "value": "RTD" },
                        { "field": "region", "op": "equals", "value": "CN" }
                    ]
                },
                "then": "chip_rtd_cn"
            },
            {
                "if": {
                    "field": "platform",
                    "op": "prefix",
                    "value": "Hi"
                },
                "then": "chip_hi"
            },
            {
                "if": {
                    "field": "score",
                    "op": "ge",
                    "value": "80"
                },
                "then": "high_score"
            }
        ],
        "fallback": "default_chip"
    }
    "#;

    // 验证规则
    validate_json(rules)?;

    // 准备参数
    let mut params = HashMap::new();
    params.insert("platform".to_string(), "RTD-2000".to_string());
    params.insert("region".to_string(), "CN".to_string());

    // 评估规则
    let result = evaluate_json(rules, &params)?;
    println!("结果: {:?}", result); // Some(String("chip_rtd_cn"))

    Ok(())
}
```

## JSON Schema 结构

### 基本结构

```json
{
    "rules": [
        {
            "if": "<条件表达式>",
            "then": "<返回值>"
        }
    ],
    "fallback": "<可选的默认返回值>"
}
```

### 条件表达式

#### 简单条件

字符串比较：
```json
{
    "field": "platform",
    "op": "equals",
    "value": "RTD"
}
```

数值比较：
```json
{
    "field": "score",
    "op": "ge",
    "value": "80"
}
```

#### AND条件
```json
{
    "and": [
        { "field": "platform", "op": "contains", "value": "RTD" },
        { "field": "region", "op": "equals", "value": "CN" }
    ]
}
```

#### OR条件
```json
{
    "or": [
        { "field": "platform", "op": "equals", "value": "MT9950" },
        { "field": "platform", "op": "equals", "value": "MT9638" }
    ]
}
```

### 返回值类型

#### 字符串返回值
```json
{
    "if": { "field": "platform", "op": "equals", "value": "RTD" },
    "then": "chip_rtd"
}
```

#### JSON对象返回值
```json
{
    "if": { "field": "platform", "op": "equals", "value": "RTD" },
    "then": {
        "chip": "rtd",
        "config": {
            "memory": "2GB",
            "cpu": "ARM"
        }
    }
}
```

## API 文档

### 主要类型

- `ConfigEvaluator`: 配置表达式评估器
- `ConfigRules`: 规则集定义
- `Condition`: 条件表达式
- `RuleResult`: 规则结果（字符串或JSON对象）
- `Operator`: 操作符枚举

### 主要方法

- `evaluate_json(json, params)`: 直接从JSON字符串评估
- `validate_json(json)`: 验证JSON规则是否合法
- `ConfigEvaluator::from_json(json)`: 从JSON创建评估器
- `evaluator.evaluate(params)`: 评估参数并返回结果

## 运行示例

```bash
cargo run --example basic_usage
```

## 运行测试

```bash
cargo test
```

## 许可证

本项目采用 MIT 或 Apache-2.0 双重许可证。
```
A JSON-based configuration expression processor.
