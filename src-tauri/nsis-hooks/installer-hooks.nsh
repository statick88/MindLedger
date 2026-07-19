; NSIS_HOOK_POSTINSTALL - runs after standard file installation
; Moves runtime DLLs (OpenSSL + WebView2) from Tauri's resources/ subdirectory
; to $INSTDIR root so the app can find them via standard Windows DLL loading.
; No hardcoded absolute paths — references Tauri-bundled files at $INSTDIR\resources\

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Installing runtime libraries to application root..."

  ; --- WebView2Loader.dll ---
  IfFileExists "$INSTDIR\resources\webview2\WebView2Loader.dll" 0 _skip_webview2
    CreateDirectory "$INSTDIR"
    CopyFiles /SILENT "$INSTDIR\resources\webview2\WebView2Loader.dll" "$INSTDIR"
    DetailPrint "Copied WebView2Loader.dll to application root"
  _skip_webview2:

  ; --- OpenSSL DLLs (architecture-agnostic) ---
  ; Try x64 names first, then arm64, then generic
  IfFileExists "$INSTDIR\resources\openssl\libcrypto-3-x64.dll" 0 _try_arm64_crypto
    CopyFiles /SILENT "$INSTDIR\resources\openssl\libcrypto-3-x64.dll" "$INSTDIR"
    Goto _check_ssl
  _try_arm64_crypto:
  IfFileExists "$INSTDIR\resources\openssl\libcrypto-3-arm64.dll" 0 _try_generic_crypto
    CopyFiles /SILENT "$INSTDIR\resources\openssl\libcrypto-3-arm64.dll" "$INSTDIR"
    Goto _check_ssl
  _try_generic_crypto:
  IfFileExists "$INSTDIR\resources\openssl\libcrypto-3.dll" 0 _skip_crypto
    CopyFiles /SILENT "$INSTDIR\resources\openssl\libcrypto-3.dll" "$INSTDIR"
  _skip_crypto:

  _check_ssl:
  IfFileExists "$INSTDIR\resources\openssl\libssl-3-x64.dll" 0 _try_arm64_ssl
    CopyFiles /SILENT "$INSTDIR\resources\openssl\libssl-3-x64.dll" "$INSTDIR"
    Goto _done
  _try_arm64_ssl:
  IfFileExists "$INSTDIR\resources\openssl\libssl-3-arm64.dll" 0 _try_generic_ssl
    CopyFiles /SILENT "$INSTDIR\resources\openssl\libssl-3-arm64.dll" "$INSTDIR"
    Goto _done
  _try_generic_ssl:
  IfFileExists "$INSTDIR\resources\openssl\libssl-3.dll" 0 _done
    CopyFiles /SILENT "$INSTDIR\resources\openssl\libssl-3.dll" "$INSTDIR"
  _done:

  DetailPrint "Runtime libraries installed."
!macroend
