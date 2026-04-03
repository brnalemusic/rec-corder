use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub encoder: String,
    pub output_dir: PathBuf,
    pub fps: u32,
    pub scale: u32,
    pub mic_enabled: bool,
    pub sys_audio_enabled: bool,
    pub system_audio_enabled: bool, // Nova flag mais clara
    pub selected_monitor: usize,
    pub selected_mic: Option<String>,
    pub selected_audio_output: Option<String>,
    pub mic_volume: u32, // 0-150, padrão 100
}

impl Default for AppConfig {
    fn default() -> Self {
        let videos_dir = dirs_next::video_dir()
            .unwrap_or_else(|| dirs_next::home_dir().unwrap_or_else(|| PathBuf::from(".")));

        Self {
            encoder: String::from("libx264"),
            output_dir: videos_dir.join("RecCorder"),
            fps: 60,
            scale: 100,
            mic_enabled: false,
            sys_audio_enabled: true,
            system_audio_enabled: true,
            selected_monitor: 0,
            selected_mic: None,
            selected_audio_output: None,
            mic_volume: 100,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let data_dir = dirs_next::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        data_dir.join("RecCorder").join("reccorder.cfg")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                return config;
            }
        }
        
        let default = Self::default();
        let _ = default.save(); // Salva os padrões se o arquivo não existir ou estiver corrompido
        default
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        
        // Garante que o diretório AppData\Local\RecCorder existe
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Não foi possível criar o diretório de configuração em {:?}: {}", parent, e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Erro ao serializar configurações: {e}"))?;
            
        fs::write(&path, content)
            .map_err(|e| format!("Erro ao gravar o arquivo {:?}: {}", path, e))?;

        println!("Configurações persistidas em: {:?}", path);
        Ok(())
    }
}
