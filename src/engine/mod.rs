pub mod archive_proc;
pub mod audio_proc;
pub mod doc_proc;
pub mod image_proc;
pub mod mime_router;
pub mod pdf_proc;
pub mod privacy_qr;
pub mod video_proc;

use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum EngineProgress {
    Started {
        job_id: String,
        title: String,
        total_items: usize,
    },
    Step {
        job_id: String,
        completed_items: usize,
        total_items: usize,
        status_text: String,
    },
    Finished {
        job_id: String,
        outputs: Vec<PathBuf>,
        summary: String,
    },
    Failed {
        job_id: String,
        error: String,
    },
}
