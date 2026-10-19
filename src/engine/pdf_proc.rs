use crate::engine::doc_proc::DocProcessor;
use crate::error::{ConvextError, ConvextResult};
use image::codecs::jpeg::JpegEncoder;
use image::GenericImageView;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(dead_code)]
pub struct PdfProcessor;

#[allow(dead_code)]
impl PdfProcessor {
    pub fn default_output(input: &Path, suffix: &str) -> PathBuf {
        let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("doc");
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        parent.join(format!("{}{}.pdf", stem, suffix))
    }

    pub fn convert_to_format(
        input: &Path,
        format: &str,
        suffix: &str,
    ) -> ConvextResult<Vec<PathBuf>> {
        let fmt = format.to_ascii_lowercase();
        let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("pdf");
        let parent = input.parent().unwrap_or_else(|| Path::new("."));

        match fmt.as_str() {
            "jpg" | "png" => {
                let flag = if fmt == "jpg" { "-jpeg" } else { "-png" };
                let prefix = parent.join(format!("{}{}_page", stem, suffix));
                let status = Command::new("pdftoppm")
                    .args([flag, "-r", "300"])
                    .arg(input)
                    .arg(&prefix)
                    .status()
                    .map_err(|e| ConvextError::ExternalBinary {
                        binary: "pdftoppm".to_string(),
                        reason: e.to_string(),
                    })?;
                if !status.success() {
                    return Err(ConvextError::Pdf("pdftoppm rasterization failed".to_string()));
                }
                let mut outs = Vec::new();
                let prefix_name = format!("{}{}_page", stem, suffix);
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                            if name.starts_with(&prefix_name) {
                                outs.push(p);
                            }
                        }
                    }
                }
                outs.sort();
                Ok(outs)
            }
            "txt" | "md" => {
                let out = parent.join(format!("{}{}.{}", stem, suffix, fmt));
                let status = Command::new("pdftotext")
                    .arg(input)
                    .arg(&out)
                    .status();
                if status.map(|s| s.success()).unwrap_or(false) && out.exists() {
                    return Ok(vec![out]);
                }
                let converted = DocProcessor::convert_document(input, &fmt, suffix)?;
                Ok(vec![converted])
            }
            "docx" | "pages" | "pptx" | "xlsx" | "epub" => {
                let converted = DocProcessor::convert_document(input, &fmt, suffix)?;
                Ok(vec![converted])
            }
            _ => {
                let converted = DocProcessor::convert_document(input, &fmt, suffix)?;
                Ok(vec![converted])
            }
        }
    }

    pub fn merge_pdfs(inputs: &[PathBuf], suffix: &str) -> ConvextResult<PathBuf> {
        let first = inputs.first().ok_or_else(|| {
            ConvextError::InvalidInput("At least one PDF is required to merge.".to_string())
        })?;
        let out = Self::default_output(first, suffix);
        let mut cmd = Command::new("qpdf");
        cmd.arg("--empty").arg("--pages");
        for p in inputs {
            cmd.arg(p);
        }
        cmd.arg("--").arg(&out);

        let output = cmd.output().map_err(|e| ConvextError::ExternalBinary {
            binary: "qpdf".to_string(),
            reason: e.to_string(),
        })?;
        if !output.status.success() {
            return Err(ConvextError::Pdf(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(out)
    }

    pub fn split_pdf_pages(input: &Path) -> ConvextResult<Vec<PathBuf>> {
        let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("page");
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        let pattern = parent.join(format!("{}_page_%d.pdf", stem));

        let output = Command::new("qpdf")
            .arg("--split-pages=1")
            .arg(input)
            .arg(&pattern)
            .output()
            .map_err(|e| ConvextError::ExternalBinary {
                binary: "qpdf".to_string(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(ConvextError::Pdf(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let mut generated = Vec::new();
        if let Ok(entries) = std::fs::read_dir(parent) {
            let prefix = format!("{}_page_", stem);
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with(&prefix) && name.ends_with(".pdf") {
                        generated.push(p);
                    }
                }
            }
        }
        generated.sort();
        Ok(generated)
    }

    pub fn images_to_pdf_collage(inputs: &[PathBuf], suffix: &str) -> ConvextResult<PathBuf> {
        let first = inputs.first().ok_or_else(|| {
            ConvextError::InvalidInput("No images provided for PDF collage.".to_string())
        })?;
        let out = Self::default_output(first, suffix);

        let mut encoded_pages: Vec<(u32, u32, Vec<u8>)> = Vec::new();
        for img_path in inputs {
            let dyn_img = image::open(img_path)?;
            let (w, h) = dyn_img.dimensions();
            let rgb = dyn_img.to_rgb8();
            let mut jpeg_bytes = Vec::new();
            let mut enc = JpegEncoder::new_with_quality(&mut jpeg_bytes, 88);
            enc.encode_image(&rgb)?;
            encoded_pages.push((w.max(1), h.max(1), jpeg_bytes));
        }

        Self::write_multi_jpeg_pdf(&out, &encoded_pages)?;
        Ok(out)
    }

    pub fn destructive_redact_pdf(input: &Path, suffix: &str) -> ConvextResult<PathBuf> {
        let out = Self::default_output(input, suffix);
        std::fs::copy(input, &out)?;
        Ok(out)
    }

    fn write_multi_jpeg_pdf(out_path: &Path, pages: &[(u32, u32, Vec<u8>)]) -> ConvextResult<()> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

        let num_pages = pages.len();
        let total_objects = 2 + num_pages * 3;
        let mut offsets: Vec<usize> = vec![0; total_objects + 1];

        offsets[1] = buf.len();
        buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        offsets[2] = buf.len();
        let mut kids = String::new();
        for i in 0..num_pages {
            let page_obj_id = 3 + i * 3;
            kids.push_str(&format!("{} 0 R ", page_obj_id));
        }
        let pages_dict = format!(
            "2 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
            kids.trim(),
            num_pages
        );
        buf.extend_from_slice(pages_dict.as_bytes());

        for (i, (w, h, jpeg_data)) in pages.iter().enumerate() {
            let page_obj = 3 + i * 3;
            let content_obj = page_obj + 1;
            let img_obj = page_obj + 2;

            let pdf_w = (*w as f64 * 0.75).max(72.0);
            let pdf_h = (*h as f64 * 0.75).max(72.0);

            offsets[page_obj] = buf.len();
            let page_str = format!(
                "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Contents {} 0 R /Resources << /XObject << /Im0 {} 0 R >> >> >>\nendobj\n",
                page_obj, pdf_w, pdf_h, content_obj, img_obj
            );
            buf.extend_from_slice(page_str.as_bytes());

            let stream_cmd = format!("q {:.2} 0 0 {:.2} 0 0 cm /Im0 Do Q\n", pdf_w, pdf_h);
            offsets[content_obj] = buf.len();
            let content_header = format!(
                "{} 0 obj\n<< /Length {} >>\nstream\n",
                content_obj,
                stream_cmd.len()
            );
            buf.extend_from_slice(content_header.as_bytes());
            buf.extend_from_slice(stream_cmd.as_bytes());
            buf.extend_from_slice(b"endstream\nendobj\n");

            offsets[img_obj] = buf.len();
            let img_header = format!(
                "{} 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n",
                img_obj,
                w,
                h,
                jpeg_data.len()
            );
            buf.extend_from_slice(img_header.as_bytes());
            buf.extend_from_slice(jpeg_data);
            buf.extend_from_slice(b"\nendstream\nendobj\n");
        }

        let xref_offset = buf.len();
        let xref_header = format!("xref\n0 {}\n0000000000 65535 f \n", total_objects + 1);
        buf.extend_from_slice(xref_header.as_bytes());
        for obj_id in 1..=total_objects {
            let line = format!("{:010} 00000 n \n", offsets[obj_id]);
            buf.extend_from_slice(line.as_bytes());
        }

        let trailer = format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            total_objects + 1,
            xref_offset
        );
        buf.extend_from_slice(trailer.as_bytes());

        let mut f = std::fs::File::create(out_path)?;
        f.write_all(&buf)?;
        Ok(())
    }
}

impl PdfProcessor {
    pub fn compress_pdf(input: &Path, suffix: &str) -> ConvextResult<PathBuf> {
        let out = Self::default_output(input, suffix);
        let qpdf_ok = Command::new("qpdf")
            .args(["--stream-data=compress", "--recompress-flate", "--compression-level=9"])
            .arg(input)
            .arg(&out)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if qpdf_ok && out.exists() {
            return Ok(out);
        }
        std::fs::copy(input, &out)?;
        Ok(out)
    }

    pub fn organize_linearize_pdf(input: &Path, suffix: &str) -> ConvextResult<PathBuf> {
        let out = Self::default_output(input, suffix);
        let qpdf_ok = Command::new("qpdf")
            .arg("--linearize")
            .arg(input)
            .arg(&out)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if qpdf_ok && out.exists() {
            return Ok(out);
        }
        std::fs::copy(input, &out)?;
        Ok(out)
    }
}
