use crate::config::CitrusConfig;
use crate::daemon::DaemonEvent;
use async_channel::Sender;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

pub struct TrayService {
    config: Arc<RwLock<CitrusConfig>>,
    tx: Sender<DaemonEvent>,
}

impl TrayService {
    pub fn new(config: Arc<RwLock<CitrusConfig>>, tx: Sender<DaemonEvent>) -> Self {
        Self { config, tx }
    }

    pub fn spawn(self) {
        let _ = std::process::Command::new("gdbus")
            .args([
                "call",
                "--session",
                "--dest=org.freedesktop.Notifications",
                "--object-path=/org/freedesktop/Notifications",
                "--method=org.freedesktop.Notifications.Notify",
                "Convext",
                "0",
                "preferences-desktop-locale",
                "Convext Menü Aktif",
                "Dosyalara sağ tıklayıp 'Convext Menü' üzerinden tüm dönüştürme ve araçları kullanabilirsiniz.",
                "[]",
                "{}",
                "3000",
            ])
            .spawn();

        let tx_clone = self.tx.clone();
        let cfg_clone = self.config.clone();

        tokio::spawn(async move {
            let sock_path = Self::control_socket_path();
            let _ = std::fs::remove_file(&sock_path);
            if let Ok(listener) = UnixListener::bind(&sock_path) {
                while let Ok((mut stream, _)) = listener.accept().await {
                    let tx_inner = tx_clone.clone();
                    let cfg_inner = cfg_clone.clone();
                    tokio::spawn(async move {
                        let (reader, mut writer) = stream.split();
                        let mut buf_reader = BufReader::new(reader);
                        let mut line = String::new();
                        if buf_reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                            let cmd = line.trim_end_matches(&['\r', '\n'][..]);
                            let response = match cmd {
                                "PAUSE_TOGGLE" => {
                                    let _ = tx_inner.send(DaemonEvent::TogglePause).await;
                                    "OK: toggled pause\n".to_string()
                                }
                                "FLOAT_TOGGLE" => {
                                    let _ = tx_inner.send(DaemonEvent::ToggleFloatShelf).await;
                                    "OK: toggled float shelf\n".to_string()
                                }
                                "WHEEL_SHOW" => "OK: Convext context menu mode active (radial window disabled)\n".to_string(),
                                "QUIT" => {
                                    let _ = tx_inner.send(DaemonEvent::QuitDaemon).await;
                                    "OK: quitting\n".to_string()
                                }
                                "STATUS" => {
                                    let paused = cfg_inner
                                        .read()
                                        .map(|c| c.daemon_paused)
                                        .unwrap_or(false);
                                    let preset = cfg_inner
                                        .read()
                                        .map(|c| c.active_preset.clone())
                                        .unwrap_or_default();
                                    format!("STATUS: paused={}, preset={}\n", paused, preset)
                                }
                                other if other.starts_with("PRESET:") => {
                                    let preset_name = other.trim_start_matches("PRESET:").trim();
                                    let _ = tx_inner
                                        .send(DaemonEvent::SelectPreset(preset_name.to_string()))
                                        .await;
                                    format!("OK: selected preset {}\n", preset_name)
                                }
                                other if other.starts_with("CONVEXT:") => {
                                    let payload = other.trim_start_matches("CONVEXT:");
                                    let mut parts = payload.split('\t');
                                    let action_code = parts.next().unwrap_or("").trim().to_string();
                                    let files: Vec<PathBuf> = parts
                                        .map(|s| s.trim())
                                        .filter(|s| !s.is_empty())
                                        .map(PathBuf::from)
                                        .collect();
                                    if !action_code.is_empty() && !files.is_empty() {
                                        let _ = tx_inner
                                            .send(DaemonEvent::ExecuteConvextCommand {
                                                action_code: action_code.clone(),
                                                files,
                                            })
                                            .await;
                                        format!("OK: Convext executed {}\n", action_code)
                                    } else {
                                        "ERR: invalid CONVEXT payload\n".to_string()
                                    }
                                }
                                _ => "ERR: unknown command. Use CONVEXT:<action>\\t<file>, FLOAT_TOGGLE, STATUS, QUIT\n".to_string(),
                            };
                            let _ = writer.write_all(response.as_bytes()).await;
                        }
                    });
                }
            }
        });
    }

    pub fn control_socket_path() -> std::path::PathBuf {
        let runtime_dir = dirs::runtime_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        runtime_dir.join("citrus-daemon-tray.sock")
    }
}
