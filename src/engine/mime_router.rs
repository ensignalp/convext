use crate::config::DependencyCapabilities;
use crate::daemon::WheelMode;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaCategory {
    Image,
    Audio,
    Video,
    Pdf,
    Document,
    Epub,
    Slides,
    Spreadsheet,
    Subtitles,
    Archive,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceIcon {
    None,
    Compress,
    Metadata,
    Edit,
    Annotate,
    AddBg,
    Crop,
    Redact,
    Mute,
    Trim,
    Speed,
    Snapshot,
    Split,
    Normalize,
    Visualize,
    Channels,
    Bleep,
    Organize,
    Extract,
    AirDrop,
    Float,
    Copy,
    Move,
    CopyPath,
    Terminal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SliceAction {
    // ConvertFormats
    ImageConvert { format: String },
    AudioConvert { format: String },
    VideoConvert { format: String },
    PdfConvert { format: String },
    DocConvert { format: String },
    SlidesConvert { format: String },
    SpreadsheetConvert { format: String },
    SubtitleConvert { format: String },
    ArchiveConvert { format: String },

    // Image Tools
    ImageCompressTool,
    StripMetadata,
    ImageEditTool,
    ImageAnnotateTool,
    ImageAddBgTool,
    ImageCropTool,
    ImageRedactTool,

    // Video Tools
    VideoCompressTool,
    VideoMuteTool,
    VideoTrimTool,
    VideoCropTool,
    VideoSpeedTool,
    VideoSnapshotTool,
    VideoSplitTool,
    VideoRedactTool,

    // Audio Tools
    AudioCompressTool,
    AudioNormalizeTool,
    AudioVisualizeTool,
    AudioTrimTool,
    AudioChannelsTool,
    AudioBleepTool,

    // PDF Tools
    PdfCompressTool,
    PdfSplitTool,
    PdfOrganizeTool,

    // Archive Tools
    ArchiveExtractHere,

    // File Actions (Control)
    ActionAirDrop,
    ActionFloat,
    ActionCopyFiles,
    ActionMoveFiles,
    ActionCopyPath,
    ActionOpenTerminal,
}

#[derive(Debug, Clone)]
pub struct WheelSlice {
    pub id: String,
    pub label: String,
    pub center_label: String,
    pub subtitle: String,
    pub icon: SliceIcon,
    pub enabled: bool,
    pub required_binary: Option<String>,
    pub action: SliceAction,
}

pub struct MimeRouter;

impl MimeRouter {
    pub fn inspect_file(path: &Path) -> MediaCategory {
        if path.is_dir() {
            return MediaCategory::Generic;
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_ascii_lowercase().as_str() {
                "jpg" | "jpeg" | "png" | "webp" | "heic" | "heif" | "tiff" | "tif" | "avif"
                | "bmp" | "svg" | "ico" => return MediaCategory::Image,
                "mp3" | "m4a" | "wav" | "flac" | "ogg" | "opus" | "aiff" | "aif" | "wma"
                | "aac" => return MediaCategory::Audio,
                "mp4" | "mov" | "mkv" | "webm" | "avi" | "wmv" | "gif" | "flv" => {
                    return MediaCategory::Video
                }
                "pdf" => return MediaCategory::Pdf,
                "epub" => return MediaCategory::Epub,
                "docx" | "doc" | "pages" | "md" | "markdown" | "txt" | "rtf" | "rtfd"
                | "odt" | "html" => return MediaCategory::Document,
                "ppt" | "pptx" | "odp" => return MediaCategory::Slides,
                "xlsx" | "xls" | "ods" | "csv" | "tsv" => return MediaCategory::Spreadsheet,
                "srt" | "vtt" => return MediaCategory::Subtitles,
                "zip" | "tar" | "gz" | "tgz" | "gzip" | "rar" | "7z" => {
                    return MediaCategory::Archive
                }
                _ => {}
            }
        }

        let mut magic = [0u8; 16];
        if let Ok(mut f) = File::open(path) {
            let n = f.read(&mut magic).unwrap_or(0);
            if n >= 8 && &magic[0..8] == b"\x89PNG\r\n\x1a\n" {
                return MediaCategory::Image;
            }
            if n >= 3 && &magic[0..3] == b"\xff\xd8\xff" {
                return MediaCategory::Image;
            }
            if n >= 12 && &magic[0..4] == b"RIFF" && &magic[8..12] == b"WEBP" {
                return MediaCategory::Image;
            }
            if n >= 6 && (&magic[0..6] == b"GIF87a" || &magic[0..6] == b"GIF89a") {
                return MediaCategory::Video;
            }
            if n >= 5 && &magic[0..5] == b"%PDF-" {
                return MediaCategory::Pdf;
            }
            if n >= 4 && &magic[0..4] == b"PK\x03\x04" {
                return MediaCategory::Archive;
            }
            if n >= 2 && &magic[0..2] == b"\x1f\x8b" {
                return MediaCategory::Archive;
            }
            if n >= 6 && &magic[0..6] == b"Rar!\x1a\x07" {
                return MediaCategory::Archive;
            }
            if n >= 3 && &magic[0..3] == b"ID3" {
                return MediaCategory::Audio;
            }
            if n >= 4 && &magic[0..4] == b"fLaC" {
                return MediaCategory::Audio;
            }
            if n >= 4 && &magic[0..4] == b"OggS" {
                return MediaCategory::Audio;
            }
            if n >= 12 && &magic[4..8] == b"ftyp" {
                return MediaCategory::Video;
            }
            if n >= 4 && &magic[0..4] == b"\x1a\x45\xdf\xa3" {
                return MediaCategory::Video;
            }
        }

        MediaCategory::Generic
    }

    pub fn build_slices_for_files(
        files: &[PathBuf],
        mode: WheelMode,
        caps: &DependencyCapabilities,
    ) -> Vec<WheelSlice> {
        if mode == WheelMode::FileActions {
            return Self::build_file_action_slices(caps);
        }

        let Some(first) = files.first() else {
            return Vec::new();
        };

        let category = Self::inspect_file(first);
        let current_ext = first
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        if mode == WheelMode::FileTools {
            return Self::build_file_tool_slices(category, &current_ext, caps);
        }

        Self::build_convert_slices(category, &current_ext, caps)
    }

    fn build_convert_slices(
        category: MediaCategory,
        current_ext: &str,
        caps: &DependencyCapabilities,
    ) -> Vec<WheelSlice> {
        let mut slices = Vec::new();

        match category {
            MediaCategory::Image => {
                // Reference Image 1: PNG, WEBP, HEIC, TIFF, AVIF, BMP, PDF, JPG
                let targets = [
                    ("png", "PNG"),
                    ("webp", "WEBP"),
                    ("heic", "HEIC"),
                    ("tiff", "TIFF"),
                    ("avif", "AVIF"),
                    ("bmp", "BMP"),
                    ("pdf", "PDF"),
                    ("jpg", "JPG"),
                ];
                for (fmt, label) in targets {
                    if current_ext != fmt && !(current_ext == "jpeg" && fmt == "jpg") {
                        slices.push(Self::conv_slice(
                            label,
                            &format!("Convert to {}", label),
                            None,
                            caps,
                            SliceAction::ImageConvert {
                                format: fmt.to_string(),
                            },
                        ));
                    }
                }
            }
            MediaCategory::Video => {
                // Reference Image 2: MP4, MKV, WEBM, AVI, WMV, GIF, MP3, MOV
                let targets = [
                    ("mp4", "MP4"),
                    ("mkv", "MKV"),
                    ("webm", "WEBM"),
                    ("avi", "AVI"),
                    ("wmv", "WMV"),
                    ("gif", "GIF"),
                    ("mp3", "MP3"),
                    ("mov", "MOV"),
                ];
                for (fmt, label) in targets {
                    if current_ext != fmt {
                        slices.push(Self::conv_slice(
                            label,
                            &format!("Convert to {}", label),
                            Some("ffmpeg"),
                            caps,
                            SliceAction::VideoConvert {
                                format: fmt.to_string(),
                            },
                        ));
                    }
                }
            }
            MediaCategory::Audio => {
                // Reference Image 3: M4A, WAV, FLAC, OGG, OPUS, AIFF, WMA, MP3
                let targets = [
                    ("m4a", "M4A"),
                    ("wav", "WAV"),
                    ("flac", "FLAC"),
                    ("ogg", "OGG"),
                    ("opus", "OPUS"),
                    ("aiff", "AIFF"),
                    ("wma", "WMA"),
                    ("mp3", "MP3"),
                ];
                for (fmt, label) in targets {
                    if current_ext != fmt {
                        slices.push(Self::conv_slice(
                            label,
                            &format!("Convert to {}", label),
                            Some("ffmpeg"),
                            caps,
                            SliceAction::AudioConvert {
                                format: fmt.to_string(),
                            },
                        ));
                    }
                }
            }
            MediaCategory::Pdf => {
                // Reference Image 4: DOCX, PAGES, PPTX, XLSX, JPG, PNG, EPUB, TXT, MD
                let targets = [
                    ("docx", "DOCX"),
                    ("pages", "PAGES"),
                    ("pptx", "PPTX"),
                    ("xlsx", "XLSX"),
                    ("jpg", "JPG"),
                    ("png", "PNG"),
                    ("epub", "EPUB"),
                    ("txt", "TXT"),
                    ("md", "MD"),
                ];
                for (fmt, label) in targets {
                    slices.push(Self::conv_slice(
                        label,
                        &format!("Convert to {}", label),
                        None,
                        caps,
                        SliceAction::PdfConvert {
                            format: fmt.to_string(),
                        },
                    ));
                }
            }
            MediaCategory::Document => {
                // Reference Image 5: PAGES, PDF, TXT, MD, HTML, EPUB, PNG, JPG, DOCX
                let targets = [
                    ("pages", "PAGES"),
                    ("pdf", "PDF"),
                    ("txt", "TXT"),
                    ("md", "MD"),
                    ("html", "HTML"),
                    ("epub", "EPUB"),
                    ("png", "PNG"),
                    ("jpg", "JPG"),
                    ("docx", "DOCX"),
                ];
                for (fmt, label) in targets {
                    if current_ext != fmt && !(current_ext == "markdown" && fmt == "md") {
                        slices.push(Self::conv_slice(
                            label,
                            &format!("Convert to {}", label),
                            None,
                            caps,
                            SliceAction::DocConvert {
                                format: fmt.to_string(),
                            },
                        ));
                    }
                }
            }
            MediaCategory::Epub => {
                // Reference Image 8: PDF, DOCX, PAGES, TXT, MD, HTML
                let targets = [
                    ("pdf", "PDF"),
                    ("docx", "DOCX"),
                    ("pages", "PAGES"),
                    ("txt", "TXT"),
                    ("md", "MD"),
                    ("html", "HTML"),
                ];
                for (fmt, label) in targets {
                    slices.push(Self::conv_slice(
                        label,
                        &format!("Convert to {}", label),
                        None,
                        caps,
                        SliceAction::DocConvert {
                            format: fmt.to_string(),
                        },
                    ));
                }
            }
            MediaCategory::Slides => {
                // Reference Image 6: PDF, JPG, PNG
                let targets = [("pdf", "PDF"), ("jpg", "JPG"), ("png", "PNG")];
                for (fmt, label) in targets {
                    if current_ext != fmt {
                        slices.push(Self::conv_slice(
                            label,
                            &format!("Convert to {}", label),
                            None,
                            caps,
                            SliceAction::SlidesConvert {
                                format: fmt.to_string(),
                            },
                        ));
                    }
                }
            }
            MediaCategory::Spreadsheet => {
                // Reference Image 7: ODS, CSV, TSV, PDF, XLSX
                let targets = [
                    ("ods", "ODS"),
                    ("csv", "CSV"),
                    ("tsv", "TSV"),
                    ("pdf", "PDF"),
                    ("xlsx", "XLSX"),
                ];
                for (fmt, label) in targets {
                    if current_ext != fmt {
                        slices.push(Self::conv_slice(
                            label,
                            &format!("Convert to {}", label),
                            None,
                            caps,
                            SliceAction::SpreadsheetConvert {
                                format: fmt.to_string(),
                            },
                        ));
                    }
                }
            }
            MediaCategory::Subtitles => {
                // Reference Image 9: VTT, TXT, SRT
                let targets = [("vtt", "VTT"), ("txt", "TXT"), ("srt", "SRT")];
                for (fmt, label) in targets {
                    if current_ext != fmt {
                        slices.push(Self::conv_slice(
                            label,
                            &format!("Convert to {}", label),
                            None,
                            caps,
                            SliceAction::SubtitleConvert {
                                format: fmt.to_string(),
                            },
                        ));
                    }
                }
            }
            MediaCategory::Archive => {
                // Reference Image 10: TAR, GZIP, ZIP
                let targets = [("tar", "TAR"), ("gzip", "GZIP"), ("zip", "ZIP")];
                for (fmt, label) in targets {
                    if current_ext != fmt
                        && !(fmt == "gzip" && (current_ext == "gz" || current_ext == "tgz"))
                    {
                        slices.push(Self::conv_slice(
                            label,
                            &format!("Convert to {}", label),
                            None,
                            caps,
                            SliceAction::ArchiveConvert {
                                format: fmt.to_string(),
                            },
                        ));
                    }
                }
            }
            MediaCategory::Generic => {
                return Self::build_file_action_slices(caps);
            }
        }

        slices
    }

    fn build_file_tool_slices(
        category: MediaCategory,
        current_ext: &str,
        caps: &DependencyCapabilities,
    ) -> Vec<WheelSlice> {
        match category {
            MediaCategory::Image => vec![
                Self::tool_slice(
                    "COMPRESS",
                    "Compress",
                    SliceIcon::Compress,
                    None,
                    caps,
                    SliceAction::ImageCompressTool,
                ),
                Self::tool_slice(
                    "METADATA",
                    "Strip Metadata",
                    SliceIcon::Metadata,
                    None,
                    caps,
                    SliceAction::StripMetadata,
                ),
                Self::tool_slice(
                    "EDIT",
                    "Edit Image",
                    SliceIcon::Edit,
                    None,
                    caps,
                    SliceAction::ImageEditTool,
                ),
                Self::tool_slice(
                    "ANNOTATE",
                    "Annotate",
                    SliceIcon::Annotate,
                    None,
                    caps,
                    SliceAction::ImageAnnotateTool,
                ),
                Self::tool_slice(
                    "ADD BG",
                    "Add Background",
                    SliceIcon::AddBg,
                    None,
                    caps,
                    SliceAction::ImageAddBgTool,
                ),
                Self::tool_slice(
                    "CROP",
                    "Crop Image",
                    SliceIcon::Crop,
                    None,
                    caps,
                    SliceAction::ImageCropTool,
                ),
                Self::tool_slice(
                    "REDACT",
                    "Redact Pixels",
                    SliceIcon::Redact,
                    None,
                    caps,
                    SliceAction::ImageRedactTool,
                ),
            ],
            MediaCategory::Video => vec![
                Self::tool_slice(
                    "COMPRESS",
                    "Compress",
                    SliceIcon::Compress,
                    Some("ffmpeg"),
                    caps,
                    SliceAction::VideoCompressTool,
                ),
                Self::tool_slice(
                    "METADATA",
                    "Strip Metadata",
                    SliceIcon::Metadata,
                    None,
                    caps,
                    SliceAction::StripMetadata,
                ),
                Self::tool_slice(
                    "MUTE",
                    "Mute Audio Track",
                    SliceIcon::Mute,
                    Some("ffmpeg"),
                    caps,
                    SliceAction::VideoMuteTool,
                ),
                Self::tool_slice(
                    "TRIM",
                    "Trim Video",
                    SliceIcon::Trim,
                    Some("ffmpeg"),
                    caps,
                    SliceAction::VideoTrimTool,
                ),
                Self::tool_slice(
                    "CROP",
                    "Crop Frame",
                    SliceIcon::Crop,
                    Some("ffmpeg"),
                    caps,
                    SliceAction::VideoCropTool,
                ),
                Self::tool_slice(
                    "SPEED",
                    "Change Speed",
                    SliceIcon::Speed,
                    Some("ffmpeg"),
                    caps,
                    SliceAction::VideoSpeedTool,
                ),
                Self::tool_slice(
                    "SNAPSHOT",
                    "Capture Frame",
                    SliceIcon::Snapshot,
                    Some("ffmpeg"),
                    caps,
                    SliceAction::VideoSnapshotTool,
                ),
                Self::tool_slice(
                    "SPLIT",
                    "Split Video",
                    SliceIcon::Split,
                    Some("ffmpeg"),
                    caps,
                    SliceAction::VideoSplitTool,
                ),
                Self::tool_slice(
                    "REDACT",
                    "Redact Video",
                    SliceIcon::Redact,
                    Some("ffmpeg"),
                    caps,
                    SliceAction::VideoRedactTool,
                ),
            ],
            MediaCategory::Audio => vec![
                Self::tool_slice(
                    "COMPRESS",
                    "Compress",
                    SliceIcon::Compress,
                    Some("ffmpeg"),
                    caps,
                    SliceAction::AudioCompressTool,
                ),
                Self::tool_slice(
                    "METADATA",
                    "Strip Metadata",
                    SliceIcon::Metadata,
                    None,
                    caps,
                    SliceAction::StripMetadata,
                ),
                Self::tool_slice(
                    "NORMALIZE",
                    "Normalize EBU R128",
                    SliceIcon::Normalize,
                    Some("ffmpeg"),
                    caps,
                    SliceAction::AudioNormalizeTool,
                ),
                Self::tool_slice(
                    "VISUALIZE",
                    "Waveform Video",
                    SliceIcon::Visualize,
                    Some("ffmpeg"),
                    caps,
                    SliceAction::AudioVisualizeTool,
                ),
                Self::tool_slice(
                    "TRIM",
                    "Trim Audio",
                    SliceIcon::Trim,
                    Some("ffmpeg"),
                    caps,
                    SliceAction::AudioTrimTool,
                ),
                Self::tool_slice(
                    "CHANNELS",
                    "Mono / Stereo",
                    SliceIcon::Channels,
                    Some("ffmpeg"),
                    caps,
                    SliceAction::AudioChannelsTool,
                ),
                Self::tool_slice(
                    "BLEEP",
                    "1kHz Censor Bleep",
                    SliceIcon::Bleep,
                    Some("ffmpeg"),
                    caps,
                    SliceAction::AudioBleepTool,
                ),
            ],
            MediaCategory::Pdf => vec![
                Self::tool_slice(
                    "COMPRESS",
                    "Compress",
                    SliceIcon::Compress,
                    None,
                    caps,
                    SliceAction::PdfCompressTool,
                ),
                Self::tool_slice(
                    "METADATA",
                    "Strip Metadata",
                    SliceIcon::Metadata,
                    None,
                    caps,
                    SliceAction::StripMetadata,
                ),
                Self::tool_slice(
                    "SPLIT",
                    "Split Pages",
                    SliceIcon::Split,
                    None,
                    caps,
                    SliceAction::PdfSplitTool,
                ),
                Self::tool_slice(
                    "ORGANIZE",
                    "Organize / Linearize",
                    SliceIcon::Organize,
                    None,
                    caps,
                    SliceAction::PdfOrganizeTool,
                ),
            ],
            MediaCategory::Archive => vec![Self::tool_slice(
                "EXTRACT",
                "Extract Archive",
                SliceIcon::Extract,
                None,
                caps,
                SliceAction::ArchiveExtractHere,
            )],
            _ => Self::build_convert_slices(category, current_ext, caps),
        }
    }

    pub fn build_file_action_slices(caps: &DependencyCapabilities) -> Vec<WheelSlice> {
        vec![
            Self::tool_slice(
                "AIRDROP",
                "AirDrop / LocalSend",
                SliceIcon::AirDrop,
                None,
                caps,
                SliceAction::ActionAirDrop,
            ),
            Self::tool_slice(
                "FLOAT",
                "Pin to Float Shelf",
                SliceIcon::Float,
                None,
                caps,
                SliceAction::ActionFloat,
            ),
            Self::tool_slice(
                "COPY",
                "Copy Files",
                SliceIcon::Copy,
                None,
                caps,
                SliceAction::ActionCopyFiles,
            ),
            Self::tool_slice(
                "MOVE",
                "Move Files",
                SliceIcon::Move,
                None,
                caps,
                SliceAction::ActionMoveFiles,
            ),
            Self::tool_slice(
                "COPY PATH",
                "Copy Full File Path",
                SliceIcon::CopyPath,
                None,
                caps,
                SliceAction::ActionCopyPath,
            ),
            Self::tool_slice(
                "TERMINAL",
                "Open in Terminal",
                SliceIcon::Terminal,
                None,
                caps,
                SliceAction::ActionOpenTerminal,
            ),
        ]
    }

    fn conv_slice(
        label: &str,
        subtitle: &str,
        required_binary: Option<&str>,
        caps: &DependencyCapabilities,
        action: SliceAction,
    ) -> WheelSlice {
        let enabled = match required_binary {
            Some(bin) => caps.is_available(bin),
            None => true,
        };
        WheelSlice {
            id: label.to_ascii_lowercase(),
            label: label.to_string(),
            center_label: label.to_string(),
            subtitle: if enabled {
                subtitle.to_string()
            } else {
                format!("Requires {}", required_binary.unwrap_or("tool"))
            },
            icon: SliceIcon::None,
            enabled,
            required_binary: required_binary.map(|s| s.to_string()),
            action,
        }
    }

    fn tool_slice(
        label: &str,
        subtitle: &str,
        icon: SliceIcon,
        required_binary: Option<&str>,
        caps: &DependencyCapabilities,
        action: SliceAction,
    ) -> WheelSlice {
        let enabled = match required_binary {
            Some(bin) => caps.is_available(bin),
            None => true,
        };
        WheelSlice {
            id: label.to_ascii_lowercase(),
            label: label.to_string(),
            center_label: label.to_string(),
            subtitle: if enabled {
                subtitle.to_string()
            } else {
                format!("Requires {}", required_binary.unwrap_or("tool"))
            },
            icon,
            enabled,
            required_binary: required_binary.map(|s| s.to_string()),
            action,
        }
    }
}

impl MimeRouter {
    pub fn build_combined_shift_slices(
        files: &[PathBuf],
        caps: &DependencyCapabilities,
    ) -> Vec<WheelSlice> {
        let Some(first) = files.first() else {
            return Self::build_file_action_slices(caps);
        };
        let category = Self::inspect_file(first);
        let current_ext = first
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        match category {
            MediaCategory::Archive => vec![
                Self::tool_slice(
                    "EXTRACT",
                    "Extract Archive",
                    SliceIcon::Extract,
                    None,
                    caps,
                    SliceAction::ArchiveExtractHere,
                ),
                Self::conv_slice(
                    "TAR",
                    "Convert to TAR",
                    None,
                    caps,
                    SliceAction::ArchiveConvert {
                        format: "tar".to_string(),
                    },
                ),
                Self::conv_slice(
                    "GZIP",
                    "Convert to GZIP",
                    None,
                    caps,
                    SliceAction::ArchiveConvert {
                        format: "gzip".to_string(),
                    },
                ),
                Self::conv_slice(
                    "ZIP",
                    "Convert to ZIP",
                    None,
                    caps,
                    SliceAction::ArchiveConvert {
                        format: "zip".to_string(),
                    },
                ),
            ],
            _ => Self::build_convert_slices(category, &current_ext, caps),
        }
    }
}
