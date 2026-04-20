use pyo3::prelude::*;
use crate::services::capture::{self, session::CaptureSession};
use crate::commands::ffmpeg::check_ffmpeg;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Classe exposta para o Python que encapsula uma sessão de gravação nativa.
/// // [IMPORTANTE] Esta struct gerencia a ponte assíncrona entre o Python e a engine em Rust.
#[pyclass]
pub struct RecorderSession {
    /// A sessão de captura responsável por gerenciar o processo do FFmpeg e arquivos locais.
    session: Option<CaptureSession>,
    /// Flag atômico para controle seguro de encerramento.
    stop_flag: Arc<AtomicBool>,
}

#[pymethods]
impl RecorderSession {
    /// Inicializa uma nova sessão de gravação a partir do Python.
    #[new]
    fn new(
        output_path: String,
        monitor_index: usize,
        fps: u32,
        scale: u32,
        encoder: String,
        mic_id: Option<String>,
        sys_id: Option<String>,
    ) -> PyResult<Self> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let path = PathBuf::from(output_path);

        let monitor = capture::resolve_monitor_index(monitor_index)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Indice de monitor invalido: {}", e)))?;
        
        let session = CaptureSession::start(
            path,
            monitor,
            mic_id,
            sys_id,
            fps,
            scale,
            &encoder,
            None,
        ).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(Self {
            session: Some(session),
            stop_flag,
        })
    }

    /// Encerra a sessão de gravação em andamento a partir do CLI Python.
    fn stop(&mut self) -> PyResult<()> {
        self.stop_flag.store(true, Ordering::Relaxed);
        
        if let Some(mut session) = self.session.take() {
            session.stop().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        }
        Ok(())
    }
}

/// Retorna os monitores disponíveis do sistema como uma string JSON para o Python.
#[pyfunction]
fn get_monitors() -> PyResult<String> {
    let monitors = capture::list_monitors().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    serde_json::to_string(&monitors).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Retorna os microfones disponíveis do sistema como uma string JSON para o Python.
#[pyfunction]
fn get_mics() -> PyResult<String> {
    let mics = capture::list_mic_devices().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    serde_json::to_string(&mics).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Retorna as saídas de áudio disponíveis do sistema como uma string JSON para o Python.
#[pyfunction]
fn get_audio_outputs() -> PyResult<String> {
    let outputs = capture::list_audio_outputs().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    serde_json::to_string(&outputs).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Verifica o status e a presença do executável FFmpeg no sistema.
#[pyfunction]
fn get_ffmpeg_status() -> PyResult<String> {
    let status = check_ffmpeg();
    serde_json::to_string(&status).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Módulo principal exportado para a biblioteca Python (via PyO3).
/// // [IMPORTANTE] Todas as funções registradas aqui tornam-se nativas no módulo `rec_corder_lib`.
#[pymodule]
fn rec_corder_lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RecorderSession>()?;
    m.add_function(wrap_pyfunction!(get_monitors, m)?)?;
    m.add_function(wrap_pyfunction!(get_mics, m)?)?;
    m.add_function(wrap_pyfunction!(get_audio_outputs, m)?)?;
    m.add_function(wrap_pyfunction!(get_ffmpeg_status, m)?)?;
    Ok(())
}
