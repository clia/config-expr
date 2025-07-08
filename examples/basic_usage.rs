use clia_config_expr::{evaluate_json, validate_json, ConfigEvaluator, RuleResult};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 示例1: 基本用法 - 直接从JSON字符串评估
    println!("=== 示例1: 基本用法 ===");

    let json_rules = r#"
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
                    "or": [
                        { "field": "platform", "op": "equals", "value": "MT9950" },
                        { "field": "platform", "op": "equals", "value": "MT9638" }
                    ]
                },
                "then": "chip_mt"
            },
            {
                "if": {
                    "field": "platform",
                    "op": "prefix",
                    "value": "Hi"
                },
                "then": "chip_hi"
            }
        ],
        "fallback": "default_chip"
    }
    "#;

    // 验证规则是否合法
    println!("验证规则...");
    validate_json(json_rules)?;
    println!("✓ 规则验证通过");

    // 测试不同的参数组合
    let test_cases = vec![
        (
            vec![("platform", "RTD-2000"), ("region", "CN")],
            "应该匹配 chip_rtd_cn",
        ),
        (vec![("platform", "MT9950")], "应该匹配 chip_mt"),
        (vec![("platform", "Hi3516")], "应该匹配 chip_hi"),
        (vec![("platform", "Unknown")], "应该使用 fallback"),
    ];

    for (params_vec, description) in test_cases {
        let mut params = HashMap::new();
        for (key, value) in params_vec {
            params.insert(key.to_string(), value.to_string());
        }

        let result = evaluate_json(json_rules, &params)?;
        println!("测试: {} -> {:?}", description, result);
    }

    // 示例2: JSON对象结果
    println!("\n=== 示例2: JSON对象结果 ===");

    let json_with_object = r#"
    {
        "rules": [
            {
                "if": {
                    "field": "platform",
                    "op": "equals",
                    "value": "RTD"
                },
                "then": {
                    "chip": "rtd",
                    "config": {
                        "memory": "2GB",
                        "cpu": "ARM",
                        "features": ["wifi", "bluetooth"]
                    }
                }
            }
        ]
    }
    "#;

    let mut params = HashMap::new();
    params.insert("platform".to_string(), "RTD".to_string());

    let result = evaluate_json(json_with_object, &params)?;
    if let Some(RuleResult::Object(obj)) = result {
        println!("匹配到JSON对象结果:");
        println!("  chip: {}", obj["chip"]);
        println!("  memory: {}", obj["config"]["memory"]);
        println!("  cpu: {}", obj["config"]["cpu"]);
        println!("  features: {:?}", obj["config"]["features"]);
    }

    // 示例3: 使用ConfigEvaluator结构体
    println!("\n=== 示例3: 使用ConfigEvaluator ===");

    let evaluator = ConfigEvaluator::from_json(json_rules)?;

    let mut params = HashMap::new();
    params.insert("platform".to_string(), "Hi3516DV300".to_string());

    let result = evaluator.evaluate(&params);
    println!("Hi3516DV300 -> {:?}", result);

    // 示例4: 正则表达式匹配
    println!("\n=== 示例4: 正则表达式匹配 ===");

    let regex_rules = r#"
    {
        "rules": [
            {
                "if": {
                    "field": "version",
                    "op": "regex",
                    "value": "^v\\d+\\.\\d+\\.\\d+$"
                },
                "then": "valid_version"
            }
        ],
        "fallback": "invalid_version"
    }
    "#;

    let test_versions = vec!["v1.2.3", "v10.0.1", "1.2.3", "v1.2"];

    for version in test_versions {
        let mut params = HashMap::new();
        params.insert("version".to_string(), version.to_string());

        let result = evaluate_json(regex_rules, &params)?;
        println!("版本 {} -> {:?}", version, result);
    }

    // 示例5: 复杂嵌套条件
    println!("\n=== 示例5: 复杂嵌套条件 ===");

    let complex_rules = r#"
    {
        "rules": [
            {
                "if": {
                    "and": [
                        {
                            "or": [
                                { "field": "platform", "op": "prefix", "value": "Hi" },
                                { "field": "platform", "op": "prefix", "value": "MT" }
                            ]
                        },
                        { "field": "region", "op": "equals", "value": "CN" },
                        { "field": "env", "op": "contains", "value": "prod" }
                    ]
                },
                "then": {
                    "chip_type": "cn_production",
                    "optimization": "high"
                }
            }
        ]
    }
    "#;

    let mut params = HashMap::new();
    params.insert("platform".to_string(), "Hi3516".to_string());
    params.insert("region".to_string(), "CN".to_string());
    params.insert("env".to_string(), "production".to_string());

    let result = evaluate_json(complex_rules, &params)?;
    println!("复杂条件匹配结果: {:?}", result);

    // 示例6: 数值比较操作符
    println!("\n=== 示例6: 数值比较操作符 ===");

    let numeric_rules = r#"
    {
        "rules": [
            {
                "if": {
                    "field": "score",
                    "op": "ge",
                    "value": "90"
                },
                "then": "excellent"
            },
            {
                "if": {
                    "field": "score",
                    "op": "ge",
                    "value": "80"
                },
                "then": "good"
            },
            {
                "if": {
                    "field": "score",
                    "op": "ge",
                    "value": "60"
                },
                "then": "pass"
            }
        ],
        "fallback": "fail"
    }
    "#;

    let test_scores = vec!["95", "85", "75", "50", "not_a_number"];

    for score in test_scores {
        let mut params = HashMap::new();
        params.insert("score".to_string(), score.to_string());

        let result = evaluate_json(numeric_rules, &params)?;
        println!("分数 {} -> {:?}", score, result);
    }

    // 示例7: 混合条件（字符串和数值）
    println!("\n=== 示例7: 混合条件 ===");

    let mixed_rules = r#"
    {
        "rules": [
            {
                "if": {
                    "and": [
                        { "field": "category", "op": "equals", "value": "premium" },
                        { "field": "price", "op": "gt", "value": "100.0" },
                        { "field": "rating", "op": "ge", "value": "4.5" }
                    ]
                },
                "then": {
                    "tier": "premium",
                    "discount": 0.1,
                    "features": ["priority_support", "advanced_analytics"]
                }
            },
            {
                "if": {
                    "and": [
                        { "field": "category", "op": "equals", "value": "standard" },
                        { "field": "price", "op": "le", "value": "50.0" }
                    ]
                },
                "then": {
                    "tier": "standard",
                    "discount": 0.05
                }
            }
        ]
    }
    "#;

    let mut params = HashMap::new();
    params.insert("category".to_string(), "premium".to_string());
    params.insert("price".to_string(), "150.99".to_string());
    params.insert("rating".to_string(), "4.8".to_string());

    let result = evaluate_json(mixed_rules, &params)?;
    println!("混合条件匹配结果: {:?}", result);

    Ok(())
}
