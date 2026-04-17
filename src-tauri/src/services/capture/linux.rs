use crate::errors::RecorderError;
use super::windows::MonitorInfo;
use std::process::Command;
use regex::Regex;

#[derive(Clone, Debug)]
pub struct LinuxMonitorInfo {
    pub index: usize,
    pub name: String,
    pub bounds: (i32, i32, i32, i32), // x, y, width, height
    pub is_primary: bool,
}

pub fn enumerate_linux_monitors() -> Result<Vec<LinuxMonitorInfo>, RecorderError> {
    let output = Command::new("xrandr")
        .arg("--query")
        .output()
        .map_err(|e| RecorderError::CaptureInit(format!("Falha ao executar xrandr: {}", e)))?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut monitors = Vec::new();
    let mut index = 0;

    // Pattern to match lines like:
    // HDMI-1 connected primary 1440x900+0+94 (normal left inverted right x axis y axis) 420mm x 240mm
    let re = Regex::new(r"^(\S+)\s+connected\s+(primary\s+)?(\d+)x(\d+)\+(\d+)\+(\d+)").unwrap();

    for line in output_str.lines() {
        if let Some(caps) = re.captures(line) {
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

#[derive(Debug, Clone)]
pub struct WpctlDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

pub fn list_wpctl_audio_devices(is_source: bool) -> Vec<WpctlDevice> {
    let section_header = if is_source { "Sources:" } else { "Sinks:" };
    list_wpctl_devices(section_header, "node.name", "node.description")
}

pub fn list_wpctl_cameras() -> Vec<WpctlDevice> {
    // Para video, primeiro encontramos a sessao "Video", depois "Devices:"
    list_wpctl_devices("Video", "api.v4l2.path", "device.description")
}

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
    
    // Regex para capturar ID e o nome (parcial) do wpctl status
    let re = Regex::new(r"(\*?)\s+(\d+)\.\s+(.+)").unwrap();

    for i in 0..lines.len() {
        let line = lines[i];
        
        // Se estamos procurando algo de Audio ou Video, primeiro achamos a sessao principal
        if !in_main_section {
            if (section_name == "Video" && line.trim() == "Video") || 
               ((section_name == "Sources:" || section_name == "Sinks:") && line.trim() == "Audio") {
                in_main_section = true;
                continue;
            }
        }

        if in_main_section {
            // Se chegamos na subsecao correta (Sinks, Sources ou Devices dentro de Video)
            if !in_sub_section && line.contains(if section_name == "Video" { "Devices:" } else { section_name }) {
                in_sub_section = true;
                continue;
            }

            if in_sub_section {
                // Se a linha comeca com um caractere nao-espaco e nao eh parte da arvore, saímos da subsecao
                if !line.starts_with(" ") && !line.is_empty() {
                    break; 
                }

                if let Some(caps) = re.captures(line) {
                    let is_default = !caps.get(1).unwrap().as_str().is_empty();
                    let numeric_id = caps.get(2).unwrap().as_str();
                    
                    if let (Some(id), Some(name)) = (
                        get_wpctl_prop(numeric_id, id_prop),
                        get_wpctl_prop(numeric_id, name_prop).or_else(|| get_wpctl_prop(numeric_id, "device.description"))
                    ) {
                        // Evita duplicatas por ID (comum em video nodes)
                        if !devices.iter().any(|d: &WpctlDevice| d.id == id) {
                            // Se for video, verificamos se eh capture
                            if section_name == "Video" {
                                if let Some(caps_prop) = get_wpctl_prop(numeric_id, "device.capabilities") {
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

    devices
}

fn get_wpctl_prop(id: &str, prop: &str) -> Option<String> {
    let output = Command::new("wpctl").args(["inspect", id]).output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    for line in stdout.lines() {
        if line.contains(prop) {
            if let Some(pos) = line.find('=') {
                return Some(line[pos+1..].trim().trim_matches('"').to_string());
            }
        }
    }
    None
}
