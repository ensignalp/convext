pub mod input_listener;
pub mod surface_bridge;
pub mod tray;

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelMode {
    ConvertFormats,
    FileTools,
    FileActions,
}

#[derive(Debug, Clone)]
pub enum DaemonEvent {
    ShowWheelAt { x: i32, y: i32 },
    SwitchWheelMode(WheelMode),
    CycleWheelMode,
    HideWheel,
    ToggleFloatShelf,
    QuickCompressActive,
    TogglePause,
    SelectPreset(String),
    ShowConvextStudio {
        files: Vec<PathBuf>,
    },
    ExecuteConvextCommand {
        action_code: String,
        files: Vec<PathBuf>,
    },
    QuitDaemon,
}
