# Config Expression

[中文版](README_zh.md)

A flexible configuration expression evaluator that supports rule systems defined by JSON schema.

## Features

- 🚀 **High Performance**: Implemented in Rust with zero-copy parsing
- 📝 **JSON Schema**: Clear and semantically explicit JSON rule definitions
- 🔧 **Extensible**: Supports multiple operators and compound conditions
- 🎯 **Type Safe**: Full Rust type system support
- ✅ **Validator**: Built-in rule validity validation
- 🌐 **Frontend/Backend Friendly**: Easy to configure on frontend and execute on backend

## Supported Operators

| Operator | Description | Example | Note |
|----------|-------------|---------|------|
| `equals` | Exact equals | `"platform" equals "RTD"` | String comparison |
| `contains` | Contains | `"platform" contains "RTD"` | String comparison |
| `prefix` | Prefix match | `"platform" prefix "Hi"` | String comparison |
| `suffix` | Suffix match | `"platform" suffix "Pro"` | String comparison |
| `regex` | Regex match | `"version" regex "^v\\d+\\.\\d+\\.\\d+$"` | String comparison |
| `gt` | Greater than | `"score" gt "80"` | Numeric comparison |
| `lt` | Less than | `"age" lt "18"` | Numeric comparison |
| `ge` | Greater than or equal | `"level" ge "5"` | Numeric comparison |
| `le` | Less than or equal | `"temperature" le "25.5"` | Numeric comparison |

## Supported Condition Types

- **Simple Condition**: Single field comparison
- **AND Condition**: All sub-conditions must be satisfied
- **OR Condition**: At least one sub-condition must be satisfied
- **Nested Conditions**: Supports arbitrary levels of condition nesting

## Getting Started

### Add Dependency

```toml
[dependencies]
clia-config-expr = "0.1.1"
```

### Basic Usage

```rust
use clia_config_expr::{evaluate_json, validate_json};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define rules
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

    // Validate rules
    validate_json(rules)?;

    // Prepare parameters
    let mut params = HashMap::new();
    params.insert("platform".to_string(), "RTD-2000".to_string());
    params.insert("region".to_string(), "CN".to_string());

    // Evaluate rules
    let result = evaluate_json(rules, &params)?;
    println!("Result: {:?}", result); // Some(String("chip_rtd_cn"))

    Ok(())
}
```

## JSON Schema Structure

### Basic Structure

```json
{
    "rules": [
        {
            "if": "<condition expression>",
            "then": "<return value>"
        }
    ],
    "fallback": "<optional default return value>"
}
```

### Condition Expressions

#### Simple Condition

String comparison:
```json
{
    "field": "platform",
    "op": "equals",
    "value": "RTD"
}
```

Numeric comparison:
```json
{
    "field": "score",
    "op": "ge",
    "value": "80"
}
```

#### AND Condition
```json
{
    "and": [
        { "field": "platform", "op": "contains", "value": "RTD" },
        { "field": "region", "op": "equals", "value": "CN" }
    ]
}
```

#### OR Condition
```json
{
    "or": [
        { "field": "platform", "op": "equals", "value": "MT9950" },
        { "field": "platform", "op": "equals", "value": "MT9638" }
    ]
}
```

### Return Value Types

#### String Return Value
```json
{
    "if": { "field": "platform", "op": "equals", "value": "RTD" },
    "then": "chip_rtd"
}
```

#### JSON Object Return Value
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

## API Documentation

### Main Types

- `ConfigEvaluator`: Configuration expression evaluator
- `ConfigRules`: Rule set definition
- `Condition`: Condition expression
- `RuleResult`: Rule result (string or JSON object)
- `Operator`: Operator enumeration

### Main Methods

- `evaluate_json(json, params)`: Directly evaluate from JSON string
- `validate_json(json)`: Validate if JSON rules are valid
- `ConfigEvaluator::from_json(json)`: Create evaluator from JSON
- `evaluator.evaluate(params)`: Evaluate parameters and return result

## Run Examples

```bash
cargo run --example basic_usage
```

## Run Tests

```bash
cargo test
```

## License

This project is dual-licensed under MIT or Apache-2.0.

```
A JSON-based configuration expression processor.
