use serde::Serialize;

/// All possible errors that can occur in the recorder.
/// Each variant maps to a user-safe message for the frontend.
#[derive(Debug, Serialize)]
pub enum RecorderError {
    CaptureInit(String),
    CaptureRuntime(String),
    AudioInit(String),
    AudioRuntime(String),
    FileIO(String),
    InvalidState(String),
    EncoderError(String),
}

impl std::fmt::Display for RecorderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CaptureInit(msg) => write!(f, "Falha ao iniciar captura: {msg}"),
            Self::CaptureRuntime(msg) => write!(f, "Erro durante captura: {msg}"),
            Self::AudioInit(msg) => write!(f, "Falha ao iniciar áudio: {msg}"),
            Self::AudioRuntime(msg) => write!(f, "Erro de áudio: {msg}"),
            Self::FileIO(msg) => write!(f, "Erro de arquivo: {msg}"),
            Self::InvalidState(msg) => write!(f, "Estado inválido: {msg}"),
            Self::EncoderError(msg) => write!(f, "Erro do encoder: {msg}"),
        }
    }
}

impl From<RecorderError> for String {
    fn from(err: RecorderError) -> Self {
        err.to_string()
    }
}

impl From<std::io::Error> for RecorderError {
    fn from(err: std::io::Error) -> Self {
        Self::FileIO(err.to_string())
    }
}
