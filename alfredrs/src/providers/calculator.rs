//! Quick maths — Alfred Calculator.

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;

pub struct CalculatorProvider;

pub fn evaluate(expr: &str) -> Option<f64> {
    let expr = expr.trim();
    if expr.is_empty() {
        return None;
    }
    // Reject obvious non-math to avoid false positives.
    if !expr
        .chars()
        .any(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '%' | '^'))
    {
        return None;
    }
    meval::eval_str(expr).ok()
}

pub fn looks_like_math(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let has_digit = s.chars().any(|c| c.is_ascii_digit());
    let has_op = s.chars().any(|c| matches!(c, '+' | '-' | '*' | '/' | '%' | '^' | '('));
    has_digit && has_op
}

impl Provider for CalculatorProvider {
    fn name(&self) -> &'static str {
        "calculator"
    }

    fn search(&self, query: &Query, _config: &Config) -> Vec<ResultItem> {
        let expr = match query.keyword.as_deref() {
            Some("=") => query.argument.as_str(),
            Some(_) => return Vec::new(),
            None if looks_like_math(query.raw.trim()) => query.raw.trim(),
            None => return Vec::new(),
        };
        let Some(value) = evaluate(expr) else {
            return Vec::new();
        };
        let rendered = if value.fract() == 0.0 && value.abs() < 1e15 {
            format!("{value:.0}")
        } else {
            format!("{value}")
        };
        vec![ResultItem::new(
            format!("calc:{expr}"),
            rendered.clone(),
            ItemKind::Calculator,
        )
        .with_subtitle(format!("{expr} = {rendered}"))
        .with_score(10_000)
        .with_actions(vec![
            Action::CopyText(rendered.clone()),
            Action::ShowLargeType(rendered),
        ])]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_expression() {
        assert_eq!(evaluate("2+2"), Some(4.0));
        assert!((evaluate("10/4").unwrap() - 2.5).abs() < 1e-9);
    }

    #[test]
    fn detects_math() {
        assert!(looks_like_math("1+2*3"));
        assert!(!looks_like_math("firefox"));
    }
}
