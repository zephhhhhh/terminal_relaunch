use std::path::PathBuf;
use std::process::Command;

use crate::{
    TargetOperatingSystem, TerminalIdentifier, TerminalProvider, TerminalSignature as TermSig,
    TerminalType, errors::TermResult,
};

use crate::RELAUNCHED_ARGUMENT;
#[allow(unused_imports)]
use crate::errors::RelaunchError;

/// Common environment variable `TERM_PROGRAM` used in terminal identification.
pub const TERM_PROGRAM_VAR: &str = "TERM_PROGRAM";
/// Common environment variable `TERM` used in terminal identification.
pub const TERM_VAR: &str = "TERM";

/// A list of known terminal identifiers with their associated signatures.
pub const TERMINAL_IDENTIFIERS: &[TerminalIdentifier] = &[
    TerminalIdentifier {
        kind: TerminalType::VSCode,
        target_os: TargetOperatingSystem::Any,
        signatures: &[TermSig::TermProgram("vscode")],
    },
    TerminalIdentifier {
        kind: TerminalType::Nvim,
        target_os: TargetOperatingSystem::Any,
        signatures: &[TermSig::EnvVarExists("NVIM")],
    },
    TerminalIdentifier {
        kind: TerminalType::ITerm2,
        target_os: TargetOperatingSystem::Any,
        signatures: &[TermSig::EnvVarExists("ITERM_SESSION_ID")],
    },
    TerminalIdentifier {
        kind: TerminalType::Alacritty,
        target_os: TargetOperatingSystem::Any,
        signatures: &[TermSig::EnvVarExists("ALACRITTY_LOG")],
    },
    TerminalIdentifier {
        kind: TerminalType::WezTerm,
        target_os: TargetOperatingSystem::Any,
        signatures: &[TermSig::TermProgram("WezTerm")],
    },
    TerminalIdentifier {
        kind: TerminalType::Kitty,
        target_os: TargetOperatingSystem::MacOS,
        signatures: &[TermSig::TermVar("xterm-kitty")],
    },
    TerminalIdentifier {
        kind: TerminalType::Ghostty,
        target_os: TargetOperatingSystem::MacOS,
        signatures: &[TermSig::TermProgram("ghostty")],
    },
];

/// A list of terminal identifiers to check last, typically for terminals that may be falsely detected when
/// checking other terminals first.
pub const FINAL_TERMINAL_IDENTIFIERS: &[TerminalIdentifier] = &[
    TerminalIdentifier {
        kind: TerminalType::WindowsTerminal,
        target_os: TargetOperatingSystem::Windows,
        signatures: &[TermSig::Any(&[
            TermSig::WindowsConsoleDelegationSet,
            TermSig::EnvVarExists("WT_SESSION"),
        ])],
    },
    TerminalIdentifier {
        kind: TerminalType::MacOS,
        target_os: TargetOperatingSystem::MacOS,
        signatures: &[TermSig::TermProgram("Apple_Terminal")],
    },
];

macro_rules! for_target {
    ($target_os: literal, $code:block) => {{
        #[cfg(target_os = $target_os)]
        $code

        #[cfg(not(target_os = $target_os))]
        {
            false
        }
    }};
    ($self:expr, $target_os: literal, $code:block) => {{
        #[cfg(not(target_os = $target_os))]
        {
            Err(RelaunchError::UnsupportedTerminalProvider(
                $self.terminal_type(),
            ))
        }

        #[cfg(target_os = $target_os)]
        $code
    }};
}

// Providers..

/// Retrieves the current executable path, working directory, and command-line arguments.
#[inline]
#[must_use]
fn get_relaunch_params() -> (PathBuf, PathBuf, Vec<String>) {
    let current_exe = std::env::current_exe().expect("Failed to get current executable path");
    let current_wd = std::env::current_dir().expect("Failed to get current working directory");
    let args: Vec<String> = [RELAUNCHED_ARGUMENT.to_string()]
        .into_iter()
        .chain(std::env::args().skip(1))
        .collect();

    (current_exe, current_wd, args)
}

/// Terminal provider for `Windows Terminal`.
pub struct WindowsTerminalProvider;

impl TerminalProvider for WindowsTerminalProvider {
    fn terminal_type(&self) -> TerminalType {
        TerminalType::WindowsTerminal
    }

    fn is_installed(&self) -> bool {
        for_target!("windows", {
            /// Path to Windows Terminal install in registry.
            const WINDOWS_TERMINAL_INSTALL_PATH: &str =
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\wt.exe";

            use winreg::RegKey;
            use winreg::enums::HKEY_CURRENT_USER;

            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            hkcu.open_subkey(WINDOWS_TERMINAL_INSTALL_PATH).is_ok()
        })
    }

    fn relaunch_in_terminal(&self) -> TermResult<()> {
        for_target!(self, "windows", {
            let (curr_exe, curr_wd, args) = get_relaunch_params();

            Command::new("wt")
                .arg("new-tab")
                .arg("--startingDirectory")
                .arg(curr_wd)
                .arg("--")
                .arg(curr_exe)
                .args(&args)
                .spawn()?;

            Ok(())
        })
    }
}

/// Escapes a string for safe embedding in a shell single-quoted string.
#[allow(dead_code)]
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// Escapes a list of arguments for safe embedding in a shell command.
#[allow(dead_code)]
fn shell_escape_args(args: &[String]) -> String {
    args.iter()
        .map(|a| shell_escape(a))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Terminal provider for `ITerm2`.
pub struct ITerm2Provider;

impl TerminalProvider for ITerm2Provider {
    fn terminal_type(&self) -> TerminalType {
        TerminalType::ITerm2
    }

    fn is_installed(&self) -> bool {
        for_target!("macos", {
            const ITERM_APP: &str = "/Applications/iTerm.app";

            std::path::Path::new(ITERM_APP).exists()
        })
    }

    fn relaunch_in_terminal(&self) -> TermResult<()> {
        for_target!(self, "macos", {
            let (curr_exe, curr_wd, args) = get_relaunch_params();

            let quoted_wd = shell_escape(&curr_wd.to_string_lossy());
            let quoted_exe = shell_escape(&curr_exe.to_string_lossy());
            let quoted_args = shell_escape_args(&args);

            let cmd = format!("cd {quoted_wd}; exec {quoted_exe} {quoted_args}");

            let script = format!(
                r#"
tell application "iTerm"
    activate
    repeat 50 times
        if (count of windows) > 0 then exit repeat
        delay 0.1
    end repeat
    if (count of windows) > 0 then
        tell current session of current window
            write text "{cmd}"
        end tell
    else
        error "no window" number 20
    end if
end tell
"#
            );

            let res = Command::new("osascript")
                .arg("-e")
                .arg(script)
                .spawn()?
                .wait()?;

            if res.success() {
                Ok(())
            } else {
                crate::logging::error!("ITerm2 launch exited unsuccessfully!");
                Err(RelaunchError::FailedToLaunchTerminal(
                    self.terminal_type(),
                    res.clone(),
                ))
            }
        })
    }
}

/// Terminal provider for `Ghostty`.
pub struct GhosttyProvider;

impl TerminalProvider for GhosttyProvider {
    fn terminal_type(&self) -> TerminalType {
        TerminalType::Ghostty
    }

    fn is_installed(&self) -> bool {
        for_target!("macos", {
            const GHOSTTY_APP: &str = "/Applications/Ghostty.app";

            std::path::Path::new(GHOSTTY_APP).exists()
        })
    }

    fn relaunch_in_terminal(&self) -> TermResult<()> {
        for_target!(self, "macos", {
            let (curr_exe, curr_wd, args) = get_relaunch_params();

            Command::new("open")
                .arg("-na")
                .arg("Ghostty")
                .arg("--args")
                .arg("-e")
                .arg(curr_exe)
                .args(args)
                .current_dir(curr_wd)
                .spawn()?;

            Ok(())
        })
    }
}

/// Terminal provider for `Kitty`.
pub struct KittyProvider;

impl TerminalProvider for KittyProvider {
    fn terminal_type(&self) -> TerminalType {
        TerminalType::Kitty
    }

    fn is_installed(&self) -> bool {
        for_target!("macos", {
            const KITTY_APP: &str = "/Applications/kitty.app";

            std::path::Path::new(KITTY_APP).exists()
        })
    }

    fn relaunch_in_terminal(&self) -> TermResult<()> {
        for_target!(self, "macos", {
            let (curr_exe, curr_wd, args) = get_relaunch_params();

            Command::new("open")
                .arg("-na")
                .arg("kitty")
                .arg("--args")
                .arg(curr_exe)
                .args(args)
                .current_dir(curr_wd)
                .spawn()?;

            Ok(())
        })
    }
}

/// Terminal provider for `Alacritty`.
pub struct AlacrittyProvider;

impl TerminalProvider for AlacrittyProvider {
    fn terminal_type(&self) -> TerminalType {
        TerminalType::Alacritty
    }

    fn is_installed(&self) -> bool {
        for_target!("macos", {
            const ALACRITTY_APP: &str = "/Applications/alacritty.app";

            std::path::Path::new(ALACRITTY_APP).exists()
        })
    }

    fn relaunch_in_terminal(&self) -> TermResult<()> {
        for_target!(self, "macos", {
            let (curr_exe, curr_wd, args) = get_relaunch_params();

            Command::new("open")
                .arg("-na")
                .arg("alacritty")
                .arg("--args")
                .arg("-e")
                .arg(curr_exe)
                .args(args)
                .current_dir(curr_wd)
                .spawn()?;

            Ok(())
        })
    }
}

/// Terminal provider for `WezTerm`.
pub struct WezTermProvider;

impl TerminalProvider for WezTermProvider {
    fn terminal_type(&self) -> TerminalType {
        TerminalType::WezTerm
    }

    fn is_installed(&self) -> bool {
        for_target!("macos", {
            const WEZ_TERM_APP: &str = "/Applications/WezTerm.app";

            std::path::Path::new(WEZ_TERM_APP).exists()
        })
    }

    fn relaunch_in_terminal(&self) -> TermResult<()> {
        for_target!(self, "macos", {
            let (curr_exe, curr_wd, args) = get_relaunch_params();

            Command::new("open")
                .arg("-na")
                .arg("WezTerm")
                .arg("--args")
                .arg("-e")
                .arg(curr_exe)
                .args(args)
                .current_dir(curr_wd)
                .spawn()?;

            Ok(())
        })
    }
}
