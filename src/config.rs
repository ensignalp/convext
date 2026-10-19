use crate::error::{CitrusError, CitrusResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutBindings {
    pub trigger_wheel: String,
    pub toggle_shelf: String,
    pub quick_compress: String,
    pub strip_metadata: String,
}

impl Default for ShortcutBindings {
    fn default() -> Self {
        Self {
            trigger_wheel: "Shift_Tap".to_string(),
            toggle_shelf: "Alt+Shift_Tap".to_string(),
            quick_compress: "Ctrl_Tap".to_string(),
            strip_metadata: "Ctrl+Shift+M".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionPreset {
    pub name: String,
    pub target_format: String,
    pub quality_percent: u8,
    pub strip_exif: bool,
    pub max_dimension: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitrusConfig {
    pub daemon_paused: bool,
    pub auto_reveal_in_file_manager: bool,
    pub wheel_outer_radius: f64,
    pub wheel_inner_radius: f64,
    pub default_output_suffix: String,
    pub overwrite_original: bool,
    pub active_preset: String,
    pub shortcuts: ShortcutBindings,
    pub presets: Vec<ConversionPreset>,
}

impl Default for CitrusConfig {
    fn default() -> Self {
        Self {
            daemon_paused: false,
            auto_reveal_in_file_manager: true,
            wheel_outer_radius: 170.0,
            wheel_inner_radius: 56.0,
            default_output_suffix: "_citrus".to_string(),
            overwrite_original: false,
            active_preset: "Balanced Web".to_string(),
            shortcuts: ShortcutBindings::default(),
            presets: vec![
                ConversionPreset {
                    name: "Balanced Web".to_string(),
                    target_format: "webp".to_string(),
                    quality_percent: 82,
                    strip_exif: true,
                    max_dimension: Some(2560),
                },
                ConversionPreset {
                    name: "Archival Lossless".to_string(),
                    target_format: "png".to_string(),
                    quality_percent: 100,
                    strip_exif: false,
                    max_dimension: None,
                },
                ConversionPreset {
                    name: "Ultra Compact Social".to_string(),
                    target_format: "jpg".to_string(),
                    quality_percent: 65,
                    strip_exif: true,
                    max_dimension: Some(1920),
                },
            ],
        }
    }
}

impl CitrusConfig {
    pub fn config_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        base.join("citrus-daemon").join("config.toml")
    }

    pub fn load_or_create() -> CitrusResult<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            toml::from_str(&content).map_err(|e| CitrusError::Config(e.to_string()))
        } else {
            let cfg = Self::default();
            cfg.save()?;
            Ok(cfg)
        }
    }

    pub fn save(&self) -> CitrusResult<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized =
            toml::to_string_pretty(self).map_err(|e| CitrusError::Config(e.to_string()))?;
        fs::write(path, serialized)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct DependencyCapabilities {
    pub available: HashMap<String, bool>,
}

impl DependencyCapabilities {
    pub const REQUIRED_BINARIES: [&'static str; 8] = [
        "ffmpeg",
        "ffprobe",
        "exiftool",
        "qpdf",
        "pandoc",
        "libreoffice",
        "7z",
        "localsend_app",
    ];

    pub fn probe_system() -> Self {
        let mut available = HashMap::new();
        for &bin in &Self::REQUIRED_BINARIES {
            let exists = Self::which_binary(bin);
            available.insert(bin.to_string(), exists);
        }
        Self { available }
    }

    pub fn is_available(&self, binary: &str) -> bool {
        self.available.get(binary).copied().unwrap_or(false)
    }

    pub fn missing_binaries(&self) -> Vec<String> {
        Self::REQUIRED_BINARIES
            .iter()
            .filter(|&&b| !self.is_available(b))
            .map(|&s| s.to_string())
            .collect()
    }

    fn which_binary(binary: &str) -> bool {
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let candidate: PathBuf = dir.join(binary);
                if Self::is_executable(&candidate) {
                    return true;
                }
            }
        }
        false
    }

    fn is_executable(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(path) {
                return metadata.permissions().mode() & 0o111 != 0;
            }
        }
        false
    }

    pub fn notify_missing_dependencies(&self) {
        let missing = self.missing_binaries();
        if missing.is_empty() {
            return;
        }
        let summary = "Tangerine-Linux: Optional Tools Missing";
        let body = format!(
            "Disabled slices requiring: {}. Install them for full offline capabilities.",
            missing.join(", ")
        );
        let _ = Command::new("gdbus")
            .args([
                "call",
                "--session",
                "--dest=org.freedesktop.Notifications",
                "--object-path=/org/freedesktop/Notifications",
                "--method=org.freedesktop.Notifications.Notify",
                "Tangerine-Linux",
                "0",
                "dialog-information",
                summary,
                &body,
                "[]",
                "{}",
                "5000",
            ])
            .spawn();
    }
}
