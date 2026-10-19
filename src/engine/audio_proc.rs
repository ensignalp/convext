use crate::engine::video_proc::VideoProcessor;
use crate::error::{CitrusError, CitrusResult};
use crate::ui::modals::media_trimmer::TrimParameters;
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(dead_code)]
pub struct AudioProcessor;

#[allow(dead_code)]
impl AudioProcessor {
    pub fn default_output(input: &Path, suffix: &str, ext: &str) -> PathBuf {
        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("audio");
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        parent.join(format!("{}{}.{}", stem, suffix, ext))
    }

    pub fn convert_format(input: &Path, format: &str, suffix: &str) -> CitrusResult<PathBuf> {
        let fmt = format.to_ascii_lowercase();
        let out = Self::default_output(input, suffix, &fmt);
        let in_str = input.to_string_lossy().to_string();
        let out_str = out.to_string_lossy().to_string();

        match fmt.as_str() {
            "mp3" => Self::run_ffmpeg(&["-y", "-i", &in_str, "-c:a", "libmp3lame", "-q:a", "2", &out_str])?,
            "m4a" => Self::run_ffmpeg(&["-y", "-i", &in_str, "-c:a", "aac", "-b:a", "192k", &out_str])?,
            "wav" => Self::run_ffmpeg(&["-y", "-i", &in_str, "-c:a", "pcm_s16le", &out_str])?,
            "flac" => Self::run_ffmpeg(&["-y", "-i", &in_str, "-c:a", "flac", &out_str])?,
            "ogg" => Self::run_ffmpeg(&["-y", "-i", &in_str, "-c:a", "libvorbis", "-q:a", "5", &out_str])?,
            "opus" => Self::run_ffmpeg(&["-y", "-i", &in_str, "-c:a", "libopus", "-b:a", "128k", &out_str])?,
            "aiff" => Self::run_ffmpeg(&["-y", "-i", &in_str, "-c:a", "pcm_s16be", &out_str])?,
            "wma" => Self::run_ffmpeg(&["-y", "-i", &in_str, "-c:a", "wmav2", "-b:a", "160k", &out_str])?,
            _ => Self::run_ffmpeg(&["-y", "-i", &in_str, &out_str])?,
        }
        Ok(out)
    }

    pub fn normalize_ebu_r128(input: &Path, suffix: &str) -> CitrusResult<PathBuf> {
        let ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp3");
        let out = Self::default_output(input, suffix, ext);
        Self::run_ffmpeg(&[
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-af",
            "loudnorm=I=-16:TP=-1.5:LRA=11",
            &out.to_string_lossy(),
        ])?;
        Ok(out)
    }

    pub fn trim_and_speed(
        input: &Path,
        params: &TrimParameters,
        suffix: &str,
    ) -> CitrusResult<PathBuf> {
        let ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp3");
        let out = Self::default_output(input, suffix, ext);
        let start = params.start_seconds.max(0.0);
        let end = params.end_seconds.max(start + 0.2);
        let speed = params.speed_multiplier.clamp(0.5, 2.0);
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
            "-filter:a",
            &af,
            &out.to_string_lossy(),
        ])?;
        Ok(out)
    }

    pub fn apply_sine_bleep(
        input: &Path,
        bleep_start: f64,
        bleep_end: f64,
        suffix: &str,
    ) -> CitrusResult<PathBuf> {
        let ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp3");
        let out = Self::default_output(input, suffix, ext);
        let dur = (bleep_end - bleep_start).max(0.4);
        let filter_complex = format!(
            "[0:a]volume=enable='between(t,{:.2},{:.2})':volume=0[main];sine=frequency=1000:duration={:.2}[beep];[beep]adelay={}s:all=1[delayed];[main][delayed]amix=inputs=2:duration=first:dropout_transition=0",
            bleep_start,
            bleep_end,
            dur,
            (bleep_start * 1000.0) as u64
        );
        Self::run_ffmpeg(&[
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-filter_complex",
            &filter_complex,
            &out.to_string_lossy(),
        ])?;
        Ok(out)
    }

    pub fn bleep_middle_segment(input: &Path, suffix: &str) -> CitrusResult<PathBuf> {
        let duration = VideoProcessor::probe_duration_secs(input).unwrap_or(6.0);
        let b_start = (duration * 0.35).max(0.2);
        let b_end = (b_start + 1.2).min(duration);
        Self::apply_sine_bleep(input, b_start, b_end, suffix)
    }

    pub fn toggle_mono_stereo(input: &Path, suffix: &str) -> CitrusResult<PathBuf> {
        let ext = input
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp3");
        let channels = Self::probe_channels(input).unwrap_or(2);
        let target_ch = if channels > 1 { "1" } else { "2" };
        let out = Self::default_output(input, suffix, ext);
        Self::run_ffmpeg(&[
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-ac",
            target_ch,
            &out.to_string_lossy(),
        ])?;
        Ok(out)
    }

    fn probe_channels(input: &Path) -> CitrusResult<u32> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "a:0",
                "-show_entries",
                "stream=channels",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
            ])
            .arg(input)
            .output()
            .map_err(|e| CitrusError::ExternalBinary {
                binary: "ffprobe".to_string(),
                reason: e.to_string(),
            })?;
        let s = String::from_utf8_lossy(&output.stdout);
        Ok(s.trim().parse::<u32>().unwrap_or(2))
    }

    fn run_ffmpeg(args: &[&str]) -> CitrusResult<()> {
        let output = Command::new("ffmpeg").args(args).output().map_err(|e| {
            CitrusError::ExternalBinary {
                binary: "ffmpeg".to_string(),
                reason: e.to_string(),
            }
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(CitrusError::ExternalBinary {
                binary: "ffmpeg".to_string(),
                reason: stderr,
            });
        }
        Ok(())
    }
}

impl AudioProcessor {
    pub fn compress_audio(input: &Path, suffix: &str) -> CitrusResult<PathBuf> {
        let out = Self::default_output(input, suffix, "mp3");
        Self::run_ffmpeg(&[
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-c:a",
            "libmp3lame",
            "-b:a",
            "96k",
            &out.to_string_lossy(),
        ])?;
        Ok(out)
    }

    pub fn visualize_waveform_mp4(input: &Path, suffix: &str) -> CitrusResult<PathBuf> {
        let out = Self::default_output(input, suffix, "mp4");
        Self::run_ffmpeg(&[
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-filter_complex",
            "[0:a]showwaves=s=1280x720:mode=line:colors=0xff4800[v]",
            "-map",
            "[v]",
            "-map",
            "0:a",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-shortest",
            &out.to_string_lossy(),
        ])?;
        Ok(out)
    }
}
