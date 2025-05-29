use anyhow::Result;
use clap::{Arg, Command};
use colored::Colorize;
use std::time::Duration;
use tokio::time::sleep;

mod config;
mod key_sender;
mod process_finder;

use config::Config;
use key_sender::KeySender;
use process_finder::ProcessFinder;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("Process Key Sender")
        .version("0.1.1")
        .author("KyleDerZweite <kyle@process-key-sender.dev>")
        .about("Cross-platform keystroke automation tool for specific processes")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
        )
        .arg(
            Arg::new("process")
                .short('p')
                .long("process")
                .value_name("PROCESS")
                .help("Target process name (e.g., 'notepad.exe')")
        )
        .arg(
            Arg::new("key")
                .short('k')
                .long("key")
                .value_name("KEY")
                .help("Key to send (e.g., 'space', 'a', 'ctrl+c')")
        )
        .arg(
            Arg::new("interval")
                .short('i')
                .long("interval")
                .value_name("DURATION")
                .help("Interval between key presses (e.g., '1000ms', '5s')")
                .default_value("1000ms")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("save-config")
                .long("save-config")
                .value_name("FILE")
                .help("Save current CLI arguments to configuration file")
        )
        .arg(
            Arg::new("max-retries")
                .long("max-retries")
                .value_name("COUNT")
                .help("Maximum retries to find process")
                .default_value("10")
        )
        .get_matches();

    // Handle config file loading or CLI argument parsing
    let config = if let Some(config_file) = matches.get_one::<String>("config") {
        load_config_file(config_file)?
    } else {
        create_config_from_args(&matches)?
    };

    // Save config if requested
    if let Some(save_path) = matches.get_one::<String>("save-config") {
        config.save_to_file(save_path)?;
        println!("{} Configuration saved to: {}", "✓".green(), save_path.cyan());
        return Ok(());
    }

    // Validate configuration
    validate_config(&config)?;

    // Print startup information
    print_startup_info(&config);

    // Initialize components
    let mut process_finder = ProcessFinder::new();
    let key_sender = KeySender::new()?;

    // Main execution loop
    run_automation(config, &mut process_finder, &key_sender).await
}

fn load_config_file(config_file: &str) -> Result<Config> {
    println!("{} Loading configuration from: {}", "📁".blue(), config_file.cyan());

    match Config::from_file(config_file) {
        Ok(config) => {
            println!("{} Configuration loaded successfully", "✓".green());
            Ok(config)
        }
        Err(e) => {
            eprintln!("{} Failed to load configuration: {}", "✗".red(), e);
            anyhow::bail!("Configuration loading failed: {}", e);
        }
    }
}

fn create_config_from_args(matches: &clap::ArgMatches) -> Result<Config> {
    let process_name = matches.get_one::<String>("process")
        .ok_or_else(|| anyhow::anyhow!("Process name is required. Use --process or --config."))?
        .clone();

    let key = matches.get_one::<String>("key")
        .ok_or_else(|| anyhow::anyhow!("Key is required. Use --key or --config."))?
        .clone();

    let interval_str = matches.get_one::<String>("interval").unwrap();
    let interval = parse_duration(interval_str)?;

    let max_retries: u32 = matches.get_one::<String>("max-retries")
        .unwrap()
        .parse()?;

    let verbose = matches.get_flag("verbose");

    Ok(Config {
        process_name,
        key_sequence: vec![config::KeyAction {
            key,
            interval_after: interval,
        }],
        independent_keys: vec![],
        max_retries,
        pause_hotkey: "ctrl+alt+r".to_string(),
        verbose,
        loop_sequence: true,
        repeat_count: 0,
    })
}

fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim().to_lowercase();

    if s.ends_with("ms") {
        let num_str = &s[..s.len() - 2];
        let ms: u64 = num_str.parse()?;
        Ok(Duration::from_millis(ms))
    } else if s.ends_with('s') {
        let num_str = &s[..s.len() - 1];
        let secs: u64 = num_str.parse()?;
        Ok(Duration::from_secs(secs))
    } else if s.ends_with('m') {
        let num_str = &s[..s.len() - 1];
        let mins: u64 = num_str.parse()?;
        Ok(Duration::from_secs(mins * 60))
    } else {
        // Default to milliseconds if no suffix
        let ms: u64 = s.parse()?;
        Ok(Duration::from_millis(ms))
    }
}

fn validate_config(config: &Config) -> Result<()> {
    if config.process_name.is_empty() {
        anyhow::bail!("Process name cannot be empty");
    }

    if config.key_sequence.is_empty() && config.independent_keys.is_empty() {
        anyhow::bail!("At least one key sequence or independent key must be configured");
    }

    if !config.key_sequence.is_empty() && !config.independent_keys.is_empty() {
        anyhow::bail!("Cannot use both key_sequence and independent_keys simultaneously. Choose one mode.");
    }

    if config.max_retries == 0 {
        anyhow::bail!("max_retries must be greater than 0");
    }

    // Validate all keys
    let key_sender = KeySender::new()?;

    for key_action in &config.key_sequence {
        validate_key(&key_sender, &key_action.key)?;
        if key_action.interval_after < Duration::from_millis(50) {
            println!("{} Warning: Very short interval ({}ms) for key '{}' may cause issues",
                     "⚠".yellow(),
                     key_action.interval_after.as_millis(),
                     key_action.key
            );
        }
    }

    for independent_key in &config.independent_keys {
        validate_key(&key_sender, &independent_key.key)?;
        if independent_key.interval < Duration::from_millis(50) {
            println!("{} Warning: Very short interval ({}ms) for key '{}' may cause issues",
                     "⚠".yellow(),
                     independent_key.interval.as_millis(),
                     independent_key.key
            );
        }
    }

    Ok(())
}

fn validate_key(key_sender: &KeySender, key: &str) -> Result<()> {
    // Try to parse the key to ensure it's valid
    key_sender.parse_key_for_validation(key)
        .map_err(|e| anyhow::anyhow!("Invalid key '{}': {}", key, e))?;
    Ok(())
}

fn print_startup_info(config: &Config) {
    println!("\n{}", "🚀 Process Key Sender v0.1.1".bold().cyan());
    println!("{}", "═".repeat(40).cyan());

    println!("{} Target Process: {}", "🎯".blue(), config.process_name.yellow());
    println!("{} Max Retries: {}", "🔄".blue(), config.max_retries.to_string().yellow());
    println!("{} Pause Hotkey: {}", "⏸".blue(), config.pause_hotkey.yellow());
    println!("{} Verbose Mode: {}", "📝".blue(), if config.verbose { "ON".green() } else { "OFF".red() });

    if !config.key_sequence.is_empty() {
        println!("\n{} Key Sequence Mode:", "⌨".blue());
        for (i, key_action) in config.key_sequence.iter().enumerate() {
            println!("  {}. {} (wait {}ms)",
                     i + 1,
                     key_action.key.cyan(),
                     key_action.interval_after.as_millis().to_string().yellow()
            );
        }
        println!("  {} Loop: {}", "🔁".blue(), if config.loop_sequence { "YES".green() } else { "NO".red() });
        if config.repeat_count > 0 {
            println!("  {} Repeat Count: {}", "🔢".blue(), config.repeat_count.to_string().yellow());
        }
    }

    if !config.independent_keys.is_empty() {
        println!("\n{} Independent Keys Mode:", "⌨".blue());
        for independent_key in &config.independent_keys {
            println!("  {} every {}ms",
                     independent_key.key.cyan(),
                     independent_key.interval.as_millis().to_string().yellow()
            );
        }
    }

    println!("{}", "═".repeat(40).cyan());
    println!("{} Press Ctrl+C to stop\n", "ℹ".blue());
}

async fn run_automation(
    config: Config,
    process_finder: &mut ProcessFinder,
    key_sender: &KeySender
) -> Result<()> {
    // Find target process
    let window_id = find_target_process(&config, process_finder).await?;

    println!("{} Process found! Starting automation...", "✓".green());

    // Run appropriate automation mode
    if !config.independent_keys.is_empty() {
        run_independent_keys(&config, key_sender, window_id).await
    } else {
        run_key_sequence(&config, key_sender, window_id).await
    }
}

async fn find_target_process(config: &Config, process_finder: &mut ProcessFinder) -> Result<u64> {
    println!("{} Searching for process: {}", "🔍".blue(), config.process_name.yellow());

    for attempt in 1..=config.max_retries {
        if config.verbose {
            println!("  Attempt {}/{}", attempt, config.max_retries);
        }

        match process_finder.find_process_window(&config.process_name) {
            Ok(Some(window_id)) => {
                println!("{} Found process window (ID: {})", "✓".green(), window_id.to_string().cyan());
                return Ok(window_id);
            }
            Ok(None) => {
                if config.verbose {
                    println!("  Process not found, retrying...");
                }
            }
            Err(e) => {
                eprintln!("{} Error searching for process: {}", "✗".red(), e);
            }
        }

        if attempt < config.max_retries {
            sleep(Duration::from_millis(1000)).await;
        }
    }

    anyhow::bail!("Could not find process '{}' after {} attempts", config.process_name, config.max_retries);
}

async fn run_independent_keys(config: &Config, key_sender: &KeySender, window_id: u64) -> Result<()> {
    println!("{} Starting independent keys automation...", "🚀".green());

    let mut handles = Vec::new();

    for independent_key in &config.independent_keys {
        let key = independent_key.key.clone();
        let interval = independent_key.interval;
        let sender = key_sender.clone();
        let wid = window_id;
        let verbose = config.verbose;

        let handle = tokio::spawn(async move {
            loop {
                match sender.send_key_to_window(wid, &key) {
                    Ok(_) => {
                        if verbose {
                            println!("✓ Sent key: {}", key.cyan());
                        }
                    }
                    Err(e) => {
                        eprintln!("{} Error sending key '{}': {}", "✗".red(), key, e);
                    }
                }

                sleep(interval).await;
            }
        });

        handles.push(handle);
    }

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    println!("\n{} Shutting down...", "🛑".yellow());

    // Cancel all tasks
    for handle in handles {
        handle.abort();
    }

    Ok(())
}

async fn run_key_sequence(config: &Config, key_sender: &KeySender, window_id: u64) -> Result<()> {
    println!("{} Starting key sequence automation...", "🚀".green());

    let mut iteration = 0u32;

    loop {
        iteration += 1;

        if config.verbose {
            println!("--- Sequence iteration {} ---", iteration.to_string().cyan());
        }

        for (i, key_action) in config.key_sequence.iter().enumerate() {
            // Check if we should stop
            if let Ok(_) = tokio::time::timeout(Duration::from_millis(1), tokio::signal::ctrl_c()).await {
                println!("\n{} Shutting down...", "🛑".yellow());
                return Ok(());
            }

            match key_sender.send_key_to_window(window_id, &key_action.key) {
                Ok(_) => {
                    if config.verbose {
                        println!("  {}. ✓ Sent key: {}", i + 1, key_action.key.cyan());
                    }
                }
                Err(e) => {
                    eprintln!("  {}. {} Error sending key '{}': {}", i + 1, "✗".red(), key_action.key, e);
                }
            }

            sleep(key_action.interval_after).await;
        }

        // Check repeat count
        if config.repeat_count > 0 && iteration >= config.repeat_count {
            println!("{} Completed {} iterations", "✓".green(), config.repeat_count.to_string().cyan());
            break;
        }

        // Check if we should loop
        if !config.loop_sequence {
            break;
        }
    }

    Ok(())
}