//! Integration test for configuration serialization and validation
//!
//! Tests that configuration can be serialized, deserialized, and validated correctly.

use unpackrr::config::{AppConfig, LogLevel};

/// Test that default configuration can be serialized and deserialized
#[test]
fn test_config_round_trip_serialization() {
    let config = AppConfig::default();

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&config).expect("Failed to serialize config");

    // Deserialize back
    let deserialized: AppConfig =
        serde_json::from_str(&json).expect("Failed to deserialize config");

    // Verify basic properties match
    assert_eq!(
        config.extraction.postfixes,
        deserialized.extraction.postfixes
    );
    assert_eq!(config.advanced.show_debug, deserialized.advanced.show_debug);
    assert_eq!(config.appearance.language, deserialized.appearance.language);
}

/// Test that configuration modifications are preserved through serialization
#[test]
fn test_modified_config_serialization() {
    // Create config with custom values
    let mut config = AppConfig::default();
    config.extraction.postfixes = vec!["Custom".to_string(), "Postfix".to_string()];
    config.extraction.ignored_files = vec!["test.ba2".to_string(), "*.log".to_string()];
    config.extraction.ignore_bad_files = true;
    config.extraction.auto_backup = true;
    config.advanced.show_debug = true;
    config.advanced.log_level = LogLevel::Debug;
    config.appearance.theme_mode = "dark".to_string();
    config.saved.directory = "/test/path".to_string();
    config.saved.threshold = 1024 * 1024; // 1MB

    // Serialize and deserialize
    let json = serde_json::to_string_pretty(&config).expect("Failed to serialize");
    let loaded: AppConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    // Verify all modifications persisted
    assert_eq!(loaded.extraction.postfixes, vec!["Custom", "Postfix"]);
    assert_eq!(loaded.extraction.ignored_files, vec!["test.ba2", "*.log"]);
    assert!(loaded.extraction.ignore_bad_files);
    assert!(loaded.extraction.auto_backup);
    assert!(loaded.advanced.show_debug);
    assert_eq!(loaded.advanced.log_level, LogLevel::Debug);
    assert_eq!(loaded.appearance.theme_mode, "dark");
    assert_eq!(loaded.saved.directory, "/test/path");
    assert_eq!(loaded.saved.threshold, 1024 * 1024);
}

/// Test that all LogLevel variants serialize correctly
#[test]
fn test_log_level_serialization() {
    let levels = vec![
        LogLevel::Fatal,
        LogLevel::Error,
        LogLevel::Warning,
        LogLevel::Info,
        LogLevel::Debug,
        LogLevel::Trace,
    ];

    for level in levels {
        let mut config = AppConfig::default();
        config.advanced.log_level = level;

        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let loaded: AppConfig = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(config.advanced.log_level, loaded.advanced.log_level);
    }
}

/// Test that config validation catches invalid postfixes
#[test]
fn test_invalid_postfix_validation() {
    let mut config = AppConfig::default();
    config.extraction.postfixes = vec!["Invalid".to_string()]; // No .ba2 extension

    let result = config.validate();
    assert!(
        result.is_err(),
        "Validation should fail for invalid postfix"
    );
}

/// Test that config validation accepts valid postfixes
#[test]
fn test_valid_postfix_validation() {
    let mut config = AppConfig::default();
    config.extraction.postfixes = vec!["- Textures.ba2".to_string()];

    let result = config.validate();
    assert!(result.is_ok(), "Validation should pass for valid postfix");
}

/// Test that config validation catches invalid regex patterns
#[test]
fn test_invalid_regex_validation() {
    let mut config = AppConfig::default();
    config.extraction.ignored_files = vec!["[invalid(".to_string()]; // Invalid regex

    let result = config.validate();
    assert!(result.is_err(), "Validation should fail for invalid regex");
}

/// Test that empty/default config is valid
#[test]
fn test_default_config_is_valid() {
    let config = AppConfig::default();
    assert!(config.validate().is_ok(), "Default config should be valid");
}

/// Test that theme mode values are preserved
#[test]
fn test_theme_mode_persistence() {
    let modes = vec!["light", "dark", "system"];

    for mode in modes {
        let mut config = AppConfig::default();
        config.appearance.theme_mode = mode.to_string();

        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let loaded: AppConfig = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(loaded.appearance.theme_mode, mode);
    }
}

/// Test that language values are preserved
#[test]
fn test_language_persistence() {
    let languages = vec!["auto", "en", "zh-CN", "zh-TW"];

    for lang in languages {
        let mut config = AppConfig::default();
        config.appearance.language = lang.to_string();

        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let loaded: AppConfig = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(loaded.appearance.language, lang);
    }
}

/// Test that deserializing from minimal JSON works (defaults fill in)
#[test]
fn test_deserialize_minimal_json() {
    let minimal_json = r##"{
        "extraction": {},
        "saved": {},
        "appearance": {
            "theme_mode": "system",
            "accent_color": "#0078d4",
            "language": "auto"
        },
        "advanced": {},
        "update": {
            "check_at_startup": true
        }
    }"##;

    let config: AppConfig =
        serde_json::from_str(minimal_json).expect("Failed to deserialize minimal JSON");

    // Verify defaults are applied for extraction
    assert!(!config.extraction.postfixes.is_empty());
    // Verify required fields are present
    assert_eq!(config.appearance.theme_mode, "system");
    assert_eq!(config.appearance.language, "auto");
    assert_eq!(config.appearance.accent_color, "#0078d4");
}

/// Test that extra fields in JSON are ignored (forward compatibility)
#[test]
fn test_deserialize_with_extra_fields() {
    let json_with_extras = r##"{
        "extraction": {
            "postfixes": ["- Textures.ba2"],
            "unknown_field": "value"
        },
        "saved": {},
        "appearance": {
            "theme_mode": "dark",
            "accent_color": "#0078d4",
            "language": "en"
        },
        "advanced": {},
        "update": {
            "check_at_startup": false
        },
        "completely_unknown": {}
    }"##;

    let config: AppConfig =
        serde_json::from_str(json_with_extras).expect("Should deserialize with extra fields");

    assert_eq!(config.extraction.postfixes, vec!["- Textures.ba2"]);
    assert_eq!(config.appearance.theme_mode, "dark");
}
