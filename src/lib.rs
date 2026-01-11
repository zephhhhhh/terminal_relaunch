//! # Terminal Relaunch
//!
//! A simple Rust library for detecting terminal capabilities and relaunching programs in better terminals
//! with enhanced feature support, such as RGB ANSI colour and full Unicode rendering support for emojis.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use terminal_relaunch::{relaunch_if_available_and_exit, CURRENT_TERMINAL};
//!
//! fn main() {
//!     // Check if we should attempt to relaunch
//!     match relaunch_if_available_and_exit() {
//!         Ok(()) => println!("Terminal features met!"),
//!         Err(e) => eprintln!("Terminal could not relaunch: {e:?}"),
//!     }
//!
//!     // Continue with your application..
//!     println!("Terminal information: {}", CURRENT_TERMINAL.verbose_format())
//! }
//! ```
//!
//! ## Usage Examples
//!
//! ### Detect Current Terminal
//!
//! ```rust
//! use terminal_relaunch::CURRENT_TERMINAL;
//!
//! println!("Terminal: {}", CURRENT_TERMINAL.verbose_format());
//! ```
//!
//! ### Check Feature Support
//!
//! ```rust
//! use terminal_relaunch::{SUPPORTS_FULL_UNICODE, SUPPORTS_RGB_ANSI_COLOURS};
//!
//! if *SUPPORTS_FULL_UNICODE {
//!     println!("✨ Unicode emojis work!");
//! }
//!
//! if *SUPPORTS_RGB_ANSI_COLOURS {
//!     println!("\x1b[38;2;255;0;0mRGB colors work!\x1b[0m");
//! }
//! ```
//!
//! ## Supported Terminals
//!
//! ### Detection
//!
//! - `Windows` Specific:
//!     - `Windows Terminal`
//!     - `CMD/PowerShell`
//! - `MacOS` Specific:
//!     - `Terminal.app`
//!     - `ITerm2`
//!     - `Kitty`
//!     - `Ghostty`
//! - Editor terminals
//!     - `VSCode`
//!     - `NVIM`
//! - Generic Linux Terminals
//!
//! ### Relaunching
//! - `Windows Terminal`
//! - `ITerm2`
//! - `Ghostty`
//! - `Kitty`
//! - `Alacritty`

#![warn(clippy::pedantic)]

pub mod errors;
pub mod logging;
pub mod terminal_providers;

use std::fmt::Display;
use std::sync::LazyLock;
use std::sync::atomic;

use strum::{EnumIter, IntoEnumIterator};

use crate::terminal_providers::AlacrittyProvider;
use crate::terminal_providers::GhosttyProvider;
use crate::terminal_providers::KittyProvider;
use crate::terminal_providers::TERM_VAR;
use crate::{
    errors::{RelaunchError, TermResult},
    terminal_providers::{ITerm2Provider, TERM_PROGRAM_VAR, WindowsTerminalProvider},
};

/// Represents the different types of terminals we can identify.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter)]
#[repr(u8)]
pub enum TerminalType {
    /// Unable to identify the terminal.
    Unknown,

    // Windows terminals..
    #[default]
    /// Default `Windows` terminal (`cmd.exe` or `powershell.exe`).
    WindowsCMD,
    /// `Windows Terminal`. (terminal app from Microsoft Store `wt.exe`)
    WindowsTerminal,

    // MacOS terminals..
    /// Default `MacOS` terminal (Terminal.app).
    MacOS,
    /// Third party `MacOS` terminal `iTerm2`.
    ITerm2,
    /// Third party `MacOS` terminal `Kitty`.
    Kitty,
    /// Third party `MacOS` terminal `Ghostty`.
    Ghostty,
    /// Third party `MacOS` terminal (`iTerm2`, `Alacritty`, etc).
    /// We just assume that if we are on `MacOS` and not in the default terminal,
    /// then we are in a third party terminal.
    ///
    /// **TODO**: Improve detection for specific third party terminals.
    ThirdPartyMacOSTerminal,

    /// Default `Linux` terminal (`GNOME Terminal`, `Konsole`, etc).
    ///
    /// **TODO**: Improve detection for specific Linux terminals.
    LinuxTerminal,

    // Cross platform editor terminals..
    WezTerm,
    Alacritty,

    // Editor terminals..
    /// `VS Code` embedded terminal.
    VSCode,
    /// `NVim` terminal (e.g. `nvim-qt`, `neovide`, etc).
    Nvim,
}

impl TerminalType {
    /// Returns the name of the terminal type.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::WindowsCMD => "Windows CMD",
            Self::WindowsTerminal => "Windows Terminal",
            Self::MacOS => "MacOS Terminal",
            Self::ITerm2 => "iTerm2",
            Self::Kitty => "Kitty",
            Self::Ghostty => "Ghostty",
            Self::ThirdPartyMacOSTerminal => "Third Party MacOS Terminal",
            Self::LinuxTerminal => "Linux Terminal",
            Self::Alacritty => "Alacritty",
            Self::WezTerm => "WezTerm",
            Self::VSCode => "VSCode Terminal",
            Self::Nvim => "NVIM Terminal",
        }
    }

    /// Returns the executable name of the terminal type, if known.
    #[inline]
    #[must_use]
    pub fn exec_name(&self) -> Option<&'static str> {
        match self {
            Self::WindowsCMD => Some("cmd.exe"),
            Self::MacOS => Some("Terminal.app"),
            Self::WindowsTerminal => Some("wt.exe"),
            Self::VSCode => Some("Code.exe"),
            Self::ITerm2 => Some("iTerm2.app"),
            _ => None,
        }
    }

    /// Returns the target operating system this terminal runs on.
    #[inline]
    #[must_use]
    pub fn target_os(&self) -> TargetOperatingSystem {
        match self {
            Self::WindowsCMD | Self::WindowsTerminal => TargetOperatingSystem::Windows,
            Self::MacOS
            | Self::ITerm2
            | Self::Ghostty
            | Self::Kitty
            | Self::ThirdPartyMacOSTerminal => TargetOperatingSystem::MacOS,
            Self::VSCode | Self::Nvim | Self::Alacritty | Self::WezTerm => {
                TargetOperatingSystem::Any
            }
            Self::LinuxTerminal => TargetOperatingSystem::Linux,
            Self::Unknown => TargetOperatingSystem::Invalid,
        }
    }

    /// Returns `true` if the terminal supports RGB (ANSI) colours.
    #[inline]
    #[must_use]
    pub fn supports_rgb_ansi_colours(&self) -> bool {
        match self {
            Self::Unknown | Self::MacOS => false,
            Self::WindowsCMD
            | Self::WindowsTerminal
            | Self::VSCode
            | Self::Nvim
            | Self::ITerm2
            | Self::ThirdPartyMacOSTerminal
            | Self::Alacritty
            | Self::WezTerm
            | Self::Kitty
            | Self::Ghostty
            | Self::LinuxTerminal => true,
        }
    }

    /// Returns `true` if the terminal supports full unicode rendering (e.g. emojis, etc.).
    #[inline]
    #[must_use]
    pub fn supports_full_unicode(&self) -> bool {
        match self {
            Self::Unknown | Self::WindowsCMD | Self::MacOS => false,
            Self::WindowsTerminal
            | Self::VSCode
            | Self::Nvim
            | Self::ITerm2
            | Self::ThirdPartyMacOSTerminal
            | Self::Alacritty
            | Self::WezTerm
            | Self::Kitty
            | Self::Ghostty
            | Self::LinuxTerminal => true,
        }
    }

    /// Returns `true` if the terminal is a preferred terminal type (i.e. supports all features).
    #[inline]
    #[must_use]
    pub fn is_preferred(&self) -> bool {
        self.supports_full_unicode() && self.supports_rgb_ansi_colours()
    }

    /// Returns a verbose formatted string of the terminal type and supported features.
    #[inline]
    #[must_use]
    pub fn verbose_format(&self) -> String {
        let unicode = if self.supports_full_unicode() {
            ", Full Unicode"
        } else {
            ""
        };
        let rgb = if self.supports_rgb_ansi_colours() {
            ", Enhanced Colours"
        } else {
            ""
        };
        let exec_name = if let Some(exec_name) = self.exec_name() {
            format!(" ({exec_name})")
        } else {
            String::new()
        };

        format!("{}{}{}{}", self.name(), exec_name, unicode, rgb)
    }
}

impl Display for TerminalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Represents the different target operating systems we can support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum OperatingSystem {
    Windows,
    MacOS,
    Linux,
    Unknown,
}

impl Display for OperatingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl OperatingSystem {
    /// Returns the current operating system from the build cfg.
    #[inline]
    #[must_use]
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOS
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else {
            Self::Unknown
        }
    }

    /// Returns the name of the operating system.
    /// # Example
    /// * `Self::Windows` => `"Windows"`
    /// * `Self::MacOS` => `"MacOS"`
    #[inline]
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::MacOS => "MacOS",
            Self::Linux => "Linux",
            Self::Unknown => "Unknown",
        }
    }

    /// Returns `true` if the operating system is compatible with the target operating system.
    /// # Example
    /// * `Self::Windows.compatible_with_target(TargetOperatingSystem::Windows)` => `true`
    /// * `Self::MacOS.compatible_with_target(TargetOperatingSystem::Windows)` => `false`
    #[inline]
    #[must_use]
    pub fn compatible_with_target(&self, other: TargetOperatingSystem) -> bool {
        if other == TargetOperatingSystem::Any {
            return true;
        }
        matches!(
            (self, other),
            (Self::Windows, TargetOperatingSystem::Windows)
                | (Self::MacOS, TargetOperatingSystem::MacOS)
                | (Self::Linux, TargetOperatingSystem::Linux)
        )
    }
}

/// Represents a target operating system for a terminal signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum TargetOperatingSystem {
    /// Akin to a `never` type, indicates an invalid target OS.
    Invalid,
    /// Targets `Windows` only.
    Windows,
    /// Targets `MacOS` only.
    MacOS,
    /// Targets `Linux` only.
    Linux,
    /// Targets any operating system.
    Any,
}

impl TargetOperatingSystem {
    /// Returns the name of the target operating system.
    /// # Example
    /// * `Self::Windows` => `"Windows"`
    /// * `Self::MacOS` => `"MacOS"`
    #[inline]
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::MacOS => "MacOS",
            Self::Linux => "Linux",
            Self::Any => "Any",
            Self::Invalid => "Invalid",
        }
    }
}

impl Display for TargetOperatingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Represents a kind of 'signature' that can be used to identify which terminal we are running in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TerminalSignature {
    /// An environment variable that must exist.
    EnvVarExists(&'static str),
    /// An environment variable that must exist, and have a specific value.
    EnvVar(&'static str, &'static str),
    /// The environment variable `TERM_PROGRAM` must have a specific value.
    TermProgram(&'static str),
    /// The environment variable `TERM` must have a specific value.
    TermVar(&'static str),
    /// Returns `true` if the windows console delegation is set to a value in the windows registry.
    WindowsConsoleDelegationSet,

    /// Returns `true` if any of the given terminal signatures are met (I.e. `OR` logic).
    Any(&'static [TerminalSignature]),
}

/// Checks if Windows console delegation is set in the registry.
/// If this is _NOT_ set, it means the default console host is being used (cmd)
/// otherwise, it is being delegated to another terminal (e.g. Windows Terminal).
#[inline]
#[must_use]
fn check_for_windows_registry_delegation() -> bool {
    #[cfg(not(target_os = "windows"))]
    {
        return false;
    }

    #[cfg(target_os = "windows")]
    {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;

        let Ok(console) = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Console") else {
            return false;
        };
        let Ok(startup) = console.open_subkey("%%Startup") else {
            return false;
        };

        let Ok(delegation_console): Result<String, _> =
            startup.get_value::<String, _>("DelegationConsole")
        else {
            return false;
        };
        let Ok(delegation_terminal): Result<String, _> =
            startup.get_value::<String, _>("DelegationTerminal")
        else {
            return false;
        };

        delegation_console != delegation_terminal
    }
}

impl TerminalSignature {
    /// Checks if the terminal signature is met.
    #[inline]
    #[must_use]
    pub fn check(&self) -> bool {
        logging::info!("Checking terminal signature: {:?}", self);
        match self {
            Self::EnvVarExists(var_name) => std::env::var(var_name).is_ok(),
            Self::EnvVar(var, value) => {
                matches!(std::env::var(var).ok().as_deref(),
                    Some(v) if v.eq_ignore_ascii_case(value))
            }
            Self::TermProgram(var_value) => {
                matches!(std::env::var(TERM_PROGRAM_VAR).ok().as_deref(),
                    Some(v) if v.eq_ignore_ascii_case(var_value))
            }
            Self::TermVar(var_value) => {
                matches!(std::env::var(TERM_VAR).ok().as_deref(),
                    Some(v) if v.eq_ignore_ascii_case(var_value))
            }
            Self::WindowsConsoleDelegationSet => check_for_windows_registry_delegation(),
            Self::Any(sigs) => sigs.iter().any(TerminalSignature::check),
        }
    }
}

/// Represents a terminal identifier, which consists of a terminal type and a set of signatures
/// that can be used to identify that terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TerminalIdentifier {
    /// The type of terminal.
    pub kind: TerminalType,
    /// The operating system this terminal runs on.
    pub target_os: TargetOperatingSystem,
    /// The signatures that can be used to identify this terminal.
    pub signatures: &'static [TerminalSignature],
}

// Identification..

/// Returns the default terminal type for a given operating system.
#[inline]
#[must_use]
pub const fn get_default_terminal_for_os(os: OperatingSystem) -> TerminalType {
    match os {
        OperatingSystem::Windows => TerminalType::WindowsCMD,
        OperatingSystem::MacOS => TerminalType::MacOS,
        OperatingSystem::Linux => TerminalType::LinuxTerminal,
        OperatingSystem::Unknown => TerminalType::Unknown,
    }
}

/// Returns an iterator over possible terminal identifiers.
#[inline]
pub fn get_all_terminal_identifiers() -> impl Iterator<Item = &'static TerminalIdentifier> {
    terminal_providers::TERMINAL_IDENTIFIERS
        .iter()
        .chain(terminal_providers::FINAL_TERMINAL_IDENTIFIERS)
}

/// Returns an iterator over possible terminal identifiers for the given operating system.
#[inline]
pub fn get_possible_terminal_identifiers_for(
    os: OperatingSystem,
) -> impl Iterator<Item = &'static TerminalIdentifier> {
    get_all_terminal_identifiers()
        .filter(move |identifier| os.compatible_with_target(identifier.target_os))
}

/// Attempts to identify the current terminal type based on a list of known terminal identification signatures.
#[inline]
#[must_use]
pub fn find_current_terminal() -> TerminalType {
    let current_os = OperatingSystem::current();

    for identifier in get_possible_terminal_identifiers_for(current_os) {
        // Check all signatures
        if identifier.signatures.iter().all(TerminalSignature::check) {
            return identifier.kind;
        }
    }

    logging::info!(
        "No terminal signatures matched for current terminal, falling back to default terminal for OS."
    );

    // If no terminal matched, return the default terminal for the current OS
    get_default_terminal_for_os(current_os)
}

/// Returns an iterator over possible preferred terminals for the given operating system.
#[inline]
pub fn get_preferred_terminals_for_os(os: OperatingSystem) -> impl Iterator<Item = TerminalType> {
    TerminalType::iter().filter(move |terminal_type| {
        terminal_type.is_preferred() && os.compatible_with_target(terminal_type.target_os())
    })
}

/// Returns `true` if the current program has been relaunched by the library in a new terminal already.
#[inline]
#[must_use]
pub fn has_been_relaunched() -> bool {
    std::env::args().any(|arg| arg == RELAUNCHED_ARGUMENT)
}

/// Constant indicating no override for is active.
const NO_OVERRIDE: u8 = 0;
/// Constant indicating override is true.
const OVERRIDE_TRUE: u8 = 1;
/// Constant indicating override is false.
const OVERRIDE_FALSE: u8 = 2;

/// Global override for unicode support detection.
static OVERRIDE_UNICODE_SUPPORT: atomic::AtomicU8 = atomic::AtomicU8::new(NO_OVERRIDE);
/// Global override for unicode support detection.
static OVERRIDE_RGB_ANSI_COLOURS: atomic::AtomicU8 = atomic::AtomicU8::new(NO_OVERRIDE);

/// Stores the override value into the given atomic target.
#[inline]
fn store_override(target: &atomic::AtomicU8, supports: Option<bool>) {
    match supports {
        Some(true) => target.store(OVERRIDE_TRUE, atomic::Ordering::SeqCst),
        Some(false) => target.store(OVERRIDE_FALSE, atomic::Ordering::SeqCst),
        None => target.store(NO_OVERRIDE, atomic::Ordering::SeqCst),
    }
}

/// Reads the override value from the given atomic target.
#[inline]
fn read_override(target: &atomic::AtomicU8) -> Option<bool> {
    match target.load(atomic::Ordering::SeqCst) {
        OVERRIDE_TRUE => Some(true),
        OVERRIDE_FALSE => Some(false),
        _ => None,
    }
}

/// Overrides the detected full unicode support for the current terminal.
///
/// # Notes
/// If `supports` is `None`, the override is cleared and automatic detection is used again.
#[inline]
pub fn set_unicode_support_override(supports: Option<bool>) {
    store_override(&OVERRIDE_UNICODE_SUPPORT, supports);
}

/// Overrides the detected RGB (ANSI) colour support for the current terminal.
///
/// # Notes
/// If `supports` is `None`, the override is cleared and automatic detection is used again.
#[inline]
pub fn set_rgb_ansi_override(supports: Option<bool>) {
    store_override(&OVERRIDE_RGB_ANSI_COLOURS, supports);
}

/// Reads the current override for full unicode support detection.
/// # Returns
/// *   `Some(overriden_state)` if full unicode support is overridden.
/// *   `None` if no override is set and automatic detection should be used.
#[inline]
#[must_use]
pub fn is_unicode_overridden() -> Option<bool> {
    read_override(&OVERRIDE_UNICODE_SUPPORT)
}

/// Reads the current override for rgb (ANSI) colour support detection.
/// # Returns
/// *   `Some(overriden_state)` if full unicode support is overridden.
/// *   `None` if no override is set and automatic detection should be used.
#[inline]
#[must_use]
pub fn is_rgb_ansi_overridden() -> Option<bool> {
    read_override(&OVERRIDE_RGB_ANSI_COLOURS)
}

/// The current terminal type detected at runtime.
pub static CURRENT_TERMINAL: LazyLock<TerminalType> = LazyLock::new(find_current_terminal);

/// If the current terminal supports full unicode rendering.
pub static SUPPORTS_FULL_UNICODE: LazyLock<bool> = LazyLock::new(|| {
    if let Some(override_state) = is_unicode_overridden() {
        override_state
    } else {
        CURRENT_TERMINAL.supports_full_unicode()
    }
});

/// If the current terminal supports full RGB (ANSI) colours.
pub static SUPPORTS_RGB_ANSI_COLOURS: LazyLock<bool> = LazyLock::new(|| {
    if let Some(override_state) = is_unicode_overridden() {
        override_state
    } else {
        CURRENT_TERMINAL.supports_rgb_ansi_colours()
    }
});

/// Argument passed to relaunched terminals to indicate a relaunch has occurred.
pub const RELAUNCHED_ARGUMENT: &str = "--relaunched-term";

/// A trait for terminal providers that can supply terminal types, check installation status and relaunch the
/// program in their terminal.
pub trait TerminalProvider {
    /// Returns the terminal type provided by this provider.
    #[must_use]
    fn terminal_type(&self) -> TerminalType;

    /// Returns `true` if the terminal is installed on the system.
    #[must_use]
    fn is_installed(&self) -> bool;

    /// Attempts to relaunch the current program in the terminal provided by this provider,
    /// with the given arguments, if installed.
    /// # Errors
    /// Returns an `std::io::Error` if any I/O operations fail.
    fn relaunch_in_terminal(&self) -> TermResult<()>;
}

/// Returns `true` if we should attempt to find and relaunch in a preferred terminal.
#[inline]
#[must_use]
pub fn should_attempt_relaunch() -> bool {
    !has_been_relaunched() && !CURRENT_TERMINAL.is_preferred()
}

/// Returns an alternative preferred terminal provider, if one is found and installed.
#[inline]
#[must_use]
pub fn find_alternative_terminal() -> Option<Box<dyn TerminalProvider>> {
    let current_os = OperatingSystem::current();

    for terminal_type in get_preferred_terminals_for_os(current_os) {
        logging::info!(
            "Testing if preferred terminal `{}` is installed.",
            terminal_type.name()
        );
        if let Some(provider) = get_provider_for_terminal(terminal_type)
            && provider.is_installed()
        {
            logging::info!("`{}` is installed!", terminal_type.name());
            return Some(provider);
        }
    }

    None
}

/// Returns a terminal provider for the given terminal type, if available.
#[inline]
#[must_use]
pub fn get_provider_for_terminal(terminal_type: TerminalType) -> Option<Box<dyn TerminalProvider>> {
    match terminal_type {
        TerminalType::WindowsTerminal => Some(Box::new(WindowsTerminalProvider)),
        TerminalType::ITerm2 => Some(Box::new(ITerm2Provider)),
        TerminalType::Ghostty => Some(Box::new(GhosttyProvider)),
        TerminalType::Kitty => Some(Box::new(KittyProvider)),
        TerminalType::Alacritty => Some(Box::new(AlacrittyProvider)),
        _ => None,
    }
}

/// Attempts to relaunch the current program in a preferred terminal, if one is found and installed.
/// # Notes
/// If this function returns `Ok(())`, the current program has been relaunched and the current instance should exit.
///
/// You should check `should_attempt_relaunch()` before calling this function to avoid unnecessary work.
/// This function is public just incase you want to call it directly, but it's recommended to use
/// `should_attempt_relaunch()` first.
///
/// # Errors
/// Returns a `RelaunchError` if no preferred terminal is found or if the relaunch fails.
///
/// # Returns
/// *   `Ok(())` if the relaunch was successful, if `Ok(())` is returned, the current instance should exit.
/// *   `Err(RelaunchError)` if no preferred terminal is found or if the relaunch fails.
#[inline]
pub fn try_relaunch_in_preferred_terminal() -> TermResult<()> {
    if let Some(provider) = find_alternative_terminal() {
        provider.relaunch_in_terminal()
    } else {
        logging::warning!("No alternative preferred terminal found for relaunch.");
        Err(RelaunchError::NoAlternativeTerminalFound)
    }
}

/// Attempts to relaunch the current program in a preferred terminal, if we have not already relaunched the application,
/// and if the current terminal does not meet the preferred terminal requirements, i.e. full unicode and RGB (ANSI) colour support.
/// and an alternative preferred terminal is found and installed.
///
/// # Notes
/// *   If this function returns `Ok(true)`, the current program has been relaunched and the current instance should exit.
/// *   If this function returns `Ok(false)`, the program was already relaunched, or the current terminal meets the feature
///     requirements, and program execution can continue as normal.
///
/// # Errors
/// Returns a `RelaunchError` if no preferred terminal is found or if the relaunch fails.
///
/// # Returns
/// *   `Ok(true)` if the relaunch was successful, if this is returned, the current instance should exit.
/// *   `Ok(false)` if the program was already relaunched, or the current terminal meets the feature requirements,
///     and program execution can continue as normal.
/// *   `Err(RelaunchError)` if no preferred terminal is found or if the relaunch fails.
#[inline]
pub fn relaunch_if_available() -> TermResult<bool> {
    // Check if we should attempt to relaunch
    if should_attempt_relaunch() {
        try_relaunch_in_preferred_terminal()?;
        Ok(true)
    } else {
        // No relaunch needed
        Ok(false)
    }
}

/// Attempts to relaunch the current program in a preferred terminal, if we have not already relaunched the application,
/// and if the current terminal does not meet the preferred terminal requirements, i.e. full unicode and RGB (ANSI) colour support.
/// and an alternative preferred terminal is found and installed.
///
/// If an alternative preferred terminal is found and the relaunch is successful, this function will exit the current process.
///
/// # Errors
/// Returns a `RelaunchError` if no preferred terminal is found or if the relaunch fails.
///
/// # Returns
/// *   `Ok(())` if the program was already relaunched, or the current terminal meets the feature requirements,
///     and program execution can continue as normal.
/// *   `Err(RelaunchError)` if no preferred terminal is found or if the relaunch fails.
#[inline]
pub fn relaunch_if_available_and_exit() -> TermResult<()> {
    const DEFAULT_EXIT_CODE: i32 = 0;

    relaunch_if_available_and_exit_with(DEFAULT_EXIT_CODE)
}

/// Attempts to relaunch the current program in a preferred terminal, if we have not already relaunched the application,
/// and if the current terminal does not meet the preferred terminal requirements, i.e. full unicode and RGB (ANSI) colour support.
/// and an alternative preferred terminal is found and installed.
///
/// If an alternative preferred terminal is found and the relaunch is successful,
/// this function will exit the current process, with the given exit code.
///
/// # Errors
/// Returns a `RelaunchError` if no preferred terminal is found or if the relaunch fails.
///
/// # Returns
/// *   `Ok(())` if the program was already relaunched, or the current terminal meets the feature requirements,
///     and program execution can continue as normal.
/// *   `Err(RelaunchError)` if no preferred terminal is found or if the relaunch fails.
#[inline]
pub fn relaunch_if_available_and_exit_with(exit_code: i32) -> TermResult<()> {
    if relaunch_if_available()? {
        std::process::exit(exit_code);
    }

    Ok(())
}
