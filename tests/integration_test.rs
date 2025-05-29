use anyhow::Result;
use process_key_sender::config::{Config, parse_duration};
use std::time::Duration;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_revolution_idle_config() {
    let json = r#"
    {
        "process_name": "Revolution Idle.exe",
        "key_sequence": [],
        "independent_keys": [
            {
                "key": "r",
                "interval": "1000ms"
            },
            {
                "key": "a", 
                "interval": "5000ms"
            }
        ],
        "max_retries": 10,
        "pause_hotkey": "ctrl+alt+r",
        "verbose": true,
        "loop_sequence": true,
        "repeat_count": 0
    }
    "#;
    
    let config: Config = serde_json::from_str(json).unwrap();
    
    assert_eq!(config.process_name, "Revolution Idle.exe");
    assert_eq!(config.independent_keys.len(), 2);
    assert_eq!(config.independent_keys[0].key, "r");
    assert_eq!(config.independent_keys[0].interval, Duration::from_millis(1000));
    assert_eq!(config.independent_keys[1].key, "a");
    assert_eq!(config.independent_keys[1].interval, Duration::from_millis(5000));
    assert_eq!(config.max_retries, 10);
    assert_eq!(config.pause_hotkey, "ctrl+alt+r");
    assert!(config.verbose);
    assert!(config.loop_sequence);
    assert_eq!(config.repeat_count, 0);
    
    // Test validation
    assert!(config.validate().is_ok());
}

#[test]
fn test_key_sequence_config() {
    let json = r#"
    {
        "process_name": "notepad.exe",
        "key_sequence": [
            {
                "key": "1",
                "interval_after": "500ms"
            },
            {
                "key": "2",
                "interval_after": "500ms"
            },
            {
                "key": "space",
                "interval_after": "1s"
            }
        ],
        "independent_keys": [],
        "max_retries": 5,
        "verbose": false,
        "loop_sequence": false,
        "repeat_count": 3
    }
    "#;
    
    let config: Config = serde_json::from_str(json).unwrap();
    
    assert_eq!(config.process_name, "notepad.exe");
    assert_eq!(config.key_sequence.len(), 3);
    assert_eq!(config.key_sequence[0].key, "1");
    assert_eq!(config.key_sequence[0].interval_after, Duration::from_millis(500));
    assert_eq!(config.key_sequence[2].key, "space");
    assert_eq!(config.key_sequence[2].interval_after, Duration::from_secs(1));
    assert_eq!(config.max_retries, 5);
    assert!(!config.verbose);
    assert!(!config.loop_sequence);
    assert_eq!(config.repeat_count, 3);
    
    // Test validation
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_file_operations() -> Result<()> {
    // Create a temporary file
    let mut temp_file = NamedTempFile::new()?;
    
    let json_content = r#"
    {
        "process_name": "test-app.exe",
        "key_sequence": [],
        "independent_keys": [
            {
                "key": "space",
                "interval": "2s"
            }
        ],
        "max_retries": 15,
        "pause_hotkey": "ctrl+shift+p",
        "verbose": true,
        "loop_sequence": true,
        "repeat_count": 0
    }
    "#;
    
    // Write JSON to file
    temp_file.write_all(json_content.as_bytes())?;
    
    // Load config from file
    let config = Config::from_file(temp_file.path().to_str().unwrap())?;
    
    assert_eq!(config.process_name, "test-app.exe");
    assert_eq!(config.independent_keys.len(), 1);
    assert_eq!(config.independent_keys[0].key, "space");
    assert_eq!(config.independent_keys[0].interval, Duration::from_secs(2));
    assert_eq!(config.max_retries, 15);
    assert_eq!(config.pause_hotkey, "ctrl+shift+p");
    
    // Test validation
    assert!(config.validate().is_ok());
    
    Ok(())
}

#[test]
fn test_duration_parsing_edge_cases() {
    // Valid cases
    assert_eq!(parse_duration("0ms").unwrap(), Duration::from_millis(0));
    assert_eq!(parse_duration("1000").unwrap(), Duration::from_millis(1000));
    assert_eq!(parse_duration("5S").unwrap(), Duration::from_secs(5)); // Case insensitive
    assert_eq!(parse_duration(" 2m ").unwrap(), Duration::from_secs(120)); // Whitespace
    
    // Invalid cases
    assert!(parse_duration("").is_err());
    assert!(parse_duration("abc").is_err());
    assert!(parse_duration("1000x").is_err());
    assert!(parse_duration("-1000ms").is_err());
}

#[test]
fn test_config_validation_errors() {
    // Empty process name
    let mut config = Config {
        process_name: "".to_string(),
        key_sequence: vec![],
        independent_keys: vec![],
        max_retries: 10,
        pause_hotkey: "ctrl+alt+r".to_string(),
        verbose: false,
        loop_sequence: true,
        repeat_count: 0,
    };
    
    assert!(config.validate().is_err());
    
    // No keys configured
    config.process_name = "test.exe".to_string();
    assert!(config.validate().is_err());
    
    // Zero retries
    config.independent_keys.push(process_key_sender::config::IndependentKey {
        key: "space".to_string(),
        interval: Duration::from_millis(1000),
    });
    config.max_retries = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_default_values() {
    let json = r#"
    {
        "process_name": "minimal.exe"
    }
    "#;
    
    // This should fail because no keys are provided
    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.process_name, "minimal.exe");
    assert_eq!(config.max_retries, 10); // default
    assert_eq!(config.pause_hotkey, "ctrl+alt+r"); // default
    assert!(!config.verbose); // default false
    assert!(config.loop_sequence); // default true
    assert_eq!(config.repeat_count, 0); // default
    assert!(config.key_sequence.is_empty()); // default empty
    assert!(config.independent_keys.is_empty()); // default empty
    
    // Should fail validation due to no keys
    assert!(config.validate().is_err());
}

#[test]
fn test_complex_key_combinations() {
    let json = r#"
    {
        "process_name": "complex-app.exe",
        "independent_keys": [
            {
                "key": "ctrl+s",
                "interval": "30s"
            },
            {
                "key": "alt+tab",
                "interval": "10s"
            },
            {
                "key": "f5",
                "interval": "5m"
            }
        ]
    }
    "#;
    
    let config: Config = serde_json::from_str(json).unwrap();
    
    assert_eq!(config.independent_keys.len(), 3);
    assert_eq!(config.independent_keys[0].key, "ctrl+s");
    assert_eq!(config.independent_keys[0].interval, Duration::from_secs(30));
    assert_eq!(config.independent_keys[1].key, "alt+tab");
    assert_eq!(config.independent_keys[1].interval, Duration::from_secs(10));
    assert_eq!(config.independent_keys[2].key, "f5");
    assert_eq!(config.independent_keys[2].interval, Duration::from_secs(300)); // 5 minutes
    
    assert!(config.validate().is_ok());
}

#[test]
fn test_mixed_duration_formats() {
    let json = r#"
    {
        "process_name": "duration-test.exe",
        "key_sequence": [
            {
                "key": "1",
                "interval_after": "500ms"
            },
            {
                "key": "2", 
                "interval_after": "1s"
            },
            {
                "key": "3",
                "interval_after": "2000"
            }
        ]
    }
    "#;
    
    let config: Config = serde_json::from_str(json).unwrap();
    
    assert_eq!(config.key_sequence[0].interval_after, Duration::from_millis(500));
    assert_eq!(config.key_sequence[1].interval_after, Duration::from_secs(1));
    assert_eq!(config.key_sequence[2].interval_after, Duration::from_millis(2000));
    
    assert!(config.validate().is_ok());
}