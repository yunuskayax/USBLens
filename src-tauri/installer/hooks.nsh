; UsbLens NSIS hooks (bundle.windows.nsis.installerHooks).

; Remove the Explorer right-click menu ("UsbLens ▸ …") on uninstall.
; During an update the app adds it back on its next start.
!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegKey HKCU "Software\Classes\Drive\shell\UsbLens"
!macroend
