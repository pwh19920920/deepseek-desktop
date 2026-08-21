; Custom NSIS hooks for DeepSeek Harness Desktop
; These macros are called by the Tauri NSIS installer at specific points

; Called before copying files - kill running processes
!macro NSIS_HOOK_PREINSTALL
  ; Kill the main app process (and its child process tree with /T)
  nsExec::ExecToStack 'taskkill /F /IM "DeepSeek Harness.exe" /T'

  ; Kill any node.exe processes that might be holding file locks
  nsExec::ExecToStack 'taskkill /F /IM "node.exe"'

  ; Wait for processes to fully terminate
  Sleep 2000
!macroend

; Called before uninstalling - kill running processes
!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToStack 'taskkill /F /IM "DeepSeek Harness.exe" /T'
  nsExec::ExecToStack 'taskkill /F /IM "node.exe"'
  Sleep 2000
!macroend