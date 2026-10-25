# Sindry — les consignes qui changent, version par version

> **Pour une IA qui a déjà une version précédente de `SCRIPTS_POUR_IA.md`** (ou de sa
> version courte) : seules les consignes **nouvelles ou modifiées**, sans le reste. Donnez
> ce fichier en plus de l'ancien guide. En cas de doute, la version complète à jour fait
> foi.
>
> Les versions les plus récentes viennent en premier. Une IA restée sur une version plus
> ancienne lit toutes les entrées jusqu'à la sienne, de bas en haut.

<!-- empreinte des consignes : d3395f8a8e29c1ab — tenue par tools/build-script-guides.mjs, ne pas modifier -->

---

## 2.3.0 — depuis la 2.2.0 : expliquer un échec, réappliquer ce qui est en place

Deux nouveautés : un échec s'**explique** (code, cible, type d'erreur, et un mode
diagnostic), et l'utilisateur peut **réappliquer** ce qui est déjà en place. Les règles 1
à 44 restent valables, aucune n'est remplacée.

### Règles nouvelles

À ajouter après la règle 44, dans une nouvelle partie « Les échecs et ce qui est déjà en
place ».

45. **Dans un `catch`, écris le code, la cible et le type d'erreur**, puis le message :
    `[ERR] E705 $Key: $($_.Exception.GetType().Name) - $($_.Exception.Message)`, jamais le
    message seul ; il devient la raison de l'échec au bilan. Recopie la fonction
    `Write-Diag` et appelle-la dans chaque `catch` (`Write-Diag $_`) : en mode diagnostic
    (`$env:SINDRY_DIAG -eq '1'`), elle écrit le détail en lignes `[LOG] Diag`, sans rien de
    secret (« Expliquer un échec »).

46. **Une action accepte ce qui est déjà en place** : avec « Réappliquer », l'utilisateur
    peut cocher un élément `state=ok`, et ton script le reçoit comme les autres. Réécrire une
    valeur déjà juste ne doit rien casser. Un choix de `[multi]` qui est un réglage rapporte
    `state=todo` ou `state=ok`, comme un `[bool]`.

### La fonction `Write-Diag`

À recopier telle quelle dans le script, avec les autres fonctions :

```powershell
# Ecrit le detail d'une erreur attrapee, en mode diagnostic seulement (SINDRY_DIAG=1).
# Recoit l'enregistrement d'erreur ($_) ; ne renvoie rien.
function Write-Diag($Erreur) {
    if ($env:SINDRY_DIAG -ne '1') { return }
    $e = $Erreur.Exception
    Write-Host ("[LOG] Diag {0} HResult=0x{1:X8}" -f $e.GetType().FullName, $e.HResult)
    Write-Host "[LOG] Diag $($Erreur.CategoryInfo)"
    Write-Host "[LOG] Diag $($Erreur.InvocationInfo.PositionMessage -replace '\s+', ' ')"
    if ($e.InnerException) { Write-Host "[LOG] Diag Inner: $($e.InnerException.Message)" }
}
```

| Variable | Valeur | Sens |
|---|---|---|
| `SINDRY_DIAG` | `1`, ou absente | **Nouveau** : le réglage 5.7 de Sindry, « Journal de diagnostic », est actif |

### Dans la vérification finale

À ajouter à la partie « Sortie » :

- [ ] Chaque `catch` écrit un `[ERR]` avec code, cible et type d'erreur, puis appelle
      `Write-Diag $_`.

À la partie « Analyse » :

- [ ] Refaire l'action sur ce qui est déjà en place (`state=ok`) ne casse rien.

---

## 2.2.0 — depuis la 2.1.0 : ranger les réglages en sections

Une action qui a beaucoup de réglages les range maintenant en **onglets** dans la fiche
Expert. Rien d'autre ne change : les règles 1 à 43 restent valables, aucune n'est
remplacée.

### Règle nouvelle

À ajouter après la règle 43, dans une nouvelle partie « Les réglages nombreux (plus de
10) ».

44. **Range-les en sections** : `[section:Cle]` entre le type et le libellé fait de chaque
    section un **onglet** de la fiche Expert ; `[section:Cle.Sous]` ajoute un sous-titre
    dans l'onglet. Déclare chaque section **et** chaque sous-titre dans `REPORT`
    (`## Cle : [section] Titre — sous-titre`, `## Cle.Sous : [section] Titre`) et
    traduis-les dans `LANG fr`. Des onglets courts (3 à 8 réglages), nommés par ce que
    l'utilisateur cherche (Affichage, Confidentialité…), jamais par la technique
    (Registre…). Un réglage sans section finit dans un dernier onglet « Autres réglages »
    (« Ranger les réglages en sections »).

### Dans les options et le bloc `REPORT`

```
## SINDRY:OPTIONS
## CompactView    : [bool] [section:View] Compact view — less space between items
## ShowExtensions : [bool] [section:View.Files] Show file extensions
## NoRecentFiles  : [bool] [section:Privacy] Do not list recent files
## SINDRY:END

## SINDRY:REPORT
## View       : [section] Display — What File Explorer shows
## View.Files : [section] Files
## Privacy    : [section] Privacy — What File Explorer remembers
## SINDRY:END

## SINDRY:LANG fr
## View       : Affichage — Ce que l'Explorateur montre
## View.Files : Fichiers
## Privacy    : Confidentialité — Ce que l'Explorateur retient
## SINDRY:END
```

| Forme | Où | Sens |
|---|---|---|
| `[section:Cle]` | Étiquette d'une option | **Nouveau** : la range dans l'onglet `Cle` |
| `[section:Cle.Sous]` | Étiquette d'une option | **Nouveau** : la range sous le sous-titre `Cle.Sous` de l'onglet `Cle` |
| `Cle : [section] Titre — sous-titre` | Bloc `REPORT` | **Nouveau** : le titre de l'onglet ou du sous-titre, traduit dans `LANG` |

Sans section, Sindry range lui-même une action de plus de 10 réglages : ce que fait
l'action, les réglages de l'analyse (`[scan]`), les réglages avancés (`[hidden]`). Il
numérote aussi les réglages (« 2.3 » = 2ᵉ onglet, 3ᵉ réglage) au-delà de 10 : un
utilisateur peut donc désigner un réglage par son numéro.

### Ce que le validateur relève désormais

| Code | Niveau | Quand |
|---|---|---|
| `SECTION_INCONNUE` | erreur | `[section:…]` vise une section, ou un sous-titre, que `REPORT` ne déclare pas `[section]` |
| `SECTION_ORPHELINE` | avertissement | Section déclarée dans `REPORT` où aucune option ne se range |
| `SECTIONS_CONSEILLEES` | note | Plus de 10 réglages sans aucune section |

### Dans la vérification finale

À ajouter à la partie « Options » :

- [ ] Au-delà de 10 réglages, chacun a sa `[section:Cle]`, et chaque section ou sous-titre
      est déclaré `[section]` dans `REPORT` et traduit.

---

## 2.1.0 — depuis la 2.0.4 : redémarrer au milieu d'un entretien

Un script peut maintenant **demander** un redémarrage au milieu d'un entretien, puis
reprendre là où il en était. Sindry redémarre, se relance seul et relance le script à la
bonne phase. Rien d'autre ne change : les règles 1 à 41 restent valables, sauf les trois
remplacées ci-dessous.

### Règles remplacées

Ces trois règles remplacent celles de même numéro.

11. `admin : true` si le script a besoin des droits ; `reversible : false` si ce qu'il fait
    ne se défait pas ; `interruptible : false` si le tuer en route laisse le système
    incohérent ; `reboot : true` s'il faut redémarrer **à la fin** ; `resume : true` s'il
    peut demander un redémarrage **au milieu**, puis reprendre (règle 42).

22. Un **marqueur en majuscules en tête** de chaque ligne utile : `[INFO]`, `[OK]`, `[WARN]`,
    `[ERR]`, `[STEP] n/m`, `[PROGRESS] 0-100`, `[CKPT]`, `[REBOOT]`, `[DONE]`,
    `[FREED] <octets>`, `[LOG] Canal texte`, `[RESTART] <phase>` (règle 42).

23. **Le seul verdict est `exit`** : termine par `exit 0` en cas de réussite, `exit 1` en
    cas d'échec, `exit 3010` seulement juste après un `[RESTART]` (règle 42). Un `[ERR]`
    seul ne fait pas échouer le script.

### Règles nouvelles

À ajouter après la règle 41, dans une nouvelle partie « Le redémarrage au milieu d'un
entretien ».

42. **Ne redémarre jamais le PC toi-même** : ni `Restart-Computer`, ni `Stop-Computer`, ni
    `shutdown /r /s /g /p` — l'entretien serait coupé net (`REDEMARRAGE_DIRECT`). Si
    l'action ne peut finir qu'après un redémarrage, **demande-le** : `## resume : true`
    dans l'entête, puis `Write-Host "[RESTART] <phase>"` et **`exit 3010`**. Sindry
    redémarre et te relance avec `$env:SINDRY_RESUME = '<phase>'` et la même
    configuration ; teste-la **juste après l'override** (et la branche d'analyse) :
    `if ($env:SINDRY_RESUME -eq 'delete-file') { …la suite… ; exit 0 }`. Phase en
    minuscules, chiffres et tirets, 32 caractères au plus ; jamais de `[RESTART]` en
    analyse ni en simulation (en `SafeTest`, dis ce que tu ferais) ; aucune option secrète
    dans un script `resume` (« Redémarrer au milieu d'un entretien »).

43. **Chaque phase doit pouvoir être rejouée** sans dommage : si le PC redémarre pendant,
    elle recommence. Pour un redémarrage nécessaire seulement **à la fin**, c'est
    `[REBOOT]` et `exit 0`, jamais `[RESTART]`. Les installeurs tiers reçoivent
    `/norestart` (MSI : `REBOOT=ReallySuppress`) ; traduis leur 3010 ou 1641 en `[REBOOT]`
    ou en `[RESTART]`.

### Dans l'entête

| Champ | Valeur | Sens |
|---|---|---|
| `reboot` | `true` / `false` | **Précisé** : un redémarrage nécessaire **à la fin** de l'entretien, que le bilan propose |
| `resume` | `true` / `false` | **Nouveau**, facultatif (`false` si absent) : le script peut demander un redémarrage **au milieu**, puis reprendre |

### Dans la sortie

```
[REBOOT]   message          un redémarrage est réellement nécessaire, à la fin de l'entretien
[RESTART]  phase            redémarrer maintenant, puis reprendre à cette phase, suivi de exit 3010
```

| Code de sortie | Sens |
|---|---|
| `exit 0` | Réussite |
| `exit 1` | Échec |
| `exit 3010` | **Nouveau** : juste après `[RESTART]`, ni succès ni échec — la suite vient après le redémarrage. Seul, c'est un échec |

Au retour, la phase arrive dans **`$env:SINDRY_RESUME`** (absente au premier passage), avec
la même configuration dans `SINDRY_CONFIG`.

### Ce que le validateur relève désormais

| Code | Niveau | Quand |
|---|---|---|
| `REDEMARRAGE_DIRECT` | erreur | Le script redémarre le PC lui-même (`Restart-Computer`, `Stop-Computer`, `shutdown /r /s /g /p`…) |
| `RESTART_SANS_RESUME` | erreur | `[RESTART]` sans `## resume : true` dans l'entête |
| `RESTART_PHASE` | erreur | Phase mal formée : minuscules, chiffres et tirets, 32 caractères au plus |
| `RESTART_PHASE_NON_TRAITEE` | erreur | Phase émise qu'aucun test de `SINDRY_RESUME` ne reprend |
| `RESTART_SANS_3010` | erreur | `[RESTART]` sans `exit 3010` dans le script |
| `RESUME_NON_LU` | erreur | `resume : true` mais `$env:SINDRY_RESUME` n'est jamais lu |
| `CODE_3010` | avertissement | `exit 3010` sans `[RESTART]` |

### Dans la vérification finale

À ajouter à la partie « Sortie » :

- [ ] Aucun redémarrage direct. Un `[RESTART] <phase>` a `## resume : true`, est suivi de
      `exit 3010`, et sa phase est reprise par un test de `$env:SINDRY_RESUME`.

### Exemple complet

Un pilote se désinscrit, le PC redémarre, puis son fichier — enfin libéré — se supprime.
Il passe le validateur en `-Strict` sans aucun constat.

```powershell
## SINDRY:START
## id            : 8d4c2f6e-1b7a-4e3d-9c5f-2a6b8e0d4f17
## lang          : en
## title         : Remove a leftover driver
## desc          : Unregisters the driver, restarts the PC, then deletes its file
## category      : apps
## icon          : package-x
## tags          : driver, leftovers
## version       : 1.0
## author        : Your name
## created       : 2026-10-24
## updated       : 2026-10-24
## admin         : true
## risk          : high
## duration      : medium
## reversible    : false
## interruptible : false
## reboot        : false
## resume        : true
## engine        : auto
## SINDRY:END

## SINDRY:OPTIONS
## SafeTest : [bool] Simulate — shows what would be done, changes nothing
## SINDRY:END

## SINDRY:REPORT
## E201 : [error] The driver could not be unregistered — Restart the PC, then run the action again
## E202 : [error] The driver file is still in use — Restart the PC once more, then run the action again
## SINDRY:END

## SINDRY:LANG fr
## title    : Retirer un pilote resté en place
## desc     : Désinscrit le pilote, redémarre le PC, puis supprime son fichier
## SafeTest : Simuler — montre ce qui serait fait, sans rien modifier
## E201     : Le pilote n'a pas pu être désinscrit — Redémarrez le PC, puis relancez l'action
## E202     : Le fichier du pilote est encore utilisé — Redémarrez le PC une fois de plus, puis relancez l'action
## SINDRY:END

$CONFIG = @{
    SafeTest = $false
}

# --- Sindry override (ne pas supprimer) ---
if ($env:SINDRY_CONFIG) {
    ($env:SINDRY_CONFIG | ConvertFrom-Json).PSObject.Properties |
        ForEach-Object { $CONFIG[$_.Name] = $_.Value }
}

# ==============================================================================

# Comparaison de CHAINE, pas de booleen : la valeur arrive par JSON.
$SafeTest = ("$($CONFIG.SafeTest)" -eq 'True')
# Le pilote vise, ecrit en dur : jamais un chemin fabrique avec une valeur recue.
$DRIVER = 'OldVendorFilter'
# Son fichier, sous le dossier des pilotes de Windows.
$FILE = "$env:SystemRoot\System32\drivers\$DRIVER.sys"

# ==============================================================================
# Phase 2, apres le redemarrage : Sindry relance le script avec SINDRY_RESUME.
# Elle doit pouvoir etre rejouee sans dommage.
# ==============================================================================
if ($env:SINDRY_RESUME -eq 'delete-file') {
    Write-Host "[STEP] 2/2 Deleting the driver file"
    if ($SafeTest) {
        Write-Host "[INFO] SafeTest - would delete $FILE"
    } elseif (Test-Path -LiteralPath $FILE) {
        try {
            Remove-Item -LiteralPath $FILE -Force -ErrorAction Stop
        } catch {
            Write-Host "[ERR] E202 $($_.Exception.Message)"
            exit 1
        }
        Write-Host "[OK]   Driver file deleted"
    }
    Write-Host "[DONE] Done"
    exit 0
}

# ==============================================================================
# Phase 1 : desinscrire le pilote. Son fichier reste charge jusqu'au redemarrage.
# ==============================================================================
Write-Host "[STEP] 1/2 Unregistering the driver"
if ($SafeTest) {
    # En simulation, jamais de redemarrage : on dit ce qu'on ferait.
    Write-Host "[INFO] SafeTest - would unregister $DRIVER, then ask for a restart"
    Write-Host "[DONE] Done"
    exit 0
}
& "$env:SystemRoot\System32\sc.exe" delete $DRIVER | Out-Null
# 1060 : le service n'existe deja plus, ce n'est pas un echec.
if ($LASTEXITCODE -ne 0 -and $LASTEXITCODE -ne 1060) {
    Write-Host "[ERR] E201 sc.exe delete returned $LASTEXITCODE"
    exit 1
}
Write-Host "[OK]   Driver unregistered"
# Sindry redemarre le PC, puis relance ce script a la phase 'delete-file'.
Write-Host "[RESTART] delete-file"
exit 3010
```
