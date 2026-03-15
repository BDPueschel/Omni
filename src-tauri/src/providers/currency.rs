use super::{ResultAction, SearchResult};
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

pub struct CurrencyProvider;

fn currency_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Matches: "100 usd to eur", "50.5 gbp in jpy", "$100 to eur", "€50 in usd"
        Regex::new(r"(?i)^([$€£¥]?)([\d.]+)\s*([a-zA-Z]{0,3})\s+(?:in|to)\s+([a-zA-Z]{3})$")
            .unwrap()
    })
}

struct RateCache {
    rates: HashMap<String, f64>,
    fetched_at: Instant,
}

fn rate_cache() -> &'static Mutex<Option<RateCache>> {
    static CACHE: OnceLock<Mutex<Option<RateCache>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

const CACHE_TTL: Duration = Duration::from_secs(6 * 3600); // 6 hours

const KNOWN_CURRENCIES: &[&str] = &[
    "USD", "EUR", "GBP", "JPY", "CAD", "AUD", "CHF", "CNY", "INR", "BRL",
    "MXN", "KRW", "SEK", "NOK", "DKK", "NZD", "SGD", "HKD", "TWD", "PLN",
    "THB", "TRY", "ZAR", "RUB", "PHP", "MYR", "IDR", "CZK", "HUF", "ILS",
    "CLP", "ARS", "COP", "PEN", "VND", "UAH", "RON", "BGN", "HRK", "ISK",
];

fn symbol_to_currency(symbol: &str) -> Option<&'static str> {
    match symbol {
        "$" => Some("USD"),
        "€" => Some("EUR"),
        "£" => Some("GBP"),
        "¥" => Some("JPY"),
        _ => None,
    }
}

fn is_valid_currency(code: &str) -> bool {
    let upper = code.to_uppercase();
    KNOWN_CURRENCIES.contains(&upper.as_str())
}

fn fetch_rates() -> Option<HashMap<String, f64>> {
    let output = std::process::Command::new("curl")
        .args([
            "-s",
            "-m", "5", // 5 second timeout
            "https://open.er-api.com/v6/latest/USD",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let body = String::from_utf8(output.stdout).ok()?;
    parse_rates_json(&body)
}

/// Parse the JSON response from the exchange rate API.
/// Exported for testing.
pub fn parse_rates_json(json: &str) -> Option<HashMap<String, f64>> {
    let parsed: serde_json::Value = serde_json::from_str(json).ok()?;
    let rates_obj = parsed.get("rates")?.as_object()?;

    let mut rates = HashMap::new();
    for (key, val) in rates_obj {
        if let Some(rate) = val.as_f64() {
            rates.insert(key.to_uppercase(), rate);
        }
    }

    if rates.is_empty() {
        None
    } else {
        Some(rates)
    }
}

fn get_rates() -> Option<HashMap<String, f64>> {
    let mut cache = rate_cache().lock().ok()?;

    // Return cached rates if still fresh
    if let Some(ref cached) = *cache {
        if cached.fetched_at.elapsed() < CACHE_TTL {
            return Some(cached.rates.clone());
        }
    }

    // Fetch fresh rates
    let rates = fetch_rates()?;
    *cache = Some(RateCache {
        rates: rates.clone(),
        fetched_at: Instant::now(),
    });
    Some(rates)
}

/// Parse input to extract (amount, from_currency, to_currency).
pub fn parse_currency_input(input: &str) -> Option<(f64, String, String)> {
    let caps = currency_regex().captures(input.trim())?;

    let symbol = &caps[1];
    let amount: f64 = caps[2].parse().ok()?;
    let code_part = caps[3].to_uppercase();
    let to_code = caps[4].to_uppercase();

    let from_code = if !symbol.is_empty() {
        // Symbol provided: $100 to EUR
        symbol_to_currency(symbol)?.to_string()
    } else if code_part.is_empty() {
        // No code and no symbol — can't determine source currency
        return None;
    } else {
        code_part
    };

    if !is_valid_currency(&from_code) || !is_valid_currency(&to_code) {
        return None;
    }

    Some((amount, from_code, to_code))
}

fn format_currency(value: f64) -> String {
    // Round to 2 decimal places for currency
    let rounded = (value * 100.0).round() / 100.0;

    if rounded.fract() == 0.0 && rounded.abs() < 1e15 {
        format_with_commas(rounded as i64)
    } else {
        let s = format!("{:.2}", rounded);
        // Add commas to integer part
        if let Some(dot_pos) = s.find('.') {
            let int_part: i64 = s[..dot_pos].parse().unwrap_or(0);
            let dec_part = &s[dot_pos..];
            format!("{}{}", format_with_commas(int_part), dec_part)
        } else {
            s
        }
    }
}

fn format_with_commas(n: i64) -> String {
    let negative = n < 0;
    let s = n.unsigned_abs().to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    let formatted: String = result.chars().rev().collect();
    if negative {
        format!("-{}", formatted)
    } else {
        formatted
    }
}

/// Convert using provided rates (for testing) or fetched rates.
pub fn convert_with_rates(
    amount: f64,
    from: &str,
    to: &str,
    rates: &HashMap<String, f64>,
) -> Option<f64> {
    // Rates are relative to USD
    let from_rate = rates.get(&from.to_uppercase())?;
    let to_rate = rates.get(&to.to_uppercase())?;

    // Convert: amount in `from` -> USD -> `to`
    let usd_amount = amount / from_rate;
    Some(usd_amount * to_rate)
}

impl CurrencyProvider {
    pub fn evaluate(input: &str) -> Vec<SearchResult> {
        let (amount, from_code, to_code) = match parse_currency_input(input) {
            Some(parsed) => parsed,
            None => return vec![],
        };

        let rates = match get_rates() {
            Some(r) => r,
            None => {
                return vec![SearchResult {
                    category: "Math".to_string(),
                    title: "Currency rates unavailable".to_string(),
                    subtitle: "Check your internet connection".to_string(),
                    action: ResultAction::Copy {
                        text: "Currency rates unavailable".to_string(),
                    },
                    icon: "currency".to_string(),
                    size: None,
                    date_modified: None,
                }];
            }
        };

        let result = match convert_with_rates(amount, &from_code, &to_code, &rates) {
            Some(r) => r,
            None => return vec![],
        };

        let formatted = format_currency(result);
        let title = format!("{} {}", formatted, to_code);
        let subtitle = format!(
            "{} {} = {} {}",
            format_currency(amount),
            from_code,
            formatted,
            to_code
        );

        vec![SearchResult {
            category: "Math".to_string(),
            title,
            subtitle,
            action: ResultAction::Copy { text: formatted },
            icon: "currency".to_string(),
            size: None,
            date_modified: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_currency_code_to_code() {
        let result = parse_currency_input("100 usd to eur");
        assert!(result.is_some());
        let (amount, from, to) = result.unwrap();
        assert_eq!(amount, 100.0);
        assert_eq!(from, "USD");
        assert_eq!(to, "EUR");
    }

    #[test]
    fn test_parse_currency_symbol() {
        let result = parse_currency_input("$100 to eur");
        assert!(result.is_some());
        let (amount, from, to) = result.unwrap();
        assert_eq!(amount, 100.0);
        assert_eq!(from, "USD");
        assert_eq!(to, "EUR");
    }

    #[test]
    fn test_parse_invalid() {
        assert!(parse_currency_input("hello world").is_none());
        assert!(parse_currency_input("100 xyz to abc").is_none());
    }
}
