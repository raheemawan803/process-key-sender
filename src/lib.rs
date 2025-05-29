//! Process Key Sender Library
//!
//! A library for sending keystrokes to specific processes across different platforms.

pub mod config;
pub mod key_sender;
pub mod process_finder;

pub use config::Config;
pub use key_sender::KeySender;
pub use process_finder::ProcessFinder;