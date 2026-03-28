use crate::errors::RecorderError;
use crate::services::audio::{
    self, AudioCaptureMode, AudioDeviceInfo, AudioTrack, NativeAudioCapture,
};
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

#[derive(Serialize)]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
}

#[derive(Serialize)]
pub struct MicInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Serialize)]
pub struct AudioOutputInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

const CREATE_NO_WINDOW: u32 = 0x08000000;
const MIN_VALID_OUTPUT_BYTES: u64 = 4 * 1024;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EncoderStrategy {
    NvidiaNvenc,
    AmdAmf,
    IntelQsv,
    SoftwareX264,
}

impl EncoderStrategy {
    pub fn label(self) -> &'static str {
        match self {
            Self::NvidiaNvenc => "h264_nvenc",
            Self::AmdAmf => "h264_amf",
            Self::IntelQsv => "h264_qsv",
            Self::SoftwareX264 => "libx264",
        }
    }

    pub fn from_label(label: &str) -> Self {
        match label {
            "h264_nvenc" => Self::NvidiaNvenc,
            "h264_amf" => Self::AmdAmf,
            "h264_qsv" => Self::IntelQsv,
            _ => Self::SoftwareX264,
        }
    }
}

pub struct CaptureSession {
    process: Child,
    final_output_path: PathBuf,
    video_output_path: PathBuf,
    log_path: PathBuf,
    encoder_label: &'static str,
    mic_capture: Option<NativeAudioCapture>,
    system_audio_capture: Option<NativeAudioCapture>,
}

fn candidate_ffmpeg_paths() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(explicit_path) = std::env::var_os("REC_CORDER_FFMPEG_PATH") {
        candidates.push(PathBuf::from(explicit_path));
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("ffmpeg.exe"));

            if let Some(target_dir) = exe_dir.parent() {
                candidates.push(target_dir.join("ffmpeg.exe"));

                if let Some(project_dir) = target_dir.parent() {
                    candidates.push(project_dir.join("ffmpeg.exe"));
                }
            }
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("ffmpeg.exe"));
    }

    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ffmpeg.exe"));

    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            candidates.push(dir.join("ffmpeg.exe"));
        }
    }

    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        candidates.push(PathBuf::from(local_app_data).join("Microsoft\\WinGet\\Links\\ffmpeg.exe"));
    }

    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        let user_profile = PathBuf::from(user_profile);
        candidates.push(user_profile.join("scoop\\shims\\ffmpeg.exe"));
        candidates.push(user_profile.join("ffmpeg\\bin\\ffmpeg.exe"));
    }

    if let Some(choco_root) = std::env::var_os("ChocolateyInstall") {
        candidates.push(PathBuf::from(choco_root).join("bin\\ffmpeg.exe"));
    }

    candidates.push(PathBuf::from(r"C:\ffmpeg\bin\ffmpeg.exe"));
    candidates.push(PathBuf::from(r"C:\ffmpeg\ffmpeg.exe"));
    candidates.push(PathBuf::from(r"C:\Program Files\ffmpeg\bin\ffmpeg.exe"));
    candidates.push(PathBuf::from(
        r"C:\Program Files (x86)\ffmpeg\bin\ffmpeg.exe",
    ));

    candidates
}

fn resolve_ffmpeg_path() -> Result<PathBuf, RecorderError> {
    let mut searched = Vec::new();

    for candidate in candidate_ffmpeg_paths() {
        let candidate_str = candidate.to_string_lossy().to_string();
        if searched.iter().any(|existing| existing == &candidate_str) {
            continue;
        }

        if candidate.is_file() {
            println!("FFmpeg encontrado em {:?}", candidate);
            return Ok(candidate);
        }

        searched.push(candidate_str);
    }

    Err(RecorderError::CaptureInit(format!(
        "FFmpeg nao foi encontrado. Adicione-o ao PATH, defina REC_CORDER_FFMPEG_PATH ou coloque o binario em src-tauri/ffmpeg.exe. Locais verificados: {}",
        searched.join(" | ")
    )))
}

fn build_video_input(monitor_index: usize, fps: u32) -> String {
    format!("ddagrab=output_idx={monitor_index}:framerate={fps}:draw_mouse=1")
}

fn build_log_path(output_path: &PathBuf) -> PathBuf {
    let mut log_dir = std::env::temp_dir();
    log_dir.push("RecCorderLogs");

    let stem = output_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("recording");

    log_dir.join(format!("{stem}.ffmpeg.log"))
}

fn build_temp_media_path(output_path: &PathBuf, suffix: &str, extension: &str) -> PathBuf {
    let parent = output_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let stem = output_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("recording");

    parent.join(format!("{stem}.{suffix}.{extension}"))
}

fn read_log_tail(log_path: &PathBuf) -> String {
    let Ok(contents) = fs::read_to_string(log_path) else {
        return String::new();
    };

    contents
        .lines()
        .rev()
        .filter(|line| !line.trim().is_empty())
        .take(8)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" | ")
}

fn cleanup_failed_attempt(output_path: &PathBuf, log_path: &PathBuf) {
    let _ = fs::remove_file(output_path);
    let _ = fs::remove_file(log_path);
}

fn append_common_inputs(cmd: &mut Command, monitor_index: usize, fps: u32) {
    let video_input = build_video_input(monitor_index, fps);
    cmd.args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i"]);
    cmd.arg(video_input);
    cmd.args(["-map", "0:v:0"]);
}

fn append_encoder_args(cmd: &mut Command, strategy: EncoderStrategy, scale_factor: u32) {
    let vf_amf = if scale_factor >= 100 {
        "hwdownload,format=bgra,format=nv12".to_string()
    } else {
        let f = scale_factor as f32 / 100.0;
        format!("hwdownload,format=bgra,scale=trunc(iw*{f}/2)*2:trunc(ih*{f}/2)*2,format=nv12")
    };

    let vf_x264 = if scale_factor >= 100 {
        "hwdownload,format=bgra,format=yuv420p".to_string()
    } else {
        let f = scale_factor as f32 / 100.0;
        format!(
            "hwdownload,format=bgra,scale=trunc(iw*{f}/2)*2:trunc(ih*{f}/2)*2,format=yuv420p"
        )
    };

    match strategy {
        EncoderStrategy::AmdAmf => {
            cmd.args([
                "-vf",
                &vf_amf,
                "-c:v",
                "h264_amf",
                "-usage",
                "lowlatency",
                "-quality",
                "speed",
                "-rc",
                "cbr",
                "-b:v",
                "5M",
                "-pix_fmt",
                "nv12",
                "-movflags",
                "+faststart",
            ]);
        }
        EncoderStrategy::NvidiaNvenc => {
            cmd.args([
                "-vf",
                &vf_x264,
                "-c:v",
                "h264_nvenc",
                "-preset",
                "p4",
                "-tune",
                "ull",
                "-rc",
                "vbr",
                "-cq",
                "23",
                "-pix_fmt",
                "yuv420p",
                "-movflags",
                "+faststart",
            ]);
        }
        EncoderStrategy::IntelQsv => {
            cmd.args([
                "-vf",
                &vf_x264,
                "-c:v",
                "h264_qsv",
                "-preset",
                "veryfast",
                "-global_quality",
                "23",
                "-pix_fmt",
                "nv12", // Intel QSV often likes nv12
                "-movflags",
                "+faststart",
            ]);
        }
        EncoderStrategy::SoftwareX264 => {
            cmd.args([
                "-vf",
                &vf_x264,
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-crf",
                "23",
                "-pix_fmt",
                "yuv420p",
                "-movflags",
                "+faststart",
            ]);
        }
    }
}

fn map_device_info(device: AudioDeviceInfo) -> (String, String, bool) {
    (device.id, device.name, device.is_default)
}

fn build_audio_filter(input_index: usize, track: &AudioTrack, label: &str) -> String {
    let channel_filter = if track.channels <= 1 {
        "pan=stereo|c0=c0|c1=c0"
    } else {
        "pan=stereo|c0=c0|c1=c1"
    };

    format!(
        "[{input_index}:a]aresample=48000,{channel_filter}[{label}]"
    )
}

fn cleanup_audio_tracks(tracks: &[AudioTrack]) {
    for track in tracks {
        let _ = fs::remove_file(&track.path);
    }
}

fn start_audio_captures(
    output_path: &PathBuf,
    mic_device_id: Option<&String>,
    system_audio_device_id: Option<&String>,
) -> Result<(Option<NativeAudioCapture>, Option<NativeAudioCapture>), RecorderError> {
    let mut mic_capture = None;
    let mut system_capture = None;

    if let Some(mic_id) = mic_device_id.filter(|value| !value.is_empty()) {
        let mic_path = build_temp_media_path(output_path, "mic", "pcm");
        match NativeAudioCapture::start(mic_id.clone(), AudioCaptureMode::Microphone, mic_path) {
            Ok(capture) => mic_capture = Some(capture),
            Err(err) => return Err(err),
        }
    }

    if let Some(system_id) = system_audio_device_id.filter(|value| !value.is_empty()) {
        let system_path = build_temp_media_path(output_path, "system", "pcm");
        match NativeAudioCapture::start(
            system_id.clone(),
            AudioCaptureMode::SystemLoopback,
            system_path,
        ) {
            Ok(capture) => system_capture = Some(capture),
            Err(err) => {
                if let Some(capture) = mic_capture.take() {
                    capture.abort();
                }
                return Err(err);
            }
        }
    }

    Ok((mic_capture, system_capture))
}

impl CaptureSession {
    pub fn start(
        output_path: PathBuf,
        monitor_index: usize,
        mic_device_id: Option<String>,
        system_audio_device_id: Option<String>,
        fps: u32,
        scale_factor: u32,
        strategy_label: &str,
    ) -> Result<Self, RecorderError> {
        let ffmpeg_path = resolve_ffmpeg_path()?;
        let log_path = build_log_path(&output_path);
        let output_dir = output_path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let log_dir = log_path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);

        fs::create_dir_all(&output_dir)?;
        fs::create_dir_all(&log_dir)?;

        let audio_requested = mic_device_id
            .as_ref()
            .is_some_and(|value| !value.is_empty())
            || system_audio_device_id
                .as_ref()
                .is_some_and(|value| !value.is_empty());
        let video_output_path = if audio_requested {
            build_temp_media_path(&output_path, "video", "mp4")
        } else {
            output_path.clone()
        };

        let strategy = EncoderStrategy::from_label(strategy_label);

        match Self::start_with_strategy(
            &ffmpeg_path,
            output_path.clone(),
            video_output_path.clone(),
            log_path.clone(),
            monitor_index,
            mic_device_id.as_ref(),
            system_audio_device_id.as_ref(),
            fps,
            scale_factor,
            strategy,
        ) {
            Ok(session) => Ok(session),
            Err(err) => {
                println!("Falha ao iniciar encoder {}: {}", strategy.label(), err);
                cleanup_failed_attempt(&video_output_path, &log_path);
                let _ = fs::remove_file(build_temp_media_path(&output_path, "mic", "pcm"));
                let _ = fs::remove_file(build_temp_media_path(&output_path, "system", "pcm"));
                Err(RecorderError::CaptureInit(format!("Encoder ({}) falhou ao iniciar: {}", strategy.label(), err)))
            }
        }
    }

    fn start_with_strategy(
        ffmpeg_path: &PathBuf,
        final_output_path: PathBuf,
        video_output_path: PathBuf,
        log_path: PathBuf,
        monitor_index: usize,
        mic_device_id: Option<&String>,
        system_audio_device_id: Option<&String>,
        fps: u32,
        scale_factor: u32,
        strategy: EncoderStrategy,
    ) -> Result<Self, RecorderError> {
        let log_file = File::create(&log_path).map_err(|e| {
            RecorderError::CaptureInit(format!(
                "Nao foi possivel criar o log do FFmpeg em {:?}: {}",
                log_path, e
            ))
        })?;

        let (mic_capture, system_audio_capture) =
            start_audio_captures(&final_output_path, mic_device_id, system_audio_device_id)?;

        let mut cmd = Command::new(ffmpeg_path);
        cmd.creation_flags(CREATE_NO_WINDOW);

        append_common_inputs(&mut cmd, monitor_index, fps);
        append_encoder_args(&mut cmd, strategy, scale_factor);

        cmd.arg("-y");
        cmd.arg(video_output_path.to_string_lossy().to_string());
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::from(log_file));

        println!(
            "Spawning FFmpeg em {:?} com encoder {}...",
            ffmpeg_path,
            strategy.label()
        );
        let process = match cmd.spawn() {
            Ok(process) => process,
            Err(e) => {
                if let Some(capture) = mic_capture {
                    capture.abort();
                }
                if let Some(capture) = system_audio_capture {
                    capture.abort();
                }
                return Err(RecorderError::CaptureInit(format!(
                    "Falha ao executar FFmpeg em {:?}. Erro: {}",
                    ffmpeg_path, e
                )));
            }
        };

        let mut session = Self {
            process,
            final_output_path,
            video_output_path,
            log_path,
            encoder_label: strategy.label(),
            mic_capture,
            system_audio_capture,
        };

        if let Err(err) = session.ensure_started() {
            if let Some(capture) = session.mic_capture.take() {
                capture.abort();
            }
            if let Some(capture) = session.system_audio_capture.take() {
                capture.abort();
            }
            let _ = session.process.kill();
            let _ = session.process.wait();
            return Err(err);
        }

        Ok(session)
    }

    fn ensure_started(&mut self) -> Result<(), RecorderError> {
        std::thread::sleep(Duration::from_millis(500));

        if let Some(status) = self.process.try_wait()? {
            let log_tail = read_log_tail(&self.log_path);
            let partial_size = fs::metadata(&self.video_output_path)
                .map(|metadata| metadata.len())
                .unwrap_or(0);

            return Err(RecorderError::CaptureInit(format!(
                "FFmpeg encerrou logo apos iniciar com {} (codigo {:?}). Arquivo parcial: {} bytes. {}",
                self.encoder_label,
                status.code(),
                partial_size,
                if log_tail.is_empty() {
                    format!("Consulte o log em {:?}", self.log_path)
                } else {
                    format!("Detalhes do FFmpeg: {}", log_tail)
                }
            )));
        }

        Ok(())
    }

    fn validate_output(&self) -> Result<(), RecorderError> {
        let file_size = fs::metadata(&self.video_output_path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);

        if file_size < MIN_VALID_OUTPUT_BYTES {
            let log_tail = read_log_tail(&self.log_path);

            return Err(RecorderError::CaptureRuntime(format!(
                "O arquivo final ficou com apenas {} bytes e nao contem video valido. Encoder: {}. {}",
                file_size,
                self.encoder_label,
                if log_tail.is_empty() {
                    format!("Consulte o log em {:?}", self.log_path)
                } else {
                    format!("Detalhes do FFmpeg: {}", log_tail)
                }
            )));
        }

        Ok(())
    }

    fn finalize_audio_tracks(&mut self) -> Result<Vec<AudioTrack>, RecorderError> {
        let mut tracks = Vec::new();

        if let Some(capture) = self.mic_capture.take() {
            let track = capture.finish()?;
            if fs::metadata(&track.path).map(|meta| meta.len()).unwrap_or(0) > 0 {
                tracks.push(track);
            } else {
                let _ = fs::remove_file(&track.path);
            }
        }

        if let Some(capture) = self.system_audio_capture.take() {
            let track = capture.finish()?;
            if fs::metadata(&track.path).map(|meta| meta.len()).unwrap_or(0) > 0 {
                tracks.push(track);
            } else {
                let _ = fs::remove_file(&track.path);
            }
        }

        Ok(tracks)
    }

    fn mux_native_audio(&self, tracks: &[AudioTrack]) -> Result<(), RecorderError> {
        let ffmpeg_path = resolve_ffmpeg_path()?;
        let mux_log_path = build_temp_media_path(&self.final_output_path, "mux", "log");
        let mux_log_file = File::create(&mux_log_path).map_err(|e| {
            RecorderError::CaptureRuntime(format!(
                "Nao foi possivel criar o log de mux do FFmpeg: {}",
                e
            ))
        })?;

        let mut cmd = Command::new(ffmpeg_path);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.args(["-hide_banner", "-loglevel", "error", "-i"]);
        cmd.arg(self.video_output_path.to_string_lossy().to_string());

        for track in tracks {
            cmd.args([
                "-f",
                track.sample_format.ffmpeg_name(),
                "-ar",
                &track.sample_rate.to_string(),
                "-ac",
                &track.channels.to_string(),
                "-i",
            ]);
            cmd.arg(track.path.to_string_lossy().to_string());
        }

        match tracks.len() {
            1 => {
                let filter = build_audio_filter(1, &tracks[0], "aout");
                cmd.args([
                    "-filter_complex",
                    &filter,
                    "-map",
                    "0:v:0",
                    "-map",
                    "[aout]",
                    "-c:v",
                    "copy",
                    "-c:a",
                    "aac",
                    "-b:a",
                    "192k",
                    "-shortest",
                    "-movflags",
                    "+faststart",
                    "-y",
                ]);
            }
            2 => {
                let filter = format!(
                    "{};{};[a1][a2]amix=inputs=2:duration=longest:normalize=0[aout]",
                    build_audio_filter(1, &tracks[0], "a1"),
                    build_audio_filter(2, &tracks[1], "a2"),
                );
                cmd.args([
                    "-filter_complex",
                    &filter,
                    "-map",
                    "0:v:0",
                    "-map",
                    "[aout]",
                    "-c:v",
                    "copy",
                    "-c:a",
                    "aac",
                    "-b:a",
                    "192k",
                    "-shortest",
                    "-movflags",
                    "+faststart",
                    "-y",
                ]);
            }
            _ => {
                return Err(RecorderError::CaptureRuntime(
                    "Quantidade de trilhas de audio nativo nao suportada".into(),
                ));
            }
        }

        cmd.arg(self.final_output_path.to_string_lossy().to_string());
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::from(mux_log_file));

        let status = cmd.status().map_err(|e| {
            RecorderError::CaptureRuntime(format!("Falha ao executar o mux final do FFmpeg: {}", e))
        })?;

        if !status.success() {
            let log_tail = read_log_tail(&mux_log_path);
            let _ = fs::remove_file(&mux_log_path);
            return Err(RecorderError::CaptureRuntime(format!(
                "Falha ao combinar o audio capturado com o video. {}",
                if log_tail.is_empty() {
                    "Consulte o log de mux do FFmpeg.".into()
                } else {
                    format!("Detalhes do FFmpeg: {}", log_tail)
                }
            )));
        }

        let _ = fs::remove_file(&mux_log_path);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), RecorderError> {
        if let Some(capture) = &self.mic_capture {
            capture.request_stop();
        }
        if let Some(capture) = &self.system_audio_capture {
            capture.request_stop();
        }

        println!("Enviando sinal de parada (q) pro FFmpeg...");
        if let Some(mut stdin) = self.process.stdin.take() {
            let _ = stdin.write_all(b"q\n");
            let _ = stdin.flush();
        }

        for _ in 0..30 {
            match self.process.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        let log_tail = read_log_tail(&self.log_path);
                        return Err(RecorderError::CaptureRuntime(format!(
                            "FFmpeg encerrou com falha ao finalizar usando {} (codigo {:?}). {}",
                            self.encoder_label,
                            status.code(),
                            if log_tail.is_empty() {
                                format!("Consulte o log em {:?}", self.log_path)
                            } else {
                                format!("Detalhes do FFmpeg: {}", log_tail)
                            }
                        )));
                    }

                    self.validate_output()?;
                    let tracks = self.finalize_audio_tracks()?;

                    if !tracks.is_empty() {
                        self.mux_native_audio(&tracks)?;
                        cleanup_audio_tracks(&tracks);
                        let _ = fs::remove_file(&self.video_output_path);
                    } else if self.video_output_path != self.final_output_path {
                        if self.final_output_path.exists() {
                            let _ = fs::remove_file(&self.final_output_path);
                        }
                        fs::rename(&self.video_output_path, &self.final_output_path).map_err(|e| {
                            RecorderError::CaptureRuntime(format!(
                                "Falha ao mover o video final sem audio para o destino: {}",
                                e
                            ))
                        })?;
                    }

                    let _ = fs::remove_file(&self.log_path);
                    return Ok(());
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(100)),
                Err(e) => {
                    return Err(RecorderError::CaptureRuntime(format!(
                        "Falha ao aguardar o encerramento do FFmpeg com {}: {}",
                        self.encoder_label, e
                    )));
                }
            }
        }

        println!("Aviso: O FFmpeg demorou muito para fechar. Forcando interrupcao...");
        let _ = self.process.kill();
        let _ = self.process.wait();

        let log_tail = read_log_tail(&self.log_path);
        Err(RecorderError::CaptureRuntime(format!(
            "FFmpeg precisou ser encerrado a forca e o MP4 pode ter ficado corrompido. Encoder: {}. {}",
            self.encoder_label,
            if log_tail.is_empty() {
                format!("Consulte o log em {:?}", self.log_path)
            } else {
                format!("Detalhes do FFmpeg: {}", log_tail)
            }
        )))
    }
}

pub fn list_monitors() -> Result<Vec<MonitorInfo>, RecorderError> {
    Ok(vec![
        MonitorInfo {
            index: 0,
            name: "Monitor Principal (1)".to_string(),
        },
        MonitorInfo {
            index: 1,
            name: "Monitor Secundario (2)".to_string(),
        },
    ])
}

pub fn list_mic_devices() -> Result<Vec<MicInfo>, RecorderError> {
    audio::list_microphones().map(|devices| {
        devices
            .into_iter()
            .map(|device| {
                let (id, name, is_default) = map_device_info(device);
                MicInfo {
                    id,
                    name,
                    is_default,
                }
            })
            .collect()
    })
}

pub fn list_audio_outputs() -> Result<Vec<AudioOutputInfo>, RecorderError> {
    audio::list_outputs().map(|devices| {
        devices
            .into_iter()
            .map(|device| {
                let (id, name, is_default) = map_device_info(device);
                AudioOutputInfo {
                    id,
                    name,
                    is_default,
                }
            })
            .collect()
    })
}

pub fn test_environment() -> String {
    let ffmpeg_path = match resolve_ffmpeg_path() {
        Ok(path) => path,
        Err(_) => return EncoderStrategy::SoftwareX264.label().to_string(),
    };

    let strategies = [
        EncoderStrategy::NvidiaNvenc,
        EncoderStrategy::AmdAmf,
        EncoderStrategy::IntelQsv,
    ];

    for strategy in strategies {
        let mut cmd = Command::new(&ffmpeg_path);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.args([
            "-f", "lavfi",
            "-i", "nullsrc=s=128x128:d=0.1",
            "-c:v", strategy.label(),
            "-f", "null",
            "-"
        ]);
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        if let Ok(mut child) = cmd.spawn() {
            if let Ok(status) = child.wait() {
                if status.success() {
                    println!("Hardware test successful for {}", strategy.label());
                    return strategy.label().to_string();
                }
            }
        }
    }
    
    println!("Hardware tests failed, falling back to libx264");
    EncoderStrategy::SoftwareX264.label().to_string()
}
