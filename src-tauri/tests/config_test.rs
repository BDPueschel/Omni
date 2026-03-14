use omni_lib::config::OmniConfig;

#[test]
fn test_default_config() {
    let config = OmniConfig::default();
    assert_eq!(config.hotkey, "Alt+Space");
    assert_eq!(config.max_results_per_category, 5);
    assert_eq!(config.search_engine, "google");
    assert!(config.start_with_windows);
    assert_eq!(config.theme_opacity, 80);
}

#[test]
fn test_config_roundtrip() {
    let config = OmniConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let loaded: OmniConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, loaded);
}

#[test]
fn test_corrupt_config_falls_back_to_defaults() {
    let bad_json = "{ broken json {{";
    let config: OmniConfig = serde_json::from_str(bad_json).unwrap_or_default();
    assert_eq!(config.hotkey, "Alt+Space");
}
