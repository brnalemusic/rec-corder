use crate::errors::RecorderError;
use crate::services::audio::{self, AudioDeviceInfo};
use super::CameraInfo;
use serde::Serialize;
use std::ffi::c_void;

#[cfg(target_os = "windows")]
use std::sync::mpsc;
#[cfg(target_os = "windows")]
use std::thread::{self, JoinHandle};
#[cfg(target_os = "windows")]
use std::process::{Command, Stdio};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
use windows::core::PCWSTR;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{BOOL, COLORREF, HWND, LPARAM, RECT};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{
    CreateRectRgn, EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR,
    MONITORINFOEXW, MonitorFromWindow, SetWindowRgn, DISPLAY_DEVICEW,
    MONITOR_DEFAULTTONEAREST,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::GetCurrentThreadId;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DispatchMessageW, GetMessageW, MSG, PostMessageW, PostThreadMessageW,
    SetLayeredWindowAttributes, SetWindowPos, TranslateMessage, HWND_TOPMOST, LWA_ALPHA,
    SWP_NOACTIVATE, SWP_SHOWWINDOW, WINDOW_EX_STYLE, WM_CLOSE, WM_QUIT, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    GetForegroundWindow, GetWindowRect, IsWindowVisible,
};
#[cfg(target_os = "windows")]
use super::ffmpeg::{resolve_ffmpeg_path, CREATE_NO_WINDOW};

/// Informações sobre um monitor, prontas para serialização no Tauri.
#[derive(Serialize)]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub is_primary: bool,
}

/// Informações sobre um dispositivo de microfone.
#[derive(Serialize)]
pub struct MicInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

/// Informações sobre um dispositivo de saída de áudio (sistema).
#[derive(Serialize)]
pub struct AudioOutputInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[cfg(target_os = "windows")]
pub const OVERLAY_ALPHA: u8 = 1;
#[cfg(target_os = "windows")]
pub const MONITORINFO_PRIMARY_FLAG: u32 = 1;

/// Estrutura nativa com dados detalhados do monitor no Windows.
#[cfg(target_os = "windows")]
#[derive(Clone, Debug)]
pub struct NativeMonitorInfo {
    pub index: usize,
    pub hmonitor: isize,
    pub name: String,
    pub bounds: RECT,
    pub is_primary: bool,
}

/// Janela invisível utilizada para habilitar captura de aplicativos em tela cheia no Windows.
#[cfg(target_os = "windows")]
pub struct CaptureGuardWindow {
    pub hwnd: isize,
    pub thread_id: u32,
    pub join_handle: Option<JoinHandle<()>>,
}

/// Dummy strutura para compatibilidade multiplataforma.
#[cfg(not(target_os = "windows"))]
pub struct CaptureGuardWindow {}

#[cfg(target_os = "windows")]
impl Drop for CaptureGuardWindow {
    /// Encerra as threads e fecha a janela invisível com segurança ao ser destruída.
    fn drop(&mut self) {
        unsafe {
            let _ = PostMessageW(HWND(self.hwnd as *mut c_void), WM_CLOSE, None, None);
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, None, None);
        }

        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

#[cfg(target_os = "windows")]
impl CaptureGuardWindow {
    /// Cria uma janela sobreposta invisível cobrindo o monitor alvo,
    /// enganando o Windows para desativar otimizações de tela cheia que impedem o Desktop Duplication.
    pub fn create(bounds: RECT) -> Result<Self, RecorderError> {
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let join_handle = thread::spawn(move || {
            let class_name: Vec<u16> = "STATIC\0".encode_utf16().collect();
            let width = (bounds.right - bounds.left).max(1);
            let height = (bounds.bottom - bounds.top).max(1);

            let result = unsafe {
                let hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE(
                        WS_EX_LAYERED.0
                            | WS_EX_TOPMOST.0
                            | WS_EX_TOOLWINDOW.0
                            | WS_EX_NOACTIVATE.0,
                    ),
                    PCWSTR(class_name.as_ptr()),
                    PCWSTR::null(),
                    WS_POPUP,
                    bounds.left,
                    bounds.top,
                    width,
                    height,
                    None,
                    None,
                    None,
                    None,
                );

                match hwnd {
                    Ok(hwnd) => {
                        let _ = SetLayeredWindowAttributes(
                            hwnd,
                            COLORREF(0),
                            OVERLAY_ALPHA,
                            LWA_ALPHA,
                        );
                        let _ = SetWindowPos(
                            hwnd,
                            HWND_TOPMOST,
                            bounds.left,
                            bounds.top,
                            width,
                            height,
                            SWP_NOACTIVATE | SWP_SHOWWINDOW,
                        );

                        let region = CreateRectRgn(0, 0, 1, 1);
                        let _ = SetWindowRgn(hwnd, region, true);

                        Ok((hwnd.0 as isize, GetCurrentThreadId()))
                    }
                    Err(_) => Err(()),
                }
            };

            if ready_tx.send(result).is_err() {
                return;
            }

            let mut msg = MSG::default();
            loop {
                let status = unsafe { GetMessageW(&mut msg, None, 0, 0) };
                if status.0 == -1 || status.0 == 0 {
                    break;
                }

                unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        });

        let (hwnd, thread_id) = ready_rx
            .recv()
            .map_err(|_| {
                RecorderError::CaptureInit(
                    "Falha ao iniciar a janela de compatibilidade para captura fullscreen".into(),
                )
            })?
            .map_err(|_| {
                RecorderError::CaptureInit(
                    "Nao foi possivel criar a janela de compatibilidade para captura fullscreen"
                        .into(),
                )
            })?;

        Ok(Self {
            hwnd,
            thread_id,
            join_handle: Some(join_handle),
        })
    }
}

/// Identifica se há uma janela em tela cheia no monitor alvo para usar modos de captura alternativos se necessário.
#[cfg(target_os = "windows")]
pub fn find_fullscreen_window_on_monitor(monitor_bounds: RECT) -> Option<isize> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() || !IsWindowVisible(hwnd).as_bool() {
            return None;
        }

        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return None;
        }

        let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if !GetMonitorInfoW(hmonitor, &mut info as *mut _ as *mut _).as_bool() {
            return None;
        }

        let m_bounds = info.monitorInfo.rcMonitor;
        if m_bounds.left != monitor_bounds.left || m_bounds.top != monitor_bounds.top {
            return None;
        }

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        let m_width = monitor_bounds.right - monitor_bounds.left;
        let m_height = monitor_bounds.bottom - monitor_bounds.top;

        if width >= (m_width * 9 / 10) && height >= (m_height * 9 / 10) {
            println!("Direct window capture enabled for potential fullscreen app (HWND: {:?})", hwnd.0);
            return Some(hwnd.0 as isize);
        }
    }
    None
}

/// Helper nativo para obter a string de dispositivo das APIs do Windows.
#[cfg(target_os = "windows")]
pub fn parse_display_device_string(device: &[u16]) -> String {
    let len = device.iter().position(|&value| value == 0).unwrap_or(device.len());
    String::from_utf16_lossy(&device[..len]).trim().to_string()
}

/// Tenta converter o nome técnico do adaptador do Windows em um nome amigável para exibição na UI.
#[cfg(target_os = "windows")]
pub fn resolve_monitor_friendly_name(adapter_name: &str) -> Option<String> {
    let adapter_name_wide: Vec<u16> = adapter_name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut device_index = 0;

    loop {
        let mut display_device = DISPLAY_DEVICEW::default();
        display_device.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;

        let result = unsafe {
            EnumDisplayDevicesW(
                PCWSTR(adapter_name_wide.as_ptr()),
                device_index,
                &mut display_device,
                0,
            )
        };

        if !result.as_bool() {
            break;
        }

        let monitor_name = parse_display_device_string(&display_device.DeviceString);
        if !monitor_name.is_empty()
            && !monitor_name.eq_ignore_ascii_case("Generic PnP Monitor")
        {
            return Some(monitor_name);
        }

        if !monitor_name.is_empty() {
            return Some(monitor_name);
        }

        device_index += 1;
    }

    None
}

/// Callback nativa (C-style) usada pelo `EnumDisplayMonitors` do Windows.
#[cfg(target_os = "windows")]
pub unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitors = &mut *(lparam.0 as *mut Vec<NativeMonitorInfo>);

    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    if GetMonitorInfoW(hmonitor, &mut info as *mut _ as *mut _).as_bool() {
        let device_name = parse_display_device_string(&info.szDevice);
        let friendly_name = resolve_monitor_friendly_name(&device_name)
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| device_name.clone());
        let bounds = info.monitorInfo.rcMonitor;
        let width = bounds.right - bounds.left;
        let height = bounds.bottom - bounds.top;
        let is_primary = (info.monitorInfo.dwFlags & MONITORINFO_PRIMARY_FLAG) != 0;
        let label = if is_primary {
            format!("{friendly_name} (Principal) - {width}x{height}")
        } else {
            format!("{friendly_name} - {width}x{height}")
        };

        monitors.push(NativeMonitorInfo {
            index: monitors.len(),
            hmonitor: hmonitor.0 as isize,
            name: label,
            bounds,
            is_primary,
        });
    }

    BOOL(1)
}

/// Varre o hardware do sistema em busca de monitores e suas posições para captura.
#[cfg(target_os = "windows")]
pub fn enumerate_native_monitors() -> Result<Vec<NativeMonitorInfo>, RecorderError> {
    let mut monitors = Vec::new();

    unsafe {
        if !EnumDisplayMonitors(
            HDC::default(),
            None,
            Some(monitor_enum_proc),
            LPARAM((&mut monitors as *mut Vec<NativeMonitorInfo>) as isize),
        )
        .as_bool()
        {
            return Err(RecorderError::CaptureInit(
                "Falha ao enumerar os monitores do Windows".into(),
            ));
        }
    }

    if monitors.is_empty() {
        return Err(RecorderError::CaptureInit(
            "Nenhum monitor ativo foi encontrado".into(),
        ));
    }

    monitors.sort_by_key(|monitor: &NativeMonitorInfo| {
        (!monitor.is_primary, monitor.bounds.left, monitor.bounds.top)
    });
    for (index, monitor) in monitors.iter_mut().enumerate() {
        monitor.index = index;
    }

    Ok(monitors)
}

/// Fallback dummy multiplataforma.
#[cfg(not(target_os = "windows"))]
pub fn enumerate_native_monitors() -> Result<Vec<()>, RecorderError> {
    Err(RecorderError::CaptureInit(
        "A enumeracao nativa de telas so esta disponivel no Windows neste modulo".into(),
    ))
}

/// Retorna o índice de monitor preferido validado ou o primário nativo como fallback de segurança.
#[cfg(target_os = "windows")]
pub fn resolve_monitor_index(preferred_index: usize) -> Result<usize, RecorderError> {
    let monitors = enumerate_native_monitors()?;

    if monitors.iter().any(|monitor| monitor.index == preferred_index) {
        return Ok(preferred_index);
    }

    Ok(monitors
        .iter()
        .find(|monitor| monitor.is_primary)
        .map(|monitor| monitor.index)
        .unwrap_or(0))
}

/// Proxy para Linux fallback resolvendo para X11/Wayland.
#[cfg(not(target_os = "windows"))]
pub fn resolve_monitor_index(preferred_index: usize) -> Result<usize, RecorderError> {
    let monitors = crate::services::capture::linux::enumerate_linux_monitors()?;

    if monitors.iter().any(|monitor| monitor.index == preferred_index) {
        return Ok(preferred_index);
    }

    Ok(monitors
        .iter()
        .find(|monitor| monitor.is_primary)
        .map(|monitor| monitor.index)
        .unwrap_or(0))
}

/// Lista todas as telas formatadas para envio ao frontend (JS/TS).
pub fn list_monitors() -> Result<Vec<MonitorInfo>, RecorderError> {
    #[cfg(target_os = "windows")]
    {
        return enumerate_native_monitors().map(|monitors| {
            monitors
                .into_iter()
                .map(|monitor| MonitorInfo {
                    index: monitor.index,
                    name: monitor.name,
                    is_primary: monitor.is_primary,
                })
                .collect()
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        return crate::services::capture::linux::enumerate_linux_monitors().map(|monitors| {
            monitors
                .into_iter()
                .map(|monitor| MonitorInfo {
                    index: monitor.index,
                    name: monitor.name,
                    is_primary: monitor.is_primary,
                })
                .collect()
        });
    }
}

fn map_device_info(device: AudioDeviceInfo) -> (String, String, bool) {
    (device.id, device.name, device.is_default)
}

/// Lista os microfones usando a API cpal/nativa do SO.
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

/// Lista os endpoints de alto-falantes para captura de desktop audio (loopback).
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

/// Lista as câmeras disponíveis no Windows (via FFmpeg DirectShow ou PowerShell como fallback).
#[cfg(target_os = "windows")]
pub fn list_cameras() -> Result<Vec<CameraInfo>, RecorderError> {
    let mut cameras = Vec::new();

    // Tenta usar FFmpeg dshow primeiro para obter hardware real compatível com DShow
    if let Ok(ffmpeg_path) = resolve_ffmpeg_path() {
        let mut cmd = Command::new(ffmpeg_path);
        cmd.creation_flags(CREATE_NO_WINDOW);

        if let Ok(output) = cmd
            .args(["-list_devices", "true", "-f", "dshow", "-i", "dummy"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            let combined_output = format!("{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
            let mut in_video_section = false;

            for line in combined_output.lines() {
                let line_lower = line.to_lowercase();
                if line_lower.contains("directshow video devices") {
                    in_video_section = true;
                    continue;
                }
                if line_lower.contains("directshow audio devices") {
                    break;
                }
                if in_video_section {
                    // Os nomes aparecem entre aspas: "Device Name"
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line[start + 1..].find('"') {
                            let name = line[start + 1..start + 1 + end].to_string();
                            // Ignora linhas "alternative name" geradas pelo FFmpeg
                            if !line_lower.contains("alternative name") {
                                cameras.push(CameraInfo {
                                    id: name.clone(),
                                    name,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: PowerShell se FFmpeg falhar ou não achar nada.
    if cameras.is_empty() {
        let mut cmd = Command::new("powershell");
        cmd.creation_flags(CREATE_NO_WINDOW);
        if let Ok(output) = cmd
            .args([
            "-NoProfile",
            "-Command",
            "Get-PnpDevice -PresentOnly | Where-Object { $_.PNPClass -eq 'Camera' -or $_.PNPClass -eq 'Image' } | Select-Object -ExpandProperty FriendlyName"
        ])
        .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let name = line.trim().to_string();
                if !name.is_empty() {
                    cameras.push(CameraInfo {
                        id: name.clone(),
                        name,
                    });
                }
            }
        }
    }
    
    Ok(cameras)
}

#[cfg(not(target_os = "windows"))]
pub fn list_cameras() -> Result<Vec<CameraInfo>, RecorderError> {
    Err(RecorderError::CaptureInit(
        "A função de câmeras do Windows não pode ser chamada em outros sistemas".into(),
    ))
}