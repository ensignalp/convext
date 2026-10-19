use crate::error::{CitrusError, CitrusResult};
use gtk4::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct DesktopActions;

impl DesktopActions {
    pub fn copy_text_to_clipboard(text: &str) {
        if let Some(display) = gdk4::Display::default() {
            let clipboard = display.clipboard();
            clipboard.set_text(text);
        }
    }

    pub fn copy_paths_to_clipboard(paths: &[PathBuf]) {
        let joined = paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("\n");
        Self::copy_text_to_clipboard(&joined);
    }

    pub fn open_terminal_in_folder(target: &Path) -> CitrusResult<()> {
        let dir = if target.is_dir() {
            target
        } else {
            target.parent().unwrap_or_else(|| Path::new("."))
        };

        let candidates: [(&str, &[&str]); 6] = [
            ("ptyxis", &["--working-directory"]),
            ("kgx", &["--working-directory"]),
            ("gnome-terminal", &["--working-directory"]),
            ("konsole", &["--workdir"]),
            ("xfce4-terminal", &["--working-directory"]),
            ("alacritty", &["--working-directory"]),
        ];

        for (bin, flag) in candidates {
            let mut cmd = Command::new(bin);
            for f in flag {
                cmd.arg(f);
            }
            cmd.arg(dir);
            if cmd.spawn().is_ok() {
                return Ok(());
            }
        }

        Command::new("x-terminal-emulator")
            .current_dir(dir)
            .spawn()
            .map_err(|e| CitrusError::Desktop(format!("Could not launch terminal: {}", e)))?;
        Ok(())
    }

    pub fn send_via_localsend(paths: &[PathBuf]) -> CitrusResult<()> {
        if paths.is_empty() {
            return Ok(());
        }
        let mut cmd = Command::new("localsend_app");
        for p in paths {
            cmd.arg(p);
        }
        cmd.spawn().map_err(|e| CitrusError::ExternalBinary {
            binary: "localsend_app".to_string(),
            reason: e.to_string(),
        })?;
        Ok(())
    }

    pub fn send_desktop_notification(summary: &str, body: &str) {
        let _ = Command::new("gdbus")
            .args([
                "call",
                "--session",
                "--dest=org.freedesktop.Notifications",
                "--object-path=/org/freedesktop/Notifications",
                "--method=org.freedesktop.Notifications.Notify",
                "Tangerine-Linux",
                "0",
                "emblem-ok-symbolic",
                summary,
                body,
                "[]",
                "{}",
                "4000",
            ])
            .spawn();
    }
}

impl DesktopActions {
    pub fn duplicate_files(files: &[PathBuf], suffix: &str) -> CitrusResult<Vec<PathBuf>> {
        let mut outs = Vec::new();
        for p in files {
            if p.is_file() {
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("copy");
                let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                let parent = p.parent().unwrap_or_else(|| Path::new("."));
                let dest = if ext.is_empty() {
                    parent.join(format!("{}{}_copy", stem, suffix))
                } else {
                    parent.join(format!("{}{}_copy.{}", stem, suffix, ext))
                };
                std::fs::copy(p, &dest)?;
                outs.push(dest);
            }
        }
        Ok(outs)
    }

    pub fn move_files_to_organized(files: &[PathBuf]) -> CitrusResult<Vec<PathBuf>> {
        let mut outs = Vec::new();
        for p in files {
            let parent = p.parent().unwrap_or_else(|| Path::new("."));
            let target_dir = parent.join("Tangerine_Moved");
            std::fs::create_dir_all(&target_dir)?;
            if let Some(fname) = p.file_name() {
                let dest = target_dir.join(fname);
                std::fs::rename(p, &dest)?;
                outs.push(dest);
            }
        }
        Ok(outs)
    }
}
