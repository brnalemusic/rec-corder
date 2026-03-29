$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

# Criar pasta de destino em AppData
$appDataPath = [Environment]::GetFolderPath([Environment+SpecialFolder]::LocalApplicationData)
$recCorderPath = Join-Path $appDataPath "RecCorder"

# Garantir que a pasta existe
if (-not (Test-Path $recCorderPath)) {
    New-Item -ItemType Directory -Path $recCorderPath -Force | Out-Null
    Write-Host "Criada pasta: $recCorderPath"
}

$ffmpegUrl = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
$zipPath = Join-Path $recCorderPath "ffmpeg.zip"
$extractFolder = Join-Path $recCorderPath "ffmpeg_unzipped"
$ffmpegExe = Join-Path $extractFolder "ffmpeg-master-latest-win64-gpl\bin\ffmpeg.exe"
$destExe = Join-Path $recCorderPath "ffmpeg.exe"

Write-Host "Downloading FFmpeg from $ffmpegUrl ..."
Invoke-WebRequest -Uri $ffmpegUrl -OutFile $zipPath

Write-Host "Unzipping FFmpeg..."
Expand-Archive -Path $zipPath -DestinationPath $extractFolder -Force

Write-Host "Moving ffmpeg.exe to $recCorderPath..."
Move-Item -Path $ffmpegExe -Destination $destExe -Force

Write-Host "Cleaning up temp files..."
Remove-Item -Path $zipPath -Force
Remove-Item -Path $extractFolder -Recurse -Force

Write-Host "FFmpeg successfully installed at $destExe!"
