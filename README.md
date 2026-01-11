# Terminal Relaunch

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)
[![Rust](https://github.com/zephhhhhh/terminal_relaunch/actions/workflows/rust.yml/badge.svg)](https://github.com/zephhhhhh/terminal_relaunch/actions/workflows/rust.yml)
[![Static Badge](https://img.shields.io/badge/pages-Docs-informational?logo=github)](https://docs.rs/terminal_relaunch)

A simple Rust library for detecting terminal capabilities and relaunching programs in better terminals with enhanced feature support,
such as RGB ANSI colour and full Unicode rendering support for emojis, etc.

## Features

- **Terminal Detection**: Automatically identifies the current terminal type (Windows Terminal, CMD, VSCode, etc.)
- **Feature Detection**: Checks for RGB/ANSI color support and full Unicode rendering
- **Smart Relaunching**: Automatically find an installed and better terminal if available to relaunch your CLI app in.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
terminal_relaunch = "0.2.2"
```

## Quick Start

```rust
use terminal_relaunch::{relaunch_if_available_and_exit, CURRENT_TERMINAL};

fn main() {
    // Check if we should attempt to relaunch
    match relaunch_if_available_and_exit() {
        Ok(()) => println!("Terminal features met!"),
        Err(e) => eprintln!("Terminal could not relaunch: {e:?}"),
    }

    // Continue with your application..
    println!("Terminal information: {}", CURRENT_TERMINAL.verbose_format())
}
```

## Usage

### Detect Current Terminal

```rust
use terminal_relaunch::CURRENT_TERMINAL;

println!("Terminal: {}", CURRENT_TERMINAL.verbose_format());
```

### Check Feature Support

```rust
use terminal_relaunch::{SUPPORTS_FULL_UNICODE, SUPPORTS_RGB_ANSI_COLOURS};

if *SUPPORTS_FULL_UNICODE {
    println!("✨ Unicode emojis work!");
}

if *SUPPORTS_RGB_ANSI_COLOURS {
    println!("\x1b[38;2;255;0;0mRGB colors work!\x1b[0m");
}
```

## Currently Supported Terminals

### Detection

- Windows Specific:
    - `Windows Terminal`
    - `CMD/PowerShell`
- MacOS Specific:
    - `Terminal.app`
    - `ITerm2`
    - `Kitty`
    - `Ghostty`
- Other terminals:
    - `Alacritty`
    - `WezTerm`
- Editor terminals:
    - `VSCode`
    - `NVIM`
- Generic Linux Terminals

### Relaunching

- `Windows Terminal`
- `ITerm2` (MacOS)
- `Ghostty` (MacOS)
- `Kitty` (MacOS)
- `Alacritty`
- `WezTerm` (MacOS)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
