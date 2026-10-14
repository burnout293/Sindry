; Crochets de l'installeur NSIS de Sindry.
; Branchés par tauri.conf.json (bundle > windows > nsis > installerHooks), et
; complétés par le modèle dérivé de Tauri (tools/modele-installeur.mjs), qui
; insère SINDRY_PAGE_DESINSTALLATION et SINDRY_EFFACER_ARBRE.
;
; DÉSINSTALLATION — la page de Sindry remplace celle de Tauri :
;
;   ( ) Garder mes données                       rien n'est effacé
;   ( ) Tout effacer, sauf mes scripts personnels
;   ( ) Choisir ce qui est effacé : réglages, historique, journaux, catalogue,
;       autorisations des scripts, emplacements protégés, données de
;       l'interface, scripts personnels
;
; Une mise à jour, une désinstallation silencieuse ou passive n'efface rien.
;
; SÉCURITÉ. Le désinstalleur tourne en administrateur, et la plupart de ces
; dossiers sont inscriptibles par n'importe quel programme du compte. Or
; « RMDir /r » de NSIS SUIT les jonctions (vérifié) : une jonction glissée dans
; l'un d'eux vers System32 ferait vider System32. Aucune suppression ici ne
; passe donc par « RMDir /r » : SINDRY_EFFACER_ARBRE parcourt lui-même les
; dossiers, et retire un lien rencontré sans jamais descendre dedans.

!include nsDialogs.nsh
!include FileFunc.nsh
!include LogicLib.nsh

; --- Effacement sûr ---------------------------------------------------------

; Efface un dossier et son contenu, sans jamais suivre une jonction ni un lien :
; un lien rencontré est retiré lui-même (le lien, pas sa cible). Paramètre : le
; chemin, sur la pile.
Function un.WtEffacerArbre
  Exch $R0
  Push $R1
  Push $R2
  Push $R3
  ${GetFileAttributes} "$R0" "REPARSE_POINT" $R3
  ${If} $R3 == 1
    RMDir "$R0"
    Goto wt_arbre_fin
  ${EndIf}
  ClearErrors
  FindFirst $R1 $R2 "$R0\*.*"
  wt_arbre_boucle:
    IfErrors wt_arbre_vu
    StrCmp $R2 "" wt_arbre_vu
    StrCmp $R2 "." wt_arbre_suivant
    StrCmp $R2 ".." wt_arbre_suivant
    ${If} ${FileExists} "$R0\$R2\*.*"
      Push "$R0\$R2"
      Call un.WtEffacerArbre
    ${Else}
      Delete "$R0\$R2"
    ${EndIf}
    wt_arbre_suivant:
    ClearErrors
    FindNext $R1 $R2
    Goto wt_arbre_boucle
  wt_arbre_vu:
  FindClose $R1
  RMDir "$R0"
  wt_arbre_fin:
  Pop $R3
  Pop $R2
  Pop $R1
  Pop $R0
FunctionEnd

!macro SINDRY_EFFACER_ARBRE CHEMIN
  ${If} ${FileExists} "${CHEMIN}\*.*"
    Push "${CHEMIN}"
    Call un.WtEffacerArbre
  ${EndIf}
!macroend

; Les données de Sindry sous BASE (%LOCALAPPDATA%\Sindry), selon les choix
; de la page. Rien du tout si BASE est lui-même un lien.
!macro SINDRY_EFFACER_DONNEES BASE
  Push $R8
  ${GetFileAttributes} "${BASE}" "REPARSE_POINT" $R8
  ${If} $R8 != 1
    ${If} $WtReglages == 1
      Delete "${BASE}\settings.json"
      Delete "${BASE}\settings-export.json"
    ${EndIf}
    ${If} $WtHistorique == 1
      Delete "${BASE}\history.json"
    ${EndIf}
    ${If} $WtJournaux == 1
      !insertmacro SINDRY_EFFACER_ARBRE "${BASE}\logs"
    ${EndIf}
    ${If} $WtCatalogue == 1
      !insertmacro SINDRY_EFFACER_ARBRE "${BASE}\sources"
    ${EndIf}
    ${If} $WtScripts == 1
      !insertmacro SINDRY_EFFACER_ARBRE "${BASE}\scripts"
    ${EndIf}
    ; Le dossier ne part que s'il est vide : ce qui n'a pas été choisi reste.
    RMDir "${BASE}"
  ${EndIf}
  Pop $R8
!macroend

; --- La page de désinstallation ---------------------------------------------

; Texte selon la langue de l'installeur (1036 : français).
!macro WT_TEXTE VAR FR EN
  ${If} $LANGUAGE = 1036
    StrCpy ${VAR} "${FR}"
  ${Else}
    StrCpy ${VAR} "${EN}"
  ${EndIf}
!macroend

!macro WT_CASE VAR X Y FR EN
  !insertmacro WT_TEXTE $R0 "${FR}" "${EN}"
  ${NSD_CreateCheckbox} ${X} ${Y} 44% 10u "$R0"
  Pop ${VAR}
!macroend

; La page « déjà installé » (1.7) : $R2 est le premier choix, coché d'office ;
; $R3 le second. Insérées par le modèle dérivé (tools/modele-installeur.mjs).
!macro SINDRY_TEXTES_MISE_A_JOUR
  !insertmacro WT_TEXTE $R2 "Mettre à jour (recommandé) — vos réglages et vos scripts sont conservés" "Update (recommended) — your settings and scripts are kept"
  !insertmacro WT_TEXTE $R3 "Désinstaller d'abord, puis installer" "Uninstall first, then install"
!macroend

!macro SINDRY_TEXTES_REPARATION
  !insertmacro WT_TEXTE $R2 "Réparer — réinstaller les fichiers de Sindry, sans toucher à vos réglages" "Repair — reinstall Sindry's files, without touching your settings"
  !insertmacro WT_TEXTE $R3 "Désinstaller Sindry" "Uninstall Sindry"
!macroend

; Insérée par le modèle dérivé à la place de la page de confirmation de Tauri,
; APRÈS la déclaration de ses variables ($PassiveMode, $UpdateMode,
; $DeleteAppDataCheckboxState) : c'est ce qui permet de les utiliser ici.
!macro SINDRY_PAGE_DESINSTALLATION
  Var WtDialogue
  Var WtRadioGarder
  Var WtRadioSimple
  Var WtRadioAvance
  Var WtCaseReglages
  Var WtCaseHistorique
  Var WtCaseJournaux
  Var WtCaseCatalogue
  Var WtCaseApprobations
  Var WtCaseGarde
  Var WtCaseInterface
  Var WtCaseScripts
  Var WtReglages
  Var WtHistorique
  Var WtJournaux
  Var WtCatalogue
  Var WtApprobations
  Var WtGarde
  Var WtInterface
  Var WtScripts

  UninstPage custom un.WtPageCreer un.WtPageQuitter

  Function un.WtPageCreer
    ; Une mise à jour ou une désinstallation passive ne demande rien, et
    ; n'efface rien.
    ${If} $PassiveMode = 1
    ${OrIf} $UpdateMode = 1
      Abort
    ${EndIf}

    !insertmacro WT_TEXTE $R1 "Désinstaller Sindry" "Uninstall Sindry"
    !insertmacro WT_TEXTE $R2 "Que doit-il rester de Sindry sur ce PC ?" "What should remain of Sindry on this PC?"
    !insertmacro MUI_HEADER_TEXT "$R1" "$R2"

    nsDialogs::Create 1018
    Pop $WtDialogue
    ${If} $WtDialogue == error
      Abort
    ${EndIf}

    !insertmacro WT_TEXTE $R0 "Les scripts que vous avez écrits ou ajoutés vous appartiennent : ils ne sont effacés que si vous le demandez." "The scripts you wrote or added are yours: they are only deleted if you ask for it."
    ${NSD_CreateLabel} 0 0 100% 20u "$R0"
    Pop $R0

    !insertmacro WT_TEXTE $R0 "Garder mes données (pour une réinstallation, par exemple)" "Keep my data (to reinstall later, for example)"
    ${NSD_CreateRadioButton} 0 24u 100% 10u "$R0"
    Pop $WtRadioGarder
    ${NSD_AddStyle} $WtRadioGarder ${WS_GROUP}
    !insertmacro WT_TEXTE $R0 "Tout effacer, sauf mes scripts personnels" "Delete everything except my personal scripts"
    ${NSD_CreateRadioButton} 0 36u 100% 10u "$R0"
    Pop $WtRadioSimple
    !insertmacro WT_TEXTE $R0 "Choisir ce qui est effacé :" "Choose what is deleted:"
    ${NSD_CreateRadioButton} 0 48u 100% 10u "$R0"
    Pop $WtRadioAvance

    !insertmacro WT_CASE $WtCaseReglages 12u 62u "Réglages" "Settings"
    !insertmacro WT_CASE $WtCaseHistorique 56% 62u "Historique" "History"
    !insertmacro WT_CASE $WtCaseJournaux 12u 74u "Journaux" "Logs"
    !insertmacro WT_CASE $WtCaseCatalogue 56% 74u "Catalogue installé" "Installed catalogue"
    !insertmacro WT_CASE $WtCaseApprobations 12u 86u "Scripts approuvés" "Approved scripts"
    !insertmacro WT_CASE $WtCaseGarde 56% 86u "Emplacements protégés" "Protected locations"
    !insertmacro WT_CASE $WtCaseInterface 12u 98u "Données de l'interface" "Interface data"
    !insertmacro WT_CASE $WtCaseScripts 56% 98u "Mes scripts personnels" "My personal scripts"

    ${NSD_OnClick} $WtRadioGarder un.WtSurChoix
    ${NSD_OnClick} $WtRadioSimple un.WtSurChoix
    ${NSD_OnClick} $WtRadioAvance un.WtSurChoix
    ${NSD_Check} $WtRadioGarder
    ; Comme un clic : la fonction attend le contrôle sur la pile.
    Push $WtRadioGarder
    Call un.WtSurChoix

    ; Cette page est la dernière avant l'effacement : son bouton le dit.
    GetDlgItem $R0 $HWNDPARENT 1
    SendMessage $R0 ${WM_SETTEXT} 0 "STR:$(^UninstallBtn)"

    nsDialogs::Show
  FunctionEnd

  ; « Garder » décoche tout, « Tout sauf mes scripts » coche tout sauf les
  ; scripts ; dans les deux cas les cases montrent le résultat sans se
  ; modifier. « Choisir » les rend modifiables.
  Function un.WtSurChoix
    Pop $R9
    ${NSD_GetState} $WtRadioAvance $R0
    ${NSD_GetState} $WtRadioSimple $R1
    ${If} $R0 == ${BST_CHECKED}
      StrCpy $R2 1
    ${Else}
      StrCpy $R2 0
      ${If} $R1 == ${BST_CHECKED}
        StrCpy $R3 ${BST_CHECKED}
      ${Else}
        StrCpy $R3 ${BST_UNCHECKED}
      ${EndIf}
      SendMessage $WtCaseReglages ${BM_SETCHECK} $R3 0
      SendMessage $WtCaseHistorique ${BM_SETCHECK} $R3 0
      SendMessage $WtCaseJournaux ${BM_SETCHECK} $R3 0
      SendMessage $WtCaseCatalogue ${BM_SETCHECK} $R3 0
      SendMessage $WtCaseApprobations ${BM_SETCHECK} $R3 0
      SendMessage $WtCaseGarde ${BM_SETCHECK} $R3 0
      SendMessage $WtCaseInterface ${BM_SETCHECK} $R3 0
      SendMessage $WtCaseScripts ${BM_SETCHECK} ${BST_UNCHECKED} 0
    ${EndIf}
    EnableWindow $WtCaseReglages $R2
    EnableWindow $WtCaseHistorique $R2
    EnableWindow $WtCaseJournaux $R2
    EnableWindow $WtCaseCatalogue $R2
    EnableWindow $WtCaseApprobations $R2
    EnableWindow $WtCaseGarde $R2
    EnableWindow $WtCaseInterface $R2
    EnableWindow $WtCaseScripts $R2
  FunctionEnd

  Function un.WtPageQuitter
    ${NSD_GetState} $WtCaseReglages $WtReglages
    ${NSD_GetState} $WtCaseHistorique $WtHistorique
    ${NSD_GetState} $WtCaseJournaux $WtJournaux
    ${NSD_GetState} $WtCaseCatalogue $WtCatalogue
    ${NSD_GetState} $WtCaseApprobations $WtApprobations
    ${NSD_GetState} $WtCaseGarde $WtGarde
    ${NSD_GetState} $WtCaseInterface $WtInterface
    ${NSD_GetState} $WtCaseScripts $WtScripts
    ${If} $WtScripts == 1
      !insertmacro WT_TEXTE $R0 "Vos scripts personnels seront effacés définitivement, y compris ceux que vous avez écrits vous-même.$\n$\nContinuer ?" "Your personal scripts will be permanently deleted, including the ones you wrote yourself.$\n$\nContinue?"
      MessageBox MB_YESNO|MB_ICONEXCLAMATION|MB_DEFBUTTON2 "$R0" IDYES wt_scripts_confirmes
      Abort
      wt_scripts_confirmes:
    ${EndIf}
    ; Les données de l'interface : c'est la section de Tauri qui les efface,
    ; par l'effacement sûr que le modèle dérivé y substitue.
    StrCpy $DeleteAppDataCheckboxState $WtInterface
  FunctionEnd
!macroend

; --- Après la désinstallation des fichiers ---------------------------------

; Migration depuis WinTool (2.0) : apres l'installation, en administrateur et
; sans fenetre, Sindry reprend les donnees et les reglages de WinTool, puis le
; desinstalle (voir src-tauri/src/migration.rs). Sans effet s'il n'y a rien a
; reprendre — une mise a jour de Sindry par exemple.
; Le raccourci de bureau de l'ancien nom, note avant que sa desinstallation
; (par la migration) ne le retire.
Var SindryAncienBureau

!macro NSIS_HOOK_POSTINSTALL
  ; La ligne de commande (2.0, §19) : Tauri installe d'office les autres
  ; binaires du paquet, dont sindry-cli.exe. Il prend ici le nom sindry.com —
  ; « sindry » dans une console le trouve avant sindry.exe (ordre de PATHEXT).
  ; Renomme, pas copie : un seul exemplaire sur le disque.
  Delete "$INSTDIR\sindry.com"
  Rename "$INSTDIR\sindry-cli.exe" "$INSTDIR\sindry.com"

  StrCpy $SindryAncienBureau 0
  ${If} ${FileExists} "$DESKTOP\WinTool.lnk"
    StrCpy $SindryAncienBureau 1
  ${EndIf}

  nsExec::Exec '"$INSTDIR\${MAINBINARYNAME}.exe" --migrer-depuis-wintool'
  Pop $0

  ; Les raccourcis (2.0.1). Arrivee par la mise a jour automatique de la 1.8,
  ; l'installeur est lance en mode mise a jour (/UPDATE) : le modele de Tauri
  ; ne cree alors aucun raccourci, il croit les avoir deja — et la migration
  ; venait de retirer ceux de l'ancien nom. La 2.0.0 laissait ainsi Sindry hors
  ; du menu Demarrer. Une seule fois par installation (marque dans la cle de
  ; desinstallation) : un raccourci retire expres ensuite n'est pas recree.
  ReadRegDWORD $0 SHCTX "${UNINSTKEY}" "SindryRaccourcis"
  ${If} $0 <> 1
  ${AndIf} $NoShortcutMode <> 1
    ${IfNot} ${FileExists} "$SMPROGRAMS\${PRODUCTNAME}.lnk"
      CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
      !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\${PRODUCTNAME}.lnk"
    ${EndIf}
    ${If} $SindryAncienBureau = 1
      ${IfNot} ${FileExists} "$DESKTOP\${PRODUCTNAME}.lnk"
        CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
        !insertmacro SetLnkAppUserModelId "$DESKTOP\${PRODUCTNAME}.lnk"
      ${EndIf}
    ${EndIf}
    WriteRegDWORD SHCTX "${UNINSTKEY}" "SindryRaccourcis" 1
  ${EndIf}

  ; Installation silencieuse avec options (§19.3) : /LANG= /THEME= /AMBIANCE=
  ; /SANSBIENVENUE /CONFIG= /PATH. sindry.com les lit dans la ligne de commande
  ; de l'installeur, les verifie, et ecrit preconfiguration.json a cote de lui.
  ; Sans option, il n'ecrit rien. Une mise a jour n'y touche pas.
  ${If} $UpdateMode <> 1
    nsExec::ExecToLog '"$INSTDIR\sindry.com" --installeur preconfigurer $CMDLINE'
    Pop $0
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ${If} $UpdateMode <> 1
    ; Le dossier d'installation quitte le PATH du systeme, s'il y etait (/PATH),
    ; et la preconfiguration part avec Sindry.
    nsExec::Exec '"$INSTDIR\sindry.com" --installeur chemin retirer'
    Pop $0
    Delete "$INSTDIR\preconfiguration.json"
  ${EndIf}
  Delete "$INSTDIR\sindry.com"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    ; Le registre 64 bits, celui qu'écrit Sindry.
    SetRegView 64
    ${If} $WtApprobations == 1
      DeleteRegKey HKLM "SOFTWARE\Sindry\Approbations"
    ${EndIf}
    ${If} $WtGarde == 1
      DeleteRegKey HKLM "SOFTWARE\Sindry\Garde"
    ${EndIf}
    DeleteRegKey /ifempty HKLM "SOFTWARE\Sindry"

    SetShellVarContext current
    !insertmacro SINDRY_EFFACER_DONNEES "$LOCALAPPDATA\Sindry"

    ; Ancien magasin d'approbations (1.0 à 1.1.1) : un fichier nommé, puis le
    ; dossier s'il est vide.
    ${If} $WtApprobations == 1
      SetShellVarContext all
      Delete "$APPDATA\Sindry\approved.json"
      RMDir "$APPDATA\Sindry"
    ${EndIf}
  ${EndIf}
!macroend
