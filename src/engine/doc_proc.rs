use crate::engine::pdf_proc::PdfProcessor;
use crate::error::{CitrusError, CitrusResult};
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(dead_code)]
pub struct DocProcessor;

#[allow(dead_code)]
impl DocProcessor {
    pub fn default_output(input: &Path, suffix: &str, ext: &str) -> PathBuf {
        let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("doc");
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        parent.join(format!("{}{}.{}", stem, suffix, ext))
    }

    pub fn convert_document(input: &Path, target_ext: &str, suffix: &str) -> CitrusResult<PathBuf> {
        let fmt = target_ext.to_ascii_lowercase();
        let out = Self::default_output(input, suffix, &fmt);

        if Command::new("pandoc")
            .arg(input)
            .arg("-o")
            .arg(&out)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
            && out.exists()
        {
            return Ok(out);
        }

        let lo_target = if fmt == "pages" { "docx" } else { fmt.as_str() };
        if let Ok(lo_out) = Self::convert_via_libreoffice(input, lo_target) {
            if lo_out != out {
                let _ = std::fs::rename(&lo_out, &out);
            }
            if out.exists() {
                return Ok(out);
            }
            if lo_out.exists() {
                return Ok(lo_out);
            }
        }

        if matches!(fmt.as_str(), "txt" | "md" | "html" | "csv" | "tsv") {
            if let Ok(text) = std::fs::read_to_string(input) {
                let converted = match fmt.as_str() {
                    "html" => format!(
                        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"></head><body><pre>{}</pre></body></html>",
                        text
                    ),
                    _ => text,
                };
                std::fs::write(&out, converted)?;
                return Ok(out);
            }
        }

        Err(CitrusError::Document(format!(
            "Could not convert {} to {}",
            input.display(),
            fmt
        )))
    }

    pub fn convert_slides(
        input: &Path,
        target_ext: &str,
        suffix: &str,
    ) -> CitrusResult<Vec<PathBuf>> {
        let fmt = target_ext.to_ascii_lowercase();
        if fmt == "jpg" || fmt == "png" {
            let pdf_path = Self::convert_via_libreoffice(input, "pdf")?;
            let rendered = PdfProcessor::convert_to_format(&pdf_path, &fmt, suffix)?;
            return Ok(rendered);
        }
        let single = Self::convert_via_libreoffice(input, &fmt)?;
        Ok(vec![single])
    }

    pub fn convert_spreadsheet(
        input: &Path,
        target_ext: &str,
        suffix: &str,
    ) -> CitrusResult<PathBuf> {
        let fmt = target_ext.to_ascii_lowercase();
        let in_ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let out = Self::default_output(input, suffix, &fmt);

        if (in_ext == "csv" && fmt == "tsv") || (in_ext == "tsv" && fmt == "csv") {
            let content = std::fs::read_to_string(input)?;
            let transformed = if fmt == "tsv" {
                content.replace(',', "\t")
            } else {
                content.replace('\t', ",")
            };
            std::fs::write(&out, transformed)?;
            return Ok(out);
        }

        if fmt == "tsv" {
            let csv_path = Self::convert_via_libreoffice(input, "csv")?;
            let content = std::fs::read_to_string(&csv_path)?;
            std::fs::write(&out, content.replace(',', "\t"))?;
            return Ok(out);
        }

        Self::convert_via_libreoffice(input, &fmt)
    }

    pub fn convert_subtitles(
        input: &Path,
        target_ext: &str,
        suffix: &str,
    ) -> CitrusResult<PathBuf> {
        let fmt = target_ext.to_ascii_lowercase();
        let out = Self::default_output(input, suffix, &fmt);
        let raw = std::fs::read_to_string(input)?;

        match fmt.as_str() {
            "vtt" => {
                let mut vtt = String::from("WEBVTT\n\n");
                for line in raw.lines() {
                    if line.contains("-->") {
                        vtt.push_str(&line.replace(',', "."));
                        vtt.push('\n');
                    } else {
                        vtt.push_str(line);
                        vtt.push('\n');
                    }
                }
                std::fs::write(&out, vtt)?;
            }
            "srt" => {
                let mut srt = String::new();
                let mut idx = 1usize;
                for block in raw.trim_start_matches("WEBVTT").trim().split("\n\n") {
                    let lines: Vec<&str> = block.lines().collect();
                    if lines.is_empty() {
                        continue;
                    }
                    let has_number = lines[0].trim().parse::<usize>().is_ok();
                    if !has_number {
                        srt.push_str(&format!("{}\n", idx));
                        idx += 1;
                    }
                    for l in lines {
                        if l.contains("-->") {
                            srt.push_str(&l.replace('.', ","));
                            srt.push('\n');
                        } else {
                            srt.push_str(l);
                            srt.push('\n');
                        }
                    }
                    srt.push('\n');
                }
                std::fs::write(&out, srt)?;
            }
            "txt" => {
                let mut txt = String::new();
                for line in raw.lines() {
                    let t = line.trim();
                    if t.is_empty()
                        || t == "WEBVTT"
                        || t.contains("-->")
                        || t.parse::<usize>().is_ok()
                    {
                        continue;
                    }
                    txt.push_str(t);
                    txt.push('\n');
                }
                std::fs::write(&out, txt)?;
            }
            _ => {
                std::fs::copy(input, &out)?;
            }
        }

        Ok(out)
    }

    pub fn convert_to_pdf_libreoffice(input: &Path) -> CitrusResult<PathBuf> {
        Self::convert_via_libreoffice(input, "pdf")
    }

    pub fn convert_with_pandoc(
        input: &Path,
        target_ext: &str,
        suffix: &str,
    ) -> CitrusResult<PathBuf> {
        Self::convert_document(input, target_ext, suffix)
    }

    fn convert_via_libreoffice(input: &Path, target_ext: &str) -> CitrusResult<PathBuf> {
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        let output = Command::new("libreoffice")
            .args(["--headless", "--convert-to", target_ext, "--outdir"])
            .arg(parent)
            .arg(input)
            .output()
            .map_err(|e| CitrusError::ExternalBinary {
                binary: "libreoffice".to_string(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(CitrusError::Document(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("doc");
        Ok(parent.join(format!("{}.{}", stem, target_ext)))
    }
}

impl DocProcessor {
    pub fn convert_document_multi(
        input: &Path,
        target_ext: &str,
        suffix: &str,
    ) -> CitrusResult<Vec<PathBuf>> {
        let fmt = target_ext.to_ascii_lowercase();
        if fmt == "png" || fmt == "jpg" {
            let pdf = Self::convert_to_pdf_libreoffice(input)?;
            return PdfProcessor::convert_to_format(&pdf, &fmt, suffix);
        }
        let single = Self::convert_document(input, &fmt, suffix)?;
        Ok(vec![single])
    }
}
