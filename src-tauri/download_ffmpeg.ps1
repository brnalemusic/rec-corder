$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$ffmpegUrl = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
$zipPath = ".\ffmpeg.zip"
$extractFolder = ".\ffmpeg_unzipped"
$ffmpegExe = "$extractFolder\ffmpeg-master-latest-win64-gpl\bin\ffmpeg.exe"
$destExe = ".\ffmpeg.exe"

Write-Host "Downloading FFmpeg from $ffmpegUrl ..."
Invoke-WebRequest -Uri $ffmpegUrl -OutFile $zipPath

Write-Host "Unzipping FFmpeg..."
Expand-Archive -Path $zipPath -DestinationPath $extractFolder -Force

Write-Host "Moving ffmpeg.exe to root..."
Move-Item -Path $ffmpegExe -Destination $destExe -Force

Write-Host "Cleaning up temp files..."
Remove-Item -Path $zipPath -Force
Remove-Item -Path $extractFolder -Recurse -Force

Write-Host "FFmpeg successfully installed at $destExe!"
