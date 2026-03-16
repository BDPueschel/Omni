use super::{ResultAction, SearchResult};
use regex::Regex;
use std::sync::OnceLock;

pub struct ColorProvider;

/// Internal RGB representation (0–255 per channel, optional alpha 0–255).
struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: Option<u8>,
}

/// Internal HSL representation (h: 0–360, s: 0–100, l: 0–100).
struct Hsl {
    h: f64,
    s: f64,
    l: f64,
}

fn hex6_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^#([0-9a-f]{6})$").unwrap())
}

fn hex8_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^#([0-9a-f]{8})$").unwrap())
}

fn hex3_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)^#([0-9a-f]{3})$").unwrap())
}

fn rgb_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^rgb\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*\)$").unwrap()
    })
}

fn hsl_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)^hsl\(\s*(\d{1,3})\s*,\s*(\d{1,3})%\s*,\s*(\d{1,3})%\s*\)$").unwrap()
    })
}

impl ColorProvider {
    pub fn evaluate(input: &str) -> Vec<SearchResult> {
        let trimmed = input.trim();
        if let Some(rgba) = Self::parse(trimmed) {
            let hex6 = format!("#{:02X}{:02X}{:02X}", rgba.r, rgba.g, rgba.b);
            let hsl = rgb_to_hsl(rgba.r, rgba.g, rgba.b);

            let rgb_str = format!("rgb({}, {}, {})", rgba.r, rgba.g, rgba.b);
            let hsl_str = format!(
                "hsl({}, {}%, {}%)",
                hsl.h.round() as i32,
                hsl.s.round() as i32,
                hsl.l.round() as i32,
            );

            let subtitle = if let Some(a) = rgba.a {
                let pct = (a as f64 / 255.0 * 100.0).round() as i32;
                format!("{} \u{00B7} {} \u{00B7} alpha {}%", rgb_str, hsl_str, pct)
            } else {
                format!("{} \u{00B7} {}", rgb_str, hsl_str)
            };

            vec![SearchResult {
                category: "Color".to_string(),
                title: hex6.clone(),
                subtitle,
                action: ResultAction::Copy { text: hex6.clone() },
                icon: format!("color:{}", hex6),
                size: None,
                date_modified: None,
            }]
        } else {
            vec![]
        }
    }

    fn parse(input: &str) -> Option<Rgba> {
        // Try hex 8-digit (with alpha) first since it's longest
        if let Some(caps) = hex8_regex().captures(input) {
            let hex = &caps[1];
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            return Some(Rgba { r, g, b, a: Some(a) });
        }

        // Hex 6-digit
        if let Some(caps) = hex6_regex().captures(input) {
            let hex = &caps[1];
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Rgba { r, g, b, a: None });
        }

        // Hex 3-digit
        if let Some(caps) = hex3_regex().captures(input) {
            let hex = &caps[1];
            let bytes: Vec<u8> = hex
                .chars()
                .map(|c| {
                    let v = u8::from_str_radix(&c.to_string(), 16).unwrap_or(0);
                    v * 16 + v // e.g. 0xF -> 0xFF
                })
                .collect();
            return Some(Rgba {
                r: bytes[0],
                g: bytes[1],
                b: bytes[2],
                a: None,
            });
        }

        // RGB
        if let Some(caps) = rgb_regex().captures(input) {
            let r: u8 = caps[1].parse().ok()?;
            let g: u8 = caps[2].parse().ok()?;
            let b: u8 = caps[3].parse().ok()?;
            return Some(Rgba { r, g, b, a: None });
        }

        // HSL
        if let Some(caps) = hsl_regex().captures(input) {
            let h: f64 = caps[1].parse().ok()?;
            let s: f64 = caps[2].parse().ok()?;
            let l: f64 = caps[3].parse().ok()?;
            if h > 360.0 || s > 100.0 || l > 100.0 {
                return None;
            }
            let (r, g, b) = hsl_to_rgb(h, s, l);
            return Some(Rgba { r, g, b, a: None });
        }

        None
    }
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> Hsl {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < 1e-10 {
        return Hsl {
            h: 0.0,
            s: 0.0,
            l: l * 100.0,
        };
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < 1e-10 {
        let mut h = (g - b) / d;
        if g < b {
            h += 6.0;
        }
        h
    } else if (max - g).abs() < 1e-10 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    Hsl {
        h: h * 60.0,
        s: s * 100.0,
        l: l * 100.0,
    }
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let s = s / 100.0;
    let l = l / 100.0;

    if s.abs() < 1e-10 {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    let h = h / 360.0;

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}
