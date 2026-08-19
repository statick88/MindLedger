; ============================================================================
; MindLedger NSIS Installer Hooks
; ============================================================================
; Custom NSIS hooks for Tauri bundler:
;   - Silent mode detection (/S or /SILENT)
;   - Post-install: copy runtime DLLs (OpenSSL + WebView2) to $INSTDIR root
;   - Uninstall: clean up runtime DLLs
;
; DLL locations after Tauri install:
;   Tauri v2 MAY flatten resources, so DLLs could be at either:
;     $INSTDIR\resources\openssl\libcrypto-3-x64.dll  (preserved structure)
;     $INSTDIR\resources\libcrypto-3-x64.dll          (flattened)
;   This script checks BOTH locations.
; ============================================================================

!include "LogicLib.nsh"
!include "FileFunc.nsh"

; ============================================================================
; GLOBAL VARIABLES
; ============================================================================
Var /GLOBAL MINDLEDGER_SILENT_MODE

; ============================================================================
; PRE-INSTALL HOOK - Runs before any installation
; ============================================================================
!macro NSIS_HOOK_PREINSTALL
  ; Detect silent mode (/S or /SILENT)
  StrCpy $MINDLEDGER_SILENT_MODE "false"
  ${GetParameters} $R0
  ${GetOptions} "$R0" "/S" $R1
  StrCmp $R1 "1" 0 _not_silent
    StrCpy $MINDLEDGER_SILENT_MODE "true"
    SetSilent silent
  _not_silent:
  ${GetOptions} "$R0" "/SILENT" $R1
  StrCmp $R1 "1" 0 _not_silent2
    StrCpy $MINDLEDGER_SILENT_MODE "true"
    SetSilent silent
  _not_silent2:

  DetailPrint "MindLedger: Preparando instalacion..."
!macroend

; ============================================================================
; POST-INSTALL HOOK - Runs after standard file installation
; Copies runtime DLLs (OpenSSL + WebView2) to $INSTDIR root
; ============================================================================
!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Instalando librerias de runtime en la raiz de la aplicacion..."

  ; --- WebView2Loader.dll ---
  ; Check subdirectory first, then flattened location
  IfFileExists "$INSTDIR\resources\webview2\WebView2Loader.dll" 0 _try_webview2_flat
    CreateDirectory "$INSTDIR"
    CopyFiles /SILENT "$INSTDIR\resources\webview2\WebView2Loader.dll" "$INSTDIR"
    DetailPrint "Copiado: WebView2Loader.dll -> $INSTDIR"
    Goto _done_webview2
  _try_webview2_flat:
  IfFileExists "$INSTDIR\resources\WebView2Loader.dll" 0 _done_webview2
    CreateDirectory "$INSTDIR"
    CopyFiles /SILENT "$INSTDIR\resources\WebView2Loader.dll" "$INSTDIR"
    DetailPrint "Copiado: WebView2Loader.dll -> $INSTDIR (flat)"
  _done_webview2:

  ; --- OpenSSL: libcrypto-3-*.dll ---
  ; Try subdirectory first ($INSTDIR\resources\openssl\)
  DetailPrint "Buscando librerias OpenSSL (libcrypto-3-*)..."
  FindFirst $R0 $R1 "$INSTDIR\resources\openssl\libcrypto-3-*.dll"
  StrCmp $R1 "" _try_crypto_flat
  _loop_crypto:
    CreateDirectory "$INSTDIR"
    CopyFiles /SILENT "$INSTDIR\resources\openssl\$R1" "$INSTDIR"
    DetailPrint "Copiado: $R1 -> $INSTDIR"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_crypto
    Goto _loop_crypto
  _try_crypto_flat:
  ; Fallback: flattened location ($INSTDIR\resources\)
  FindFirst $R0 $R1 "$INSTDIR\resources\libcrypto-3-*.dll"
  StrCmp $R1 "" _done_crypto
  _loop_crypto_flat:
    CreateDirectory "$INSTDIR"
    CopyFiles /SILENT "$INSTDIR\resources\$R1" "$INSTDIR"
    DetailPrint "Copiado: $R1 -> $INSTDIR (flat)"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_crypto
    Goto _loop_crypto_flat
  _done_crypto:
  FindClose $R0

  ; --- OpenSSL: libssl-3-*.dll ---
  ; Try subdirectory first
  DetailPrint "Buscando librerias OpenSSL (libssl-3-*)..."
  FindFirst $R0 $R1 "$INSTDIR\resources\openssl\libssl-3-*.dll"
  StrCmp $R1 "" _try_ssl_flat
  _loop_ssl:
    CreateDirectory "$INSTDIR"
    CopyFiles /SILENT "$INSTDIR\resources\openssl\$R1" "$INSTDIR"
    DetailPrint "Copiado: $R1 -> $INSTDIR"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_ssl
    Goto _loop_ssl
  _try_ssl_flat:
  ; Fallback: flattened location
  FindFirst $R0 $R1 "$INSTDIR\resources\libssl-3-*.dll"
  StrCmp $R1 "" _done_ssl
  _loop_ssl_flat:
    CreateDirectory "$INSTDIR"
    CopyFiles /SILENT "$INSTDIR\resources\$R1" "$INSTDIR"
    DetailPrint "Copiado: $R1 -> $INSTDIR (flat)"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_ssl
    Goto _loop_ssl_flat
  _done_ssl:
  FindClose $R0

  ; --- Final verification ---
  IfFileExists "$INSTDIR\libcrypto-3-x64.dll" 0 _warn_missing
    DetailPrint "Verificacion: libcrypto-3-x64.dll encontrado en $INSTDIR"
    Goto _verify_done
  _warn_missing:
    DetailPrint "ADVERTENCIA: libcrypto-3-x64.dll no encontrado en $INSTDIR - la aplicacion puede fallar"
  _verify_done:

  DetailPrint "Librerias de runtime instaladas correctamente."
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
  IfFileExists "$INSTDIR\WebView2Loader.dll" 0 _skip_webview2_uninst
    Delete "$INSTDIR\WebView2Loader.dll"
    DetailPrint "Eliminado: WebView2Loader.dll"
  _skip_webview2_uninst:

  ; libcrypto-3-*.dll
  FindFirst $R0 $R1 "$INSTDIR\libcrypto-3-*.dll"
  StrCmp $R1 "" _done_crypto_uninst
  _loop_crypto_uninst:
    Delete "$INSTDIR\$R1"
    DetailPrint "Eliminado: $R1"
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
    DetailPrint "Eliminado: $R1"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_ssl_uninst
    Goto _loop_ssl_uninst
  _done_ssl_uninst:
  FindClose $R0

  DetailPrint "MindLedger desinstalado completamente."
!macroend

; ============================================================================
; SILENT INSTALL SUPPORT
; ============================================================================
; Usage: MindLedger_Setup.exe /S    or    MindLedger_Setup.exe /SILENT
;
; In silent mode:
; - No UI pages are shown
; - Install directory defaults to $LOCALAPPDATA\MindLedger (per-user, no UAC)
; - Auto-close on completion
; ============================================================================
