use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Representa as preferências e configurações do aplicativo.
/// // [IMPORTANTE] Esta estrutura é o estado base persistido que sincroniza a GUI e o CLI.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    /// O encoder de vídeo preferido (ex: `libx264`, `h264_nvenc`).
    pub encoder: String,
    /// Caminho do diretório de saída para onde as gravações serão salvas.
    pub output_dir: PathBuf,
    /// Taxa de quadros (frames per second) alvo para a gravação.
    pub fps: u32,
    /// Fator de escala da tela, usado no filtro do FFmpeg (ex: 100 para tamanho original).
    pub scale: u32,
    /// Se o microfone está habilitado na gravação (legado).
    pub mic_enabled: bool,
    /// Se a captura do sistema de áudio está habilitada (legado).
    pub sys_audio_enabled: bool,
    /// Nova flag mais clara se a captura do sistema está ativada.
    pub system_audio_enabled: bool, 
    /// Índice do monitor selecionado para ser gravado.
    pub selected_monitor: usize,
    /// O nome/ID do microfone selecionado (se houver).
    pub selected_mic: Option<String>,
    /// O nome/ID da saída de áudio do sistema selecionada.
    pub selected_audio_output: Option<String>,
    /// Volume do microfone (0-150, padrão 100).
    pub mic_volume: u32, 
    /// Verdadeiro se o overlay de webcam estiver ativado.
    pub webcam_enabled: bool,
    /// Nome do dispositivo DirectShow da webcam (ex: "Integrated Camera").
    pub webcam_device: Option<String>,   
    /// Posição do overlay ("top-left" | "top-right" | "bottom-left" | "bottom-right" | "center").
    pub webcam_position: String,          
    /// Percentual do tamanho base da webcam (200px). Range: 50–300. Default: 100.
    pub webcam_size: u32,                 
}

impl Default for AppConfig {
    /// Configurações padrões quando o arquivo de prefs não existe.
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
            webcam_enabled: false,
            webcam_device: None,
            webcam_position: String::from("bottom-right"),
            webcam_size: 100,
        }
    }
}

impl AppConfig {
    /// Retorna o caminho estático de onde as configurações devem ser lidas/salvas.
    pub fn config_path() -> PathBuf {
        let data_dir = dirs_next::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        data_dir.join("RecCorder").join("reccorder.cfg")
    }

    /// Carrega as configurações persistidas no disco rígido ou cria valores padrão.
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

    /// Salva e persiste as alterações da configuração no disco local.
    /// // [IMPORTANTE] Toda gravação/alteração de estado em `AppConfig` deve chamar esta função.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        
        // Garante que o diretório base de configuração (ex: AppData\Local\RecCorder) exista.
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
