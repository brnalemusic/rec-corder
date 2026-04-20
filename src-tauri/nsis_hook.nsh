; Hook customizado para o instalador NSIS do Tauri 2.
; O arquivo e referenciado em bundle.windows.nsis.installerHooks.
; A versao abaixo e mantida em sincronia por scripts/sync.js usando version.txt.

!macro NSIS_HOOK_PREINSTALL
  DetailPrint "Preparando a instalacao do Rec Corder v1.0.0-beta.5"
!macroend

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Rec Corder instalado com sucesso."

  ; Em instalacao silenciosa (/S), instala o CLI direto
  IfSilent InstallCLI

  ; Dialogo para instalacao manual
  MessageBox MB_YESNO|MB_ICONQUESTION "Deseja instalar o Rec Corder CLI? (Permite usar o comando 'reccorder' no terminal para gravar via TUI)" IDYES InstallCLI IDNO SkipCLI

InstallCLI:
  DetailPrint "Configurando o Rec Corder CLI..."

  ; Cria o wrapper
  FileOpen $0 "$INSTDIR\reccorder.bat" w
  FileWrite $0 "@echo off$\r$\n"
  FileWrite $0 "$\"$INSTDIR\python_env\python.exe$\" $\"$INSTDIR\cli\main.py$\" %*$\r$\n"
  FileClose $0

  ; Adiciona ao PATH do usuario via PowerShell
  ExecWait 'powershell -NoProfile -Command "$p = [Environment]::GetEnvironmentVariable(\"PATH\", \"User\"); if ($p -notmatch [regex]::Escape(\"$INSTDIR\")) { [Environment]::SetEnvironmentVariable(\"PATH\", $p + \";$INSTDIR\", \"User\") }"'

SkipCLI:
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Removendo o Rec Corder..."
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DetailPrint "Rec Corder removido."
  
  ; Remove o wrapper explicitamente (criado via script, não pelo instalador)
  Delete "$INSTDIR\reccorder.bat"

  ; Remove resíduos de execução do Python caso existam
  RMDir /r "$INSTDIR\cli\__pycache__"
  
  ; Remove do PATH
  ExecWait 'powershell -NoProfile -Command "$p = [Environment]::GetEnvironmentVariable(\"PATH\", \"User\"); if ($p -match [regex]::Escape(\"$INSTDIR\")) { [Environment]::SetEnvironmentVariable(\"PATH\", ($p -split \";\" | Where-Object { $_ -ne \"$INSTDIR\" }) -join \";\", \"User\") }"'
!macroend
