use crate::error::{ConvextError, ConvextResult};
use gtk4::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct KdeConnectDevice {
    pub id: String,
    pub name: String,
}

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

    pub fn open_terminal_in_folder(target: &Path) -> ConvextResult<()> {
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
            .map_err(|e| ConvextError::Desktop(format!("Could not launch terminal: {}", e)))?;
        Ok(())
    }

    pub fn list_kdeconnect_devices() -> Vec<KdeConnectDevice> {
        let mut devices = Vec::new();
        if let Ok(output) = Command::new("kdeconnect-cli")
            .args(["-a", "--id-name-only"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Some((id, name)) = trimmed.split_once(' ') {
                        devices.push(KdeConnectDevice {
                            id: id.trim().to_string(),
                            name: name.trim().to_string(),
                        });
                    }
                }
            }
        }
        devices
    }

    pub fn send_via_kdeconnect(paths: &[PathBuf], device_id: Option<&str>) -> ConvextResult<String> {
        if paths.is_empty() {
            return Ok("Dosya seçilmedi.".to_string());
        }

        let reachable = Self::list_kdeconnect_devices();
        let target_dev = if let Some(explicit_id) = device_id {
            reachable
                .iter()
                .find(|d| d.id == explicit_id)
                .cloned()
                .or_else(|| {
                    Some(KdeConnectDevice {
                        id: explicit_id.to_string(),
                        name: explicit_id.to_string(),
                    })
                })
        } else {
            reachable.first().cloned()
        };

        if let Some(dev) = target_dev {
            let mut sent_count = 0usize;
            for p in paths {
                let status = Command::new("kdeconnect-cli")
                    .args(["-d", &dev.id, "--share"])
                    .arg(p)
                    .status();
                if status.map(|s| s.success()).unwrap_or(false) {
                    sent_count += 1;
                }
            }
            if sent_count > 0 {
                return Ok(format!(
                    "KDE Connect ile '{}' cihazına {} dosya gönderildi.",
                    dev.name, sent_count
                ));
            }
        }

        // Fallback to kdeconnect-handler GUI dialog if no direct device matched or CLI share failed
        let mut handler_launched = false;
        for p in paths {
            if Command::new("kdeconnect-handler").arg(p).spawn().is_ok() {
                handler_launched = true;
            }
        }
        if handler_launched {
            return Ok("KDE Connect paylaşım penceresi açıldı.".to_string());
        }

        Err(ConvextError::ExternalBinary {
            binary: "kdeconnect-cli".to_string(),
            reason: "Ulaşılabilir KDE Connect cihazı bulunamadı.".to_string(),
        })
    }

    pub fn send_via_localsend(paths: &[PathBuf]) -> ConvextResult<String> {
        if paths.is_empty() {
            return Ok("Dosya seçilmedi.".to_string());
        }

        // 1. Try native binaries in PATH
        for bin in ["localsend_app", "localsend"] {
            let mut cmd = Command::new(bin);
            for p in paths {
                cmd.arg(p);
            }
            if cmd.spawn().is_ok() {
                return Ok("LocalSend başlatıldı.".to_string());
            }
        }

        // 2. Try Flatpak LocalSend (org.localsend.localsend_app)
        let mut fp_cmd = Command::new("flatpak");
        fp_cmd.args([
            "run",
            "--file-forwarding",
            "org.localsend.localsend_app",
            "@@",
        ]);
        for p in paths {
            fp_cmd.arg(p);
        }
        fp_cmd.arg("@@");
        if fp_cmd.spawn().is_ok() {
            return Ok("LocalSend (Flatpak) ile dosya gönderimi başlatıldı.".to_string());
        }

        Err(ConvextError::ExternalBinary {
            binary: "LocalSend".to_string(),
            reason: "LocalSend (native veya Flatpak) bulunamadı.".to_string(),
        })
    }

    pub fn send_via_kdeconnect_or_localsend(paths: &[PathBuf]) -> ConvextResult<String> {
        if paths.is_empty() {
            return Ok("Dosya seçilmedi.".to_string());
        }

        // First try active KDE Connect devices (zero-click instant share to paired phone/PC)
        let devices = Self::list_kdeconnect_devices();
        if !devices.is_empty() {
            if let Ok(msg) = Self::send_via_kdeconnect(paths, Some(&devices[0].id)) {
                return Ok(msg);
            }
        }

        // Next try LocalSend (native or Flatpak)
        if let Ok(msg) = Self::send_via_localsend(paths) {
            return Ok(msg);
        }

        // Finally try KDE Connect handler dialog
        Self::send_via_kdeconnect(paths, None)
    }

    pub fn send_desktop_notification(summary: &str, body: &str) {
        let _ = Command::new("gdbus")
            .args([
                "call",
                "--session",
                "--dest=org.freedesktop.Notifications",
                "--object-path=/org/freedesktop/Notifications",
                "--method=org.freedesktop.Notifications.Notify",
                "Convext",
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
    pub fn duplicate_files(files: &[PathBuf], suffix: &str) -> ConvextResult<Vec<PathBuf>> {
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

    pub fn move_files_to_organized(files: &[PathBuf]) -> ConvextResult<Vec<PathBuf>> {
        let mut outs = Vec::new();
        for p in files {
            let parent = p.parent().unwrap_or_else(|| Path::new("."));
            let target_dir = parent.join("Convext_Organized");
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
