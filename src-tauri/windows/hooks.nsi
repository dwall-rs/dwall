!include "StrFunc.nsh"
${StrStr}
${UnStrStr}

!macro NSIS_HOOK_PREINSTALL
  nsis_tauri_utils::FindProcess "dwall.exe"
  Pop $R0
  ${If} $R0 = 0
    nsExec::ExecToStack `wmic process where "name='dwall.exe'" get ExecutablePath`
    Pop $R1
    Pop $R2
    ${StrStr} $R3 "$R2" "$INSTDIR"
    ${If} $R3 != ""
      nsis_tauri_utils::KillProcess "dwall.exe"
      Pop $R0
      Sleep 1000
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsis_tauri_utils::FindProcess "dwall.exe"
  Pop $R0
  ${If} $R0 = 0
    nsExec::ExecToStack `wmic process where "name='dwall.exe'" get ExecutablePath`
    Pop $R1
    Pop $R2
    ${UnStrStr} $R3 "$R2" "$INSTDIR"
    ${If} $R3 != ""
      nsis_tauri_utils::KillProcess "dwall.exe"
      Pop $R0
      Sleep 1000
    ${EndIf}
  ${EndIf}

  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Dwall"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  RMDir /r "$APPDATA\dwall"
  RMDir /r "$LOCALAPPDATA\dwall"
!macroend