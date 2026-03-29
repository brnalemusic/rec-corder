#!/bin/bash
# Script para rebuild completo do Rec Corder v0.1.6
# Uso: ./build.sh ou bash build.sh

set -e

echo "🔨 Rec Corder v0.1.6 - Build Completo"
echo "======================================="

# Verificar se estamos na pasta correta
if [ ! -f "package.json" ]; then
    echo "❌ Erro: Execute este script na raiz do projeto"
    exit 1
fi

echo "📦 Limpando builds anteriores..."
cd src-tauri
cargo clean
cd ..

echo "📥 Atualizando dependências..."
cd src-tauri
cargo update
cd ..

echo "🔨 Compilando para desenvolvimento..."
npm run tauri dev &
DEV_PID=$!
sleep 10
kill $DEV_PID 2>/dev/null || true

echo "📦 Buildando versão release..."
cd src-tauri
cargo tauri build --config=src-tauri/tauri.conf.json
cd ..

echo "✅ Build concluído!"
echo ""
echo "Saída:"
echo "- .exe:  src-tauri/target/release/bundle/nsis/"
echo "- .msi:  src-tauri/target/release/bundle/msi/"
echo ""
echo "O instalador automaticamente:"
echo "1. Instala Rec Corder v0.1.6"
echo "2. Baixa FFmpeg para %LOCALAPPDATA%\\RecCorder\\"
echo "3. Detecta acelerador de vídeo"
echo "4. Mostra na splash screen"
echo ""
echo "🎉 Pronto para distribuição!"
