use std::path::PathBuf;

fn main() {
    // Verificar se FFmpeg já existe no bundle
    let bundle_ffmpeg = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("src-tauri"))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ffmpeg.exe");

    if !bundle_ffmpeg.exists() {
        println!("cargo:warning=FFmpeg não encontrado em src-tauri/ffmpeg.exe");
        println!("cargo:warning=O aplicativo tentará usar FFmpeg do PATH ou AppData");
        // Não falha - deixa para o runtime tentar encontrar
    }

    tauri_build::build()
}
