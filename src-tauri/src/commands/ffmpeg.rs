use std::fs;
use std::path::PathBuf;
use std::process::Command;
use serde::Serialize;

#[derive(Serialize)]
pub struct FfmpegStatus {
    pub found: bool,
    pub path: Option<String>,
}

/// Verifica se o FFmpeg está disponível no sistema
#[tauri::command]
pub fn check_ffmpeg() -> FfmpegStatus {
    let rec_corder_path = get_rec_corder_path();
    let ffmpeg_path = rec_corder_path.join("ffmpeg.exe");
    
    if ffmpeg_path.exists() {
        return FfmpegStatus {
            found: true,
            path: Some(ffmpeg_path.to_string_lossy().to_string()),
        };
    }
    
    // Tenta encontrar em outros locais
    if let Ok(output) = Command::new("where")
        .arg("ffmpeg.exe")
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .map(|s| s.trim().to_string());
            
            return FfmpegStatus {
                found: true,
                path,
            };
        }
    }
    
    FfmpegStatus {
        found: false,
        path: None,
    }
}

/// Baixa o FFmpeg para a pasta AppData do RecCorder
#[tauri::command]
pub async fn download_ffmpeg() -> Result<String, String> {
    let rec_corder_path = get_rec_corder_path();
    
    // Cria a pasta se não existir
    fs::create_dir_all(&rec_corder_path)
        .map_err(|e| format!("Erro ao criar pasta RecCorder: {}", e))?;
    
    let ffmpeg_url = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip";
    let zip_path = rec_corder_path.join("ffmpeg.zip");
    let extract_folder = rec_corder_path.join("ffmpeg_unzipped");
    let ffmpeg_exe = extract_folder.join("ffmpeg-master-latest-win64-gpl\\bin\\ffmpeg.exe");
    let dest_exe = rec_corder_path.join("ffmpeg.exe");
    
    println!("Iniciando download de FFmpeg...");
    
    // Download
    let response = reqwest::Client::new()
        .get(ffmpeg_url)
        .send()
        .await
        .map_err(|e| format!("Erro ao baixar FFmpeg: {}", e))?;
    
    let content = response
        .bytes()
        .await
        .map_err(|e| format!("Erro ao ler resposta: {}", e))?;
    
    fs::write(&zip_path, content)
        .map_err(|e| format!("Erro ao salvar arquivo ZIP: {}", e))?;
    
    println!("ZIP baixado, extraindo...");
    
    // Extrai usando Command
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(format!(
            "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
            zip_path.display(),
            extract_folder.display()
        ))
        .output()
        .map_err(|e| format!("Erro ao extrair ZIP: {}", e))?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Erro na extração: {}", error));
    }
    
    println!("Movendo ffmpeg.exe...");
    
    // Move ffmpeg.exe
    if ffmpeg_exe.exists() {
        fs::rename(&ffmpeg_exe, &dest_exe)
            .map_err(|e| format!("Erro ao mover ffmpeg.exe: {}", e))?;
    } else {
        return Err(format!("FFmpeg não encontrado em: {}", ffmpeg_exe.display()));
    }
    
    // Limpa arquivos temporários
    let _ = fs::remove_file(&zip_path);
    let _ = fs::remove_dir_all(&extract_folder);
    
    println!("FFmpeg instalado com sucesso em: {}", dest_exe.display());
    
    Ok(format!("FFmpeg instalado em: {}", dest_exe.display()))
}

/// Obtém a pasta de dados do aplicativo RecCorder
fn get_rec_corder_path() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data).join("RecCorder")
    } else if let Ok(user_profile) = std::env::var("USERPROFILE") {
        PathBuf::from(user_profile).join("AppData\\Local\\RecCorder")
    } else {
        PathBuf::from(".\\RecCorder")
    }
}
