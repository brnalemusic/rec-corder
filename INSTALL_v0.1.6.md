# Rec Corder v0.1.6 - Instalação e FFmpeg

## 📦 Processo de Instalação

### Windows (.exe / .msi)
1. **Download do instalador** → Usuário baixa Rec_Corder_Setup.exe ou Rec_Corder.msi
2. **Execução** → Clica para instalar
3. **Instalação do aplicativo** → Tauri copia arquivos
4. **Hook de pré-instalação** → Script PowerShell executa automaticamente
5. **Download de FFmpeg** → `pre_install.ps1` baixa FFmpeg (~200MB) via GitHub
6. **Instalação de FFmpeg** → Salvo em `%LOCALAPPDATA%\RecCorder\ffmpeg.exe`
7. **Finalização** → Instalador conclui, aplicativo pronto para uso

### Primeira Execução
1. **Splash screen** → Mostra "Inicializando..."
2. **Verificação de FFmpeg** → Check se FFmpeg existe
3. **Detecção de encoder** → Teste de aceleração (NVIDIA/AMD/Intel/Software)
4. **Aplicação inicia** → Interface principal carrega

## 🔍 Fluxo de FFmpeg

### Se FFmpeg for encontrado
```
Instalador → [FFmpeg já em AppData] → Splash mostra "FFmpeg detectado ✓"
```

### Se FFmpeg não for encontrado (fallback)
```
Splash screen → Detecta ausência → Oferece download no app → Baixa → Usa
```

## 📁 Locais de Busca de FFmpeg

O aplicativo procura FFmpeg nesta ordem:
1. `%LOCALAPPDATA%\RecCorder\ffmpeg.exe` ← **Instalação primária**
2. Variável de ambiente `REC_CORDER_FFMPEG_PATH`
3. Pasta local do executável
4. Pastas pai (desenvolvimento)
5. Diretórios globais (C:\ffmpeg\, Program Files\)
6. PATH do sistema

## 🔧 Configuração do Instalador

### NSIS Hook
- Arquivo: `src-tauri/nsis_hook.nsh`
- Função: Executa `pre_install.ps1` durante instalação
- Suporta atualizações (FFmpeg não é re-baixado se já existe)

### Script de Pré-Instalação
- Arquivo: `src-tauri/pre_install.ps1`
- Função: Baixa FFmpeg de https://github.com/BtbN/FFmpeg-Builds/
- Timeout: 600 segundos (para conexões lentas)
- Erro: Não falha instalação se FFmpeg não conseguir baixar

## 💾 Tamanho do Download

- **Rec Corder**: ~50-100MB
- **FFmpeg**: ~200MB
- **Total**: ~250-300MB

## 🚀 Para Buildear v0.1.6

```bash
cd src-tauri
cargo tauri build
```

Isso gerará:
- `rec-corder_0.1.6_x64-setup.exe` (NSIS com FFmpeg automático)
- `rec-corder_0.1.6_x64.msi` (WiX com FFmpeg automático)
- `rec-corder_0.1.6_x64_en-US.msi.zip` (Portable)

## ⚠️ Troubleshooting

Se o download de FFmpeg falhar durante instalação:
1. Usuário pode executar manualmente: `src-tauri\download_ffmpeg.ps1`
2. Ou colocar ffmpeg.exe em `%LOCALAPPDATA%\RecCorder\`
3. Ao iniciar app, splash oferecerá opção de download

## ✅ v0.1.6 Checklist

- [x] FFmpeg integrado no instalador
- [x] Download automático sem intervenção do usuário
- [x] Fallback para download na primeira execução
- [x] Interfaces visuais atualizadas
- [x] Versão sincronizada (0.1.6)
- [x] Documentação de instalação
