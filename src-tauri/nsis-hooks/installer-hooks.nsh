; NSIS_HOOK_POSTINSTALL - runs after standard file installation
; Adds runtime DLLs (OpenSSL + WebView2) to the installer
; File commands are compile-time: they add files to the NSIS package
; Paths are relative to the NSIS build directory (Tauri sets this up)

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Installing runtime libraries..."
  SetOutPath "$INSTDIR"
  File /nonfatal "resources\webview2\WebView2Loader.dll"
  File /nonfatal "resources\openssl\libcrypto-3-x64.dll"
  File /nonfatal "resources\openssl\libssl-3-x64.dll"
!macroend
