!include "StrFunc.nsh"

!macro TerminateDwallProcess
  nsis_tauri_utils::FindProcess "dwall.exe"
  Pop $R0
  ${If} $R0 = 0
    DetailPrint "Found dwall.exe process running, attempting to terminate..."
    nsis_tauri_utils::KillProcess "dwall.exe"
    Pop $R0

    ${If} $R0 = 0
      DetailPrint "Successfully terminated dwall.exe process"
    ${Else}
      DetailPrint "Failed to terminate process using nsis_tauri_utils, trying taskkill..."
      # If above method failed, try using taskkill
      nsExec::ExecToStack `taskkill /F /IM dwall.exe`
      Pop $R1
      Pop $R2
      ${If} $R1 = 0
        DetailPrint "Successfully terminated dwall.exe process using taskkill"
      ${Else}
        DetailPrint "Failed to terminate dwall.exe process: $R2"
      ${EndIf}
    ${EndIf}

    # Add delay to ensure process is fully terminated
    DetailPrint "Waiting for process to fully terminate..."
    ${For} $R3 1 10
        nsis_tauri_utils::FindProcess "dwall.exe"
        Pop $R0
        ${If} $R0 != 0
            ${Break}
        ${EndIf}
        Sleep 200
    ${Next}
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREINSTALL
  # NOTE: Clear incomplete thumbnails saved by old versions
  # ---- start: This hook will be removed in the future ----
  # Only the thumbnail cache, not the whole cache directory: the logs live beside it
  # and are the record of a problem the user has already hit, so an upgrade must not
  # take them away. $LOCALAPPDATA\dwall is the pre-identifier layout of older versions.
  RMDir /r "$LOCALAPPDATA\com.thep0y.dwall\thumbnails"
  RMDir /r "$LOCALAPPDATA\dwall"
  # ---- end ----

  !insertmacro TerminateDwallProcess
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro TerminateDwallProcess

  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Dwall"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  # $APPDATA\dwall holds config.toml, the downloaded themes and the custom themes. The
  # custom ones are the only files here that cannot be downloaded again, and deleting
  # the directory destroyed them without asking, so an uninstall leaves it alone. What
  # is cleared below is regenerated on the next run.
  RMDir /r "$LOCALAPPDATA\com.thep0y.dwall"
  RMDir /r "$LOCALAPPDATA\dwall"
!macroend