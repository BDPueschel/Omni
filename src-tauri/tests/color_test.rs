use omni_lib::providers::color::ColorProvider;

#[test]
fn test_parse_hex_6digit() {
    let results = ColorProvider::evaluate("#FF5733");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "#FF5733");
    assert_eq!(results[0].category, "Color");
    assert!(results[0].subtitle.contains("rgb(255, 87, 51)"));
    assert!(results[0].icon.starts_with("color:#"));
}

#[test]
fn test_parse_hex_6digit_lowercase() {
    let results = ColorProvider::evaluate("#ff5733");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "#FF5733");
}

#[test]
fn test_parse_hex_3digit() {
    let results = ColorProvider::evaluate("#F53");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "#FF5533");
    assert!(results[0].subtitle.contains("rgb(255, 85, 51)"));
}

#[test]
fn test_parse_rgb() {
    let results = ColorProvider::evaluate("rgb(255, 87, 51)");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "#FF5733");
    assert!(results[0].subtitle.contains("rgb(255, 87, 51)"));
}

#[test]
fn test_parse_rgb_no_spaces() {
    let results = ColorProvider::evaluate("rgb(255,87,51)");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "#FF5733");
}

#[test]
fn test_parse_hsl() {
    let results = ColorProvider::evaluate("hsl(11, 100%, 60%)");
    assert_eq!(results.len(), 1);
    // HSL(11, 100%, 60%) should produce a reddish-orange
    assert!(results[0].subtitle.contains("hsl("));
    assert!(results[0].icon.starts_with("color:#"));
}

#[test]
fn test_parse_hsl_no_spaces() {
    let results = ColorProvider::evaluate("hsl(11,100%,60%)");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_parse_hex_8digit_alpha() {
    let results = ColorProvider::evaluate("#FF573380");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "#FF5733");
    assert!(results[0].subtitle.contains("alpha 50%"));
}

#[test]
fn test_non_color_returns_empty() {
    assert!(ColorProvider::evaluate("hello world").is_empty());
    assert!(ColorProvider::evaluate("12345").is_empty());
    assert!(ColorProvider::evaluate("#ZZZZZZ").is_empty());
    assert!(ColorProvider::evaluate("rgb(300, 0, 0)").is_empty()); // 300 overflows u8
}

#[test]
fn test_copy_action() {
    let results = ColorProvider::evaluate("#FF5733");
    assert_eq!(results.len(), 1);
    match &results[0].action {
        omni_lib::providers::ResultAction::Copy { text } => {
            assert_eq!(text, "#FF5733");
        }
        _ => panic!("Expected Copy action"),
    }
}

#[test]
fn test_roundtrip_hex_rgb_hsl() {
    // Parse a hex, check the subtitle contains valid rgb and hsl
    let results = ColorProvider::evaluate("#FF0000");
    assert_eq!(results.len(), 1);
    assert!(results[0].subtitle.contains("rgb(255, 0, 0)"));
    assert!(results[0].subtitle.contains("hsl(0, 100%, 50%)"));
}

#[test]
fn test_roundtrip_pure_green() {
    let results = ColorProvider::evaluate("#00FF00");
    assert_eq!(results.len(), 1);
    assert!(results[0].subtitle.contains("rgb(0, 255, 0)"));
    assert!(results[0].subtitle.contains("hsl(120, 100%, 50%)"));
}

#[test]
fn test_roundtrip_pure_blue() {
    let results = ColorProvider::evaluate("#0000FF");
    assert_eq!(results.len(), 1);
    assert!(results[0].subtitle.contains("rgb(0, 0, 255)"));
    assert!(results[0].subtitle.contains("hsl(240, 100%, 50%)"));
}

#[test]
fn test_black_and_white() {
    let black = ColorProvider::evaluate("#000000");
    assert_eq!(black.len(), 1);
    assert!(black[0].subtitle.contains("rgb(0, 0, 0)"));
    assert!(black[0].subtitle.contains("hsl(0, 0%, 0%)"));

    let white = ColorProvider::evaluate("#FFFFFF");
    assert_eq!(white.len(), 1);
    assert!(white[0].subtitle.contains("rgb(255, 255, 255)"));
    assert!(white[0].subtitle.contains("hsl(0, 0%, 100%)"));
}
