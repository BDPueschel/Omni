use omni_lib::providers::math::MathProvider;

#[test]
fn test_simple_addition() {
    let results = MathProvider::evaluate("2 + 3");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "5");
    assert_eq!(results[0].category, "Math");
}

#[test]
fn test_complex_expression() {
    let results = MathProvider::evaluate("(10 + 5) * 2");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "30");
}

#[test]
fn test_not_math_plain_text() {
    let results = MathProvider::evaluate("hello world");
    assert!(results.is_empty());
}

#[test]
fn test_not_math_file_path() {
    let results = MathProvider::evaluate("C:\\Users\\test");
    assert!(results.is_empty());
}

#[test]
fn test_not_math_single_number() {
    let results = MathProvider::evaluate("42");
    assert!(results.is_empty());
}

#[test]
fn test_sqrt_function() {
    let results = MathProvider::evaluate("sqrt(16)");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "4");
}

#[test]
fn test_decimal_result() {
    let results = MathProvider::evaluate("10 / 3");
    assert_eq!(results.len(), 1);
    assert!(results[0].title.starts_with("3.333"));
}

#[test]
fn test_heuristic_rejects_no_operator() {
    let results = MathProvider::evaluate("123");
    assert!(results.is_empty());
}
