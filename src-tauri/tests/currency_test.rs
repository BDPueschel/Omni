use omni_lib::providers::currency::{convert_with_rates, parse_currency_input, parse_rates_json};
use std::collections::HashMap;

#[test]
fn test_parse_rates_json() {
    let json = r#"{
        "result": "success",
        "rates": {
            "USD": 1.0,
            "EUR": 0.85,
            "GBP": 0.73,
            "JPY": 110.0
        }
    }"#;
    let rates = parse_rates_json(json);
    assert!(rates.is_some());
    let rates = rates.unwrap();
    assert_eq!(rates.get("USD"), Some(&1.0));
    assert_eq!(rates.get("EUR"), Some(&0.85));
    assert_eq!(rates.get("JPY"), Some(&110.0));
}

#[test]
fn test_parse_rates_json_invalid() {
    assert!(parse_rates_json("not json").is_none());
    assert!(parse_rates_json(r#"{"result": "error"}"#).is_none());
}

#[test]
fn test_symbol_detection_dollar() {
    let result = parse_currency_input("$100 to eur");
    assert!(result.is_some());
    let (amount, from, to) = result.unwrap();
    assert_eq!(amount, 100.0);
    assert_eq!(from, "USD");
    assert_eq!(to, "EUR");
}

#[test]
fn test_symbol_detection_euro() {
    let result = parse_currency_input("€50 to usd");
    assert!(result.is_some());
    let (amount, from, to) = result.unwrap();
    assert_eq!(amount, 50.0);
    assert_eq!(from, "EUR");
    assert_eq!(to, "USD");
}

#[test]
fn test_symbol_detection_pound() {
    let result = parse_currency_input("£200 to jpy");
    assert!(result.is_some());
    let (amount, from, _) = result.unwrap();
    assert_eq!(amount, 200.0);
    assert_eq!(from, "GBP");
}

#[test]
fn test_code_to_code() {
    let result = parse_currency_input("100 usd to eur");
    assert!(result.is_some());
    let (amount, from, to) = result.unwrap();
    assert_eq!(amount, 100.0);
    assert_eq!(from, "USD");
    assert_eq!(to, "EUR");
}

#[test]
fn test_code_in_code() {
    let result = parse_currency_input("50 gbp in jpy");
    assert!(result.is_some());
    let (_, from, to) = result.unwrap();
    assert_eq!(from, "GBP");
    assert_eq!(to, "JPY");
}

#[test]
fn test_conversion_math() {
    let mut rates = HashMap::new();
    rates.insert("USD".to_string(), 1.0);
    rates.insert("EUR".to_string(), 0.85);
    rates.insert("GBP".to_string(), 0.73);

    // 100 USD to EUR: 100 / 1.0 * 0.85 = 85.0
    let result = convert_with_rates(100.0, "USD", "EUR", &rates);
    assert!(result.is_some());
    assert!((result.unwrap() - 85.0).abs() < 0.01);

    // 100 EUR to USD: 100 / 0.85 * 1.0 = ~117.65
    let result = convert_with_rates(100.0, "EUR", "USD", &rates);
    assert!(result.is_some());
    assert!((result.unwrap() - 117.647).abs() < 0.01);

    // 100 GBP to EUR: 100 / 0.73 * 0.85 = ~116.44
    let result = convert_with_rates(100.0, "GBP", "EUR", &rates);
    assert!(result.is_some());
    assert!((result.unwrap() - 116.44).abs() < 0.1);
}

#[test]
fn test_invalid_currency_code() {
    assert!(parse_currency_input("100 xyz to abc").is_none());
}

#[test]
fn test_plain_text_no_match() {
    assert!(parse_currency_input("hello world").is_none());
}

#[test]
fn test_decimal_amount() {
    let result = parse_currency_input("50.5 usd to eur");
    assert!(result.is_some());
    let (amount, _, _) = result.unwrap();
    assert_eq!(amount, 50.5);
}
