use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Parser)]
#[command(name = "pks")]
#[command(about = "A tool for sending keystrokes to specific processes")]
#[command(version = "0.1.0")]
#[command(author = "KyleDerZweite")]
pub struct Args {
    /// Name of the target process (e.g., "game.exe", "notepad")
    #[arg(short, long)]
    pub process: Option<String>,  // Made optional

    /// Key to send (e.g., "r", "space", "f1", "ctrl+c")
    #[arg(short, long)]
    pub key: Option<String>,

    /// Interval between keystrokes in milliseconds
    #[arg(short, long, default_value = "1000")]
    pub interval: u64,

    /// Key sequence with custom intervals (e.g., "r:1000,space:500,e:2000")
    #[arg(short = 's', long)]
    pub sequence: Option<String>,

    /// Multiple independent keys with intervals (e.g., "r:1000;a:5000")
    #[arg(long)]
    pub independent_keys: Option<String>,

    /// Maximum number of attempts to find the process
    #[arg(short = 'r', long, default_value = "10")]
    pub max_retries: u32,

    /// Hotkey to pause/resume (e.g., "ctrl+alt+r")
    #[arg(long)]
    pub pause_hotkey: Option<String>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Load configuration from file
    #[arg(short, long)]
    pub config: Option<String>,

    /// Save current configuration to file
    #[arg(long)]
    pub save_config: Option<String>,

    /// Loop the sequence indefinitely
    #[arg(long, default_value = "true")]
    pub loop_sequence: bool,

    /// Number of times to repeat the sequence (0 = infinite)
    #[arg(long, default_value = "0")]
    pub repeat_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyAction {
    pub key: String,
    pub interval_after: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndependentKey {
    pub key: String,
    pub interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub process_name: String,
    pub key_sequence: Vec<KeyAction>,
    pub independent_keys: Vec<IndependentKey>,
    pub max_retries: u32,
    pub pause_hotkey: Option<String>,
    pub verbose: bool,
    pub loop_sequence: bool,
    pub repeat_count: u32,
}

impl Config {
    pub fn from_args(args: Args) -> Result<Self> {
        let mut config = if let Some(config_path) = &args.config {
            Self::load_from_file(config_path)?
        } else {
            Self::default()
        };

        // Override with CLI arguments (only if provided)
        if let Some(process) = args.process {
            config.process_name = process;
        }

        config.max_retries = args.max_retries;
        config.verbose = args.verbose;
        config.loop_sequence = args.loop_sequence;
        config.repeat_count = args.repeat_count;

        if args.pause_hotkey.is_some() {
            config.pause_hotkey = args.pause_hotkey;
        }

        // Parse independent keys first (takes priority)
        if let Some(independent_str) = args.independent_keys {
            config.independent_keys = Self::parse_independent_keys(&independent_str)?;
            config.key_sequence.clear(); // Clear sequence mode
        }
        // Parse key sequence
        else if let Some(sequence_str) = args.sequence {
            config.key_sequence = Self::parse_key_sequence(&sequence_str)?;
            config.independent_keys.clear(); // Clear independent mode
        }
        // Single key mode
        else if let Some(single_key) = args.key {
            config.key_sequence = vec![KeyAction {
                key: single_key,
                interval_after: Duration::from_millis(args.interval),
            }];
            config.independent_keys.clear(); // Clear independent mode
        }

        // Validate that we have a process name
        if config.process_name.is_empty() {
            anyhow::bail!("No process name specified. Use --process or provide a config file with process_name.");
        }

        // Validate that we have at least one key action
        if config.key_sequence.is_empty() && config.independent_keys.is_empty() {
            anyhow::bail!("No key actions specified. Use --key, --sequence, or --independent-keys, or provide a config file with key actions.");
        }

        // Save config if requested
        if let Some(save_path) = &args.save_config {
            config.save_to_file(save_path)?;
        }

        Ok(config)
    }

    fn parse_independent_keys(keys_str: &str) -> Result<Vec<IndependentKey>> {
        let mut keys = Vec::new();

        for part in keys_str.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let components: Vec<&str> = part.split(':').collect();
            match components.len() {
                1 => {
                    // Just a key, use default interval of 1000ms
                    keys.push(IndependentKey {
                        key: components[0].trim().to_string(),
                        interval: Duration::from_millis(1000),
                    });
                }
                2 => {
                    // Key with custom interval
                    let key = components[0].trim().to_string();
                    let interval = components[1].trim().parse::<u64>()
                        .map_err(|_| anyhow::anyhow!("Invalid interval '{}' for key '{}'", components[1], key))?;

                    keys.push(IndependentKey {
                        key,
                        interval: Duration::from_millis(interval),
                    });
                }
                _ => {
                    anyhow::bail!("Invalid independent key format '{}'. Use 'key:interval' or just 'key'", part);
                }
            }
        }

        if keys.is_empty() {
            anyhow::bail!("Empty independent keys provided");
        }

        Ok(keys)
    }

    fn parse_key_sequence(sequence_str: &str) -> Result<Vec<KeyAction>> {
        let mut actions = Vec::new();

        for part in sequence_str.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let components: Vec<&str> = part.split(':').collect();
            match components.len() {
                1 => {
                    actions.push(KeyAction {
                        key: components[0].trim().to_string(),
                        interval_after: Duration::from_millis(1000),
                    });
                }
                2 => {
                    let key = components[0].trim().to_string();
                    let interval = components[1].trim().parse::<u64>()
                        .map_err(|_| anyhow::anyhow!("Invalid interval '{}' in sequence", components[1]))?;

                    actions.push(KeyAction {
                        key,
                        interval_after: Duration::from_millis(interval),
                    });
                }
                _ => {
                    anyhow::bail!("Invalid sequence format '{}'. Use 'key:interval' or just 'key'", part);
                }
            }
        }

        if actions.is_empty() {
            anyhow::bail!("Empty key sequence provided");
        }

        Ok(actions)
    }

    fn load_from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn save_to_file(&self, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        println!("Configuration saved to: {}", path);
        Ok(())
    }

    pub fn is_independent_mode(&self) -> bool {
        !self.independent_keys.is_empty()
    }

    pub fn get_independent_keys_description(&self) -> String {
        self.independent_keys.iter()
            .map(|key| format!("'{}' every {}ms", key.key, key.interval.as_millis()))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn total_sequence_duration(&self) -> Duration {
        self.key_sequence.iter()
            .map(|action| action.interval_after)
            .sum()
    }

    pub fn get_sequence_description(&self) -> String {
        self.key_sequence.iter()
            .map(|action| format!("'{}' ({}ms)", action.key, action.interval_after.as_millis()))
            .collect::<Vec<_>>()
            .join(" → ")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            process_name: String::new(),
            key_sequence: vec![KeyAction {
                key: "r".to_string(),
                interval_after: Duration::from_millis(1000),
            }],
            independent_keys: Vec::new(),
            max_retries: 10,
            pause_hotkey: Some("ctrl+alt+r".to_string()),
            verbose: false,
            loop_sequence: true,
            repeat_count: 0,
        }
    }
}