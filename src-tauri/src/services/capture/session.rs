use super::ffmpeg::{
    append_encoder_args, append_video_input, append_webcam_input, build_capture_filter,
    build_ddagrab_input, build_gdigrab_input, build_gfxcapture_input, build_webcam_overlay_filter,
    resolve_ffmpeg_path, CaptureInput, EncoderStrategy, CREATE_NO_WINDOW,
};
#[cfg(target_os = "windows")]
use super::windows::enumerate_native_monitors;
use crate::errors::RecorderError;
use crate::services::audio::{AudioCaptureMode, AudioTrack, NativeAudioCapture};
use std::fs::{self, File};
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

pub const MIN_VALID_OUTPUT_BYTES: u64 = 4 * 1024;
pub const STOP_POLL_INTERVAL_MS: u64 = 100;
pub const STOP_MAX_WAIT_MS: u64 = 30_000;

/// Configuration for the webcam overlay, passed from the frontend config.
pub struct WebcamOverlayConfig {
    pub device_name: String,
    pub position: String,
    pub size_percent: u32,
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

pub fn build_log_path(output_path: &PathBuf) -> PathBuf {
    let mut log_dir = std::env::temp_dir();
    log_dir.push("RecCorderLogs");

    let stem = output_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("recording");

    log_dir.join(format!("{stem}.ffmpeg.log"))
}

pub fn build_temp_media_path(output_path: &PathBuf, suffix: &str, extension: &str) -> PathBuf {
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

pub fn read_log_tail(log_path: &PathBuf) -> String {
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

pub fn cleanup_failed_attempt(output_path: &PathBuf, log_path: &PathBuf) {
    let _ = fs::remove_file(output_path);
    let _ = fs::remove_file(log_path);
}

pub fn build_audio_filter(input_index: usize, track: &AudioTrack, label: &str) -> String {
    let channel_filter = if track.channels <= 1 {
        "pan=stereo|c0=c0|c1=c0"
    } else {
        "pan=stereo|c0=c0|c1=c1"
    };

    format!("[{input_index}:a]aresample=48000,{channel_filter}[{label}]")
}

pub fn cleanup_audio_tracks(tracks: &[AudioTrack]) {
    for track in tracks {
        let _ = fs::remove_file(&track.path);
    }
}

pub fn start_audio_captures(
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

#[cfg(target_os = "windows")]
fn build_capture_inputs(
    monitor_index: usize,
    fps: u32,
) -> Result<Vec<CaptureInput>, RecorderError> {
    let monitors = enumerate_native_monitors()?;
    let monitor = monitors
        .into_iter()
        .find(|monitor| monitor.index == monitor_index)
        .ok_or_else(|| {
            RecorderError::CaptureInit(format!(
                "O monitor selecionado ({monitor_index}) nao esta mais disponivel"
            ))
        })?;

    let width = (monitor.bounds.right - monitor.bounds.left).max(1);
    let height = (monitor.bounds.bottom - monitor.bounds.top).max(1);
    let mut inputs = Vec::new();

    inputs.push(build_gfxcapture_input(
        Some(monitor.hmonitor),
        monitor.index,
        None,
        fps,
    ));
    inputs.push(build_gfxcapture_input(None, monitor.index, None, fps));

    if let Some(output_idx) = monitor.dxgi_output_index {
        inputs.push(build_ddagrab_input(output_idx, fps));
    }

    inputs.push(build_gdigrab_input(
        monitor.bounds.left,
        monitor.bounds.top,
        width,
        height,
        fps,
    ));

    Ok(inputs)
}

#[cfg(not(target_os = "windows"))]
fn build_capture_inputs(
    monitor_index: usize,
    fps: u32,
) -> Result<Vec<CaptureInput>, RecorderError> {
    Ok(vec![build_gfxcapture_input(None, monitor_index, None, fps)])
}

impl CaptureSession {
    fn build_runtime_exit_message(&self, status_code: Option<i32>, context: &str) -> String {
        let log_tail = read_log_tail(&self.log_path);
        let partial_size = fs::metadata(&self.video_output_path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);

        format!(
            "FFmpeg encerrou inesperadamente {} usando {} (codigo {:?}). Arquivo parcial: {} bytes. {}",
            context,
            self.encoder_label,
            status_code,
            partial_size,
            if log_tail.is_empty() {
                format!("Consulte o log em {:?}", self.log_path)
            } else {
                format!("Detalhes do FFmpeg: {}", log_tail)
            }
        )
    }

    pub fn start(
        output_path: PathBuf,
        monitor_index: usize,
        mic_device_id: Option<String>,
        system_audio_device_id: Option<String>,
        fps: u32,
        scale_factor: u32,
        strategy_label: &str,
        webcam_config: Option<WebcamOverlayConfig>,
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
        let capture_inputs = build_capture_inputs(monitor_index, fps)?;
        let mut attempt_errors = Vec::new();

        for capture_input in capture_inputs {
            match Self::start_with_strategy(
                &ffmpeg_path,
                output_path.clone(),
                video_output_path.clone(),
                log_path.clone(),
                mic_device_id.as_ref(),
                system_audio_device_id.as_ref(),
                fps,
                scale_factor,
                strategy,
                &capture_input,
                webcam_config.as_ref(),
            ) {
                Ok(session) => return Ok(session),
                Err(err) => {
                    println!(
                        "Falha ao iniciar backend {} com encoder {}: {}",
                        capture_input.backend.label(),
                        strategy.label(),
                        err
                    );
                    cleanup_failed_attempt(&video_output_path, &log_path);
                    let _ = fs::remove_file(build_temp_media_path(&output_path, "mic", "pcm"));
                    let _ = fs::remove_file(build_temp_media_path(&output_path, "system", "pcm"));
                    attempt_errors.push(format!("{}: {}", capture_input.backend.label(), err));
                }
            }
        }

        Err(RecorderError::CaptureInit(format!(
            "Nenhum backend de captura conseguiu iniciar com {}. {}",
            strategy.label(),
            attempt_errors.join(" | ")
        )))
    }

    fn start_with_strategy(
        ffmpeg_path: &PathBuf,
        final_output_path: PathBuf,
        video_output_path: PathBuf,
        log_path: PathBuf,
        mic_device_id: Option<&String>,
        system_audio_device_id: Option<&String>,
        fps: u32,
        scale_factor: u32,
        strategy: EncoderStrategy,
        capture_input: &CaptureInput,
        webcam_config: Option<&WebcamOverlayConfig>,
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

        append_video_input(&mut cmd, capture_input);

        // Add webcam input if configured (must come after screen input, before encoder args)
        if let Some(wc) = webcam_config {
            append_webcam_input(&mut cmd, &wc.device_name);
        }

        // When webcam is active, we use -filter_complex instead of -vf
        // to compose the overlay. Otherwise, use the standard encoder args.
        if let Some(wc) = webcam_config {
            let pixel_format = match strategy {
                EncoderStrategy::AmdAmf => "nv12",
                _ => "yuv420p",
            };
            let base_vf =
                build_capture_filter(scale_factor, fps, pixel_format, capture_input.backend);
            let filter_complex =
                build_webcam_overlay_filter(&base_vf, &wc.position, wc.size_percent);

            cmd.args(["-filter_complex", &filter_complex, "-map", "[out]"]);

            // Encoder codec and quality settings (without -vf, which is in filter_complex)
            match strategy {
                EncoderStrategy::AmdAmf => {
                    cmd.args([
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
                    ]);
                }
                EncoderStrategy::NvidiaNvenc => {
                    cmd.args([
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
                    ]);
                }
                EncoderStrategy::IntelQsv => {
                    cmd.args([
                        "-c:v",
                        "h264_qsv",
                        "-preset",
                        "veryfast",
                        "-global_quality",
                        "23",
                        "-pix_fmt",
                        "nv12",
                    ]);
                }
                EncoderStrategy::SoftwareX264 => {
                    cmd.args([
                        "-c:v",
                        "libx264",
                        "-preset",
                        "ultrafast",
                        "-crf",
                        "23",
                        "-pix_fmt",
                        "yuv420p",
                    ]);
                }
            }

            if video_output_path == final_output_path {
                cmd.args(["-movflags", "+faststart"]);
            }
        } else {
            append_encoder_args(
                &mut cmd,
                strategy,
                fps,
                scale_factor,
                capture_input.backend,
                video_output_path == final_output_path,
            );
            cmd.args(["-map", "0:v:0"]);
        }

        cmd.arg("-y");
        cmd.arg(video_output_path.to_string_lossy().to_string());
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::from(log_file));

        println!(
            "Spawning FFmpeg em {:?} com backend {} e encoder {}...",
            ffmpeg_path,
            capture_input.backend.label(),
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
            return Err(RecorderError::CaptureInit(self.build_runtime_exit_message(
                status.code(),
                "logo apos iniciar",
            )));
        }

        Ok(())
    }

    pub fn poll_runtime_error(&mut self) -> Result<Option<String>, RecorderError> {
        match self.process.try_wait() {
            Ok(Some(status)) => Ok(Some(self.build_runtime_exit_message(
                status.code(),
                "durante a gravacao",
            ))),
            Ok(None) => Ok(None),
            Err(err) => Err(RecorderError::CaptureRuntime(format!(
                "Falha ao verificar o processo de captura com {}: {}",
                self.encoder_label, err
            ))),
        }
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
            if fs::metadata(&track.path)
                .map(|meta| meta.len())
                .unwrap_or(0)
                > 0
            {
                tracks.push(track);
            } else {
                let _ = fs::remove_file(&track.path);
            }
        }

        if let Some(capture) = self.system_audio_capture.take() {
            let track = capture.finish()?;
            if fs::metadata(&track.path)
                .map(|meta| meta.len())
                .unwrap_or(0)
                > 0
            {
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

    fn complete_recording_files(&mut self) -> Result<(), RecorderError> {
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

        let attempts = (STOP_MAX_WAIT_MS / STOP_POLL_INTERVAL_MS) as usize;

        for _ in 0..attempts {
            match self.process.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        self.complete_recording_files()?;
                        return Err(RecorderError::CaptureRuntime(format!(
                            "A captura foi interrompida inesperadamente, mas um arquivo parcial foi salvo. {}",
                            self.build_runtime_exit_message(status.code(), "ao finalizar")
                        )));
                    }

                    self.complete_recording_files()?;
                    return Ok(());
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(STOP_POLL_INTERVAL_MS)),
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
            "FFmpeg precisou ser encerrado a forca apos {} ms e o MP4 pode ter ficado corrompido. Encoder: {}. {}",
            STOP_MAX_WAIT_MS,
            self.encoder_label,
            if log_tail.is_empty() {
                format!("Consulte o log em {:?}", self.log_path)
            } else {
                format!("Detalhes do FFmpeg: {}", log_tail)
            }
        )))
    }

    pub fn kill(&mut self) {
        if let Some(capture) = &self.mic_capture {
            capture.request_stop();
        }
        if let Some(capture) = &self.system_audio_capture {
            capture.request_stop();
        }
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}
