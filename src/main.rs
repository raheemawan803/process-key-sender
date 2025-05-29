mod config;
mod key_sender;
mod process_finder;

use anyhow::Result;
use clap::Parser;
use colored::*;
use log::{info, warn, error};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

use crate::config::{Args, Config, KeyAction};
use crate::key_sender::KeySender;
use crate::process_finder::ProcessFinder;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    let config = Config::from_args(args)?;

    // Display header and disclaimer
    display_header_and_disclaimer();

    let app = App::new(config)?;
    app.run().await
}

fn display_header_and_disclaimer() {
    println!("{}", "Process Key Sender v0.1.0".bright_cyan().bold());
    println!("{}", "by KyleDerZweite".dim());
    println!();

    // Disclaimer warning
    println!("{}", "⚠️  DISCLAIMER WARNING ⚠️".bright_red().bold());
    println!("{}", "This tool is intended for offline/single-player games ONLY!".bright_yellow());
    println!("{}", "DO NOT use with online games or anti-cheat systems.".bright_yellow());
    println!("{}", "Using this tool with online games may result in permanent bans.".bright_red());
    println!("{}", "Use at your own risk and responsibility.".dim());
    println!();
}

struct App {
    config: Config,
    key_sender: KeySender,
    process_finder: ProcessFinder,
    running: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
}

impl App {
    fn new(config: Config) -> Result<Self> {
        let running = Arc::new(AtomicBool::new(true));
        let paused = Arc<AtomicBool::new(false));

        let key_sender = KeySender::new()?;
        let process_finder = ProcessFinder::new();

        // Setup Ctrl+C handler
        let running_clone = Arc::clone(&running);
        ctrlc::set_handler(move || {
            running_clone.store(false, Ordering::SeqCst);
        })?;

        Ok(Self {
            config,
            key_sender,
            process_finder,
            running,
            paused,
        })
    }

    async fn run(&self) -> Result<()> {
        info!("Starting Process Key Sender");

        // Display configuration
        self.display_config();

        // Setup hotkeys if enabled
        if let Some(ref hotkey) = self.config.pause_hotkey {
            self.setup_pause_hotkey(hotkey)?;
        }

        // Find target process
        let target_window = self.find_target_process().await?;

        println!("{}", "✓ Process found and ready!".bright_green());

        if self.config.key_sequence.len() == 1 {
            let action = &self.config.key_sequence[0];
            println!("{}", format!("Sending '{}' every {}ms",
                                   action.key.bright_yellow(),
                                   action.interval_after.as_millis()).bright_blue());
        } else {
            println!("{}", "Key sequence:".bright_blue());
            println!("  {}", self.config.get_sequence_description());
            println!("  {}", format!("Total cycle time: {}ms",
                                     self.config.total_sequence_duration().as_millis()).dim());

            if self.config.loop_sequence {
                if self.config.repeat_count > 0 {
                    println!("  {}", format!("Will repeat {} times", self.config.repeat_count).dim());
                } else {
                    println!("  {}", "Will loop indefinitely".dim());
                }
            } else {
                println!("  {}", "Will run once".dim());
            }
        }

        if self.config.pause_hotkey.is_some() {
            println!("{}", format!("Press {} to pause/resume",
                                   self.config.pause_hotkey.as_ref().unwrap().bright_magenta()));
        }
        println!("{}", "Press Ctrl+C to exit".dim());
        println!();

        // Main execution loop
        self.execution_loop(target_window).await
    }

    fn display_config(&self) {
        println!("{}", "Configuration:".bright_white().underline());
        println!("  Process: {}", self.config.process_name.bright_yellow());

        if self.config.key_sequence.len() == 1 {
            let action = &self.config.key_sequence[0];
            println!("  Key: {}", action.key.bright_yellow());
            println!("  Interval: {}ms", action.interval_after.as_millis());
        } else {
            println!("  Sequence: {} keys", self.config.key_sequence.len());
            for (i, action) in self.config.key_sequence.iter().enumerate() {
                println!("    {}: '{}' → wait {}ms",
                         (i + 1).to_string().bright_cyan(),
                         action.key.bright_yellow(),
                         action.interval_after.as_millis());
            }
        }

        println!("  Max attempts: {}", self.config.max_retries);
        if let Some(ref hotkey) = self.config.pause_hotkey {
            println!("  Pause hotkey: {}", hotkey.bright_magenta());
        }
        println!();
    }

    async fn find_target_process(&self) -> Result<u64> {
        println!("{}", format!("🔍 Looking for process: {}", self.config.process_name).bright_blue());

        let mut attempts = 0;

        while attempts < self.config.max_retries && self.running.load(Ordering::SeqCst) {
            if let Some(window_id) = self.process_finder.find_process_window(&self.config.process_name)? {
                return Ok(window_id);
            }

            attempts += 1;
            if attempts < self.config.max_retries {
                println!("{}", format!("Process not found, retrying... ({}/{})",
                                       attempts, self.config.max_retries).yellow());
                time::sleep(Duration::from_secs(2)).await;
            }
        }

        anyhow::bail!("Could not find process '{}' after {} attempts",
            self.config.process_name, self.config.max_retries);
    }

    async fn execution_loop(&self, target_window: u64) -> Result<()> {
        let mut consecutive_failures = 0;
        let mut current_sequence_index = 0;
        let mut cycles_completed = 0;

        while self.running.load(Ordering::SeqCst) {
            // Check if process is still running
            if !self.process_finder.is_process_running(&self.config.process_name)? {
                println!("{}", "⚠️  Target process has been closed. Stopping...".bright_red());
                break;
            }

            // Check if we've completed the required number of cycles
            if self.config.repeat_count > 0 && cycles_completed >= self.config.repeat_count {
                println!("{}", format!("✓ Completed {} cycles as requested. Stopping...",
                                       cycles_completed).bright_green());
                break;
            }

            // Send key if not paused
            if !self.paused.load(Ordering::SeqCst) {
                let current_action = &self.config.key_sequence[current_sequence_index];

                match self.key_sender.send_key_to_window(target_window, &current_action.key) {
                    Ok(_) => {
                        if self.config.verbose {
                            let step_info = if self.config.key_sequence.len() > 1 {
                                format!(" [step {}/{}]", current_sequence_index + 1, self.config.key_sequence.len())
                            } else {
                                String::new()
                            };

                            println!("{}", format!("✓ Sent '{}'{}",
                                                   current_action.key, step_info).green());
                        }
                        consecutive_failures = 0;
                    }
                    Err(e) => {
                        consecutive_failures += 1;
                        warn!("Failed to send key '{}': {}", current_action.key, e);

                        if consecutive_failures >= 5 {
                            error!("Too many consecutive failures, stopping...");
                            break;
                        }
                    }
                }

                // Wait for the interval specified for this key
                time::sleep(current_action.interval_after).await;

                // Move to next key in sequence
                current_sequence_index += 1;

                // Check if we've completed the sequence
                if current_sequence_index >= self.config.key_sequence.len() {
                    cycles_completed += 1;

                    if self.config.verbose && self.config.key_sequence.len() > 1 {
                        println!("{}", format!("🔄 Completed cycle {} of sequence",
                                               cycles_completed).bright_blue());
                    }

                    if self.config.loop_sequence {
                        current_sequence_index = 0; // Reset to beginning
                    } else {
                        // Single run, we're done
                        println!("{}", "✓ Sequence completed (single run). Stopping...".bright_green());
                        break;
                    }
                }
            } else {
                if self.config.verbose {
                    println!("{}", "⏸️  Paused".yellow());
                }
                time::sleep(Duration::from_millis(100)).await; // Small sleep when paused
            }
        }

        println!("{}", "👋 Stopping key sender...".bright_blue());
        Ok(())
    }

    fn setup_pause_hotkey(&self, _hotkey: &str) -> Result<()> {
        // TODO: Implement global hotkey setup
        // This would require parsing the hotkey string and setting up global hotkey manager
        info!("Hotkey setup not yet implemented");
        Ok(())
    }
}