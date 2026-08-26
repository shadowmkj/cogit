// ==============================================================================
// Cross-Platform Clipboard Helper
// ==============================================================================
//
// Copies generated Markdown or commit texts to the user's OS system clipboard
// using native platform utilities (pbcopy on macOS, clip on Windows, wl-copy/xclip on Linux).

use anyhow::{Context, Result, bail};
use std::io::Write;
use std::process::{Command, Stdio};

/// Copies text content to the OS system clipboard using native clipboard tools.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .context("spawn pbcopy command")?;

    #[cfg(target_os = "windows")]
    let mut child = Command::new("clip")
        .stdin(Stdio::piped())
        .spawn()
        .context("spawn clip command")?;

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut child = {
        if Command::new("wl-copy").arg("--version").output().is_ok() {
            Command::new("wl-copy")
                .stdin(Stdio::piped())
                .spawn()
                .context("spawn wl-copy command")?
        } else if Command::new("xclip").arg("-version").output().is_ok() {
            Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(Stdio::piped())
                .spawn()
                .context("spawn xclip command")?
        } else {
            bail!("No supported clipboard tool found (install wl-copy or xclip).");
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .context("write text to clipboard tool stdin")?;
    }

    let status = child
        .wait()
        .context("wait for clipboard command to complete")?;
    if !status.success() {
        bail!("Clipboard utility exited with non-zero status");
    }

    Ok(())
}
