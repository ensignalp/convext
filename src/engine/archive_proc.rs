use crate::error::{CitrusError, CitrusResult};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[allow(dead_code)]
pub struct ArchiveProcessor;

#[allow(dead_code)]
impl ArchiveProcessor {
    pub fn convert_archive(input: &Path, format: &str, suffix: &str) -> CitrusResult<PathBuf> {
        let unpacked_dir = Self::extract_archive(input)?;
        let entries = vec![unpacked_dir.clone()];
        let fmt = format.to_ascii_lowercase();
        let res = match fmt.as_str() {
            "zip" => Self::create_zip(&entries, suffix),
            "tar" => Self::create_tar(&entries, suffix),
            "gzip" | "gz" | "tgz" => Self::create_tar_gz(&entries, suffix),
            _ => Self::create_zip(&entries, suffix),
        };
        let _ = fs::remove_dir_all(&unpacked_dir);
        res
    }

    pub fn extract_archive(input: &Path) -> CitrusResult<PathBuf> {
        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("extracted");
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        let out_dir = parent.join(format!("{}_unpacked", stem));
        fs::create_dir_all(&out_dir)?;

        let file_name_lower = input
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        if file_name_lower.ends_with(".zip") {
            let file = File::open(input)?;
            let mut archive =
                ZipArchive::new(file).map_err(|e| CitrusError::Archive(e.to_string()))?;
            for i in 0..archive.len() {
                let mut entry = archive
                    .by_index(i)
                    .map_err(|e| CitrusError::Archive(e.to_string()))?;
                let Some(enclosed) = entry.enclosed_name().map(|p| p.to_path_buf()) else {
                    continue;
                };
                let dest = out_dir.join(enclosed);
                if entry.is_dir() {
                    fs::create_dir_all(&dest)?;
                } else {
                    if let Some(p) = dest.parent() {
                        fs::create_dir_all(p)?;
                    }
                    let mut out_file = File::create(&dest)?;
                    std::io::copy(&mut entry, &mut out_file)?;
                }
            }
            return Ok(out_dir);
        }

        if file_name_lower.ends_with(".tar.gz") || file_name_lower.ends_with(".tgz") {
            let file = File::open(input)?;
            let gz = GzDecoder::new(file);
            let mut tar_arc = tar::Archive::new(gz);
            tar_arc
                .unpack(&out_dir)
                .map_err(|e| CitrusError::Archive(e.to_string()))?;
            return Ok(out_dir);
        }

        if file_name_lower.ends_with(".tar") {
            let file = File::open(input)?;
            let mut tar_arc = tar::Archive::new(file);
            tar_arc
                .unpack(&out_dir)
                .map_err(|e| CitrusError::Archive(e.to_string()))?;
            return Ok(out_dir);
        }

        let out_arg = format!("-o{}", out_dir.to_string_lossy());
        let output = Command::new("7z")
            .arg("x")
            .arg("-y")
            .arg(&out_arg)
            .arg(input)
            .output()
            .map_err(|e| CitrusError::ExternalBinary {
                binary: "7z".to_string(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(CitrusError::Archive(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(out_dir)
    }

    pub fn create_zip(inputs: &[PathBuf], suffix: &str) -> CitrusResult<PathBuf> {
        let first = inputs.first().ok_or_else(|| {
            CitrusError::InvalidInput("No files selected for ZIP creation.".to_string())
        })?;
        let stem = first
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("archive");
        let parent = first.parent().unwrap_or_else(|| Path::new("."));
        let out_path = parent.join(format!("{}{}.zip", stem, suffix));

        let file = File::create(&out_path)?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644);

        for path in inputs {
            if path.is_file() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("item");
                zip.start_file(name, options)
                    .map_err(|e| CitrusError::Archive(e.to_string()))?;
                let mut f = File::open(path)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                zip.write_all(&buf)?;
            } else if path.is_dir() {
                Self::add_dir_to_zip(&mut zip, path, path, options)?;
            }
        }

        zip.finish()
            .map_err(|e| CitrusError::Archive(e.to_string()))?;
        Ok(out_path)
    }

    pub fn create_tar(inputs: &[PathBuf], suffix: &str) -> CitrusResult<PathBuf> {
        let first = inputs.first().ok_or_else(|| {
            CitrusError::InvalidInput("No files selected for TAR creation.".to_string())
        })?;
        let stem = first
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("archive");
        let parent = first.parent().unwrap_or_else(|| Path::new("."));
        let out_path = parent.join(format!("{}{}.tar", stem, suffix));

        let tar_file = File::create(&out_path)?;
        let mut tar = tar::Builder::new(tar_file);

        for path in inputs {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("item");
            if path.is_file() {
                tar.append_path_with_name(path, name)?;
            } else if path.is_dir() {
                tar.append_dir_all(name, path)?;
            }
        }

        tar.finish()?;
        Ok(out_path)
    }

    fn add_dir_to_zip<W: Write + std::io::Seek>(
        zip: &mut ZipWriter<W>,
        root: &Path,
        current: &Path,
        options: SimpleFileOptions,
    ) -> CitrusResult<()> {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            let rel = path
                .strip_prefix(root.parent().unwrap_or(root))
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            if path.is_dir() {
                Self::add_dir_to_zip(zip, root, &path, options)?;
            } else if path.is_file() {
                zip.start_file(&rel, options)
                    .map_err(|e| CitrusError::Archive(e.to_string()))?;
                let mut f = File::open(&path)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                zip.write_all(&buf)?;
            }
        }
        Ok(())
    }

    pub fn create_tar_gz(inputs: &[PathBuf], suffix: &str) -> CitrusResult<PathBuf> {
        let first = inputs.first().ok_or_else(|| {
            CitrusError::InvalidInput("No files selected for TAR.GZ creation.".to_string())
        })?;
        let stem = first
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("archive");
        let parent = first.parent().unwrap_or_else(|| Path::new("."));
        let out_path = parent.join(format!("{}{}.tar.gz", stem, suffix));

        let tar_gz = File::create(&out_path)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = tar::Builder::new(enc);

        for path in inputs {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("item");
            if path.is_file() {
                tar.append_path_with_name(path, name)?;
            } else if path.is_dir() {
                tar.append_dir_all(name, path)?;
            }
        }

        let enc = tar.into_inner()?;
        enc.finish()?;
        Ok(out_path)
    }
}
