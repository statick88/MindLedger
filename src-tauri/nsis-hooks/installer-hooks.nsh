; ============================================================================
; MindLedger NSIS Installer Hooks
; ============================================================================
; Custom NSIS hooks for Tauri v2 bundler:
;   - Post-install: copy runtime DLLs (OpenSSL + WebView2) to $INSTDIR root
;   - Uninstall: clean up runtime DLLs
;
; Tauri v2 resource bundling behavior:
;   Resources listed in bundle.resources are installed to $INSTDIR\resources\
;   preserving subdirectory structure. However, the exact layout depends on
;   how Tauri resolves the paths. This script searches multiple locations
;   to handle both preserved and flattened structures.
; ============================================================================

!include "LogicLib.nsh"

; ============================================================================
; POST-INSTALL HOOK - Runs after standard file installation
; ============================================================================
!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "MindLedger: Instalando librerias de runtime..."

  ; --- WebView2Loader.dll ---
  ; Search in subdirectory first, then flattened
  IfFileExists "$INSTDIR\resources\webview2\WebView2Loader.dll" 0 _try_webview2_flat
    CopyFiles /SILENT "$INSTDIR\resources\webview2\WebView2Loader.dll" "$INSTDIR"
    DetailPrint "  WebView2Loader.dll (from webview2\)"
    Goto _done_webview2
  _try_webview2_flat:
  IfFileExists "$INSTDIR\resources\WebView2Loader.dll" 0 _done_webview2
    CopyFiles /SILENT "$INSTDIR\resources\WebView2Loader.dll" "$INSTDIR"
    DetailPrint "  WebView2Loader.dll (flat)"
  _done_webview2:

  ; --- OpenSSL: libcrypto-3-*.dll ---
  ; Strategy: search $INSTDIR\resources\openssl\ first, then $INSTDIR\resources\,
  ; then any subdirectory of $INSTDIR\resources\ as last resort.
  DetailPrint "  Buscando libcrypto..."

  ; Location 1: $INSTDIR\resources\openssl\ (preserved structure)
  IfFileExists "$INSTDIR\resources\openssl\libcrypto-3-*.dll" 0 _try_crypto_flat
    FindFirst $R0 $R1 "$INSTDIR\resources\openssl\libcrypto-3-*.dll"
    StrCmp $R1 "" _try_crypto_flat
  _loop_crypto_sub:
    CopyFiles /SILENT "$INSTDIR\resources\openssl\$R1" "$INSTDIR"
    DetailPrint "  $R1 (from resources\openssl\)"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_crypto
    Goto _loop_crypto_sub
  FindClose $R0
  Goto _done_crypto

  ; Location 2: $INSTDIR\resources\ (flattened)
  _try_crypto_flat:
  IfFileExists "$INSTDIR\resources\libcrypto-3-*.dll" 0 _try_crypto_deep
    FindFirst $R0 $R1 "$INSTDIR\resources\libcrypto-3-*.dll"
    StrCmp $R1 "" _try_crypto_deep
  _loop_crypto_flat:
    CopyFiles /SILENT "$INSTDIR\resources\$R1" "$INSTDIR"
    DetailPrint "  $R1 (flat from resources\)"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_crypto
    Goto _loop_crypto_flat
  FindClose $R0
  Goto _done_crypto

  ; Location 3: any subdirectory of $INSTDIR\resources\ (last resort)
  _try_crypto_deep:
  FindFirst $R0 $R1 "$INSTDIR\resources\*\libcrypto-3-*.dll"
  StrCmp $R1 "" _done_crypto
  _loop_crypto_deep:
    CopyFiles /SILENT "$INSTDIR\resources\$R1" "$INSTDIR"
    DetailPrint "  $R1 (from resources\*\)"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_crypto
    Goto _loop_crypto_deep
  FindClose $R0

  _done_crypto:

  ; --- OpenSSL: libssl-3-*.dll ---
  DetailPrint "  Buscando libssl..."

  ; Location 1: $INSTDIR\resources\openssl\
  IfFileExists "$INSTDIR\resources\openssl\libssl-3-*.dll" 0 _try_ssl_flat
    FindFirst $R0 $R1 "$INSTDIR\resources\openssl\libssl-3-*.dll"
    StrCmp $R1 "" _try_ssl_flat
  _loop_ssl_sub:
    CopyFiles /SILENT "$INSTDIR\resources\openssl\$R1" "$INSTDIR"
    DetailPrint "  $R1 (from resources\openssl\)"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_ssl
    Goto _loop_ssl_sub
  FindClose $R0
  Goto _done_ssl

  ; Location 2: $INSTDIR\resources\ (flattened)
  _try_ssl_flat:
  IfFileExists "$INSTDIR\resources\libssl-3-*.dll" 0 _try_ssl_deep
    FindFirst $R0 $R1 "$INSTDIR\resources\libssl-3-*.dll"
    StrCmp $R1 "" _try_ssl_deep
  _loop_ssl_flat:
    CopyFiles /SILENT "$INSTDIR\resources\$R1" "$INSTDIR"
    DetailPrint "  $R1 (flat from resources\)"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_ssl
    Goto _loop_ssl_flat
  FindClose $R0
  Goto _done_ssl

  ; Location 3: any subdirectory of $INSTDIR\resources\
  _try_ssl_deep:
  FindFirst $R0 $R1 "$INSTDIR\resources\*\libssl-3-*.dll"
  StrCmp $R1 "" _done_ssl
  _loop_ssl_deep:
    CopyFiles /SILENT "$INSTDIR\resources\$R1" "$INSTDIR"
    DetailPrint "  $R1 (from resources\*\)"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_ssl
    Goto _loop_ssl_deep
  FindClose $R0

  _done_ssl:

  ; --- Final verification ---
  DetailPrint "Verificando librerias de runtime..."
  IfFileExists "$INSTDIR\libcrypto-3-x64.dll" 0 _verify_no_crypto
    DetailPrint "  OK: libcrypto-3-x64.dll"
    Goto _check_ssl
  _verify_no_crypto:
  IfFileExists "$INSTDIR\libcrypto-3-arm64.dll" 0 _verify_warn
    DetailPrint "  OK: libcrypto-3-arm64.dll"
    Goto _check_ssl
  _verify_warn:
    DetailPrint "  ADVERTENCIA: libcrypto no encontrado - la aplicacion puede fallar"

  _check_ssl:
  IfFileExists "$INSTDIR\libssl-3-x64.dll" 0 _check_ssl_arm
    DetailPrint "  OK: libssl-3-x64.dll"
    Goto _verify_done
  _check_ssl_arm:
  IfFileExists "$INSTDIR\libssl-3-arm64.dll" 0 _verify_done
    DetailPrint "  OK: libssl-3-arm64.dll"

  _verify_done:
  DetailPrint "MindLedger: Librerias de runtime instaladas."
!macroend

; ============================================================================
; UNINSTALL HOOKS
; ============================================================================
!macro NSIS_HOOK_UNINSTALL_PRE
  DetailPrint "Desinstalando MindLedger..."
!macroend

!macro NSIS_HOOK_UNINSTALL_POST
  DetailPrint "Limpiando librerias de runtime..."

  ; WebView2Loader.dll
  IfFileExists "$INSTDIR\WebView2Loader.dll" 0 _skip_webview2
    Delete "$INSTDIR\WebView2Loader.dll"
  _skip_webview2:

  ; libcrypto-3-*.dll
  FindFirst $R0 $R1 "$INSTDIR\libcrypto-3-*.dll"
  StrCmp $R1 "" _done_crypto_uninst
  _loop_crypto_uninst:
    Delete "$INSTDIR\$R1"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_crypto_uninst
    Goto _loop_crypto_uninst
  _done_crypto_uninst:
  FindClose $R0

  ; libssl-3-*.dll
  FindFirst $R0 $R1 "$INSTDIR\libssl-3-*.dll"
  StrCmp $R1 "" _done_ssl_uninst
  _loop_ssl_uninst:
    Delete "$INSTDIR\$R1"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_ssl_uninst
    Goto _loop_ssl_uninst
  _done_ssl_uninst:
  FindClose $R0

  DetailPrint "MindLedger desinstalado."
!macroend
