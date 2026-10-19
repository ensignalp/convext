use crate::engine::pdf_proc::PdfProcessor;
use crate::error::{CitrusError, CitrusResult};
use crate::ui::modals::image_editor::{RedactionBox, RedactionTool};
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(dead_code)]
pub struct ImageProcessor;

#[allow(dead_code)]
impl ImageProcessor {
    pub fn output_path_with_ext(input: &Path, suffix: &str, ext: &str) -> PathBuf {
        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        parent.join(format!("{}{}.{}", stem, suffix, ext))
    }

    pub fn convert_format(input: &Path, format: &str, suffix: &str) -> CitrusResult<PathBuf> {
        let clean_ext = format.to_ascii_lowercase();
        if clean_ext == "pdf" {
            return PdfProcessor::images_to_pdf_collage(&[input.to_path_buf()], suffix);
        }

        let out = Self::output_path_with_ext(input, suffix, &clean_ext);
        let input_ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        if matches!(clean_ext.as_str(), "heic" | "avif")
            || matches!(input_ext.as_str(), "svg" | "heic" | "heif" | "avif")
        {
            if Self::convert_via_magick_or_ffmpeg(input, &out).is_ok() {
                return Ok(out);
            }
        }

        let img = image::open(input)?;
        let fmt = match clean_ext.as_str() {
            "png" => ImageFormat::Png,
            "jpg" | "jpeg" => ImageFormat::Jpeg,
            "webp" => ImageFormat::WebP,
            "gif" => ImageFormat::Gif,
            "bmp" => ImageFormat::Bmp,
            "ico" => ImageFormat::Ico,
            "tiff" | "tif" => ImageFormat::Tiff,
            _ => ImageFormat::Png,
        };
        img.save_with_format(&out, fmt)?;
        Ok(out)
    }

    fn convert_via_magick_or_ffmpeg(input: &Path, out: &Path) -> CitrusResult<()> {
        if let Ok(status) = Command::new("magick").arg(input).arg(out).status() {
            if status.success() {
                return Ok(());
            }
        }
        let status = Command::new("ffmpeg")
            .args(["-y", "-i"])
            .arg(input)
            .arg(out)
            .status()
            .map_err(|e| CitrusError::ExternalBinary {
                binary: "magick/ffmpeg".to_string(),
                reason: e.to_string(),
            })?;
        if status.success() {
            Ok(())
        } else {
            Err(CitrusError::InvalidInput(
                "External image conversion failed.".to_string(),
            ))
        }
    }

    pub fn compress_percentage(input: &Path, quality: u8, suffix: &str) -> CitrusResult<PathBuf> {
        let img = image::open(input)?;
        let out = Self::output_path_with_ext(input, suffix, "jpg");
        let file = File::create(&out)?;
        let writer = BufWriter::new(file);
        let rgb = img.to_rgb8();
        let mut encoder = JpegEncoder::new_with_quality(writer, quality.clamp(5, 100));
        encoder.encode_image(&rgb)?;
        Ok(out)
    }

    pub fn compress_to_target_bytes(
        input: &Path,
        target_bytes: u64,
        suffix: &str,
    ) -> CitrusResult<PathBuf> {
        let orig = image::open(input)?;
        let out = Self::output_path_with_ext(input, suffix, "jpg");
        let mut best_buf: Vec<u8> = Vec::new();

        for scale_step in 0..5 {
            let scale_factor = 1.0 - (scale_step as f64 * 0.16);
            let scaled: DynamicImage = if scale_step == 0 {
                orig.clone()
            } else {
                let (w, h) = orig.dimensions();
                let nw = ((w as f64 * scale_factor) as u32).max(64);
                let nh = ((h as f64 * scale_factor) as u32).max(64);
                orig.resize(nw, nh, image::imageops::FilterType::Lanczos3)
            };

            let rgb = scaled.to_rgb8();
            let mut low: u8 = 5;
            let mut high: u8 = 98;
            let mut matched_in_scale = false;

            while low <= high {
                let mid = low + (high - low) / 2;
                let mut candidate = Vec::new();
                {
                    let mut enc = JpegEncoder::new_with_quality(&mut candidate, mid);
                    enc.encode_image(&rgb)?;
                }
                let sz = candidate.len() as u64;
                if sz <= target_bytes {
                    best_buf = candidate;
                    matched_in_scale = true;
                    low = mid.saturating_add(1);
                    if mid == 98 {
                        break;
                    }
                } else {
                    if mid <= 5 {
                        if best_buf.is_empty() {
                            best_buf = candidate;
                        }
                        break;
                    }
                    high = mid.saturating_sub(1);
                }
            }

            if matched_in_scale {
                break;
            }
        }

        if best_buf.is_empty() {
            return Err(CitrusError::InvalidInput(
                "Could not encode image to target size.".to_string(),
            ));
        }

        std::fs::write(&out, &best_buf)?;
        Ok(out)
    }

    pub fn apply_redactions(
        input: &Path,
        boxes: &[RedactionBox],
        suffix: &str,
    ) -> CitrusResult<PathBuf> {
        let img = image::open(input)?;
        let mut rgba: RgbaImage = img.to_rgba8();
        let (mut img_w, mut img_h) = rgba.dimensions();

        for b in boxes {
            let x0 = b.x.min(img_w.saturating_sub(1));
            let y0 = b.y.min(img_h.saturating_sub(1));
            let x1 = (b.x.saturating_add(b.width)).min(img_w);
            let y1 = (b.y.saturating_add(b.height)).min(img_h);
            if x1 <= x0 || y1 <= y0 {
                continue;
            }

            match b.tool {
                RedactionTool::Blackout => {
                    for py in y0..y1 {
                        for px in x0..x1 {
                            rgba.put_pixel(px, py, Rgba([0, 0, 0, 255]));
                        }
                    }
                }
                RedactionTool::Pixelate => {
                    let block: u32 = 14;
                    let mut by = y0;
                    while by < y1 {
                        let mut bx = x0;
                        while bx < x1 {
                            let bw = block.min(x1 - bx);
                            let bh = block.min(y1 - by);
                            let mut r_sum: u64 = 0;
                            let mut g_sum: u64 = 0;
                            let mut b_sum: u64 = 0;
                            let count = (bw * bh).max(1) as u64;

                            for py in by..(by + bh) {
                                for px in bx..(bx + bw) {
                                    let p = rgba.get_pixel(px, py);
                                    r_sum += p[0] as u64;
                                    g_sum += p[1] as u64;
                                    b_sum += p[2] as u64;
                                }
                            }
                            let avg = Rgba([
                                (r_sum / count) as u8,
                                (g_sum / count) as u8,
                                (b_sum / count) as u8,
                                255,
                            ]);
                            for py in by..(by + bh) {
                                for px in bx..(bx + bw) {
                                    rgba.put_pixel(px, py, avg);
                                }
                            }
                            bx += block;
                        }
                        by += block;
                    }
                }
                RedactionTool::Crop => {
                    let cropped =
                        image::imageops::crop_imm(&rgba, x0, y0, x1 - x0, y1 - y0).to_image();
                    rgba = cropped;
                    let dims = rgba.dimensions();
                    img_w = dims.0;
                    img_h = dims.1;
                }
            }
        }

        let ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_ascii_lowercase();
        let out = Self::output_path_with_ext(input, suffix, &ext);
        DynamicImage::ImageRgba8(rgba).save(&out)?;
        Ok(out)
    }
}

impl ImageProcessor {
    pub fn add_solid_background(input: &Path, suffix: &str) -> CitrusResult<PathBuf> {
        let img = image::open(input)?.to_rgba8();
        let (w, h) = img.dimensions();
        let pad = ((w.max(h) as f64) * 0.08) as u32;
        let nw = w + pad * 2;
        let nh = h + pad * 2;
        let mut canvas = RgbaImage::from_pixel(nw, nh, Rgba([24, 25, 30, 255]));
        for y in 0..h {
            for x in 0..w {
                let p = img.get_pixel(x, y);
                let alpha = p[3] as f64 / 255.0;
                let bg = canvas.get_pixel(x + pad, y + pad);
                let r = (p[0] as f64 * alpha + bg[0] as f64 * (1.0 - alpha)) as u8;
                let g = (p[1] as f64 * alpha + bg[1] as f64 * (1.0 - alpha)) as u8;
                let b = (p[2] as f64 * alpha + bg[2] as f64 * (1.0 - alpha)) as u8;
                canvas.put_pixel(x + pad, y + pad, Rgba([r, g, b, 255]));
            }
        }
        let out = Self::output_path_with_ext(input, suffix, "png");
        DynamicImage::ImageRgba8(canvas).save(&out)?;
        Ok(out)
    }
}
