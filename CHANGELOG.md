# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-05-29

### Added
- Initial release of Process Key Sender
- Cross-platform keystroke automation for specific processes
- Support for single keys, key sequences, and independent key timers
- Configuration file support (JSON format)
- Comprehensive CLI interface with clap
- Process detection and monitoring
- Safety disclaimers and ethical usage guidelines
- Support for key combinations (Ctrl+C, Alt+Tab, etc.)
- Verbose logging and colored terminal output
- Pause/resume functionality (hotkey support planned)
- Windows implementation with winapi
- Example configuration for Revolution Idle

### Features
- **Independent Key Mode**: Send multiple keys on different timers simultaneously
- **Sequential Mode**: Send keys in a specific sequence with custom intervals
- **Process Targeting**: Automatically find and target specific processes
- **Configuration Files**: Save and load settings from JSON files
- **Cross-platform**: Windows support (Linux planned)
- **Safety First**: Built-in warnings for responsible usage

### Technical Details
- Built with Rust 🦀
- Uses modern async/await with Tokio
- Comprehensive error handling with anyhow
- Structured logging with env_logger
- Beautiful CLI with clap and colored output

[Unreleased]: https://github.com/KyleDerZweite/process-key-sender/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/KyleDerZweite/process-key-sender/releases/tag/v0.1.0