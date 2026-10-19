use crate::error::{ConvextError, ConvextResult};
use std::path::PathBuf;
use std::process::Command;

pub struct FileManagerBridge;

impl FileManagerBridge {
    pub fn show_items(paths: &[PathBuf]) -> ConvextResult<()> {
        if paths.is_empty() {
            return Ok(());
        }

        let mut uris = Vec::new();
        for p in paths {
            if let Ok(canon) = std::fs::canonicalize(p) {
                if let Ok(u) = url::Url::from_file_path(&canon) {
                    uris.push(format!("'{}'", u.as_str()));
                }
            } else if let Ok(u) = url::Url::from_file_path(p) {
                uris.push(format!("'{}'", u.as_str()));
            }
        }

        if uris.is_empty() {
            return Ok(());
        }

        let array_arg = format!("[{}]", uris.join(", "));
        let status = Command::new("gdbus")
            .args([
                "call",
                "--session",
                "--dest=org.freedesktop.FileManager1",
                "--object-path=/org/freedesktop/FileManager1",
                "--method=org.freedesktop.FileManager1.ShowItems",
                &array_arg,
                "convext",
            ])
            .status()
            .map_err(|e| ConvextError::Desktop(e.to_string()))?;

        if !status.success() {
            if let Some(parent) = paths[0].parent() {
                let _ = Command::new("xdg-open").arg(parent).spawn();
            }
        }
        Ok(())
    }
}

impl FileManagerBridge {
    pub fn detect_active_files() -> Vec<PathBuf> {
        let mut results = Vec::new();
        let runtime_dir = dirs::runtime_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        let sel_file = runtime_dir.join("convext-selected-files.txt");
        if let Ok(content) = std::fs::read_to_string(&sel_file) {
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    let p = PathBuf::from(trimmed);
                    if p.exists() && !results.contains(&p) {
                        results.push(p);
                    }
                }
            }
            if !results.is_empty() {
                return results;
            }
        }

        let clip_cmds: [(&str, &[&str]); 4] = [
            ("wl-paste", &["--no-newline", "--type", "text/uri-list"]),
            ("wl-paste", &["--no-newline"]),
            ("xclip", &["-selection", "clipboard", "-o"]),
            ("xclip", &["-selection", "primary", "-o"]),
        ];

        for (bin, args) in clip_cmds {
            if let Ok(out) = Command::new(bin).args(args).output() {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    for line in text.lines() {
                        let t = line.trim();
                        if t.starts_with("file://") {
                            if let Ok(u) = url::Url::parse(t) {
                                if let Ok(p) = u.to_file_path() {
                                    if p.exists() && !results.contains(&p) {
                                        results.push(p);
                                    }
                                }
                            }
                        } else if t.starts_with('/') {
                            let p = PathBuf::from(t);
                            if p.exists() && !results.contains(&p) {
                                results.push(p);
                            }
                        }
                    }
                    if !results.is_empty() {
                        return results;
                    }
                }
            }
        }

        results
    }
}
