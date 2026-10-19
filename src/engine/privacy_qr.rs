use crate::engine::mime_router::{MediaCategory, MimeRouter};
use crate::error::{CitrusError, CitrusResult};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct PrivacyQrProcessor;

impl PrivacyQrProcessor {
    pub fn strip_metadata(input: &Path, suffix: &str) -> CitrusResult<PathBuf> {
        let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("clean");
        let ext = input.extension().and_then(|s| s.to_str()).unwrap_or("dat");
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        let out = parent.join(format!("{}{}.{}", stem, suffix, ext));

        std::fs::copy(input, &out)?;
        let exif_status = Command::new("exiftool")
            .args(["-all=", "-overwrite_original"])
            .arg(&out)
            .status();

        if exif_status.map(|s| s.success()).unwrap_or(false) {
            return Ok(out);
        }

        if MimeRouter::inspect_file(input) == MediaCategory::Image {
            let img = image::open(input)?;
            img.save(&out)?;
            return Ok(out);
        }

        Ok(out)
    }

    pub fn decode_qr_from_image(input: &Path) -> CitrusResult<String> {
        let img = image::open(input)?.to_luma8();
        let mut prepared = rqrr::PreparedImage::prepare(img);
        let grids = prepared.detect_grids();
        if grids.is_empty() {
            return Err(CitrusError::PrivacyQr(
                "No QR code detected in the selected image.".to_string(),
            ));
        }

        let mut decoded_payloads = Vec::new();
        for grid in grids {
            if let Ok((_meta, content)) = grid.decode() {
                decoded_payloads.push(content);
            }
        }

        if decoded_payloads.is_empty() {
            return Err(CitrusError::PrivacyQr(
                "QR grid found, but payload decoding failed.".to_string(),
            ));
        }

        Ok(decoded_payloads.join("\n"))
    }
}
