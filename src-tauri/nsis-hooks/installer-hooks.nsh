; ============================================================================
; MindLedger NSIS Installer Hooks
; ============================================================================
; Custom NSIS hooks for Tauri bundler to create a One-Click / Silent installer
; with minimal UI and first-run bootstrapping support.
; ============================================================================

!include "LogicLib.nsh"
!include "FileFunc.nsh"
!include "WinCore.nsh"

; ============================================================================
; GLOBAL VARIABLES
; ============================================================================
Var /GLOBAL MINDLEDGER_RUN_ON_FINISH
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
    ; In silent mode, disable UI completely
    SetSilent silent
  _not_silent:
  ${GetOptions} "$R0" "/SILENT" $R1
  StrCmp $R1 "1" 0 _not_silent2
    StrCpy $MINDLEDGER_SILENT_MODE "true"
    SetSilent silent
  _not_silent2:

  ; Pre-install: ensure clean state for DLLs
  DetailPrint "MindLedger: Preparando instalación..."
!macroend

; ============================================================================
; POST-INSTALL HOOK - Runs after standard file installation
; Copies runtime DLLs (OpenSSL + WebView2) to $INSTDIR root
; Architecture-agnostic: scans resources/openssl/ for ALL matching DLLs
; ============================================================================
!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Instalando librerías de runtime en la raíz de la aplicación..."

  ; --- WebView2Loader.dll ---
  IfFileExists "$INSTDIR\resources\webview2\WebView2Loader.dll" 0 _skip_webview2
    CreateDirectory "$INSTDIR"
    CopyFiles /SILENT "$INSTDIR\resources\webview2\WebView2Loader.dll" "$INSTDIR"
    DetailPrint "Copiado: WebView2Loader.dll -> $INSTDIR"
  _skip_webview2:

  ; --- OpenSSL DLLs: libcrypto-3-*.dll (architecture-agnostic scan) ---
  DetailPrint "Buscando librerías OpenSSL (libcrypto-3-*)..."
  FindFirst $R0 $R1 "$INSTDIR\resources\openssl\libcrypto-3-*.dll"
  StrCmp $R1 "" _done_crypto
  _loop_crypto:
    CreateDirectory "$INSTDIR"
    CopyFiles /SILENT "$INSTDIR\resources\openssl\$R1" "$INSTDIR"
    DetailPrint "Copiado: $R1 -> $INSTDIR"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_crypto
    Goto _loop_crypto
  _done_crypto:
  FindClose $R0

  ; --- OpenSSL DLLs: libssl-3-*.dll (architecture-agnostic scan) ---
  DetailPrint "Buscando librerías OpenSSL (libssl-3-*)..."
  FindFirst $R0 $R1 "$INSTDIR\resources\openssl\libssl-3-*.dll"
  StrCmp $R1 "" _done_ssl
  _loop_ssl:
    CreateDirectory "$INSTDIR"
    CopyFiles /SILENT "$INSTDIR\resources\openssl\$R1" "$INSTDIR"
    DetailPrint "Copiado: $R1 -> $INSTDIR"
    FindNext $R0 $R1
    StrCmp $R1 "" _done_ssl
    Goto _loop_ssl
  _done_ssl:
  FindClose $R0

  DetailPrint "Librerías de runtime instaladas correctamente."
!macroend

; ============================================================================
; CUSTOM UI PAGES - Minimal "One-Click" Installer Flow
; ============================================================================

; Custom page: Simplified Welcome / Install page (replaces default welcome + dir + install pages)
!macro MINDLEDGER_CUSTOM_INSTALL_PAGE
  !define MUI_PAGE_CUSTOMFUNCTION_PRE MINDLEDGER_ON_INSTALL_PAGE_PRE
  !define MUI_PAGE_CUSTOMFUNCTION_LEAVE MINDLEDGER_ON_INSTALL_PAGE_LEAVE
  Page Custom MINDLEDGER_CREATE_INSTALL_PAGE MINDLEDGER_DESTROY_INSTALL_PAGE
!macroend

; Custom page: Simplified Finish page with "Run MindLedger" checkbox
!macro MINDLEDGER_CUSTOM_FINISH_PAGE
  !define MUI_PAGE_CUSTOMFUNCTION_PRE MINDLEDGER_ON_FINISH_PAGE_PRE
  !define MUI_PAGE_CUSTOMFUNCTION_LEAVE MINDLEDGER_ON_FINISH_PAGE_LEAVE
  Page Custom MINDLEDGER_CREATE_FINISH_PAGE MINDLEDGER_DESTROY_FINISH_PAGE
!macroend

; ============================================================================
; CUSTOM PAGE: INSTALL (Minimal UI - Just "Install" button)
; ============================================================================
Function MINDLEDGER_CREATE_INSTALL_PAGE
  ; Skip custom page entirely in silent mode
  StrCmp $MINDLEDGER_SILENT_MODE "true" 0 _create_page
    Abort
  _create_page:

  ; Create a simple page with minimal UI
  nsDialogs::Create 1018
  Pop $R0

  ; Title
  ${NSD_CreateLabel} 0 10 100% 30u "MindLedger - Instalación Simplificada"
  Pop $R0
  SendMessage $R0 ${WM_SETFONT} ${MSGOTHIC_FONT} 1

  ; Subtitle
  ${NSD_CreateLabel} 0 40u 100% 20u "El software se instalará en el directorio predeterminado:"
  Pop $R0

  ; Install path display (read-only)
  ${NSD_CreateLabel} 20 65u 90% 15u "$INSTDIR"
  Pop $R0
  EnableWindow $R0 0

  ; Install button (big, prominent)
  ${NSD_CreateButton} 40% 100u 20% 30u "Instalar"
  Pop $MINDLEDGER_INSTALL_BTN
  ${NSD_OnClick} $MINDLEDGER_INSTALL_BTN MINDLEDGER_ON_INSTALL_CLICK

  nsDialogs::Show
FunctionEnd

Function MINDLEDGER_DESTROY_INSTALL_PAGE
  nsDialogs::Destroy
FunctionEnd

Function MINDLEDGER_ON_INSTALL_PAGE_PRE
  ; Set install directory to default (per-user AppData or Program Files)
  ; Using per-user install by default for no-UAC experience
  StrCmp $MINDLEDGER_SILENT_MODE "true" 0 _not_silent_pre
    ; In silent mode, just continue
    Abort
  _not_silent_pre:
FunctionEnd

Function MINDLEDGER_ON_INSTALL_CLICK
  ; User clicked "Install" - proceed to installation
  ; Disable button to prevent double-click
  EnableWindow $MINDLEDGER_INSTALL_BTN 0
  SetWindowText $MINDLEDGER_INSTALL_BTN "Instalando..."
  ; Signal NSIS to continue to the actual install phase
  Push "next"
  GetFunctionAddress $R0 MINDLEDGER_INSTALL_DONE
  Call $R0
FunctionEnd

Function MINDLEDGER_INSTALL_DONE
  ; This is called after the actual file installation completes
  ; We just continue to the next page (Finish)
FunctionEnd

Function MINDLEDGER_ON_INSTALL_PAGE_LEAVE
  ; Nothing special needed
FunctionEnd

; ============================================================================
; CUSTOM PAGE: FINISH (with "Run MindLedger" checkbox)
; ============================================================================
Function MINDLEDGER_CREATE_FINISH_PAGE
  StrCmp $MINDLEDGER_SILENT_MODE "true" 0 _create_finish
    Abort
  _create_finish:

  nsDialogs::Create 1018
  Pop $R0

  ; Success message
  ${NSD_CreateLabel} 0 10 100% 30u "¡MindLedger se ha instalado correctamente!"
  Pop $R0
  SendMessage $R0 ${WM_SETFONT} ${MSGOTHIC_FONT} 1

  ; Description
  ${NSD_CreateLabel} 0 50u 100% 20u "La aplicación está lista para usarse. La base de datos y las claves"
  Pop $R0
  ${NSD_CreateLabel} 0 70u 100% 20u "criptográficas se inicializarán automáticamente en el primer inicio."
  Pop $R0

  ; "Run MindLedger" checkbox
  ${NSD_CreateCheckbox} 20 105u 90% 20u "Ejecutar MindLedger ahora"
  Pop $MINDLEDGER_RUN_CHECKBOX
  SendMessage $MINDLEDGER_RUN_CHECKBOX ${BM_SETCHECK} ${BST_CHECKED} 0
  StrCpy $MINDLEDGER_RUN_ON_FINISH "true"

  ${NSD_OnClick} $MINDLEDGER_RUN_CHECKBOX MINDLEDGER_ON_RUN_CHECKBOX_CLICK

  ; Finish button
  ${NSD_CreateButton} 40% 145u 20% 30u "Finalizar"
  Pop $R0
  ${NSD_OnClick} $R0 MINDLEDGER_ON_FINISH_CLICK

  nsDialogs::Show
FunctionEnd

Function MINDLEDGER_DESTROY_FINISH_PAGE
  nsDialogs::Destroy
FunctionEnd

Function MINDLEDGER_ON_RUN_CHECKBOX_CLICK
  Pop $R0
  ${NSD_GetState} $MINDLEDGER_RUN_CHECKBOX $R0
  StrCmp $R0 ${BST_CHECKED} 0 _unchecked
    StrCpy $MINDLEDGER_RUN_ON_FINISH "true"
    Goto _done_cb
  _unchecked:
    StrCpy $MINDLEDGER_RUN_ON_FINISH "false"
  _done_cb:
FunctionEnd

Function MINDLEDGER_ON_FINISH_PAGE_PRE
  StrCmp $MINDLEDGER_SILENT_MODE "true" 0 _not_silent_finish_pre
    ; In silent mode, don't run the app automatically unless explicitly requested
    StrCpy $MINDLEDGER_RUN_ON_FINISH "false"
    Abort
  _not_silent_finish_pre:
FunctionEnd

Function MINDLEDGER_ON_FINISH_CLICK
  ; Check if we should run the app
  StrCmp $MINDLEDGER_RUN_ON_FINISH "true" 0 _no_run
    ; Launch the installed application
    Exec '"$INSTDIR\MindLedger.exe"'
  _no_run:
  ; Close installer
  Quit
FunctionEnd

Function MINDLEDGER_ON_FINISH_PAGE_LEAVE
FunctionEnd

; ============================================================================
; MUI PAGE OVERRIDES - Inject our custom pages into the standard flow
; ============================================================================
!macro MINDLEDGER_INSERT_CUSTOM_PAGES
  ; Replace standard pages with our minimal flow:
  ; 1. Custom Install page (instead of Welcome + Dir + Install)
  ; 2. Custom Finish page (instead of standard Finish)
  !insertmacro MINDLEDGER_CUSTOM_INSTALL_PAGE
  !insertmacro MINDLEDGER_CUSTOM_FINISH_PAGE
!macroend

; ============================================================================
; UNINSTALL HOOKS
; ============================================================================
!macro NSIS_HOOK_UNINSTALL_PRE
  DetailPrint "Desinstalando MindLedger..."
!macroend

!macro NSIS_HOOK_UNINSTALL_POST
  ; Clean up runtime DLLs that were copied to root (architecture-agnostic)
  DetailPrint "Limpiando librerías de runtime..."

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
; Usage: MindLedger_Setup.exe /S    or    MindLedger_Setup.exe /SILENT
; ============================================================================
; The silent mode is detected in NSIS_HOOK_PREINSTALL above.
; In silent mode:
; - No UI pages are shown
; - Install directory defaults to $LOCALAPPDATA\MindLedger (per-user, no UAC)
; - No "Run on finish" (unless we add /RUN flag support)
; - Auto-close on completion
; ============================================================================