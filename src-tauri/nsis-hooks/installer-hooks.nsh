; ============================================================================
; MindLedger NSIS Installer Hooks
; ============================================================================
; Post-install: copy runtime DLLs (OpenSSL + WebView2) to $INSTDIR root.
; Searches 3 locations: subdirectory, flat, and any nested subdirectory.
; ============================================================================

!include "LogicLib.nsh"

!macro NSIS_HOOK_PREINSTALL
  DetailPrint "MindLedger: Preparando instalacion..."
!macroend

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "MindLedger: Instalando librerias de runtime..."

  ; === WebView2Loader.dll ===
  IfFileExists "$INSTDIR\resources\webview2\WebView2Loader.dll" 0 _wv2_flat
    CopyFiles /SILENT "$INSTDIR\resources\webview2\WebView2Loader.dll" "$INSTDIR"
    DetailPrint "  WebView2Loader.dll (webview2\)"
    Goto _wv2_done
  _wv2_flat:
  IfFileExists "$INSTDIR\resources\WebView2Loader.dll" 0 _wv2_done
    CopyFiles /SILENT "$INSTDIR\resources\WebView2Loader.dll" "$INSTDIR"
    DetailPrint "  WebView2Loader.dll (flat)"
  _wv2_done:

  ; === OpenSSL: libcrypto-3-*.dll (3-location search) ===
  ; Location 1: $INSTDIR\resources\openssl\
  IfFileExists "$INSTDIR\resources\openssl\libcrypto-3-*.dll" 0 _crypto_flat
    FindFirst $R0 $R1 "$INSTDIR\resources\openssl\libcrypto-3-*.dll"
    StrCmp $R1 "" _crypto_flat
  _crypto_sub_lp:
    CopyFiles /SILENT "$INSTDIR\resources\openssl\$R1" "$INSTDIR"
    DetailPrint "  $R1 (resources\openssl\)"
    FindNext $R0 $R1
    StrCmp $R1 "" _crypto_sub_dn
    Goto _crypto_sub_lp
  _crypto_sub_dn:
    FindClose $R0
    Goto _crypto_done

  ; Location 2: $INSTDIR\resources\ (flat)
  _crypto_flat:
  IfFileExists "$INSTDIR\resources\libcrypto-3-*.dll" 0 _crypto_deep
    FindFirst $R0 $R1 "$INSTDIR\resources\libcrypto-3-*.dll"
    StrCmp $R1 "" _crypto_deep
  _crypto_flat_lp:
    CopyFiles /SILENT "$INSTDIR\resources\$R1" "$INSTDIR"
    DetailPrint "  $R1 (resources\ flat)"
    FindNext $R0 $R1
    StrCmp $R1 "" _crypto_flat_dn
    Goto _crypto_flat_lp
  _crypto_flat_dn:
    FindClose $R0
    Goto _crypto_done

  ; Location 3: $INSTDIR\resources\*\ (any nested subdirectory)
  _crypto_deep:
  FindFirst $R0 $R1 "$INSTDIR\resources\*\libcrypto-3-*.dll"
  StrCmp $R1 "" _crypto_done
  _crypto_deep_lp:
    CopyFiles /SILENT "$INSTDIR\resources\$R1" "$INSTDIR"
    DetailPrint "  $R1 (resources\*\ deep)"
    FindNext $R0 $R1
    StrCmp $R1 "" _crypto_deep_dn
    Goto _crypto_deep_lp
  _crypto_deep_dn:
    FindClose $R0
  _crypto_done:

  ; === OpenSSL: libssl-3-*.dll (3-location search) ===
  ; Location 1: $INSTDIR\resources\openssl\
  IfFileExists "$INSTDIR\resources\openssl\libssl-3-*.dll" 0 _ssl_flat
    FindFirst $R0 $R1 "$INSTDIR\resources\openssl\libssl-3-*.dll"
    StrCmp $R1 "" _ssl_flat
  _ssl_sub_lp:
    CopyFiles /SILENT "$INSTDIR\resources\openssl\$R1" "$INSTDIR"
    DetailPrint "  $R1 (resources\openssl\)"
    FindNext $R0 $R1
    StrCmp $R1 "" _ssl_sub_dn
    Goto _ssl_sub_lp
  _ssl_sub_dn:
    FindClose $R0
    Goto _ssl_done

  ; Location 2: $INSTDIR\resources\ (flat)
  _ssl_flat:
  IfFileExists "$INSTDIR\resources\libssl-3-*.dll" 0 _ssl_deep
    FindFirst $R0 $R1 "$INSTDIR\resources\libssl-3-*.dll"
    StrCmp $R1 "" _ssl_deep
  _ssl_flat_lp:
    CopyFiles /SILENT "$INSTDIR\resources\$R1" "$INSTDIR"
    DetailPrint "  $R1 (resources\ flat)"
    FindNext $R0 $R1
    StrCmp $R1 "" _ssl_flat_dn
    Goto _ssl_flat_lp
  _ssl_flat_dn:
    FindClose $R0
    Goto _ssl_done

  ; Location 3: $INSTDIR\resources\*\ (any nested subdirectory)
  _ssl_deep:
  FindFirst $R0 $R1 "$INSTDIR\resources\*\libssl-3-*.dll"
  StrCmp $R1 "" _ssl_done
  _ssl_deep_lp:
    CopyFiles /SILENT "$INSTDIR\resources\$R1" "$INSTDIR"
    DetailPrint "  $R1 (resources\*\ deep)"
    FindNext $R0 $R1
    StrCmp $R1 "" _ssl_deep_dn
    Goto _ssl_deep_lp
  _ssl_deep_dn:
    FindClose $R0
  _ssl_done:

  ; === Verification ===
  DetailPrint "Verificando librerias de runtime..."
  IfFileExists "$INSTDIR\libcrypto-3-x64.dll" 0 _v_no_crypto
    DetailPrint "  OK: libcrypto-3-x64.dll"
    Goto _v_check_ssl
  _v_no_crypto:
  IfFileExists "$INSTDIR\libcrypto-3-arm64.dll" 0 _v_warn
    DetailPrint "  OK: libcrypto-3-arm64.dll"
    Goto _v_check_ssl
  _v_warn:
    DetailPrint "  ADVERTENCIA: libcrypto no encontrado"

  _v_check_ssl:
  IfFileExists "$INSTDIR\libssl-3-x64.dll" 0 _v_check_ssl_arm
    DetailPrint "  OK: libssl-3-x64.dll"
    Goto _v_done
  _v_check_ssl_arm:
  IfFileExists "$INSTDIR\libssl-3-arm64.dll" 0 _v_done
    DetailPrint "  OK: libssl-3-arm64.dll"
  _v_done:

  DetailPrint "MindLedger: Librerias de runtime instaladas."
!macroend

!macro NSIS_HOOK_UNINSTALL_PRE
  DetailPrint "Desinstalando MindLedger..."
!macroend

!macro NSIS_HOOK_UNINSTALL_POST
  DetailPrint "Limpiando librerias de runtime..."
  IfFileExists "$INSTDIR\WebView2Loader.dll" 0 _u_wv2
    Delete "$INSTDIR\WebView2Loader.dll"
  _u_wv2:
  FindFirst $R0 $R1 "$INSTDIR\libcrypto-3-*.dll"
  StrCmp $R1 "" _u_crypto_dn
  _u_crypto_lp:
    Delete "$INSTDIR\$R1"
    FindNext $R0 $R1
    StrCmp $R1 "" _u_crypto_dn
    Goto _u_crypto_lp
  _u_crypto_dn:
  FindClose $R0
  FindFirst $R0 $R1 "$INSTDIR\libssl-3-*.dll"
  StrCmp $R1 "" _u_ssl_dn
  _u_ssl_lp:
    Delete "$INSTDIR\$R1"
    FindNext $R0 $R1
    StrCmp $R1 "" _u_ssl_dn
    Goto _u_ssl_lp
  _u_ssl_dn:
  FindClose $R0
  DetailPrint "MindLedger desinstalado."
!macroend
