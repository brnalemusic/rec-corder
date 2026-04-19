use crate::errors::RecorderError;
use crate::services::audio::{self, AudioDeviceInfo};
use serde::Serialize;
#[cfg(target_os = "windows")]
use windows::core::PCWSTR;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW, DISPLAY_DEVICEW, HDC, HMONITOR,
    MONITORINFOEXW,
};

#[derive(Serialize)]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub is_primary: bool,
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

#[cfg(target_os = "windows")]
pub const MONITORINFO_PRIMARY_FLAG: u32 = 1;

#[cfg(target_os = "windows")]
#[derive(Clone, Debug)]
pub struct NativeMonitorInfo {
    pub index: usize,
    pub hmonitor: isize,
    pub dxgi_output_index: Option<usize>,
    pub name: String,
    pub bounds: RECT,
    pub is_primary: bool,
}

#[cfg(target_os = "windows")]
pub fn parse_display_device_string(device: &[u16]) -> String {
    let len = device
        .iter()
        .position(|&value| value == 0)
        .unwrap_or(device.len());
    String::from_utf16_lossy(&device[..len]).trim().to_string()
}

#[cfg(target_os = "windows")]
pub fn resolve_monitor_friendly_name(adapter_name: &str) -> Option<String> {
    let adapter_name_wide: Vec<u16> = adapter_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
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
        if !monitor_name.is_empty() && !monitor_name.eq_ignore_ascii_case("Generic PnP Monitor") {
            return Some(monitor_name);
        }

        if !monitor_name.is_empty() {
            return Some(monitor_name);
        }

        device_index += 1;
    }

    None
}

#[cfg(target_os = "windows")]
pub fn resolve_dxgi_output_index(target_hmonitor: HMONITOR) -> Option<usize> {
    unsafe {
        let factory = CreateDXGIFactory1::<IDXGIFactory1>().ok()?;
        let mut adapter_index = 0;
        let mut output_index = 0usize;

        loop {
            let adapter = match factory.EnumAdapters1(adapter_index) {
                Ok(adapter) => adapter,
                Err(_) => break,
            };

            let mut adapter_output_index = 0;
            loop {
                let output = match adapter.EnumOutputs(adapter_output_index) {
                    Ok(output) => output,
                    Err(_) => break,
                };

                if let Ok(desc) = output.GetDesc() {
                    if desc.Monitor.0 == target_hmonitor.0 {
                        return Some(output_index);
                    }
                }

                output_index += 1;
                adapter_output_index += 1;
            }

            adapter_index += 1;
        }
    }

    None
}

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
        let dxgi_output_index = resolve_dxgi_output_index(hmonitor);
        let label = if is_primary {
            format!("{friendly_name} (Principal) - {width}x{height}")
        } else {
            format!("{friendly_name} - {width}x{height}")
        };

        monitors.push(NativeMonitorInfo {
            index: monitors.len(),
            hmonitor: hmonitor.0 as isize,
            dxgi_output_index,
            name: label,
            bounds,
            is_primary,
        });
    }

    BOOL(1)
}

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

#[cfg(not(target_os = "windows"))]
pub fn enumerate_native_monitors() -> Result<Vec<()>, RecorderError> {
    Err(RecorderError::CaptureInit(
        "A captura de tela so esta disponivel no Windows".into(),
    ))
}

#[cfg(target_os = "windows")]
pub fn resolve_monitor_index(preferred_index: usize) -> Result<usize, RecorderError> {
    let monitors = enumerate_native_monitors()?;

    if monitors
        .iter()
        .any(|monitor| monitor.index == preferred_index)
    {
        return Ok(preferred_index);
    }

    Ok(monitors
        .iter()
        .find(|monitor| monitor.is_primary)
        .map(|monitor| monitor.index)
        .unwrap_or(0))
}

#[cfg(not(target_os = "windows"))]
pub fn resolve_monitor_index(preferred_index: usize) -> Result<usize, RecorderError> {
    Ok(preferred_index)
}

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
        Err(RecorderError::CaptureInit(
            "A captura de tela so esta disponivel no Windows".into(),
        ))
    }
}

fn map_device_info(device: AudioDeviceInfo) -> (String, String, bool) {
    (device.id, device.name, device.is_default)
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
