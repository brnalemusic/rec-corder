# Script executado durante a instalação do Rec Corder v0.2.0-alpha
# Baixa e instala FFmpeg automaticamente no AppData do usuário

$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

# Obter caminho AppData
$appDataPath = [Environment]::GetFolderPath([Environment+SpecialFolder]::LocalApplicationData)
$recCorderPath = Join-Path $appDataPath "RecCorder"

# Criar pasta se não existir
if (-not (Test-Path $recCorderPath)) {
    New-Item -ItemType Directory -Path $recCorderPath -Force | Out-Null
}

$ffmpegPath = Join-Path $recCorderPath "ffmpeg.exe"

# Se FFmpeg já existe, não baixar novamente
if (Test-Path $ffmpegPath) {
    Write-Host "FFmpeg já instalado em: $ffmpegPath"
    exit 0
}

Write-Host "Instalando FFmpeg v0.2.0-alpha"

$ffmpegUrl = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
$zipPath = Join-Path $recCorderPath "ffmpeg_download.zip"
$extractFolder = Join-Path $recCorderPath "ffmpeg_temp"

try {
    # Download
    Write-Host "Baixando FFmpeg de: $ffmpegUrl"
    $ProgressPreference = "SilentlyContinue"
    Invoke-WebRequest -Uri $ffmpegUrl -OutFile $zipPath -TimeoutSec 600
    
    # Extrair
    Write-Host "Extraindo arquivos..."
    if (Test-Path $extractFolder) {
        Remove-Item -Path $extractFolder -Recurse -Force
    }
    Expand-Archive -Path $zipPath -DestinationPath $extractFolder -Force
    
    # Encontrar e mover ffmpeg.exe
    $ffmpegExe = Get-ChildItem -Path $extractFolder -Filter "ffmpeg.exe" -Recurse | Select-Object -First 1
    if ($ffmpegExe) {
        Move-Item -Path $ffmpegExe.FullName -Destination $ffmpegPath -Force
        Write-Host "FFmpeg instalado com sucesso em: $ffmpegPath"
    } else {
        Write-Host "AVISO: ffmpeg.exe não encontrado no arquivo ZIP"
    }
    
    # Limpar arquivos temporários
    if (Test-Path $zipPath) {
        Remove-Item -Path $zipPath -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path $extractFolder) {
        Remove-Item -Path $extractFolder -Recurse -Force -ErrorAction SilentlyContinue
    }
    
    exit 0
} catch {
    Write-Host "AVISO: Falha ao baixar FFmpeg: $_"
    Write-Host "O aplicativo tentará usar FFmpeg do PATH ou você pode instalá-lo manualmente."
    exit 0
}
