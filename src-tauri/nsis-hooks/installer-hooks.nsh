; NSIS_HOOK_POSTINSTALL - runs after standard file installation
; Adds runtime DLLs (OpenSSL + WebView2) to the installer
; File commands are compile-time: they add files to the NSIS package
; NOTE: NSIS compiles from target/release/nsis/<arch>/ which does NOT have resources/
;       Use absolute paths to src-tauri/resources/ instead

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Installing runtime libraries..."
  SetOutPath "$INSTDIR"
  File "C:\Users\statick\Desktop\MindLedger\src-tauri\resources\webview2\WebView2Loader.dll"
  File "C:\Users\statick\Desktop\MindLedger\src-tauri\resources\openssl\libcrypto-3-x64.dll"
  File "C:\Users\statick\Desktop\MindLedger\src-tauri\resources\openssl\libssl-3-x64.dll"
!macroend
