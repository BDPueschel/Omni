use super::{ResultAction, SearchResult};
use regex::Regex;
use std::sync::OnceLock;

pub struct MathProvider;

fn func_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[a-z]+\(").unwrap())
}

impl MathProvider {
    pub fn is_math_expression(input: &str) -> bool {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return false;
        }
        let starts_valid = trimmed.starts_with(|c: char| c.is_ascii_digit() || c == '(')
            || trimmed.starts_with("sqrt")
            || trimmed.starts_with("sin")
            || trimmed.starts_with("cos")
            || trimmed.starts_with("tan")
            || trimmed.starts_with("abs")
            || trimmed.starts_with("ln")
            || trimmed.starts_with("log")
            || trimmed.starts_with("exp");

        if !starts_valid {
            return false;
        }

        let has_operator = trimmed.contains('+')
            || trimmed.contains('-')
            || trimmed.contains('*')
            || trimmed.contains('/')
            || trimmed.contains('^')
            || trimmed.contains('%');

        let has_function = func_regex().is_match(trimmed);

        has_operator || has_function
    }

    pub fn evaluate(input: &str) -> Vec<SearchResult> {
        if !Self::is_math_expression(input) {
            return vec![];
        }

        match meval::eval_str(input) {
            Ok(result) => {
                let formatted = if result.fract() == 0.0 && result.abs() < 1e15 {
                    format!("{}", result as i64)
                } else {
                    format!("{:.10}", result)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string()
                };

                vec![SearchResult {
                    category: "Math".to_string(),
                    title: formatted.clone(),
                    subtitle: format!("{} =", input.trim()),
                    action: ResultAction::Copy { text: formatted },
                    icon: "calculator".to_string(),
                }]
            }
            Err(_) => vec![],
        }
    }
}
