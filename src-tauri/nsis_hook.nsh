; Hook customizado para o instalador NSIS do Tauri 2.
; O arquivo e referenciado em bundle.windows.nsis.installerHooks.
; A versao abaixo e mantida em sincronia por scripts/sync.js usando version.txt.

!macro NSIS_HOOK_PREINSTALL
  DetailPrint "Preparando a instalacao do Rec Corder v0.3.0"
!macroend

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Rec Corder instalado com sucesso."
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Removendo o Rec Corder..."
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DetailPrint "Rec Corder removido."
!macroend
