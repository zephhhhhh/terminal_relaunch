use crate::{
    TargetOperatingSystem, TerminalIdentifier, TerminalProvider, TerminalSignature as TermSig,
    TerminalType, errors::TermResult,
};

#[allow(unused_imports)]
use crate::errors::RelaunchError;

/// Common environment variable names used in terminal identification.
pub const TERM_PROGRAM_VAR: &str = "TERM_PROGRAM";

/// Value of `TERM_PROGRAM` defined inside a `VSCode` terminal.
pub const VSCODE_TERM_PROGRAM: &str = "vscode";
/// Value of `TERM_PROGRAM` defined inside a default `MacOS` terminal.
pub const APPLE_TERMINAL_TERM_PROGRAM: &str = "Apple_Terminal";

/// Environment variable for Windows Terminal session.
pub const WINDOWS_TERMINAL_VAR: &str = "WT_SESSION";

/// A list of known terminal identifiers with their associated signatures.
pub const TERMINAL_IDENTIFIERS: &[TerminalIdentifier] = &[
    TerminalIdentifier {
        kind: TerminalType::WindowsTerminal,
        target_os: TargetOperatingSystem::Windows,
        signatures: &[TermSig::EnvVarExists(WINDOWS_TERMINAL_VAR)],
    },
    TerminalIdentifier {
        kind: TerminalType::VSCode,
        target_os: TargetOperatingSystem::Any,
        signatures: &[TermSig::EnvVar(TERM_PROGRAM_VAR, VSCODE_TERM_PROGRAM)],
    },
    TerminalIdentifier {
        kind: TerminalType::MacOS,
        target_os: TargetOperatingSystem::MacOS,
        signatures: &[TermSig::EnvVar(
            TERM_PROGRAM_VAR,
            APPLE_TERMINAL_TERM_PROGRAM,
        )],
    },
];

// Providers..

/// Terminal provider for Windows Terminal.
pub struct WindowsTerminalProvider;

impl TerminalProvider for WindowsTerminalProvider {
    fn terminal_type(&self) -> TerminalType {
        TerminalType::WindowsTerminal
    }

    fn is_installed(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            /// Path to Windows Terminal install in registry.
            const WINDOWS_TERMINAL_INSTALL_PATH: &str =
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\wt.exe";

            use winreg::RegKey;
            use winreg::enums::HKEY_CURRENT_USER;

            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            hkcu.open_subkey(WINDOWS_TERMINAL_INSTALL_PATH).is_ok()
        }

        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }

    fn relaunch_in_terminal(&self) -> TermResult<()> {
        #[cfg(not(windows))]
        {
            Err(RelaunchError::UnsupportedTerminalProvider(
                TerminalType::WindowsTerminal,
            ))
        }

        #[cfg(windows)]
        {
            use std::env;
            use std::process::Command;

            use crate::RELAUNCHED_ARGUMENT;

            let current_exe = env::current_exe()?;
            let current_wd = env::current_dir()?;
            let args: Vec<String> = env::args().skip(1).collect();

            Command::new("wt")
                .arg("new-tab")
                .arg("--startingDirectory")
                .arg(current_wd)
                .arg("--")
                .arg(current_exe)
                .arg(RELAUNCHED_ARGUMENT)
                .args(&args)
                .spawn()?;

            Ok(())
        }
    }
}
