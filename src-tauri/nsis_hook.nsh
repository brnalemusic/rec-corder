; Hook customizado para Tauri NSIS - Rec Corder v0.2.0
; Este arquivo é incluído automaticamente no script NSIS gerado pelo Tauri

!macro customInstall
  ; Executar script PowerShell de pré-instalação de FFmpeg
  DetailPrint "Preparando dependências (FFmpeg)..."
  
  ; Obter caminho do script
  StrCpy $0 "$INSTDIR\..\..\..\src-tauri\pre_install.ps1"
  
  ; Executar PowerShell com ExecutionPolicy Bypass
  ExecWait 'powershell.exe -ExecutionPolicy Bypass -NoProfile -File "$0"' $1
  
  ${If} $1 = 0
    DetailPrint "Dependências instaladas com sucesso"
  ${Else}
    DetailPrint "Aviso: Erro ao instalar dependências (código: $1)"
  ${EndIf}
!macroend

!macro customUnInstall
  ; Remover FFmpeg quando desinstalar (opcional)
  ; Descomentar se desejar limpar FFmpeg da desinstalação
  ; StrCpy $0 "$LOCALAPPDATA\RecCorder\ffmpeg.exe"
  ; ${If} ${FileExists} "$0"
  ;   Delete "$0"
  ; ${EndIf}
!macroend
