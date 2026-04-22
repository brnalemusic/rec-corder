use crate::errors::RecorderError;
use serde::Serialize;
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Clone, Copy)]
pub enum AudioCaptureMode {
    Microphone,
    SystemLoopback,
}

impl AudioCaptureMode {
    fn label(self) -> &'static str {
        match self {
            Self::Microphone => "microfone",
            Self::SystemLoopback => "audio do sistema",
        }
    }
}

#[derive(Clone)]
pub enum AudioSampleFormat {
    I16,
    I24,
    I32,
    F32,
    F64,
}

impl AudioSampleFormat {
    pub fn ffmpeg_name(&self) -> &'static str {
        match self {
            Self::I16 => "s16le",
            Self::I24 => "s24le",
            Self::I32 => "s32le",
            Self::F32 => "f32le",
            Self::F64 => "f64le",
        }
    }

    fn bytes_per_sample(&self) -> usize {
        match self {
            Self::I16 => 2,
            Self::I24 => 3,
            Self::I32 | Self::F32 => 4,
            Self::F64 => 8,
        }
    }
}

#[derive(Clone)]
pub struct AudioTrack {
    pub path: PathBuf,
    pub sample_format: AudioSampleFormat,
    pub sample_rate: u32,
    pub channels: u16,
}

pub struct NativeAudioCapture {
    stop_flag: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<Result<AudioTrack, RecorderError>>>,
    temp_path: PathBuf,
}

impl NativeAudioCapture {
    pub fn start(
        device_id: String,
        mode: AudioCaptureMode,
        temp_path: PathBuf,
    ) -> Result<Self, RecorderError> {
        if let Some(parent) = temp_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let stop_flag = Arc::new(AtomicBool::new(false));
        let worker_flag = Arc::clone(&stop_flag);
        let worker_path = temp_path.clone();
        let (init_tx, init_rx) = mpsc::sync_channel(1);

        let join_handle = thread::spawn(move || {
            capture_audio_thread(device_id, mode, worker_path, worker_flag, init_tx)
        });

        match init_rx.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(_)) => Ok(Self {
                stop_flag,
                join_handle: Some(join_handle),
                temp_path,
            }),
            Ok(Err(message)) => {
                stop_flag.store(true, Ordering::Relaxed);
                let _ = join_handle.join();
                let _ = fs::remove_file(&temp_path);
                Err(RecorderError::AudioInit(message))
            }
            Err(_) => {
                stop_flag.store(true, Ordering::Relaxed);
                let _ = join_handle.join();
                let _ = fs::remove_file(&temp_path);
                Err(RecorderError::AudioInit(format!(
                    "A captura de {} demorou demais para inicializar",
                    mode.label()
                )))
            }
        }
    }

    pub fn request_stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    pub fn finish(mut self) -> Result<AudioTrack, RecorderError> {
        self.stop_flag.store(true, Ordering::Relaxed);

        let Some(join_handle) = self.join_handle.take() else {
            return Err(RecorderError::AudioRuntime(
                "Thread de captura de audio indisponivel".into(),
            ));
        };

        join_handle
            .join()
            .map_err(|_| RecorderError::AudioRuntime("Thread de captura de audio falhou".into()))?
    }

    pub fn abort(mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
        let _ = fs::remove_file(&self.temp_path);
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use std::io::Write;
    use std::slice;
    use windows::core::{PCWSTR, PROPVARIANT};
    use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
    use windows::Win32::Media::Audio::{
        eCapture, eMultimedia, eRender, IAudioCaptureClient, IAudioClient, IMMDevice,
        IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
        AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK,
        WAVEFORMATEX, WAVEFORMATEXTENSIBLE, WAVE_FORMAT_PCM,
    };
    use windows::Win32::Media::KernelStreaming::{
        KSDATAFORMAT_SUBTYPE_PCM, WAVE_FORMAT_EXTENSIBLE,
    };
    use windows::Win32::Media::Multimedia::{KSDATAFORMAT_SUBTYPE_IEEE_FLOAT, WAVE_FORMAT_IEEE_FLOAT};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
        COINIT_MULTITHREADED, STGM_READ,
    };
    use windows::Win32::System::Com::StructuredStorage::{
        PropVariantClear, PropVariantToStringAlloc,
    };
    use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, PROPERTYKEY};

    struct ComGuard;

    impl ComGuard {
        fn init() -> Result<Self, RecorderError> {
            unsafe {
                if let Err(e) = CoInitializeEx(None, COINIT_MULTITHREADED).ok() {
                    if e.code().0 != -2147417850 { // RPC_E_CHANGED_MODE
                        return Err(map_win_err(e));
                    }
                }
            }
            Ok(Self)
        }
    }

    impl Drop for ComGuard {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }

    fn map_win_err(error: windows::core::Error) -> RecorderError {
        RecorderError::AudioRuntime(error.message().to_string())
    }

    fn get_property_string(store: &IPropertyStore, key: &PROPERTYKEY) -> Result<String, RecorderError> {
        unsafe {
            let mut value: PROPVARIANT = store.GetValue(key).map_err(map_win_err)?;
            let buffer = PropVariantToStringAlloc(&value).map_err(map_win_err)?;
            let _ = PropVariantClear(&mut value);

            let text = buffer
                .to_string()
                .map_err(|e| RecorderError::AudioRuntime(e.to_string()))?;
            CoTaskMemFree(Some(buffer.0 as _));
            Ok(text)
        }
    }

    fn get_device_id(device: &IMMDevice) -> Result<String, RecorderError> {
        unsafe {
            let id = device.GetId().map_err(map_win_err)?;
            let text = id
                .to_string()
                .map_err(|e| RecorderError::AudioRuntime(e.to_string()))?;
            CoTaskMemFree(Some(id.0 as _));
            Ok(text)
        }
    }

    fn get_device_name(device: &IMMDevice) -> Result<String, RecorderError> {
        unsafe {
            let store = device.OpenPropertyStore(STGM_READ).map_err(map_win_err)?;
            match get_property_string(&store, &PKEY_Device_FriendlyName) {
                Ok(name) => Ok(name),
                Err(_) => Ok("Dispositivo Desconhecido".to_string()),
            }
        }
    }

    fn enumerate_devices(flow: windows::Win32::Media::Audio::EDataFlow) -> Result<Vec<AudioDeviceInfo>, RecorderError> {
        let _com = ComGuard::init()?;
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| {
                    println!("Erro ao criar enumerator: {}", e.message());
                    map_win_err(e)
                })?;

            let default_id = match enumerator.GetDefaultAudioEndpoint(flow, eMultimedia) {
                Ok(default_device) => get_device_id(&default_device).unwrap_or_default(),
                Err(_) => String::new(),
            };

            let collection = enumerator
                .EnumAudioEndpoints(flow, DEVICE_STATE_ACTIVE)
                .map_err(|e| {
                    println!("Erro ao enumerar associacoes state=active: {}", e.message());
                    map_win_err(e)
                })?;

            let count = collection.GetCount().unwrap_or(0);
            let mut devices = Vec::with_capacity(count as usize);

            for index in 0..count {
                if let Ok(device) = collection.Item(index) {
                    if let Ok(id) = get_device_id(&device) {
                        let name = get_device_name(&device).unwrap_or_else(|_| "Dispositivo Desconhecido".to_string());
                        devices.push(AudioDeviceInfo {
                            is_default: !id.is_empty() && id == default_id,
                            id,
                            name,
                        });
                    }
                }
            }

            Ok(devices)
        }
    }

    pub fn list_microphones() -> Result<Vec<AudioDeviceInfo>, RecorderError> {
        enumerate_devices(eCapture)
    }

    pub fn list_outputs() -> Result<Vec<AudioDeviceInfo>, RecorderError> {
        enumerate_devices(eRender)
    }

    fn parse_wave_format(format_ptr: *mut WAVEFORMATEX) -> Result<(AudioSampleFormat, u32, u16), RecorderError> {
        if format_ptr.is_null() {
            return Err(RecorderError::AudioInit(
                "Formato de audio do Windows nao foi informado".into(),
            ));
        }

        unsafe {
            let format = *format_ptr;
            let channels = format.nChannels;
            let sample_rate = format.nSamplesPerSec;

            let sample_format = match format.wFormatTag as u32 {
                WAVE_FORMAT_PCM => match format.wBitsPerSample {
                    16 => AudioSampleFormat::I16,
                    24 => AudioSampleFormat::I24,
                    32 => AudioSampleFormat::I32,
                    bits => {
                        return Err(RecorderError::AudioInit(format!(
                            "Formato PCM de {} bits nao suportado",
                            bits
                        )))
                    }
                },
                WAVE_FORMAT_IEEE_FLOAT => match format.wBitsPerSample {
                    32 => AudioSampleFormat::F32,
                    64 => AudioSampleFormat::F64,
                    bits => {
                        return Err(RecorderError::AudioInit(format!(
                            "Formato float de {} bits nao suportado",
                            bits
                        )))
                    }
                },
                WAVE_FORMAT_EXTENSIBLE => {
                    let ext = *(format_ptr as *const WAVEFORMATEXTENSIBLE);
                    let sub_format = ext.SubFormat;

                    if sub_format == KSDATAFORMAT_SUBTYPE_PCM {
                        match format.wBitsPerSample {
                            16 => AudioSampleFormat::I16,
                            24 => AudioSampleFormat::I24,
                            32 => AudioSampleFormat::I32,
                            bits => {
                                return Err(RecorderError::AudioInit(format!(
                                    "Formato PCM extensivel de {} bits nao suportado",
                                    bits
                                )))
                            }
                        }
                    } else if sub_format == KSDATAFORMAT_SUBTYPE_IEEE_FLOAT {
                        match format.wBitsPerSample {
                            32 => AudioSampleFormat::F32,
                            64 => AudioSampleFormat::F64,
                            bits => {
                                return Err(RecorderError::AudioInit(format!(
                                    "Formato float extensivel de {} bits nao suportado",
                                    bits
                                )))
                            }
                        }
                    } else {
                        return Err(RecorderError::AudioInit(
                            "Subformato de audio extensivel nao suportado".into(),
                        ));
                    }
                }
                other => {
                    return Err(RecorderError::AudioInit(format!(
                        "Formato de audio do Windows nao suportado: {}",
                        other
                    )))
                }
            };

            Ok((sample_format, sample_rate, channels))
        }
    }

    fn open_selected_device(
        enumerator: &IMMDeviceEnumerator,
        device_id: &str,
    ) -> Result<IMMDevice, RecorderError> {
        let wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe { enumerator.GetDevice(PCWSTR(wide.as_ptr())).map_err(map_win_err) }
    }

    pub fn capture_audio_thread(
        device_id: String,
        mode: AudioCaptureMode,
        output_path: PathBuf,
        stop_flag: Arc<AtomicBool>,
        init_tx: mpsc::SyncSender<Result<AudioTrack, String>>,
    ) -> Result<AudioTrack, RecorderError> {
        let _com = ComGuard::init()?;

        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(map_win_err)?;
            let device = open_selected_device(&enumerator, &device_id)?;
            let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None).map_err(map_win_err)?;

            let format_ptr = audio_client.GetMixFormat().map_err(map_win_err)?;
            let (sample_format, sample_rate, channels) = parse_wave_format(format_ptr)?;
            let block_align = channels as usize * sample_format.bytes_per_sample();

            let stream_flags = match mode {
                AudioCaptureMode::Microphone => 0,
                AudioCaptureMode::SystemLoopback => AUDCLNT_STREAMFLAGS_LOOPBACK,
            };

            audio_client
                .Initialize(
                    AUDCLNT_SHAREMODE_SHARED,
                    stream_flags,
                    0,
                    0,
                    format_ptr,
                    None,
                )
                .map_err(map_win_err)?;

            CoTaskMemFree(Some(format_ptr as _));

            let capture_client: IAudioCaptureClient = audio_client.GetService().map_err(map_win_err)?;
            let mut file = File::create(&output_path)?;

            let descriptor = AudioTrack {
                path: output_path.clone(),
                sample_format: sample_format.clone(),
                sample_rate,
                channels,
            };

            let _ = init_tx.send(Ok(descriptor.clone()));

            audio_client.Start().map_err(map_win_err)?;

            loop {
                let mut packet_size = capture_client.GetNextPacketSize().map_err(map_win_err)?;

                while packet_size > 0 {
                    let mut data_ptr = std::ptr::null_mut();
                    let mut frame_count = 0u32;
                    let mut flags = 0u32;

                    capture_client
                        .GetBuffer(
                            &mut data_ptr,
                            &mut frame_count,
                            &mut flags,
                            None,
                            None,
                        )
                        .map_err(map_win_err)?;

                    let bytes_to_write = frame_count as usize * block_align;

                    if flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 {
                        file.write_all(&vec![0u8; bytes_to_write])?;
                    } else if bytes_to_write > 0 {
                        let bytes = slice::from_raw_parts(data_ptr as *const u8, bytes_to_write);
                        file.write_all(bytes)?;
                    }

                    capture_client.ReleaseBuffer(frame_count).map_err(map_win_err)?;
                    packet_size = capture_client.GetNextPacketSize().map_err(map_win_err)?;
                }

                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }

                thread::sleep(Duration::from_millis(8));
            }

            audio_client.Stop().map_err(map_win_err)?;
            file.flush()?;

            Ok(descriptor)
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::*;
    use std::process::{Command, Stdio};
    use crate::services::capture::ffmpeg::resolve_ffmpeg_path;

    pub fn list_microphones() -> Result<Vec<AudioDeviceInfo>, RecorderError> {
        let wpctl_devices = crate::services::capture::linux::list_wpctl_audio_devices(true);
        if !wpctl_devices.is_empty() {
            return Ok(wpctl_devices.into_iter().map(|d| AudioDeviceInfo {
                id: d.id,
                name: d.name,
                is_default: d.is_default,
            }).collect());
        }

        // Fallback
        Ok(vec![AudioDeviceInfo {
            id: "default".to_string(),
            name: "Microfone Padrão (PulseAudio)".to_string(),
            is_default: true,
        }])
    }

    pub fn list_outputs() -> Result<Vec<AudioDeviceInfo>, RecorderError> {
        let wpctl_devices = crate::services::capture::linux::list_wpctl_audio_devices(false);
        if !wpctl_devices.is_empty() {
            return Ok(wpctl_devices.into_iter().map(|d| AudioDeviceInfo {
                id: d.id,
                name: d.name,
                is_default: d.is_default,
            }).collect());
        }

        // Fallback
        Ok(vec![AudioDeviceInfo {
            id: "default".to_string(),
            name: "Áudio do Sistema Padrão (PulseAudio)".to_string(),
            is_default: true,
        }])
    }

    pub fn capture_audio_thread(
        device_id: String,
        mode: AudioCaptureMode,
        output_path: PathBuf,
        stop_flag: Arc<AtomicBool>,
        init_tx: mpsc::SyncSender<Result<AudioTrack, String>>,
    ) -> Result<AudioTrack, RecorderError> {
        let ffmpeg_path = match resolve_ffmpeg_path() {
            Ok(p) => p,
            Err(e) => {
                let _ = init_tx.send(Err(format!("FFmpeg nao encontrado: {}", e)));
                return Err(RecorderError::AudioInit("FFmpeg nao encontrado".into()));
            }
        };

        let mut pulse_input = device_id.clone();
        if pulse_input == "default" || pulse_input.is_empty() {
            match mode {
                AudioCaptureMode::Microphone => pulse_input = "default".into(),
                AudioCaptureMode::SystemLoopback => {
                    // Tenta descobrir o monitor do sink padrao via pactl
                    if let Ok(output) = Command::new("sh").args(["-c", "pactl info | awk '/Default Sink:/ {print $3\".monitor\"}'"]).output() {
                        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !stdout.is_empty() {
                            pulse_input = stdout;
                        } else {
                            pulse_input = "default".into();
                        }
                    } else {
                        pulse_input = "default".into();
                    }
                }
            }
        }

        let mut cmd = Command::new(ffmpeg_path);
        cmd.args([
            "-f", "pulse",
            "-i", &pulse_input,
            "-f", "s16le",
            "-acodec", "pcm_s16le",
            "-ar", "48000",
            "-ac", "2",
            "-y",
        ]);
        cmd.arg(&output_path);
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = init_tx.send(Err(format!("Falha ao iniciar FFmpeg pulse: {}", e)));
                return Err(RecorderError::AudioInit("Falha ao iniciar ffmpeg".into()));
            }
        };

        let track = AudioTrack {
            path: output_path.clone(),
            sample_format: AudioSampleFormat::I16,
            sample_rate: 48000,
            channels: 2,
        };

        let _ = init_tx.send(Ok(track.clone()));

        loop {
            if stop_flag.load(Ordering::Relaxed) {
                // Envia q pra fechar graciosamente o pcm
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(b"q\n");
                }
                thread::sleep(Duration::from_millis(100));
                let _ = child.kill();
                let _ = child.wait();
                break;
            }
            if let Ok(Some(_)) = child.try_wait() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        Ok(track)
    }
}

pub fn list_microphones() -> Result<Vec<AudioDeviceInfo>, RecorderError> {
    platform::list_microphones()
}

pub fn list_outputs() -> Result<Vec<AudioDeviceInfo>, RecorderError> {
    platform::list_outputs()
}

fn capture_audio_thread(
    device_id: String,
    mode: AudioCaptureMode,
    output_path: PathBuf,
    stop_flag: Arc<AtomicBool>,
    init_tx: mpsc::SyncSender<Result<AudioTrack, String>>,
) -> Result<AudioTrack, RecorderError> {
    platform::capture_audio_thread(device_id, mode, output_path, stop_flag, init_tx)
}
