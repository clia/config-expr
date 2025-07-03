use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// 配置表达式错误类型
#[derive(Error, Debug)]
pub enum ConfigExprError {
    #[error("Invalid operator: {0}")]
    InvalidOperator(String),
    #[error("Field not found: {0}")]
    FieldNotFound(String),
    #[error("Regex compilation error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// 操作符枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Operator {
    Equals,
    Contains,
    Prefix,
    Suffix,
    Regex,
}

impl Operator {
    /// 验证操作符是否有效
    pub fn is_valid(&self) -> bool {
        matches!(
            self,
            Operator::Equals
                | Operator::Contains
                | Operator::Prefix
                | Operator::Suffix
                | Operator::Regex
        )
    }
}

/// 条件表达式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Condition {
    /// 简单条件：字段比较
    Simple {
        field: String,
        op: Operator,
        value: String,
    },
    /// AND 条件：所有子条件都必须满足
    And { and: Vec<Condition> },
    /// OR 条件：至少一个子条件满足
    Or { or: Vec<Condition> },
}

/// 规则的返回值，支持字符串或JSON对象
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleResult {
    String(String),
    Object(serde_json::Value),
}

/// 单个规则定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    #[serde(rename = "if")]
    pub condition: Condition,
    #[serde(rename = "then")]
    pub result: RuleResult,
}

/// 配置规则集
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRules {
    pub rules: Vec<Rule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<RuleResult>,
}

/// 配置表达式评估器
pub struct ConfigEvaluator {
    rules: ConfigRules,
}

impl ConfigEvaluator {
    /// 创建新的评估器
    pub fn new(rules: ConfigRules) -> Result<Self, ConfigExprError> {
        // 验证规则集
        Self::validate_rules(&rules)?;
        Ok(Self { rules })
    }

    /// 从JSON字符串创建评估器
    pub fn from_json(json: &str) -> Result<Self, ConfigExprError> {
        let rules: ConfigRules = serde_json::from_str(json)?;
        Self::new(rules)
    }

    /// 评估请求参数，返回匹配的结果
    pub fn evaluate(&self, params: &HashMap<String, String>) -> Option<RuleResult> {
        for rule in &self.rules.rules {
            if self.evaluate_condition(&rule.condition, params) {
                return Some(rule.result.clone());
            }
        }
        self.rules.fallback.clone()
    }

    /// 评估单个条件
    fn evaluate_condition(&self, condition: &Condition, params: &HashMap<String, String>) -> bool {
        match condition {
            Condition::Simple { field, op, value } => {
                self.evaluate_simple_condition(field, op, value, params)
            }
            Condition::And { and } => and.iter().all(|cond| self.evaluate_condition(cond, params)),
            Condition::Or { or } => or.iter().any(|cond| self.evaluate_condition(cond, params)),
        }
    }

    /// 评估简单条件
    fn evaluate_simple_condition(
        &self,
        field: &str,
        op: &Operator,
        value: &str,
        params: &HashMap<String, String>,
    ) -> bool {
        let field_value = match params.get(field) {
            Some(v) => v,
            None => return false,
        };

        match op {
            Operator::Equals => field_value == value,
            Operator::Contains => field_value.contains(value),
            Operator::Prefix => field_value.starts_with(value),
            Operator::Suffix => field_value.ends_with(value),
            Operator::Regex => {
                match Regex::new(value) {
                    Ok(regex) => regex.is_match(field_value),
                    Err(_) => false, // 正则表达式无效时返回false
                }
            }
        }
    }

    /// 验证规则集是否合法
    fn validate_rules(rules: &ConfigRules) -> Result<(), ConfigExprError> {
        if rules.rules.is_empty() {
            return Err(ConfigExprError::ValidationError(
                "Rules cannot be empty".to_string(),
            ));
        }

        for (index, rule) in rules.rules.iter().enumerate() {
            Self::validate_condition(&rule.condition, index)?;
        }

        Ok(())
    }

    /// 验证条件是否合法
    fn validate_condition(condition: &Condition, rule_index: usize) -> Result<(), ConfigExprError> {
        match condition {
            Condition::Simple { field, op, value } => {
                if field.is_empty() {
                    return Err(ConfigExprError::ValidationError(format!(
                        "Field name cannot be empty in rule {}",
                        rule_index
                    )));
                }

                if !op.is_valid() {
                    return Err(ConfigExprError::InvalidOperator(format!("{:?}", op)));
                }

                // 验证正则表达式
                if matches!(op, Operator::Regex) {
                    Regex::new(value).map_err(|e| {
                        ConfigExprError::ValidationError(format!(
                            "Invalid regex '{}' in rule {}: {}",
                            value, rule_index, e
                        ))
                    })?;
                }
            }
            Condition::And { and } => {
                if and.is_empty() {
                    return Err(ConfigExprError::ValidationError(format!(
                        "AND condition cannot be empty in rule {}",
                        rule_index
                    )));
                }
                for cond in and {
                    Self::validate_condition(cond, rule_index)?;
                }
            }
            Condition::Or { or } => {
                if or.is_empty() {
                    return Err(ConfigExprError::ValidationError(format!(
                        "OR condition cannot be empty in rule {}",
                        rule_index
                    )));
                }
                for cond in or {
                    Self::validate_condition(cond, rule_index)?;
                }
            }
        }
        Ok(())
    }
}

/// 便利方法：直接从JSON字符串评估
pub fn evaluate_json(
    json: &str,
    params: &HashMap<String, String>,
) -> Result<Option<RuleResult>, ConfigExprError> {
    let evaluator = ConfigEvaluator::from_json(json)?;
    Ok(evaluator.evaluate(params))
}

/// 便利方法：验证JSON规则是否合法
pub fn validate_json(json: &str) -> Result<(), ConfigExprError> {
    let rules: ConfigRules = serde_json::from_str(json)?;
    ConfigEvaluator::validate_rules(&rules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_condition() {
        let json = r#"
        {
            "rules": [
                {
                    "if": {
                        "field": "platform",
                        "op": "equals",
                        "value": "RTD"
                    },
                    "then": "chip_rtd"
                }
            ]
        }
        "#;

        let mut params = HashMap::new();
        params.insert("platform".to_string(), "RTD".to_string());

        let result = evaluate_json(json, &params).unwrap();
        assert!(result.is_some());

        if let Some(RuleResult::String(s)) = result {
            assert_eq!(s, "chip_rtd");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_and_condition() {
        let json = r#"
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
                }
            ]
        }
        "#;

        let mut params = HashMap::new();
        params.insert("platform".to_string(), "RTD-2000".to_string());
        params.insert("region".to_string(), "CN".to_string());

        let result = evaluate_json(json, &params).unwrap();
        assert!(result.is_some());

        if let Some(RuleResult::String(s)) = result {
            assert_eq!(s, "chip_rtd_cn");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_or_condition() {
        let json = r#"
        {
            "rules": [
                {
                    "if": {
                        "or": [
                            { "field": "platform", "op": "equals", "value": "MT9950" },
                            { "field": "platform", "op": "equals", "value": "MT9638" }
                        ]
                    },
                    "then": "chip_mt"
                }
            ]
        }
        "#;

        let mut params = HashMap::new();
        params.insert("platform".to_string(), "MT9950".to_string());

        let result = evaluate_json(json, &params).unwrap();
        assert!(result.is_some());

        if let Some(RuleResult::String(s)) = result {
            assert_eq!(s, "chip_mt");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_prefix_condition() {
        let json = r#"
        {
            "rules": [
                {
                    "if": {
                        "field": "platform",
                        "op": "prefix",
                        "value": "Hi"
                    },
                    "then": "chip_hi"
                }
            ]
        }
        "#;

        let mut params = HashMap::new();
        params.insert("platform".to_string(), "Hi3516".to_string());

        let result = evaluate_json(json, &params).unwrap();
        assert!(result.is_some());

        if let Some(RuleResult::String(s)) = result {
            assert_eq!(s, "chip_hi");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_json_object_result() {
        let json = r#"
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
                            "cpu": "ARM"
                        }
                    }
                }
            ]
        }
        "#;

        let mut params = HashMap::new();
        params.insert("platform".to_string(), "RTD".to_string());

        let result = evaluate_json(json, &params).unwrap();
        assert!(result.is_some());

        if let Some(RuleResult::Object(obj)) = result {
            assert_eq!(obj["chip"], "rtd");
            assert_eq!(obj["config"]["memory"], "2GB");
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_fallback() {
        let json = r#"
        {
            "rules": [
                {
                    "if": {
                        "field": "platform",
                        "op": "equals",
                        "value": "RTD"
                    },
                    "then": "chip_rtd"
                }
            ],
            "fallback": "default_chip"
        }
        "#;

        let mut params = HashMap::new();
        params.insert("platform".to_string(), "Unknown".to_string());

        let result = evaluate_json(json, &params).unwrap();
        assert!(result.is_some());

        if let Some(RuleResult::String(s)) = result {
            assert_eq!(s, "default_chip");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_no_match_no_fallback() {
        let json = r#"
        {
            "rules": [
                {
                    "if": {
                        "field": "platform",
                        "op": "equals",
                        "value": "RTD"
                    },
                    "then": "chip_rtd"
                }
            ]
        }
        "#;

        let mut params = HashMap::new();
        params.insert("platform".to_string(), "Unknown".to_string());

        let result = evaluate_json(json, &params).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_regex_condition() {
        let json = r#"
        {
            "rules": [
                {
                    "if": {
                        "field": "platform",
                        "op": "regex",
                        "value": "^Hi\\d+"
                    },
                    "then": "chip_hi"
                }
            ]
        }
        "#;

        let mut params = HashMap::new();
        params.insert("platform".to_string(), "Hi3516".to_string());

        let result = evaluate_json(json, &params).unwrap();
        assert!(result.is_some());

        if let Some(RuleResult::String(s)) = result {
            assert_eq!(s, "chip_hi");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_suffix_condition() {
        let json = r#"
        {
            "rules": [
                {
                    "if": {
                        "field": "platform",
                        "op": "suffix",
                        "value": "Pro"
                    },
                    "then": "chip_pro"
                }
            ]
        }
        "#;

        let mut params = HashMap::new();
        params.insert("platform".to_string(), "RTD-Pro".to_string());

        let result = evaluate_json(json, &params).unwrap();
        assert!(result.is_some());

        if let Some(RuleResult::String(s)) = result {
            assert_eq!(s, "chip_pro");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_validation_empty_rules() {
        let json = r#"
        {
            "rules": []
        }
        "#;

        let result = validate_json(json);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Rules cannot be empty")
        );
    }

    #[test]
    fn test_validation_empty_field() {
        let json = r#"
        {
            "rules": [
                {
                    "if": {
                        "field": "",
                        "op": "equals",
                        "value": "RTD"
                    },
                    "then": "chip_rtd"
                }
            ]
        }
        "#;

        let result = validate_json(json);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Field name cannot be empty")
        );
    }

    #[test]
    fn test_validation_invalid_regex() {
        let json = r#"
        {
            "rules": [
                {
                    "if": {
                        "field": "platform",
                        "op": "regex",
                        "value": "[invalid"
                    },
                    "then": "chip_rtd"
                }
            ]
        }
        "#;

        let result = validate_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid regex"));
    }

    #[test]
    fn test_validation_empty_and_condition() {
        let json = r#"
        {
            "rules": [
                {
                    "if": {
                        "and": []
                    },
                    "then": "chip_rtd"
                }
            ]
        }
        "#;

        let result = validate_json(json);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("AND condition cannot be empty")
        );
    }

    #[test]
    fn test_validation_empty_or_condition() {
        let json = r#"
        {
            "rules": [
                {
                    "if": {
                        "or": []
                    },
                    "then": "chip_rtd"
                }
            ]
        }
        "#;

        let result = validate_json(json);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("OR condition cannot be empty")
        );
    }

    #[test]
    fn test_complex_nested_conditions() {
        let json = r#"
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
                            { "field": "region", "op": "equals", "value": "CN" }
                        ]
                    },
                    "then": "chip_cn"
                }
            ]
        }
        "#;

        let mut params = HashMap::new();
        params.insert("platform".to_string(), "Hi3516".to_string());
        params.insert("region".to_string(), "CN".to_string());

        let result = evaluate_json(json, &params).unwrap();
        assert!(result.is_some());

        if let Some(RuleResult::String(s)) = result {
            assert_eq!(s, "chip_cn");
        } else {
            panic!("Expected string result");
        }
    }
}
