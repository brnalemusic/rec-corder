use crate::errors::RecorderError;
use crate::services::audio::{
    AudioCaptureMode, AudioTrack, NativeAudioCapture,
};
use super::ffmpeg::{
    append_common_inputs, append_encoder_args, append_webcam_input,
    build_webcam_overlay_filter, build_capture_filter, resolve_ffmpeg_path,
    EncoderStrategy,
};
#[cfg(target_os = "windows")]
use super::ffmpeg::CREATE_NO_WINDOW;
#[cfg(target_os = "windows")]
use super::windows::{
    enumerate_native_monitors, find_fullscreen_window_on_monitor, CaptureGuardWindow,
};
use std::fs::{self, File};
use std::io::Write;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

pub const MIN_VALID_OUTPUT_BYTES: u64 = 4 * 1024;
pub const STOP_POLL_INTERVAL_MS: u64 = 100;
pub const STOP_MAX_WAIT_MS: u64 = 30_000;

/// Configuração de overlay da webcam, vinda da configuração do frontend.
pub struct WebcamOverlayConfig {
    pub device_name: String,
    pub position: String,
    pub size_percent: u32,
}

/// Sessão ativa de captura de tela e áudio.
/// // [IMPORTANTE] Controla o ciclo de vida do processo FFmpeg e os arquivos de áudio locais.
pub struct CaptureSession {
    process: Child,
    final_output_path: PathBuf,
    video_output_path: PathBuf,
    log_path: PathBuf,
    encoder_label: &'static str,
    #[cfg(target_os = "windows")]
    _capture_guard: Option<CaptureGuardWindow>,
    #[cfg(not(target_os = "windows"))]
    _capture_guard: Option<()>,
    mic_capture: Option<NativeAudioCapture>,
    system_audio_capture: Option<NativeAudioCapture>,
}

/// Constrói o caminho para o arquivo de log do FFmpeg.
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

/// Constrói um caminho temporário para arquivos de mídia intermediários.
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

/// Lê as últimas linhas do arquivo de log do FFmpeg para debug.
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

/// Limpa os arquivos parciais de uma tentativa falha de captura.
pub fn cleanup_failed_attempt(output_path: &PathBuf, log_path: &PathBuf) {
    let _ = fs::remove_file(output_path);
    let _ = fs::remove_file(log_path);
}

/// Constrói a string de filtro do FFmpeg para mixagem de áudio.
pub fn build_audio_filter(input_index: usize, track: &AudioTrack, label: &str) -> String {
    let channel_filter = if track.channels <= 1 {
        "pan=stereo|c0=c0|c1=c0"
    } else {
        "pan=stereo|c0=c0|c1=c1"
    };

    format!(
        "[{input_index}:a]aresample=48000,{channel_filter}[{label}]"
    )
}

/// Remove arquivos PCM temporários após o mux final.
pub fn cleanup_audio_tracks(tracks: &[AudioTrack]) {
    for track in tracks {
        let _ = fs::remove_file(&track.path);
    }
}

/// Inicia a captura de áudio nativo (Microfone e Loopback) paralelamente.
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

impl CaptureSession {
    /// Inicia uma nova sessão de captura delegando para a estratégia de encoder escolhida.
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
            webcam_config.as_ref(),
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

    /// Configura e invoca o processo filho do FFmpeg com os argumentos de gravação.
    /// // [IMPORTANTE] A injeção de pipes stdin/stderr é crítica para não travar a aplicação base.
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
        webcam_config: Option<&WebcamOverlayConfig>,
    ) -> Result<Self, RecorderError> {
        let log_file = File::create(&log_path).map_err(|e| {
            RecorderError::CaptureInit(format!(
                "Nao foi possivel criar o log do FFmpeg em {:?}: {}",
                log_path, e
            ))
        })?;

        #[cfg(target_os = "windows")]
        let (capture_guard, fullscreen_window, selected_hmonitor) = match enumerate_native_monitors() {
            Ok(monitors) => {
                let monitor = monitors.into_iter().find(|m| m.index == monitor_index);
                let guard = monitor
                    .as_ref()
                    .map(|m| CaptureGuardWindow::create(m.bounds))
                    .transpose();
                let window = monitor
                    .as_ref()
                    .and_then(|m| find_fullscreen_window_on_monitor(m.bounds));
                let hmonitor = monitor.as_ref().map(|m| m.hmonitor);
                (guard, window, hmonitor)
            }
            Err(err) => {
                println!(
                    "Aviso: falha ao enumerar monitores para o modo fullscreen: {}",
                    err
                );
                (Ok(None), None, None)
            }
        };

        #[cfg(not(target_os = "windows"))]
        let (capture_guard, fullscreen_window, selected_hmonitor): (Result<Option<()>, &'static str>, Option<isize>, Option<isize>) = (Ok(None), None, None);

        let capture_guard = capture_guard.map_err(|err| {
            RecorderError::CaptureInit(format!(
                "Falha ao preparar a compatibilidade com fullscreen para o monitor selecionado: {}",
                err
            ))
        })?;

        let (mic_capture, system_audio_capture) =
            start_audio_captures(&final_output_path, mic_device_id, system_audio_device_id)?;

        let mut cmd = Command::new(ffmpeg_path);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        append_common_inputs(
            &mut cmd,
            selected_hmonitor,
            monitor_index,
            fullscreen_window,
            fps,
        );

        // Adiciona input da webcam se configurado
        if let Some(wc) = webcam_config {
            append_webcam_input(&mut cmd, &wc.device_name);
        }

        // Quando a webcam está ativa, usamos filter_complex para compor overlay.
        if let Some(wc) = webcam_config {
            let pixel_format = match strategy {
                EncoderStrategy::AmdAmf => "nv12",
                _ => "yuv420p",
            };
            let base_vf = build_capture_filter(scale_factor, fps, pixel_format);
            let filter_complex = build_webcam_overlay_filter(&base_vf, &wc.position, wc.size_percent);

            cmd.args(["-filter_complex", &filter_complex, "-map", "[out]"]);

            // Encoder codec e qualidade (sem -vf, que está no filter_complex)
            match strategy {
                EncoderStrategy::AmdAmf => {
                    cmd.args(["-c:v", "h264_amf", "-usage", "lowlatency", "-quality", "speed", "-rc", "cbr", "-b:v", "5M", "-pix_fmt", "nv12"]);
                }
                EncoderStrategy::NvidiaNvenc => {
                    cmd.args(["-c:v", "h264_nvenc", "-preset", "p4", "-tune", "ull", "-rc", "vbr", "-cq", "23", "-pix_fmt", "yuv420p"]);
                }
                EncoderStrategy::IntelQsv => {
                    cmd.args(["-c:v", "h264_qsv", "-preset", "veryfast", "-global_quality", "23", "-pix_fmt", "nv12"]);
                }
                EncoderStrategy::SoftwareX264 => {
                    cmd.args(["-c:v", "libx264", "-preset", "ultrafast", "-crf", "23", "-pix_fmt", "yuv420p"]);
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
            _capture_guard: capture_guard,
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

    /// Verifica se o FFmpeg não fechou prematuramente após o spawn.
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

    /// Valida se o arquivo gerado possui um tamanho mínimo aceitável.
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

    /// Para e mescla as trilhas de áudio nativo geradas (PCM).
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

    /// Realiza o mux (combinação) do vídeo finalizado com as trilhas de áudio gravadas.
    /// // [IMPORTANTE] Este processo é bloqueante e I/O bound.
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
        #[cfg(target_os = "windows")]
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

    /// Envia um sinal para encerramento gracioso (`q`) pro FFmpeg e aguarda a finalização.
    /// // [IMPORTANTE] Aguarda o término de streams de I/O em polling.
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

    /// Mata agressivamente o processo do FFmpeg e de áudio (Force exit).
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
