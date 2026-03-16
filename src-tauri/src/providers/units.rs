use super::{ResultAction, SearchResult};
use regex::Regex;
use std::sync::OnceLock;

pub struct UnitProvider;

fn unit_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^([\d.]+)\s*([a-zA-Z°/%]+)\s+(?:in|to)\s+([a-zA-Z°/%]+)$").unwrap()
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum UnitCategory {
    Length,
    Weight,
    Temperature,
    Volume,
    Data,
    Speed,
    Time,
}

/// Returns (canonical_name, category, factor_to_base) for a unit string.
/// For temperature, factor is unused (special formulas).
fn normalize_unit(raw: &str) -> Option<(&'static str, UnitCategory, f64)> {
    let lower = raw.to_lowercase();
    let s = lower.as_str();

    // Length — base: meters
    let length = match s {
        "mm" | "millimeter" | "millimeters" | "millimetre" | "millimetres" => Some(("mm", 0.001)),
        "cm" | "centimeter" | "centimeters" | "centimetre" | "centimetres" => Some(("cm", 0.01)),
        "m" | "meter" | "meters" | "metre" | "metres" => Some(("m", 1.0)),
        "km" | "kilometer" | "kilometers" | "kilometre" | "kilometres" => Some(("km", 1000.0)),
        "in" | "inch" | "inches" => Some(("in", 0.0254)),
        "ft" | "foot" | "feet" => Some(("ft", 0.3048)),
        "yd" | "yard" | "yards" => Some(("yd", 0.9144)),
        "mi" | "mile" | "miles" => Some(("mi", 1609.344)),
        _ => None,
    };
    if let Some((name, factor)) = length {
        return Some((name, UnitCategory::Length, factor));
    }

    // Weight — base: grams
    let weight = match s {
        "mg" | "milligram" | "milligrams" => Some(("mg", 0.001)),
        "g" | "gram" | "grams" => Some(("g", 1.0)),
        "kg" | "kilogram" | "kilograms" | "kilo" | "kilos" => Some(("kg", 1000.0)),
        "lb" | "lbs" | "pound" | "pounds" => Some(("lb", 453.592)),
        "oz" | "ounce" | "ounces" => Some(("oz", 28.3495)),
        _ => None,
    };
    if let Some((name, factor)) = weight {
        return Some((name, UnitCategory::Weight, factor));
    }

    // Temperature — base: celsius (factor unused)
    let temp = match s {
        "c" | "celsius" => Some("c"),
        "f" | "fahrenheit" => Some("f"),
        "k" | "kelvin" => Some("k"),
        _ => None,
    };
    if let Some(name) = temp {
        return Some((name, UnitCategory::Temperature, 0.0));
    }

    // Volume — base: milliliters
    let volume = match s {
        "ml" | "milliliter" | "milliliters" | "millilitre" | "millilitres" => Some(("ml", 1.0)),
        "l" | "liter" | "liters" | "litre" | "litres" => Some(("l", 1000.0)),
        "gal" | "gallon" | "gallons" => Some(("gal", 3785.41)),
        "floz" | "fl oz" => Some(("floz", 29.5735)),
        "cup" | "cups" => Some(("cup", 236.588)),
        "pt" | "pint" | "pints" => Some(("pt", 473.176)),
        "qt" | "quart" | "quarts" => Some(("qt", 946.353)),
        _ => None,
    };
    if let Some((name, factor)) = volume {
        return Some((name, UnitCategory::Volume, factor));
    }

    // Data — base: bytes
    let data = match s {
        "b" | "byte" | "bytes" => Some(("B", 1.0)),
        "kb" | "kilobyte" | "kilobytes" => Some(("KB", 1024.0)),
        "mb" | "megabyte" | "megabytes" => Some(("MB", 1_048_576.0)),
        "gb" | "gigabyte" | "gigabytes" => Some(("GB", 1_073_741_824.0)),
        "tb" | "terabyte" | "terabytes" => Some(("TB", 1_099_511_627_776.0)),
        "pb" | "petabyte" | "petabytes" => Some(("PB", 1_125_899_906_842_624.0)),
        _ => None,
    };
    if let Some((name, factor)) = data {
        return Some((name, UnitCategory::Data, factor));
    }

    // Speed — base: m/s
    let speed = match s {
        "mph" => Some(("mph", 0.44704)),
        "kmh" | "kph" | "kmph" => Some(("km/h", 0.277778)),
        "ms" => Some(("m/s", 1.0)),
        "knots" | "knot" | "kn" | "kt" => Some(("knots", 0.514444)),
        _ => None,
    };
    if let Some((name, factor)) = speed {
        return Some((name, UnitCategory::Speed, factor));
    }

    // Time — base: seconds
    let time = match s {
        "ms" | "millisecond" | "milliseconds" => {
            // "ms" conflicts with speed m/s — but in time context it's milliseconds.
            // We handle this by checking category compatibility later.
            // For now, if we reach here it means speed didn't match (it did above).
            // So this won't actually be reached for "ms".
            Some(("ms", 0.001))
        }
        "s" | "sec" | "second" | "seconds" => Some(("s", 1.0)),
        "min" | "minute" | "minutes" => Some(("min", 60.0)),
        "hr" | "hour" | "hours" => Some(("hr", 3600.0)),
        "day" | "days" => Some(("day", 86400.0)),
        "week" | "weeks" => Some(("week", 604800.0)),
        "year" | "years" | "yr" => Some(("year", 31_557_600.0)), // Julian year
        _ => None,
    };
    if let Some((name, factor)) = time {
        return Some((name, UnitCategory::Time, factor));
    }

    None
}

/// Special handling: "ms" can be speed (m/s) or time (milliseconds).
/// Disambiguate based on the other unit in the conversion.
fn normalize_unit_with_context(raw: &str, other_raw: &str) -> Option<(&'static str, UnitCategory, f64)> {
    let lower = raw.to_lowercase();
    let other_lower = other_raw.to_lowercase();

    // "ms" disambiguation
    if lower == "ms" {
        // If the other unit is a time unit, treat "ms" as milliseconds
        let other_norm = normalize_unit(&other_lower);
        if let Some((_, UnitCategory::Time, _)) = other_norm {
            return Some(("ms", UnitCategory::Time, 0.001));
        }
        // Otherwise treat as m/s (speed)
        if let Some((_, UnitCategory::Speed, _)) = other_norm {
            return Some(("m/s", UnitCategory::Speed, 1.0));
        }
        // Default: speed
        return Some(("m/s", UnitCategory::Speed, 1.0));
    }

    normalize_unit(raw)
}

fn convert_temperature(value: f64, from: &str, to: &str) -> Option<f64> {
    // Convert to Celsius first
    let celsius = match from {
        "c" => value,
        "f" => (value - 32.0) * 5.0 / 9.0,
        "k" => value - 273.15,
        _ => return None,
    };
    // Convert from Celsius to target
    match to {
        "c" => Some(celsius),
        "f" => Some(celsius * 9.0 / 5.0 + 32.0),
        "k" => Some(celsius + 273.15),
        _ => None,
    }
}

fn format_number(value: f64) -> String {
    // Round to 6 significant decimal places to avoid floating-point noise
    let rounded = (value * 1_000_000.0).round() / 1_000_000.0;

    if rounded.fract() == 0.0 && rounded.abs() < 1e15 {
        // Integer — format with commas
        format_with_commas(rounded as i64)
    } else {
        // Decimal — strip trailing zeros, add commas to integer part
        let s = format!("{:.6}", rounded);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        // Split into integer and decimal parts for comma formatting
        if let Some(dot_pos) = s.find('.') {
            let int_part: i64 = s[..dot_pos].parse().unwrap_or(0);
            let dec_part = &s[dot_pos..];
            format!("{}{}", format_with_commas(int_part), dec_part)
        } else {
            let int_part: i64 = s.parse().unwrap_or(0);
            format_with_commas(int_part)
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

impl UnitProvider {
    pub fn evaluate(input: &str) -> Vec<SearchResult> {
        let trimmed = input.trim();
        let caps = match unit_regex().captures(trimmed) {
            Some(c) => c,
            None => return vec![],
        };

        let value: f64 = match caps[1].parse() {
            Ok(v) => v,
            Err(_) => return vec![],
        };

        let from_raw = &caps[2];
        let to_raw = &caps[3];

        let from = match normalize_unit_with_context(from_raw, to_raw) {
            Some(u) => u,
            None => return vec![],
        };
        let to = match normalize_unit_with_context(to_raw, from_raw) {
            Some(u) => u,
            None => return vec![],
        };

        // Units must be in the same category
        if from.1 != to.1 {
            return vec![];
        }

        let result = if from.1 == UnitCategory::Temperature {
            match convert_temperature(value, from.0, to.0) {
                Some(r) => r,
                None => return vec![],
            }
        } else {
            // Convert: value * from_factor / to_factor
            let base_value = value * from.2;
            base_value / to.2
        };

        let formatted = format_number(result);
        let title = format!("{} {}", formatted, to.0);
        let subtitle = format!("{} {} = {} {}", format_number(value), from.0, formatted, to.0);

        vec![SearchResult {
            category: "Math".to_string(),
            title,
            subtitle,
            action: ResultAction::Copy {
                text: format_number(result),
            },
            icon: "unit".to_string(),
            size: None,
            date_modified: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_with_commas() {
        assert_eq!(format_with_commas(1234), "1,234");
        assert_eq!(format_with_commas(1234567), "1,234,567");
        assert_eq!(format_with_commas(42), "42");
    }

    #[test]
    fn test_normalize_unit() {
        assert!(normalize_unit("km").is_some());
        assert!(normalize_unit("miles").is_some());
        assert!(normalize_unit("xyz").is_none());
    }
}
