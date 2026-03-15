use omni_lib::providers::units::UnitProvider;

#[test]
fn test_basic_length_km_to_miles() {
    let results = UnitProvider::evaluate("5km in miles");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].category, "Math");
    assert_eq!(results[0].icon, "unit");
    // 5 km = 3.10686 miles
    assert!(results[0].title.contains("mi"));
    assert!(results[0].title.contains("3.10"));
}

#[test]
fn test_length_with_space() {
    let results = UnitProvider::evaluate("5 km in miles");
    assert_eq!(results.len(), 1);
    assert!(results[0].title.contains("mi"));
}

#[test]
fn test_temperature_f_to_c() {
    let results = UnitProvider::evaluate("100f to c");
    assert_eq!(results.len(), 1);
    // 100F = 37.7778C
    assert!(results[0].title.contains("37.77"));
}

#[test]
fn test_temperature_0c_to_f() {
    let results = UnitProvider::evaluate("0c to f");
    assert_eq!(results.len(), 1);
    assert!(results[0].title.contains("32"));
}

#[test]
fn test_temperature_k_to_c() {
    let results = UnitProvider::evaluate("273.15k to c");
    assert_eq!(results.len(), 1);
    // 273.15K = 0C
    assert!(results[0].title.contains("0"));
}

#[test]
fn test_data_mb_to_gb() {
    let results = UnitProvider::evaluate("1024mb to gb");
    assert_eq!(results.len(), 1);
    assert!(results[0].title.contains("1"));
    assert!(results[0].title.contains("GB"));
}

#[test]
fn test_weight_kg_to_lb() {
    let results = UnitProvider::evaluate("1kg in lb");
    assert_eq!(results.len(), 1);
    // 1 kg = 2.20462 lb
    assert!(results[0].title.contains("2.2046"));
}

#[test]
fn test_invalid_unit_returns_empty() {
    let results = UnitProvider::evaluate("5xyz in abc");
    assert!(results.is_empty());
}

#[test]
fn test_no_match_plain_text() {
    let results = UnitProvider::evaluate("hello world");
    assert!(results.is_empty());
}

#[test]
fn test_cross_category_returns_empty() {
    // km (length) to kg (weight) — should not convert
    let results = UnitProvider::evaluate("5km to kg");
    assert!(results.is_empty());
}

#[test]
fn test_case_insensitive() {
    let results = UnitProvider::evaluate("5KM in Miles");
    assert_eq!(results.len(), 1);
    assert!(results[0].title.contains("mi"));
}

#[test]
fn test_volume_liters_to_gallons() {
    let results = UnitProvider::evaluate("1l to gal");
    assert_eq!(results.len(), 1);
    // 1L = ~0.264172 gal
    assert!(results[0].title.contains("0.264"));
}

#[test]
fn test_speed_mph_to_kmh() {
    let results = UnitProvider::evaluate("60mph to kmh");
    assert_eq!(results.len(), 1);
    // 60 mph = ~96.56 km/h
    assert!(results[0].title.contains("96."));
}

#[test]
fn test_time_hours_to_minutes() {
    let results = UnitProvider::evaluate("2hr to min");
    assert_eq!(results.len(), 1);
    assert!(results[0].title.contains("120"));
}

#[test]
fn test_copy_action() {
    let results = UnitProvider::evaluate("1km to m");
    assert_eq!(results.len(), 1);
    match &results[0].action {
        omni_lib::providers::ResultAction::Copy { text } => {
            assert_eq!(text, "1,000");
        }
        _ => panic!("Expected Copy action"),
    }
}
