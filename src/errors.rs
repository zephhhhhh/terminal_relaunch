use std::process::ExitStatus;

use thiserror::Error;

use crate::TerminalType;

/// Errors that can occur during terminal relaunch operations.
#[derive(Error, Debug)]
pub enum RelaunchError {
    /// No alternative terminal was found to relaunch in.
    #[error("No alternative terminal found to relaunch in.")]
    NoAlternativeTerminalFound,
    /// The terminal provider is unsupported on this platform.
    #[error("The terminal provider for {0} is unsupported on this platform.")]
    UnsupportedTerminalProvider(TerminalType),
    /// An error occured when trying to relaunch in the specified terminal.
    #[error("Failed to launch terminal `{0}`. Exit status: {1:?}")]
    FailedToLaunchTerminal(TerminalType, ExitStatus),
    /// An I/O error occurred.
    #[error("I/O error occurred: {0:?}")]
    IOError(#[from] std::io::Error),
}

/// A specialized `Result` type for terminal relaunch operations.
pub type TermResult<T> = Result<T, RelaunchError>;
