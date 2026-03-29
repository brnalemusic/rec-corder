# Script para rebuild completo do Rec Corder v0.1.6 (Windows PowerShell)
# Uso: .\build.ps1

$ErrorActionPreference = "Stop"

Write-Host "🔨 Rec Corder v0.1.6 - Build Completo" -ForegroundColor Cyan
Write-Host "=======================================" -ForegroundColor Cyan

# Verificar se estamos na pasta correta
if (-not (Test-Path "package.json")) {
    Write-Host "❌ Erro: Execute este script na raiz do projeto" -ForegroundColor Red
    exit 1
}

Write-Host "📦 Limpando builds anteriores..." -ForegroundColor Yellow
Set-Location src-tauri
cargo clean
Set-Location ..

Write-Host "📥 Atualizando dependências..." -ForegroundColor Yellow
Set-Location src-tauri
cargo update
Set-Location ..

Write-Host "🔨 Buildando versão release..." -ForegroundColor Yellow
Set-Location src-tauri
cargo tauri build --config=src-tauri/tauri.conf.json
Set-Location ..

Write-Host "✅ Build concluído!" -ForegroundColor Green
Write-Host ""
Write-Host "📦 Saída:" -ForegroundColor Cyan
Write-Host "- .exe:  src-tauri/target/release/bundle/nsis/" 
Write-Host "- .msi:  src-tauri/target/release/bundle/msi/"
Write-Host ""
Write-Host "O instalador automaticamente:" -ForegroundColor Cyan
Write-Host "1. Instala Rec Corder v0.1.6"
Write-Host "2. Baixa FFmpeg para %LOCALAPPDATA%\RecCorder\"
Write-Host "3. Detecta acelerador de vídeo"
Write-Host "4. Mostra na splash screen"
Write-Host ""
Write-Host "🎉 Pronto para distribuição!" -ForegroundColor Green
