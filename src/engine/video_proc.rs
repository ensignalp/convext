use crate::error::{ConvextError, ConvextResult};
use crate::ui::modals::media_trimmer::TrimParameters;
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(dead_code)]
pub struct VideoProcessor;

#[allow(dead_code)]
impl VideoProcessor {
    pub fn default_output(input: &Path, suffix: &str, ext: &str) -> PathBuf {
        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("video");
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        parent.join(format!("{}{}.{}", stem, suffix, ext))
    }

    pub fn convert_format(input: &Path, format: &str, suffix: &str) -> ConvextResult<PathBuf> {
        let fmt = format.to_ascii_lowercase();
        if fmt == "mp3" {
            return Self::extract_audio_mp3(input, suffix);
        }
        let out = Self::default_output(input, suffix, &fmt);
        let in_str = input.to_string_lossy().to_string();
        let out_str = out.to_string_lossy().to_string();

        match fmt.as_str() {
            "mp4" | "mov" | "mkv" => Self::run_ffmpeg(&[
                "-y", "-i", &in_str, "-c:v", "libx264", "-preset", "fast", "-crf", "23", "-c:a",
                "aac", "-b:a", "128k", &out_str,
            ])?,
            "webm" => Self::run_ffmpeg(&[
                "-y", "-i", &in_str, "-c:v", "libvpx-vp9", "-crf", "30", "-b:v", "0", "-c:a",
                "libopus", &out_str,
            ])?,
            "avi" => Self::run_ffmpeg(&[
                "-y", "-i", &in_str, "-c:v", "mpeg4", "-q:v", "5", "-c:a", "libmp3lame", &out_str,
            ])?,
            "wmv" => Self::run_ffmpeg(&[
                "-y", "-i", &in_str, "-c:v", "wmv2", "-b:v", "2M", "-c:a", "wmav2", &out_str,
            ])?,
            "gif" => Self::run_ffmpeg(&[
                "-y",
                "-i",
                &in_str,
                "-vf",
                "fps=12,scale=480:-1:flags=lanczos",
                &out_str,
            ])?,
            _ => Self::run_ffmpeg(&["-y", "-i", &in_str, &out_str])?,
        }
        Ok(out)
    }

    pub fn probe_duration_secs(input: &Path) -> ConvextResult<f64> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
            ])
            .arg(input)
            .output()
            .map_err(|e| ConvextError::ExternalBinary {
                binary: "ffprobe".to_string(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            return Ok(10.0);
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let parsed = text.trim().parse::<f64>().unwrap_or(10.0);
        Ok(parsed.max(0.5))
    }

    pub fn convert_to_mp4(input: &Path, suffix: &str) -> ConvextResult<PathBuf> {
        Self::convert_format(input, "mp4", suffix)
    }

    pub fn extract_audio_mp3(input: &Path, suffix: &str) -> ConvextResult<PathBuf> {
        let out = Self::default_output(input, suffix, "mp3");
        Self::run_ffmpeg(&[
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-vn",
            "-acodec",
            "libmp3lame",
            "-q:a",
            "2",
            &out.to_string_lossy(),
        ])?;
        Ok(out)
    }

    pub fn capture_snapshot_png(input: &Path, suffix: &str) -> ConvextResult<PathBuf> {
        let duration = Self::probe_duration_secs(input).unwrap_or(4.0);
        let timestamp = (duration * 0.25).clamp(0.0, duration);
        let ts_str = format!("{:.2}", timestamp);
        let out = Self::default_output(input, suffix, "png");
        Self::run_ffmpeg(&[
            "-y",
            "-ss",
            &ts_str,
            "-i",
            &input.to_string_lossy(),
            "-frames:v",
            "1",
            &out.to_string_lossy(),
        ])?;
        Ok(out)
    }

    pub fn quick_compress(input: &Path, suffix: &str) -> ConvextResult<PathBuf> {
        let out = Self::default_output(input, suffix, "mp4");
        Self::run_ffmpeg(&[
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-c:v",
            "libx264",
            "-preset",
            "fast",
            "-crf",
            "28",
            "-c:a",
            "aac",
            "-b:a",
            "96k",
            &out.to_string_lossy(),
        ])?;
        Ok(out)
    }

    pub fn compress_to_target_bytes(
        input: &Path,
        target_bytes: u64,
        suffix: &str,
    ) -> ConvextResult<PathBuf> {
        let duration = Self::probe_duration_secs(input)?;
        let total_bits = (target_bytes as f64) * 8.0 * 0.92;
        let total_kbps = (total_bits / duration / 1000.0).max(160.0);
        let audio_kbps = 96.0;
        let video_kbps = (total_kbps - audio_kbps).max(64.0) as u32;
        let v_bitrate = format!("{}k", video_kbps);
        let max_rate = format!("{}k", (video_kbps as f64 * 1.25) as u32);
        let buf_size = format!("{}k", video_kbps * 2);

        let out = Self::default_output(input, suffix, "mp4");
        Self::run_ffmpeg(&[
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-b:v",
            &v_bitrate,
            "-maxrate",
            &max_rate,
            "-bufsize",
            &buf_size,
            "-c:a",
            "aac",
            "-b:a",
            "96k",
            &out.to_string_lossy(),
        ])?;
        Ok(out)
    }

    pub fn trim_and_speed(
        input: &Path,
        params: &TrimParameters,
        suffix: &str,
    ) -> ConvextResult<PathBuf> {
        let ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp4");
        let out = Self::default_output(input, suffix, ext);
        let start = params.start_seconds.max(0.0);
        let end = params.end_seconds.max(start + 0.2);
        let speed = params.speed_multiplier.clamp(0.5, 2.0);
        let pts_mult = 1.0 / speed;

        let vf = format!("setpts={:.4}*PTS", pts_mult);
        let af = format!("atempo={:.4}", speed);
        let start_s = format!("{:.2}", start);
        let end_s = format!("{:.2}", end);

        Self::run_ffmpeg(&[
            "-y",
            "-ss",
            &start_s,
            "-to",
            &end_s,
            "-i",
            &input.to_string_lossy(),
            "-filter:v",
            &vf,
            "-filter:a",
            &af,
            &out.to_string_lossy(),
        ])?;
        Ok(out)
    }

    pub fn split_in_half(input: &Path) -> ConvextResult<Vec<PathBuf>> {
        let duration = Self::probe_duration_secs(input)?;
        let mid = duration / 2.0;
        let mid_s = format!("{:.2}", mid);
        let ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp4");

        let part1 = Self::default_output(input, "_part1", ext);
        let part2 = Self::default_output(input, "_part2", ext);

        Self::run_ffmpeg(&[
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-t",
            &mid_s,
            "-c",
            "copy",
            &part1.to_string_lossy(),
        ])?;

        Self::run_ffmpeg(&[
            "-y",
            "-ss",
            &mid_s,
            "-i",
            &input.to_string_lossy(),
            "-c",
            "copy",
            &part2.to_string_lossy(),
        ])?;

        Ok(vec![part1, part2])
    }

    pub fn join_videos(inputs: &[PathBuf], suffix: &str) -> ConvextResult<PathBuf> {
        let first = inputs.first().ok_or_else(|| {
            ConvextError::InvalidInput("No videos provided for join.".to_string())
        })?;
        let out = Self::default_output(first, suffix, "mp4");
        let list_path = std::env::temp_dir().join(format!(
            "convext_concat_{}.txt",
            chrono::Utc::now().timestamp_millis()
        ));

        let mut list_content = String::new();
        for p in inputs {
            let escaped = p.to_string_lossy().replace('\'', "'\\''");
            list_content.push_str(&format!("file '{}'\n", escaped));
        }
        std::fs::write(&list_path, list_content)?;

        let res = Self::run_ffmpeg(&[
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            &list_path.to_string_lossy(),
            "-c",
            "copy",
            &out.to_string_lossy(),
        ]);
        let _ = std::fs::remove_file(&list_path);
        res?;
        Ok(out)
    }

    fn run_ffmpeg(args: &[&str]) -> ConvextResult<()> {
        let output = Command::new("ffmpeg").args(args).output().map_err(|e| {
            ConvextError::ExternalBinary {
                binary: "ffmpeg".to_string(),
                reason: e.to_string(),
            }
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(ConvextError::ExternalBinary {
                binary: "ffmpeg".to_string(),
                reason: stderr,
            });
        }
        Ok(())
    }

    pub fn mute_audio(input: &Path, suffix: &str) -> ConvextResult<PathBuf> {
        let ext = input.extension().and_then(|e| e.to_str()).unwrap_or("mp4");
        let out = Self::default_output(input, suffix, ext);
        Self::run_ffmpeg(&[
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-c:v",
            "copy",
            "-an",
            &out.to_string_lossy(),
        ])?;
        Ok(out)
    }

    pub fn crop_center_video(input: &Path, suffix: &str) -> ConvextResult<PathBuf> {
        let ext = input.extension().and_then(|e| e.to_str()).unwrap_or("mp4");
        let out = Self::default_output(input, suffix, ext);
        Self::run_ffmpeg(&[
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-vf",
            "crop=in_w*0.85:in_h*0.85",
            "-c:a",
            "copy",
            &out.to_string_lossy(),
        ])?;
        Ok(out)
    }

    pub fn redact_video_center(input: &Path, suffix: &str) -> ConvextResult<PathBuf> {
        let ext = input.extension().and_then(|e| e.to_str()).unwrap_or("mp4");
        let out = Self::default_output(input, suffix, ext);
        Self::run_ffmpeg(&[
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-vf",
            "drawbox=x=in_w*0.25:y=in_h*0.35:w=in_w*0.5:h=in_h*0.3:color=black@1:t=fill",
            "-c:a",
            "copy",
            &out.to_string_lossy(),
        ])?;
        Ok(out)
    }
}
