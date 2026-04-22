use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use parking_lot::Mutex;

/// Estado central da aplicação gerenciado pelo Tauri.
/// Usa atômicos (atomics) para verificações de status sem bloqueio de thread a partir do frontend.
/// // [IMPORTANTE] O estado é compartilhado entre Tauri e a API Python (via PyO3) indiretamente,
/// sendo a fonte da verdade para o controle de concorrência.
pub struct AppState {
    /// Flag atômica para indicar se a gravação está em andamento.
    pub is_recording: AtomicBool,
    /// Instante de início da gravação para cálculo de tempo decorrido.
    pub recording_start: Mutex<Option<Instant>>,
    /// Diretório de saída configurado atualmente.
    pub output_dir: Mutex<PathBuf>,
    /// Caminho do arquivo de gravação atual (se houver).
    pub current_file: Mutex<Option<PathBuf>>,

    /// Marcador persistido em disco para recuperação em caso de falha (crash recovery).
    /// // [IMPORTANTE] Mantido com Mutex de escopo reduzido para evitar gargalos durante I/O.
    pub crash_marker: Mutex<Option<PathBuf>>,
    /// Configurações gerais da aplicação (carregadas e salvas em disco).
    pub config: Mutex<crate::config::AppConfig>,
}

impl AppState {
    /// Inicializa o estado da aplicação e carrega as configurações do disco.
    pub fn new() -> Self {
        let config = crate::config::AppConfig::load();
        let output = config.output_dir.clone();

        Self {
            is_recording: AtomicBool::new(false),
            recording_start: Mutex::new(None),
            output_dir: Mutex::new(output),
            current_file: Mutex::new(None),
            crash_marker: Mutex::new(None),
            config: Mutex::new(config),
        }
    }

    /// Retorna verdadeiro se uma gravação estiver em andamento (leitura atômica rápida).
    pub fn recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }

    /// Atualiza o status atômico de gravação.
    pub fn set_recording(&self, val: bool) {
        self.is_recording.store(val, Ordering::Relaxed);
    }
    
    /// Calcula o tempo decorrido desde o início da gravação em segundos.
    pub fn elapsed_secs(&self) -> u64 {
        self.recording_start
            .lock()
            .map(|start| start.elapsed().as_secs())
            .unwrap_or(0)
    }
}

impl Default for AppState {
    /// Implementação padrão para inicialização do estado.
    fn default() -> Self {
        Self::new()
    }
}
