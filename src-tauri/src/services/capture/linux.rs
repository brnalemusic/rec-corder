use crate::errors::RecorderError;
use super::CameraInfo;
use std::process::Command;
use regex::Regex;
use once_cell::sync::Lazy;

/// Regex estáticos compilados uma única vez para melhor performance no Linux.
static XRANDR_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\S+)\s+connected\s+(primary\s+)?(\d+)x(\d+)\+(\d+)\+(\d+)").unwrap()
});

static WPCTL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\*?)\s+(\d+)\.\s+(.+)").unwrap()
});

/// Estrutura contendo informações nativas de monitores no Linux (X11/Wayland via xrandr).
#[derive(Clone, Debug)]
pub struct LinuxMonitorInfo {
    /// Índice numérico sequencial.
    pub index: usize,
    /// Nome amigável retornado pelo xrandr.
    pub name: String,
    /// Limites da tela no formato: (x, y, largura, altura).
    pub bounds: (i32, i32, i32, i32),
    /// Verdadeiro se for o monitor principal.
    pub is_primary: bool,
}

/// Enumera os monitores disponíveis no Linux usando a ferramenta `xrandr`.
/// Retorna uma lista estruturada pronta para consumo.
pub fn enumerate_linux_monitors() -> Result<Vec<LinuxMonitorInfo>, RecorderError> {
    let output = Command::new("xrandr")
        .arg("--query")
        .output()
        .map_err(|e| RecorderError::CaptureInit(format!("Falha ao executar xrandr: {}", e)))?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut monitors = Vec::new();
    let mut index = 0;

    for line in output_str.lines() {
        if let Some(caps) = XRANDR_RE.captures(line) {
            let name = caps.get(1).unwrap().as_str().to_string();
            let is_primary = caps.get(2).is_some();
            let width: i32 = caps.get(3).unwrap().as_str().parse().unwrap_or(0);
            let height: i32 = caps.get(4).unwrap().as_str().parse().unwrap_or(0);
            let x: i32 = caps.get(5).unwrap().as_str().parse().unwrap_or(0);
            let y: i32 = caps.get(6).unwrap().as_str().parse().unwrap_or(0);

            let label = if is_primary {
                format!("{} (Principal) - {}x{}", name, width, height)
            } else {
                format!("{} - {}x{}", name, width, height)
            };

            monitors.push(LinuxMonitorInfo {
                index,
                name: label,
                bounds: (x, y, width, height),
                is_primary,
            });
            index += 1;
        }
    }

    if monitors.is_empty() {
        return Err(RecorderError::CaptureInit("Nenhum monitor encontrado via xrandr".into()));
    }

    Ok(monitors)
}

/// Estrutura para dispositivos abstraídos do PipeWire (wpctl).
#[derive(Debug, Clone)]
pub struct WpctlDevice {
    /// ID nativo PipeWire / V4L2.
    pub id: String,
    /// Nome amigável.
    pub name: String,
    /// Verdadeiro se for o dispositivo padrão do sistema.
    pub is_default: bool,
}

/// Lista de dispositivos de áudio nativos baseados em PipeWire.
pub fn list_wpctl_audio_devices(is_source: bool) -> Vec<WpctlDevice> {
    let section_header = if is_source { "Sources:" } else { "Sinks:" };
    list_wpctl_devices(section_header, "node.name", "node.description")
}

/// Lista dispositivos de vídeo disponíveis utilizando PipeWire.
pub fn list_wpctl_cameras() -> Vec<WpctlDevice> {
    // Para vídeo, primeiro encontramos a sessão "Video", depois "Devices:"
    list_wpctl_devices("Video", "api.v4l2.path", "device.description")
}

/// Helper para fazer o parse da árvore do wpctl.
fn list_wpctl_devices(section_name: &str, id_prop: &str, name_prop: &str) -> Vec<WpctlDevice> {
    let mut devices = Vec::new();
    
    let output = match Command::new("wpctl").arg("status").output() {
        Ok(o) => o,
        Err(_) => return devices,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    
    let mut in_main_section = section_name != "Video" && section_name != "Sources:" && section_name != "Sinks:";
    let mut in_sub_section = false;
    
    for i in 0..lines.len() {
        let line = lines[i];
        
        // Se estamos procurando algo de Áudio ou Vídeo, primeiro achamos a sessão principal
        if !in_main_section {
            if (section_name == "Video" && line.trim() == "Video") || 
               ((section_name == "Sources:" || section_name == "Sinks:") && line.trim() == "Audio") {
                in_main_section = true;
                continue;
            }
        }

        if in_main_section {
            // Se chegamos na subseção correta (Sinks, Sources ou Devices dentro de Video)
            if !in_sub_section && line.contains(if section_name == "Video" { "Devices:" } else { section_name }) {
                in_sub_section = true;
                continue;
            }

            if in_sub_section {
                // Se a linha começa com um caractere não-espaço e não é parte da árvore, saímos da subseção
                if !line.starts_with(" ") && !line.is_empty() {
                    break; 
                }

                if let Some(caps) = WPCTL_RE.captures(line) {
                    let is_default = !caps.get(1).unwrap().as_str().is_empty();
                    let numeric_id = caps.get(2).unwrap().as_str();

                    if let Some(props) = get_all_wpctl_props(numeric_id) {
                        let id = props.get(id_prop).cloned()
                            .or_else(|| props.get("node.name").cloned());
                        let name = props.get(name_prop).cloned()
                            .or_else(|| props.get("device.description").cloned())
                            .or_else(|| props.get("node.description").cloned());

                        if let (Some(id), Some(name)) = (id, name) {
                            // Evita duplicatas por ID (comum em video nodes V4L2)
                            if !devices.iter().any(|d: &WpctlDevice| d.id == id) {
                                // Se for vídeo, verificamos se a capacidade é de captura (capture)
                                if section_name == "Video" {
                                    if let Some(caps_prop) = props.get("device.capabilities") {
                                        if !caps_prop.contains(":capture:") {
                                            continue;
                                        }
                                    }
                                }

                                devices.push(WpctlDevice {
                                    id,
                                    name,
                                    is_default,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    devices
}

/// Obtém todas as propriedades de um objeto via wpctl inspect de uma vez só.
fn get_all_wpctl_props(id: &str) -> Option<std::collections::HashMap<String, String>> {
    let output = Command::new("wpctl").args(["inspect", id]).output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut props = std::collections::HashMap::new();

    for line in stdout.lines() {
        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim().trim_matches('"');
            let value = line[pos+1..].trim().trim_matches('"');
            props.insert(key.to_string(), value.to_string());
        }
    }

    if props.is_empty() { None } else { Some(props) }
}

/// Lista todas as câmeras disponíveis no ambiente Linux, utilizando preferencialmente o wpctl e com fallback para /dev/video.
pub fn list_cameras() -> Result<Vec<CameraInfo>, RecorderError> {
    let wpctl_cameras = list_wpctl_cameras();
    if !wpctl_cameras.is_empty() {
        return Ok(wpctl_cameras.into_iter().map(|c| CameraInfo {
            id: c.id,
            name: c.name,
        }).collect());
    }

    // Fallback: busca por diretórios em /dev/video*
    let mut cameras = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/dev") {
        let mut devices: Vec<_> = entries.filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().starts_with("video"))
            .collect();
        devices.sort_by_key(|e| e.file_name());
        for entry in devices {
            if let Ok(name) = entry.file_name().into_string() {
                cameras.push(CameraInfo {
                    id: format!("/dev/{}", name),
                    name: format!("Camera {}", name),
                });
            }
        }
    }
    
    Ok(cameras)
}

/// Verifica se as dependências essenciais do sistema Linux estão instaladas.
pub fn validate_linux_system_deps() -> Result<(), Vec<String>> {
    let mut missing = Vec::new();
    let tools = vec![
        ("xrandr", "--version"),
        ("wpctl", "status"),
        ("pactl", "info"),
        ("awk", "--version"),
    ];

    for (tool, arg) in tools {
        let exists = Command::new(tool)
            .arg(arg)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !exists {
            missing.push(tool.to_string());
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}
