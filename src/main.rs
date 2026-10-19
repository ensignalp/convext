#![allow(dead_code)]

mod config;
mod daemon;
mod desktop;
mod engine;
mod error;
mod ui;

use crate::config::{ConvextConfig, DependencyCapabilities};
use crate::daemon::tray::TrayService;
use crate::daemon::DaemonEvent;
use crate::desktop::actions::DesktopActions;
use crate::desktop::file_manager::FileManagerBridge;
use crate::engine::archive_proc::ArchiveProcessor;
use crate::engine::audio_proc::AudioProcessor;
use crate::engine::doc_proc::DocProcessor;
use crate::engine::image_proc::ImageProcessor;
use crate::engine::mime_router::SliceAction;
use crate::engine::pdf_proc::PdfProcessor;
use crate::engine::privacy_qr::PrivacyQrProcessor;
use crate::engine::video_proc::VideoProcessor;
use crate::engine::EngineProgress;
use crate::error::{ConvextError, ConvextResult};
use crate::ui::convext_studio::ConvextStudioWindow;
use crate::ui::float_shelf::FloatShelfWindow;
use crate::ui::modals::batch_rename::BatchRenameModal;
use crate::ui::modals::image_editor::ImageEditorModal;
use crate::ui::modals::media_trimmer::MediaTrimmerModal;
use crate::ui::modals::target_size::TargetSizeModal;
use async_channel::Sender;
use gtk4::prelude::*;
use libadwaita as adw;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

struct SingleInstanceGuard {
    _file: std::fs::File,
}

impl SingleInstanceGuard {
    fn acquire() -> ConvextResult<Self> {
        let runtime_dir = dirs::runtime_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let lock_path = runtime_dir.join("convext-daemon.lock");
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&lock_path)?;
        let fd = file.as_raw_fd();
        let rc = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
        if rc != 0 {
            return Err(ConvextError::InstanceLock(
                "Another instance of convext is already running.".to_string(),
            ));
        }
        Ok(Self { _file: file })
    }
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(include_str!("../assets/style.css"));
    if let Some(display) = gdk4::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn dispatch_convext_command(
    app: &gtk4::Application,
    shelf: &FloatShelfWindow,
    action_code: &str,
    files: Vec<PathBuf>,
    config: Arc<RwLock<ConvextConfig>>,
    prog_tx: Sender<EngineProgress>,
) {
    if files.is_empty() {
        return;
    }

    if action_code == "OPEN_STUDIO" {
        let app_c = app.clone();
        let shelf_c = shelf.clone();
        let cfg_c = config.clone();
        let tx_c = prog_tx.clone();
        ConvextStudioWindow::show(app, files, move |selected_code, target_files| {
            dispatch_convext_command(
                &app_c,
                &shelf_c,
                &selected_code,
                target_files,
                cfg_c.clone(),
                tx_c.clone(),
            );
        });
        return;
    }

    if let Some(fmt) = action_code.strip_prefix("IMG_CONVERT:") {
        dispatch_batch_action(
            app,
            shelf,
            SliceAction::ImageConvert {
                format: fmt.to_string(),
            },
            files,
            config,
            prog_tx,
        );
        return;
    }
    if let Some(fmt) = action_code.strip_prefix("VID_CONVERT:") {
        dispatch_batch_action(
            app,
            shelf,
            SliceAction::VideoConvert {
                format: fmt.to_string(),
            },
            files,
            config,
            prog_tx,
        );
        return;
    }
    if let Some(fmt) = action_code.strip_prefix("AUD_CONVERT:") {
        dispatch_batch_action(
            app,
            shelf,
            SliceAction::AudioConvert {
                format: fmt.to_string(),
            },
            files,
            config,
            prog_tx,
        );
        return;
    }
    if let Some(fmt) = action_code.strip_prefix("PDF_CONVERT:") {
        dispatch_batch_action(
            app,
            shelf,
            SliceAction::PdfConvert {
                format: fmt.to_string(),
            },
            files,
            config,
            prog_tx,
        );
        return;
    }
    if let Some(fmt) = action_code.strip_prefix("DOC_CONVERT:") {
        dispatch_batch_action(
            app,
            shelf,
            SliceAction::DocConvert {
                format: fmt.to_string(),
            },
            files,
            config,
            prog_tx,
        );
        return;
    }
    if let Some(fmt) = action_code.strip_prefix("SLIDES_CONVERT:") {
        dispatch_batch_action(
            app,
            shelf,
            SliceAction::SlidesConvert {
                format: fmt.to_string(),
            },
            files,
            config,
            prog_tx,
        );
        return;
    }
    if let Some(fmt) = action_code.strip_prefix("SHEET_CONVERT:") {
        dispatch_batch_action(
            app,
            shelf,
            SliceAction::SpreadsheetConvert {
                format: fmt.to_string(),
            },
            files,
            config,
            prog_tx,
        );
        return;
    }
    if let Some(fmt) = action_code.strip_prefix("SUB_CONVERT:") {
        dispatch_batch_action(
            app,
            shelf,
            SliceAction::SubtitleConvert {
                format: fmt.to_string(),
            },
            files,
            config,
            prog_tx,
        );
        return;
    }
    if let Some(fmt) = action_code.strip_prefix("ARC_CONVERT:") {
        dispatch_batch_action(
            app,
            shelf,
            SliceAction::ArchiveConvert {
                format: fmt.to_string(),
            },
            files,
            config,
            prog_tx,
        );
        return;
    }

    match action_code {
        "ARC_EXTRACT" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::ArchiveExtractHere,
            files,
            config,
            prog_tx,
        ),
        "ARC_CREATE_ZIP" | "ARC_CREATE_TAR" | "ARC_CREATE_GZIP" => {
            let code = action_code.to_string();
            let suffix = config
                .read()
                .map(|c| c.default_output_suffix.clone())
                .unwrap_or_else(|_| "_convext".to_string());
            let auto_reveal = config
                .read()
                .map(|c| c.auto_reveal_in_file_manager)
                .unwrap_or(true);
            tokio::spawn(async move {
                let res = match code.as_str() {
                    "ARC_CREATE_TAR" => ArchiveProcessor::create_tar(&files, &suffix),
                    "ARC_CREATE_GZIP" => ArchiveProcessor::create_tar_gz(&files, &suffix),
                    _ => ArchiveProcessor::create_zip(&files, &suffix),
                };
                match res {
                    Ok(out) => {
                        if auto_reveal {
                            let _ = FileManagerBridge::show_items(&[out.clone()]);
                        }
                        let _ = prog_tx
                            .send(EngineProgress::Finished {
                                job_id: "arc_create".to_string(),
                                outputs: vec![out],
                                summary: "Arşiv oluşturuldu.".to_string(),
                            })
                            .await;
                    }
                    Err(e) => {
                        let _ = prog_tx
                            .send(EngineProgress::Failed {
                                job_id: "arc_create".to_string(),
                                error: e.to_string(),
                            })
                            .await;
                    }
                }
            });
        }
        "IMG_COMPRESS_85" | "IMG_COMPRESS_65" | "IMG_COMPRESS_40" => {
            let quality: u8 = match action_code {
                "IMG_COMPRESS_85" => 85,
                "IMG_COMPRESS_65" => 65,
                _ => 40,
            };
            let suffix = config
                .read()
                .map(|c| c.default_output_suffix.clone())
                .unwrap_or_else(|_| "_compressed".to_string());
            let auto_reveal = config
                .read()
                .map(|c| c.auto_reveal_in_file_manager)
                .unwrap_or(true);
            tokio::spawn(async move {
                let mut outs = Vec::new();
                for f in &files {
                    match ImageProcessor::compress_percentage(f, quality, &suffix) {
                        Ok(o) => outs.push(o),
                        Err(e) => {
                            let _ = prog_tx
                                .send(EngineProgress::Failed {
                                    job_id: "img_comp".to_string(),
                                    error: e.to_string(),
                                })
                                .await;
                            return;
                        }
                    }
                }
                if auto_reveal && !outs.is_empty() {
                    let _ = FileManagerBridge::show_items(&outs);
                }
                let _ = prog_tx
                    .send(EngineProgress::Finished {
                        job_id: "img_comp".to_string(),
                        outputs: outs,
                        summary: format!("Görsel sıkıştırıldı (%{} kalite).", quality),
                    })
                    .await;
            });
        }
        "IMG_TARGET_SIZE" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::ImageCompressTool,
            files,
            config,
            prog_tx,
        ),
        "IMG_EDITOR" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::ImageEditTool,
            files,
            config,
            prog_tx,
        ),
        "IMG_ADD_BG" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::ImageAddBgTool,
            files,
            config,
            prog_tx,
        ),
        "IMG_COLLAGE_PDF" => {
            let suffix = config
                .read()
                .map(|c| c.default_output_suffix.clone())
                .unwrap_or_else(|_| "_collage".to_string());
            let auto_reveal = config
                .read()
                .map(|c| c.auto_reveal_in_file_manager)
                .unwrap_or(true);
            tokio::spawn(async move {
                match PdfProcessor::images_to_pdf_collage(&files, &suffix) {
                    Ok(out) => {
                        if auto_reveal {
                            let _ = FileManagerBridge::show_items(&[out.clone()]);
                        }
                        let _ = prog_tx
                            .send(EngineProgress::Finished {
                                job_id: "img_pdf".to_string(),
                                outputs: vec![out],
                                summary: "Görseller tek PDF dosyasında birleştirildi.".to_string(),
                            })
                            .await;
                    }
                    Err(e) => {
                        let _ = prog_tx
                            .send(EngineProgress::Failed {
                                job_id: "img_pdf".to_string(),
                                error: e.to_string(),
                            })
                            .await;
                    }
                }
            });
        }
        "IMG_QR_DECODE" => {
            tokio::spawn(async move {
                if let Some(first) = files.first() {
                    match PrivacyQrProcessor::decode_qr_from_image(first) {
                        Ok(text) => {
                            if let Ok(mut child) = std::process::Command::new("wl-copy")
                                .stdin(std::process::Stdio::piped())
                                .spawn()
                            {
                                if let Some(mut stdin) = child.stdin.take() {
                                    let _ = stdin.write_all(text.as_bytes());
                                }
                            }
                            DesktopActions::send_desktop_notification(
                                "Convext QR Okuyucu (Panoya Kopyalandı)",
                                &text,
                            );
                        }
                        Err(e) => {
                            DesktopActions::send_desktop_notification(
                                "Convext QR Okuyucu",
                                &e.to_string(),
                            );
                        }
                    }
                }
            });
        }
        "VID_QUICK_COMPRESS" => {
            let suffix = config
                .read()
                .map(|c| c.default_output_suffix.clone())
                .unwrap_or_else(|_| "_compressed".to_string());
            let auto_reveal = config
                .read()
                .map(|c| c.auto_reveal_in_file_manager)
                .unwrap_or(true);
            tokio::spawn(async move {
                let mut outs = Vec::new();
                for f in &files {
                    match VideoProcessor::quick_compress(f, &suffix) {
                        Ok(o) => outs.push(o),
                        Err(e) => {
                            let _ = prog_tx
                                .send(EngineProgress::Failed {
                                    job_id: "vid_qcomp".to_string(),
                                    error: e.to_string(),
                                })
                                .await;
                            return;
                        }
                    }
                }
                if auto_reveal && !outs.is_empty() {
                    let _ = FileManagerBridge::show_items(&outs);
                }
                let _ = prog_tx
                    .send(EngineProgress::Finished {
                        job_id: "vid_qcomp".to_string(),
                        outputs: outs,
                        summary: "Video arka planda sıkıştırıldı.".to_string(),
                    })
                    .await;
            });
        }
        "VID_TARGET_SIZE" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::VideoCompressTool,
            files,
            config,
            prog_tx,
        ),
        "VID_JOIN" => {
            let suffix = config
                .read()
                .map(|c| c.default_output_suffix.clone())
                .unwrap_or_else(|_| "_joined".to_string());
            let auto_reveal = config
                .read()
                .map(|c| c.auto_reveal_in_file_manager)
                .unwrap_or(true);
            tokio::spawn(async move {
                match VideoProcessor::join_videos(&files, &suffix) {
                    Ok(out) => {
                        if auto_reveal {
                            let _ = FileManagerBridge::show_items(&[out.clone()]);
                        }
                        let _ = prog_tx
                            .send(EngineProgress::Finished {
                                job_id: "vid_join".to_string(),
                                outputs: vec![out],
                                summary: "Videolar tek MP4 dosyasında birleştirildi.".to_string(),
                            })
                            .await;
                    }
                    Err(e) => {
                        let _ = prog_tx
                            .send(EngineProgress::Failed {
                                job_id: "vid_join".to_string(),
                                error: e.to_string(),
                            })
                            .await;
                    }
                }
            });
        }
        "VID_TRIM" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::VideoTrimTool,
            files,
            config,
            prog_tx,
        ),
        "VID_SPLIT" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::VideoSplitTool,
            files,
            config,
            prog_tx,
        ),
        "VID_CROP" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::VideoCropTool,
            files,
            config,
            prog_tx,
        ),
        "VID_MUTE" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::VideoMuteTool,
            files,
            config,
            prog_tx,
        ),
        "VID_SNAPSHOT" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::VideoSnapshotTool,
            files,
            config,
            prog_tx,
        ),
        "VID_REDACT" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::VideoRedactTool,
            files,
            config,
            prog_tx,
        ),
        "AUD_COMPRESS" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::AudioCompressTool,
            files,
            config,
            prog_tx,
        ),
        "AUD_TRIM" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::AudioTrimTool,
            files,
            config,
            prog_tx,
        ),
        "AUD_NORMALIZE" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::AudioNormalizeTool,
            files,
            config,
            prog_tx,
        ),
        "AUD_VISUALIZE" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::AudioVisualizeTool,
            files,
            config,
            prog_tx,
        ),
        "AUD_CHANNELS" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::AudioChannelsTool,
            files,
            config,
            prog_tx,
        ),
        "AUD_BLEEP" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::AudioBleepTool,
            files,
            config,
            prog_tx,
        ),
        "PDF_COMPRESS" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::PdfCompressTool,
            files,
            config,
            prog_tx,
        ),
        "PDF_SPLIT" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::PdfSplitTool,
            files,
            config,
            prog_tx,
        ),
        "PDF_ORGANIZE" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::PdfOrganizeTool,
            files,
            config,
            prog_tx,
        ),
        "PDF_MERGE" => {
            let suffix = config
                .read()
                .map(|c| c.default_output_suffix.clone())
                .unwrap_or_else(|_| "_merged".to_string());
            let auto_reveal = config
                .read()
                .map(|c| c.auto_reveal_in_file_manager)
                .unwrap_or(true);
            tokio::spawn(async move {
                match PdfProcessor::merge_pdfs(&files, &suffix) {
                    Ok(out) => {
                        if auto_reveal {
                            let _ = FileManagerBridge::show_items(&[out.clone()]);
                        }
                        let _ = prog_tx
                            .send(EngineProgress::Finished {
                                job_id: "pdf_merge".to_string(),
                                outputs: vec![out],
                                summary: "PDF dosyaları birleştirildi.".to_string(),
                            })
                            .await;
                    }
                    Err(e) => {
                        let _ = prog_tx
                            .send(EngineProgress::Failed {
                                job_id: "pdf_merge".to_string(),
                                error: e.to_string(),
                            })
                            .await;
                    }
                }
            });
        }
        "STRIP_METADATA" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::StripMetadata,
            files,
            config,
            prog_tx,
        ),
        "ACT_BATCH_RENAME" => {
            let auto_reveal = config
                .read()
                .map(|c| c.auto_reveal_in_file_manager)
                .unwrap_or(true);
            let tx_clone = prog_tx.clone();
            BatchRenameModal::show(app, files, move |target_files, spec| {
                let tx = tx_clone.clone();
                tokio::spawn(async move {
                    match BatchRenameModal::execute_batch_rename(&target_files, &spec) {
                        Ok(outs) => {
                            if auto_reveal && !outs.is_empty() {
                                let _ = FileManagerBridge::show_items(&outs);
                            }
                            let _ = tx
                                .send(EngineProgress::Finished {
                                    job_id: "rename".to_string(),
                                    outputs: outs,
                                    summary: "Dosyalar yeniden adlandırıldı.".to_string(),
                                })
                                .await;
                        }
                        Err(e) => {
                            let _ = tx
                                .send(EngineProgress::Failed {
                                    job_id: "rename".to_string(),
                                    error: e.to_string(),
                                })
                                .await;
                        }
                    }
                });
            });
        }
        "ACT_FLOAT" => {
            dispatch_batch_action(app, shelf, SliceAction::ActionFloat, files, config, prog_tx)
        }
        "ACT_AIRDROP" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::ActionAirDrop,
            files,
            config,
            prog_tx,
        ),
        "ACT_COPY_FILES" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::ActionCopyFiles,
            files,
            config,
            prog_tx,
        ),
        "ACT_MOVE_ORGANIZED" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::ActionMoveFiles,
            files,
            config,
            prog_tx,
        ),
        "ACT_COPY_PATH" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::ActionCopyPath,
            files,
            config,
            prog_tx,
        ),
        "ACT_TERMINAL" => dispatch_batch_action(
            app,
            shelf,
            SliceAction::ActionOpenTerminal,
            files,
            config,
            prog_tx,
        ),
        _ => {}
    }
}

fn dispatch_batch_action(
    app: &gtk4::Application,
    shelf: &FloatShelfWindow,
    action: SliceAction,
    files: Vec<PathBuf>,
    config: Arc<RwLock<ConvextConfig>>,
    prog_tx: Sender<EngineProgress>,
) {
    let active_files = if files.is_empty() {
        FileManagerBridge::detect_active_files()
    } else {
        files
    };

    if active_files.is_empty() {
        return;
    }

    match action {
        SliceAction::ActionFloat => {
            shelf.add_files(active_files);
            shelf.window.set_visible(true);
            shelf.window.present();
        }
        SliceAction::ActionCopyPath => {
            DesktopActions::copy_paths_to_clipboard(&active_files);
            DesktopActions::send_desktop_notification(
                "Convext — Yol Kopyalandı",
                "Dosya yolu panoya kopyalandı.",
            );
        }
        SliceAction::ActionOpenTerminal => {
            if let Some(first) = active_files.first() {
                if let Err(e) = DesktopActions::open_terminal_in_folder(first) {
                    DesktopActions::send_desktop_notification("Terminal Hatası", &e.to_string());
                }
            }
        }
        SliceAction::ActionAirDrop => {
            if let Err(e) = DesktopActions::send_via_localsend(&active_files) {
                DesktopActions::send_desktop_notification("AirDrop / LocalSend", &e.to_string());
            }
        }
        SliceAction::ImageEditTool
        | SliceAction::ImageAnnotateTool
        | SliceAction::ImageCropTool
        | SliceAction::ImageRedactTool => {
            if let Some(first) = active_files.first().cloned() {
                let cfg_clone = config.clone();
                let tx_clone = prog_tx.clone();
                ImageEditorModal::show(app, first, move |path, boxes| {
                    let suffix = cfg_clone
                        .read()
                        .map(|c| c.default_output_suffix.clone())
                        .unwrap_or_else(|_| "_edited".to_string());
                    let auto_reveal = cfg_clone
                        .read()
                        .map(|c| c.auto_reveal_in_file_manager)
                        .unwrap_or(true);
                    let tx = tx_clone.clone();
                    tokio::spawn(async move {
                        match ImageProcessor::apply_redactions(&path, &boxes, &suffix) {
                            Ok(out) => {
                                if auto_reveal {
                                    let _ = FileManagerBridge::show_items(&[out.clone()]);
                                }
                                let _ = tx
                                    .send(EngineProgress::Finished {
                                        job_id: "img_edit".to_string(),
                                        outputs: vec![out],
                                        summary: "Görsel düzenlendi ve kaydedildi.".to_string(),
                                    })
                                    .await;
                            }
                            Err(e) => {
                                let _ = tx
                                    .send(EngineProgress::Failed {
                                        job_id: "img_edit".to_string(),
                                        error: e.to_string(),
                                    })
                                    .await;
                            }
                        }
                    });
                });
            }
        }
        SliceAction::VideoTrimTool
        | SliceAction::VideoSpeedTool
        | SliceAction::AudioTrimTool => {
            let is_audio = action == SliceAction::AudioTrimTool;
            let cfg_clone = config.clone();
            let tx_clone = prog_tx.clone();
            MediaTrimmerModal::show(app, active_files, move |target_files, params| {
                let suffix = cfg_clone
                    .read()
                    .map(|c| c.default_output_suffix.clone())
                    .unwrap_or_else(|_| "_trimmed".to_string());
                let auto_reveal = cfg_clone
                    .read()
                    .map(|c| c.auto_reveal_in_file_manager)
                    .unwrap_or(true);
                let tx = tx_clone.clone();
                tokio::spawn(async move {
                    let mut outputs = Vec::new();
                    for p in &target_files {
                        let res = if is_audio {
                            AudioProcessor::trim_and_speed(p, &params, &suffix)
                        } else {
                            VideoProcessor::trim_and_speed(p, &params, &suffix)
                        };
                        match res {
                            Ok(out) => outputs.push(out),
                            Err(e) => {
                                let _ = tx
                                    .send(EngineProgress::Failed {
                                        job_id: "trim".to_string(),
                                        error: e.to_string(),
                                    })
                                    .await;
                                return;
                            }
                        }
                    }
                    if auto_reveal {
                        let _ = FileManagerBridge::show_items(&outputs);
                    }
                    let _ = tx
                        .send(EngineProgress::Finished {
                            job_id: "trim".to_string(),
                            outputs,
                            summary: "Medya kırpma ve hız işlemi tamamlandı.".to_string(),
                        })
                        .await;
                });
            });
        }
        SliceAction::ImageCompressTool | SliceAction::VideoCompressTool => {
            let is_video = action == SliceAction::VideoCompressTool;
            let cfg_clone = config.clone();
            let tx_clone = prog_tx.clone();
            TargetSizeModal::show(app, active_files, is_video, move |target_files, spec, video_mode| {
                let suffix = cfg_clone
                    .read()
                    .map(|c| c.default_output_suffix.clone())
                    .unwrap_or_else(|_| "_compressed".to_string());
                let auto_reveal = cfg_clone
                    .read()
                    .map(|c| c.auto_reveal_in_file_manager)
                    .unwrap_or(true);
                let tx = tx_clone.clone();
                tokio::spawn(async move {
                    let mut outputs = Vec::new();
                    for p in &target_files {
                        let res = if video_mode {
                            VideoProcessor::compress_to_target_bytes(p, spec.target_bytes, &suffix)
                        } else {
                            ImageProcessor::compress_to_target_bytes(p, spec.target_bytes, &suffix)
                        };
                        match res {
                            Ok(out) => outputs.push(out),
                            Err(e) => {
                                let _ = tx
                                    .send(EngineProgress::Failed {
                                        job_id: "compress".to_string(),
                                        error: e.to_string(),
                                    })
                                    .await;
                                return;
                            }
                        }
                    }
                    if auto_reveal {
                        let _ = FileManagerBridge::show_items(&outputs);
                    }
                    let _ = tx
                        .send(EngineProgress::Finished {
                            job_id: "compress".to_string(),
                            outputs,
                            summary: "Hedef boyuta sıkıştırma tamamlandı.".to_string(),
                        })
                        .await;
                });
            });
        }
        bg_action => {
            let suffix = config
                .read()
                .map(|c| c.default_output_suffix.clone())
                .unwrap_or_else(|_| "_convext".to_string());
            let auto_reveal = config
                .read()
                .map(|c| c.auto_reveal_in_file_manager)
                .unwrap_or(true);

            tokio::spawn(async move {
                execute_silent_background_action(
                    bg_action,
                    active_files,
                    suffix,
                    auto_reveal,
                    prog_tx,
                )
                .await;
            });
        }
    }
}

async fn execute_silent_background_action(
    action: SliceAction,
    files: Vec<PathBuf>,
    suffix: String,
    auto_reveal: bool,
    prog_tx: Sender<EngineProgress>,
) {
    let mut outputs = Vec::new();

    match action {
        SliceAction::ActionCopyFiles => {
            DesktopActions::copy_paths_to_clipboard(&files);
            match DesktopActions::duplicate_files(&files, &suffix) {
                Ok(mut out) => outputs.append(&mut out),
                Err(e) => {
                    let _ = prog_tx
                        .send(EngineProgress::Failed {
                            job_id: "copy".to_string(),
                            error: e.to_string(),
                        })
                        .await;
                    return;
                }
            }
        }
        SliceAction::ActionMoveFiles => match DesktopActions::move_files_to_organized(&files) {
            Ok(mut out) => outputs.append(&mut out),
            Err(e) => {
                let _ = prog_tx
                    .send(EngineProgress::Failed {
                        job_id: "move".to_string(),
                        error: e.to_string(),
                    })
                    .await;
                return;
            }
        },
        per_file_action => {
            for path in &files {
                let single_res: ConvextResult<Vec<PathBuf>> = match &per_file_action {
                    SliceAction::ImageConvert { format } => {
                        ImageProcessor::convert_format(path, format, &suffix).map(|o| vec![o])
                    }
                    SliceAction::AudioConvert { format } => {
                        AudioProcessor::convert_format(path, format, &suffix).map(|o| vec![o])
                    }
                    SliceAction::VideoConvert { format } => {
                        VideoProcessor::convert_format(path, format, &suffix).map(|o| vec![o])
                    }
                    SliceAction::PdfConvert { format } => {
                        PdfProcessor::convert_to_format(path, format, &suffix)
                    }
                    SliceAction::DocConvert { format } => {
                        DocProcessor::convert_document_multi(path, format, &suffix)
                    }
                    SliceAction::SlidesConvert { format } => {
                        DocProcessor::convert_slides(path, format, &suffix)
                    }
                    SliceAction::SpreadsheetConvert { format } => {
                        DocProcessor::convert_spreadsheet(path, format, &suffix).map(|o| vec![o])
                    }
                    SliceAction::SubtitleConvert { format } => {
                        DocProcessor::convert_subtitles(path, format, &suffix).map(|o| vec![o])
                    }
                    SliceAction::ArchiveConvert { format } => {
                        ArchiveProcessor::convert_archive(path, format, &suffix).map(|o| vec![o])
                    }
                    SliceAction::ArchiveExtractHere => {
                        ArchiveProcessor::extract_archive(path).map(|o| vec![o])
                    }
                    SliceAction::StripMetadata => {
                        PrivacyQrProcessor::strip_metadata(path, &suffix).map(|o| vec![o])
                    }
                    SliceAction::ImageAddBgTool => {
                        ImageProcessor::add_solid_background(path, &suffix).map(|o| vec![o])
                    }
                    SliceAction::VideoMuteTool => {
                        VideoProcessor::mute_audio(path, &suffix).map(|o| vec![o])
                    }
                    SliceAction::VideoCropTool => {
                        VideoProcessor::crop_center_video(path, &suffix).map(|o| vec![o])
                    }
                    SliceAction::VideoSnapshotTool => {
                        VideoProcessor::capture_snapshot_png(path, &suffix).map(|o| vec![o])
                    }
                    SliceAction::VideoSplitTool => VideoProcessor::split_in_half(path),
                    SliceAction::VideoRedactTool => {
                        VideoProcessor::redact_video_center(path, &suffix).map(|o| vec![o])
                    }
                    SliceAction::AudioCompressTool => {
                        AudioProcessor::compress_audio(path, &suffix).map(|o| vec![o])
                    }
                    SliceAction::AudioNormalizeTool => {
                        AudioProcessor::normalize_ebu_r128(path, &suffix).map(|o| vec![o])
                    }
                    SliceAction::AudioVisualizeTool => {
                        AudioProcessor::visualize_waveform_mp4(path, &suffix).map(|o| vec![o])
                    }
                    SliceAction::AudioChannelsTool => {
                        AudioProcessor::toggle_mono_stereo(path, &suffix).map(|o| vec![o])
                    }
                    SliceAction::AudioBleepTool => {
                        AudioProcessor::bleep_middle_segment(path, &suffix).map(|o| vec![o])
                    }
                    SliceAction::PdfCompressTool => {
                        PdfProcessor::compress_pdf(path, &suffix).map(|o| vec![o])
                    }
                    SliceAction::PdfSplitTool => PdfProcessor::split_pdf_pages(path),
                    SliceAction::PdfOrganizeTool => {
                        PdfProcessor::organize_linearize_pdf(path, &suffix).map(|o| vec![o])
                    }
                    _ => Ok(vec![]),
                };

                match single_res {
                    Ok(mut produced) => {
                        outputs.append(&mut produced);
                    }
                    Err(e) => {
                        let _ = prog_tx
                            .send(EngineProgress::Failed {
                                job_id: "bg".to_string(),
                                error: e.to_string(),
                            })
                            .await;
                        return;
                    }
                }
            }
        }
    }

    if auto_reveal && !outputs.is_empty() {
        let _ = FileManagerBridge::show_items(&outputs);
    }

    let _ = prog_tx
        .send(EngineProgress::Finished {
            job_id: "bg".to_string(),
            outputs,
            summary: "İşlem arka planda tamamlandı.".to_string(),
        })
        .await;
}

fn try_send_cli_convext(action_code: &str, files: &[String]) -> bool {
    let sock_path = TrayService::control_socket_path();
    let payload = format!("CONVEXT:{}\t{}\n", action_code, files.join("\t"));
    if let Ok(mut stream) = UnixStream::connect(&sock_path) {
        if stream.write_all(payload.as_bytes()).is_ok() {
            return true;
        }
    }
    false
}

fn main() -> ConvextResult<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 4 && args[1] == "--convext" {
        let action_code = &args[2];
        let file_args = &args[3..];
        if try_send_cli_convext(action_code, file_args) {
            return Ok(());
        }
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new(exe)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
            for _ in 0..25 {
                std::thread::sleep(std::time::Duration::from_millis(120));
                if try_send_cli_convext(action_code, file_args) {
                    return Ok(());
                }
            }
        }
        return Ok(());
    }

    let _instance_guard = SingleInstanceGuard::acquire()?;
    let config = Arc::new(RwLock::new(ConvextConfig::load_or_create()?));
    let _capabilities = Arc::new(DependencyCapabilities::probe_system());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let _rt_guard = rt.enter();

    let (tx, rx) = async_channel::unbounded::<DaemonEvent>();
    let (prog_tx, prog_rx) = async_channel::unbounded::<EngineProgress>();

    let tray = TrayService::new(config.clone(), tx.clone());
    tray.spawn();

    let _ = adw::init();
    let style_mgr = adw::StyleManager::default();
    style_mgr.set_color_scheme(adw::ColorScheme::PreferDark);

    let app = gtk4::Application::builder()
        .application_id("org.convext.Daemon")
        .build();

    let config_activate = config.clone();
    let prog_tx_activate = prog_tx.clone();
    let tx_activate = tx.clone();

    app.connect_activate(move |app| {
        load_css();

        let hold_guard = app.hold();
        Box::leak(Box::new(hold_guard));

        let prog_rx_loop = prog_rx.clone();
        glib::spawn_future_local(async move {
            while let Ok(ev) = prog_rx_loop.recv().await {
                match ev {
                    EngineProgress::Finished { outputs, summary, .. } => {
                        let count = outputs.len();
                        let body = if count > 0 {
                            format!("{} ({} dosya hazır)", summary, count)
                        } else {
                            summary
                        };
                        DesktopActions::send_desktop_notification("Convext Menü", &body);
                    }
                    EngineProgress::Failed { error, .. } => {
                        DesktopActions::send_desktop_notification("Convext Hata", &error);
                    }
                    _ => {}
                }
            }
        });

        let tx_shelf = tx_activate.clone();
        let shelf = FloatShelfWindow::new(app, move |files| {
            let _ = tx_shelf.send_blocking(DaemonEvent::ShowConvextStudio { files });
        });

        let rx_loop = rx.clone();
        let shelf_loop = shelf.clone();
        let config_loop = config_activate.clone();
        let prog_tx_loop = prog_tx_activate.clone();
        let app_clone = app.clone();

        glib::spawn_future_local(async move {
            while let Ok(ev) = rx_loop.recv().await {
                match ev {
                    DaemonEvent::ShowConvextStudio { files } => {
                        let app_c = app_clone.clone();
                        let shelf_c = shelf_loop.clone();
                        let cfg_c = config_loop.clone();
                        let tx_c = prog_tx_loop.clone();
                        ConvextStudioWindow::show(&app_clone, files, move |code, target_files| {
                            dispatch_convext_command(
                                &app_c,
                                &shelf_c,
                                &code,
                                target_files,
                                cfg_c.clone(),
                                tx_c.clone(),
                            );
                        });
                    }
                    DaemonEvent::ExecuteConvextCommand { action_code, files } => {
                        let paused = config_loop
                            .read()
                            .map(|c| c.daemon_paused)
                            .unwrap_or(false);
                        if !paused {
                            dispatch_convext_command(
                                &app_clone,
                                &shelf_loop,
                                &action_code,
                                files,
                                config_loop.clone(),
                                prog_tx_loop.clone(),
                            );
                        }
                    }
                    DaemonEvent::ToggleFloatShelf => {
                        let vis = shelf_loop.window.is_visible();
                        shelf_loop.window.set_visible(!vis);
                        if !vis {
                            shelf_loop.window.present();
                        }
                    }
                    DaemonEvent::TogglePause => {
                        if let Ok(mut cfg) = config_loop.write() {
                            cfg.daemon_paused = !cfg.daemon_paused;
                            let _ = cfg.save();
                        }
                    }
                    DaemonEvent::SelectPreset(name) => {
                        if let Ok(mut cfg) = config_loop.write() {
                            cfg.active_preset = name;
                            let _ = cfg.save();
                        }
                    }
                    DaemonEvent::QuitDaemon => {
                        app_clone.quit();
                        break;
                    }
                    _ => {}
                }
            }
        });
    });

    let empty_args: Vec<String> = Vec::new();
    app.run_with_args(&empty_args);
    Ok(())
}
