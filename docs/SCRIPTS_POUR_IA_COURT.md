# Sindry — écrire un script PowerShell conforme (version courte)

> **Version courte, pour une IA à petite mémoire** (un modèle local, par exemple). Générée
> par `tools/build-script-guides.mjs` (Sindry 2.0.4), comme la version complète
> `SCRIPTS_POUR_IA.md`. Ne la modifiez pas à la main.

Ce document garde l'essentiel pour écrire un script que Sindry accepte : la mission, le
contexte, **toutes les règles impératives**, les données de référence, les extraits du
contrat qui servent à chaque script (squelette, marqueurs, analyse) et **un exemple complet**,
puis la liste de vérification. Le reste du contrat est dans la version complète ; en cas de
doute, **elle fait foi**.
---

## 1. Ta mission

Tu écris **un script PowerShell (`.ps1`) pour Sindry**, à partir de la demande de
l'utilisateur.

- **Avant d'écrire**, si la demande laisse un doute sur **ce qui sera modifié ou supprimé**,
  sur le besoin de droits administrateur ou sur le caractère réversible, pose la question.
  Ne devine pas une action destructive.
- **Livre** :
  1. le fichier complet, dans un seul bloc de code ;
  2. son nom proposé, `NNN_NOM_EN_ANGLAIS.ps1` (section 5.3) ;
  3. en trois à six lignes : ce que fait le script, ses options, ses codes d'erreur, tes
     hypothèses ;
  4. le rappel d'enregistrer le fichier en **UTF-8 avec BOM** et de le vérifier avec
     `.\tools\lint-scripts.ps1 -Path <dossier> -Strict`.
- **Le validateur tranche**, pas toi : écris pour qu'il ne relève rien en `-Strict`
  (section 7).

---

## 2. Le contexte

**Sindry** (anciennement WinTool) est une application de maintenance pour **Windows 10 et
11** : nettoyage, performance, vie privée, applications, santé du PC. Elle est faite en
Tauri (Rust et une interface web) et elle **exécute des scripts PowerShell**, le plus
souvent **en administrateur**.

| Élément | Ce qu'il faut savoir |
|---|---|
| Deux modes | **Simple**, pour un débutant (un assistant en trois étapes, aucun terme technique) ; **Expert**, une fiche par script avec tous ses réglages |
| Deux langues | L'interface est en français et en anglais. Le script écrit sa sortie en anglais ; les textes affichés viennent de ses blocs `##`, traduits |
| D'où viennent les scripts | Du **catalogue officiel**, signé (dépôt GitHub `burnout293/Sindry-Catalogue`), ou du dossier personnel de l'utilisateur |
| Ce que Sindry lit | Les blocs `## SINDRY:…` en tête du fichier : ils construisent l'interface |
| Ce que Sindry transmet | Les réglages choisis, en JSON, dans la variable d'environnement `SINDRY_CONFIG` ; `SINDRY_MODE=scan` pour une analyse |
| Ce que Sindry écoute | La sortie du script, ligne par ligne, par ses **marqueurs** (`[STEP]`, `[OK]`, `[ERR]`…) |
| Le verdict | **Le code de sortie** : `exit 0` réussite, `exit 1` échec |
| Le contrôle | `tools/lint-scripts.ps1` : en `-Strict`, le catalogue officiel ne tolère ni erreur ni avertissement |

---

## 3. Les règles impératives

Chacune est détaillée dans le contrat (section 6), sous le titre indiqué entre parenthèses.

### Le fichier

1. **UTF-8 avec BOM.** Sans BOM, PowerShell 5.1 lit mal le moindre accent (« Encodage »).
2. Code **compatible PowerShell 5.1** : pas de `??`, de `?:`, de `&&` ou de `||` entre
   commandes, sauf avec `engine : pwsh` (« Le cas de `engine` »).
3. Ordre des blocs : `SINDRY:START`, `SINDRY:OPTIONS`, `SINDRY:REPORT` (s'il y en a un),
   `SINDRY:LANG fr`, puis `$CONFIG`, la ligne d'override, et le code (« Le squelette »).

### La langue

4. **Anglais** : noms de fichier, variables, fonctions, clés, libellés de base (avec
   `lang : en`) et **toute la sortie**. Aucun accent dans un message affiché.
5. **Français** : les commentaires, et le bloc `SINDRY:LANG fr`, seule place du français
   pour l'interface (« La langue »).

### L'entête `SINDRY:START`

6. Champs **obligatoires** : `id`, `lang`, `title`, `desc`, `category`, `icon`, `version`,
   `admin`, `risk`, `duration`, `reversible`, `interruptible`, `reboot`, `engine`.
7. `id` : un **GUID v4 neuf**, jamais copié d'un exemple. Écris-en un au hasard et signale
   qu'il peut être remplacé par le résultat de `New-Guid`.
8. `category` : une des catégories de la section 5.1 — **jamais** un lot (`maintenance`
   n'est pas une catégorie).
9. `icon` : un nom **Lucide** de la section 5.4, en minuscules avec tirets. **Jamais
   d'emoji.**
10. Valeurs fermées : `risk` `low|medium|high` ; `duration` `fast|medium|slow` ; `engine`
    `auto|winps|pwsh` (préfère `auto`) ; booléens `true|false`.
11. `admin : true` si le script a besoin des droits ; `reversible : false` si ce qu'il fait
    ne se défait pas ; `interruptible : false` si le tuer en route laisse le système
    incohérent ; `reboot : true` s'il faut redémarrer ensuite.
12. **Toujours présents et à jour** : `author` (le créateur), `created` (date de création),
    `updated` (date de la dernière modification) et `version`. Dates `AAAA-MM-JJ`, `updated`
    jamais avant `created`, aucune date future ; `version` et `updated` changent à chaque
    modification. Le validateur ne bloque pas leur absence, mais le style du catalogue les
    exige.

### Les options : `SINDRY:OPTIONS` et `$CONFIG`

13. Une ligne par option : `## Cle : [type] Libellé — Description`. **Un** espace après `##`
    pour une option, **deux ou plus** pour un choix. Le séparateur est ` — ` (tiret
    cadratin entouré d'espaces).
14. Types, **en minuscules** : `bool`, `number`, `string`, `select`, `multi`, `hidden`,
    `items`. `select` et `multi` ont au moins deux choix ; `hidden` est **un booléen
    caché**, rien d'autre.
15. La **même clé** (sans espace) aux trois endroits : `OPTIONS`, `LANG fr`, `$CONFIG`.
    Chaque option a sa valeur par défaut dans `$CONFIG` ; un `select` vaut un choix, un
    `multi` ou un `items` un tableau (`@()` permis).
16. `$CONFIG = @{` sur une ligne ; l'accolade fermante **seule** sur la sienne.
17. Nomme une option par l'action : `$true` veut dire « fais-le ».
18. **Simulation recommandée** : l'option
    `## SafeTest : [bool] Simulate — shows what would be done, changes nothing`, défaut
    `$false`, honorée **partout** (réseau, licences, réglages globaux compris). En
    simulation, dis ce que tu *ferais*, jamais ce que tu *aurais trouvé* (« 4. La
    simulation »).

### La traduction : `SINDRY:LANG fr`

19. Code de langue sur **deux lettres**. Traduit `title`, `desc`, **chaque** option,
    **chaque** choix et **chaque** clé de `REPORT`, sans répéter `[type]` ni étiquettes.
    Aucune traduction orpheline.

### La ligne d'override

20. Recopie-la **à l'identique**, après `$CONFIG`, jamais en commentaire :

    ```powershell
    # --- Sindry override (ne pas supprimer) ---
    if ($env:SINDRY_CONFIG) {
        ($env:SINDRY_CONFIG | ConvertFrom-Json).PSObject.Properties |
            ForEach-Object { $CONFIG[$_.Name] = $_.Value }
    }
    ```

    **Jamais** de `Get-Content` sur `SINDRY_CONFIG` : Sindry refuserait de lancer le script.
21. Lis un booléen par comparaison de chaîne : `$SafeTest = ("$($CONFIG.SafeTest)" -eq 'True')`.

### La sortie

22. Un **marqueur en majuscules en tête** de chaque ligne utile : `[INFO]`, `[OK]`, `[WARN]`,
    `[ERR]`, `[STEP] n/m`, `[PROGRESS] 0-100`, `[CKPT]`, `[REBOOT]`, `[DONE]`,
    `[FREED] <octets>`, `[LOG] Canal texte`.
23. **Le seul verdict est `exit`** : termine par `exit 0` en cas de réussite, `exit 1` en
    cas d'échec. Un `[ERR]` seul ne fait pas échouer le script.
24. **Codes d'erreur** : déclare-les dans `REPORT`
    (`## E104 : [error] Description — Solution`), traduis-les dans `LANG`, et écris le code
    en **premier mot** : `[ERR] E104 <détail technique>`. Forme : une à trois majuscules,
    puis deux à quatre chiffres. Tout code émis est déclaré, tout code déclaré est utilisé.
25. `[CKPT]` dès que l'état redevient cohérent dans un long traitement ; `[FREED]` si le
    script libère de l'espace.

### Le mode analyse (si le script sait montrer avant d'agir)

26. `## scan : true`, puis, **juste après l'override et avant toute modification** :
    `if ($env:SINDRY_MODE -eq 'scan') { …mesurer, décrire… ; exit 0 }`. En analyse, **rien
    n'est modifié**.
27. Chaque ligne `[FIND]`, `[ITEM]`, `[METRIC]`, `[NOTE]` vise une option, un choix ou une
    clé de `REPORT` **déclarés**. Tailles en **octets** ; valeurs avec espaces entre
    guillemets.
28. Une option `[items]` vaut `@()` par défaut. Ses `id=` sont **stables** et calculés à
    partir de ce qui ne bouge pas ; à l'action, retrouve les éléments par la même fonction et
    n'utilise l'id reçu **que comme clé**.
29. Pièges de l'écran : `size=` sur un `[ITEM]` compte comme de l'espace libéré (n'en écris
    pas pour ce qui s'installe ou s'ajoute) ; `state=ok` rend la case inactive ; `exe=` ne
    vaut que pour `kind=app|startup|service|task`, avec un chemin local absolu (« Ce que
    l'écran d'analyse fait vraiment d'un `[ITEM]` »).
30. **Les icônes d'un `[ITEM]`**, de la plus précise à la plus générale : `exe=` (le
    fichier d'une application installée), `appicon=<nom>` (l'icône officielle, téléchargée par
    Sindry depuis le dépôt `Sindry-Media-Library` : un nom, jamais une adresse), `icon=<nom Lucide>`
    (une icône générique, aussi pour une catégorie). Écris-en plusieurs : chacune sert de repli.
31. Une action à **longue liste** (plus de 12 éléments) ou dont un élément est **expliqué**
    gagne d'elle-même l'écran de liste : onglets Choisir / Récapitulatif, recherche,
    catégories, filtres et tri, fiche à droite. Explique ce qui le mérite : `about=` (de quoi
    il s'agit), `why=` (pourquoi c'est proposé), `warn=` (ce qu'on perd), chacun **une clé
    `[note:…]` du `REPORT`**, jamais une phrase ; `advice=caution` pour ce qui est à double
    tranchant, `advice=discouraged` pour ce qu'il vaut mieux garder (un antivirus) — jamais
    cochés d'office ; `reboot=true` si le retirer demande un redémarrage, puis `[REBOOT]` à
    la fin s'il a eu lieu. Pour qu'en mode Simple elle montre sa liste plutôt qu'un résumé,
    déclare `## view : tree`.

### La sécurité

32. **N'exécute jamais une valeur** : ni `Invoke-Expression`, ni `[scriptblock]::Create`,
    ni commande construite avec une valeur de `$CONFIG`. Passe les valeurs en paramètres
    (`-LiteralPath`).
33. Outils du système par leur **chemin complet** (`"$env:SystemRoot\System32\…"`) ; jamais
    le dossier courant ; `$PSScriptRoot` pour tes propres fichiers.
34. **Avant de supprimer ou d'écraser**, vérifie que le chemin ne vise ni Windows, ni
    Program Files, ni ProgramData, ni la racine d'un lecteur, ni un autre profil. Borne les
    nombres. Aucun secret dans une option.
35. Données de restauration : sous `HKLM\SOFTWARE\Sindry\Scripts\<id>`, jamais dans un
    dossier de `%ProgramData%`.

### Le style du catalogue officiel

36. Commentaires **en français** : chaque variable commentée **sur sa propre ligne**, en fin
    de ligne ; chaque fonction commentée **à son début** (ce qu'elle fait, reçoit, renvoie).
37. Au-delà de **200 lignes**, des chapitres balisés et adaptés au script : Sommaire, Résumé,
    Variables, Fonctions, Préparation, Exécution, Journal (logs), Annexes…
38. Un **maximum de drapeaux** — booléens, listes, ou tout autre type utile — pour activer,
    désactiver ou adapter vite chaque fonctionnalité, **regroupés avec les variables** en
    tête. Les listes de comportement par défaut (ce qui est coché d'office, ce qui est exclu)
    s'y modifient sans toucher au reste.
39. Un **maximum de codes d'erreur** : chacun déclaré `[error]` avec sa description et, si
    possible, sa solution, et **traduit en français** dans `LANG fr` (règle 24).
40. Un journal **clair pour un débutant**, lisible d'un coup d'œil : aéré, sans pavé, avec
    des tableaux et des listes alignés par des espaces quand ça s'y prête (le journal de
    Sindry est en chasse fixe). `desc` annonce les changements sensibles (services, tâches
    planifiées, démarrage…).
41. **Le journal est en anglais, et une seule langue.** Sindry ne traduit pas une ligne de
    sortie : ce que l'utilisateur doit lire dans sa langue passe par les **codes d'erreur**
    (description et solution traduites) et, en analyse, par les mots du bloc `REPORT`.
    N'écris jamais deux versions d'un message.

---

## 4. La méthode

1. **Comprendre** : ce qui change sur le PC, les droits nécessaires, la réversibilité, le
   redémarrage, la durée, et si le script peut **montrer avant d'agir** (mode analyse).
2. **Choisir** : la catégorie (5.1), le numéro du fichier (5.3), l'icône (5.4), un `id` neuf.
3. **Écrire**, dans l'ordre : `START` → `OPTIONS` → `REPORT` → `LANG fr` → `$CONFIG` →
   override → analyse (si `scan`) → action → `exit`. Pars du **squelette** du contrat ;
   pour un script analysable, de l'**exemple complet** « faire de la place ».
4. **Vérifier** avec la liste de la section 7, point par point.
5. **Livrer** comme dit en section 1.

---

## 5. Les données de référence

### 5.1 Les catégories — champ `category`

Écris l'identifiant anglais avec `lang : en` (le cas des scripts officiels), l'identifiant
français avec `lang : fr`. Rien d'autre n'est admis.

| Anglais | Français | Nom | Numéros de fichier |
|---|---|---|---|
| `cleaning` | `nettoyage` | Nettoyage / Cleaning | `1xx` |
| `performance` | `performance` | Performance / Performance | `2xx` |
| `privacy` | `vieprivee` | Vie privée / Privacy | `3xx` |
| `apps` | `applications` | Applications / Applications | `4xx` |
| `health` | `sante` | Santé / Health | `5xx` |
| `tools` | `outillage` | Outillage / Tools | `6xx` |
| `customize` | `personnalisation` | Personnalisation / Customize | `7xx` |

### 5.2 Les lots d'usine

Un script entre dans le lot qui accueille sa catégorie. Il ne vise jamais un lot.

| Lot | Accueille |
|---|---|
| Faire le ménage | `cleaning` |
| Performance | `performance` |
| Vie privée | `privacy` |
| Applications | `apps` |
| Santé | `health` |
| Outillage | `tools` |
| Personnalisation | `customize` |
| Entretien complet | tout ce qui est rangé dans les autres lots (aucun script ne le vise) |

### 5.3 Le nom du fichier

`NNN_NOM_EN_ANGLAIS.ps1` : trois chiffres dans la plage de la catégorie (5.1), un tiret
bas, le nom en majuscules, mots séparés par des tirets bas — par exemple
`105_CLEAN_THUMBNAIL_CACHE.ps1`. Le numéro fixe l'ordre par défaut dans le lot.

### 5.4 Les icônes Lucide — champ `icon`

Une sélection pour un outil de maintenance. Le validateur admet les 2112 noms de
Lucide (liste complète dans `SCRIPTS_POUR_IA.md`) : prends ici celui qui convient.

| Thème | Noms |
|---|---|
| Nettoyage et disque | trash-2, trash, eraser, brush-cleaning, broom, sparkles, recycle, archive, hard-drive, hard-drive-download, database, folder, folder-x, folder-cog, folder-search, file, file-x, files, copy, scan-search |
| Performance et matériel | zap, gauge, rocket, cpu, memory-stick, monitor, monitor-cog, laptop, battery, battery-charging, power, plug, thermometer, fan, timer, activity |
| Vie privée et sécurité | shield, shield-check, shield-alert, shield-off, lock, lock-open, key, fingerprint, eye, eye-off, user-x, ban, bug |
| Applications et Windows | package, package-x, package-plus, package-check, download, app-window, blocks, puzzle, layout-grid, store, box, boxes, rotate-ccw, refresh-cw, history |
| Réseau | globe, wifi, wifi-off, network, router, server, cloud, cloud-off, cable, bluetooth |
| Santé et diagnostic | heart-pulse, stethoscope, circle-check, triangle-alert, info, search, list-checks, clipboard-list, scan |
| Outils et réglages | settings, settings-2, sliders-horizontal, wrench, hammer, terminal, code, calendar-clock, clock, bell, bell-off |
| Périphériques et médias | printer, keyboard, mouse, gamepad-2, usb, music, play, image, camera, mic, volume-2, volume-x, languages, palette, sun, moon |

---

## 6. Extraits du contrat — docs/FORMAT_SCRIPT.md

Les parties du contrat dont chaque script se sert, telles quelles. Les autres titres cités
entre « » dans les règles sont dans la version complète, `SCRIPTS_POUR_IA.md`.

### Le squelette

```powershell
## SINDRY:START
## id            : 3f2b1a9c-7e4d-4c6a-9b0e-1d5f6a8c2e0b
## lang          : en
## title         : Set fast DNS
## desc          : Speeds up browsing with a faster resolver
## category      : performance
## icon          : zap
## tags          : dns, network, latency
## version       : 2.0
## author        : Your name
## created       : 2026-10-17
## updated       : 2026-10-17
## admin         : true
## risk          : medium
## duration      : fast
## reversible    : true
## interruptible : true
## reboot        : false
## engine        : auto
## SINDRY:END

## SINDRY:OPTIONS
## DnsProvider  : [select] DNS provider — the service that resolves website addresses
##   cloudflare : Cloudflare — 1.1.1.1, fastest on most connections
##   google     : Google — 8.8.8.8, very reliable
##   quad9      : Quad9 — 9.9.9.9, blocks known malicious domains
## ApplyToIPv6  : [bool]   Apply to IPv6 — equivalent resolvers
## FlushCache   : [hidden] Flush the resolver cache afterwards
## SafeTest     : [bool]   Simulate — shows what would be done, changes nothing
## SINDRY:END

## SINDRY:LANG fr
## title        : Utiliser un Internet plus rapide
## desc         : Accélère la navigation avec un résolveur plus rapide
## DnsProvider  : Fournisseur DNS — le service qui traduit les adresses des sites
##   cloudflare : Cloudflare — 1.1.1.1, le plus rapide sur la plupart des connexions
##   google     : Google — 8.8.8.8, très fiable
##   quad9      : Quad9 — 9.9.9.9, bloque les domaines malveillants connus
## ApplyToIPv6  : Appliquer à l'IPv6 — résolveurs équivalents
## FlushCache   : Vider le cache de résolution ensuite
## SafeTest     : Simuler — montre ce qui serait fait, sans rien modifier
## SINDRY:END

$CONFIG = @{
    DnsProvider = "cloudflare"
    ApplyToIPv6 = $true
    FlushCache  = $true
    SafeTest    = $false
}

# --- Sindry override (ne pas supprimer) ---
if ($env:SINDRY_CONFIG) {
    ($env:SINDRY_CONFIG | ConvertFrom-Json).PSObject.Properties |
        ForEach-Object { $CONFIG[$_.Name] = $_.Value }
}

# ==============================================================================

# Comparaison de CHAINE, pas de booleen : la valeur arrive par JSON et peut
# valoir $true, "True" ou 1 selon le chemin emprunte.
$SafeTest = ("$($CONFIG.SafeTest)" -eq 'True')

Write-Host "[STEP] 1/1 Applying DNS settings"
if ($SafeTest) {
    Write-Host "[INFO] SafeTest - would set the resolver to $($CONFIG.DnsProvider)"
} else {
    Write-Host "[OK]   Provider set to $($CONFIG.DnsProvider)"
}
Write-Host "[DONE] Done"
exit 0
```

---

### Les marqueurs de sortie

La sortie est **en anglais** (§10). En mode Simple elle est masquée derrière « Voir le
détail technique » ; c'est l'interface qui traduit la progression à partir des marqueurs.

```
[INFO]     message          ligne neutre
[OK]       message          succès d'une étape
[WARN]     message          avertissement, n'échoue pas
[ERR]      message          erreur affichée et comptée — ne suffit PAS à faire échouer le script
[STEP]     3/7 message      alimente la barre de progression
[PROGRESS] 0-100            la progression en pourcentage, quand [STEP] ne convient pas
[CKPT]     message          « interruption sans risque à partir d'ici »
[REBOOT]   message          un redémarrage est réellement nécessaire
[DONE]     message          fin nominale
[FREED]    octets           espace réellement libéré, repris dans le bilan
[LOG]      Canal texte      une ligne de journal rangée dans un canal (mode Expert)
[FIND]     Option champs    mode analyse : un constat sur une case — voir « Le mode analyse »
[ITEM]     Option id=…      mode analyse : un élément trouvé
[METRIC]   Libellé value=…  mode analyse : une mesure
[NOTE]     Note [cible]     mode analyse : une phrase du bloc REPORT
```

> ### Le seul verdict est `exit`
>
> `[ERR]` n'est qu'un affichage : il colore une ligne et incrémente un compteur. Il
> **n'échoue pas** le script. Un script qui écrit `[ERR] ...` puis se termine par `exit 0`
> — ou sans `exit` du tout, PowerShell renvoyant alors 0 — est enregistré comme une
> **réussite**, avec « 1 erreur » en petit à côté.
>
> Si votre script a rencontré une erreur, il **doit** se terminer par `exit 1`.
>
> C'est exactement le « succès fictif » que la v0.3 produisait et que cette refonte corrige :
> ne le réintroduisez pas depuis le script.

**Le verdict de réussite vient du code de sortie** (`exit 0` = succès), jamais du fait que
le script ait démarré. Terminez donc explicitement par `exit 0` ou `exit 1`. La v0.3 se
contentait de constater le démarrage, et enregistrait ainsi des succès fictifs.

#### Les codes d'erreur : `[ERR] E104` (2.0.3)

La sortie reste en anglais, mais une erreur peut être **expliquée** dans la langue de
l'utilisateur. Déclarez-la dans le bloc `REPORT` avec l'étiquette `[error]` : ce qui s'est
passé, puis, après « — », ce qu'il faut faire (facultatif). Traduisez-la dans `LANG` comme
le reste :

```
## SINDRY:REPORT
## E104 : [error] Access denied on a protected key — Restart the PC, then run the action again
## SINDRY:END

## SINDRY:LANG fr
## E104 : Accès refusé sur une clé protégée — Redémarrez le PC, puis relancez l'action
## SINDRY:END
```

Puis écrivez le code en **premier mot** du message, le détail technique ensuite :

```powershell
Write-Output "[ERR] E104 HKLM\SOFTWARE\WRData (0x80070005)"
```

- **Le code** : une à trois majuscules, puis deux à quatre chiffres (`E104`, `SVC12`). Il vaut
  aussi pour `[WARN] W201 …`, déclaré lui aussi `[error]` : c'est la ligne qui dit le niveau.
- **Ce que Sindry en fait** : dans le journal, la description traduite suit le code, puis le
  détail technique ; la solution s'affiche en dessous. Le dernier `[ERR]` codé d'une action
  qui échoue devient la raison de l'échec, dans la liste des tâches et dans le verdict.
- **Le verdict ne change pas** : un code explique une erreur, il ne fait pas échouer le
  script. Terminez toujours par `exit 1`.
- Un premier mot qui n'est pas un code déclaré laisse la ligne telle quelle. Le validateur
  relève un code écrit dans un `[ERR]` ou un `[WARN]` littéral sans être déclaré
  (`CODE_ERREUR_INCONNU`), et un code déclaré qui n'apparaît nulle part dans le script
  (`CODE_ERREUR_INUTILISE`). Une fonction d'aide qui écrit `[ERR] $Code` est permise : le
  code compte dès qu'il figure dans son appel (`Write-Err 'E104' $detail`).

#### `[CKPT]`, le marqueur qui rend l'annulation sûre

Au premier clic sur « Arrêter », Sindry laisse le script en cours aller au bout. Au
second, il ne le tue que si le script est `interruptible`, **ou** si un `[CKPT]` a été
franchi. Émettez-en un dès que l'état du système redevient cohérent :

```powershell
Write-Host "[CKPT] Backup complete - safe to interrupt from here"
```

---

### Le contrat, en quatre règles

**1. Sindry pose `SINDRY_MODE=scan`.** C'est une variable d'environnement, distincte de
`SINDRY_CONFIG`. Absente — lancement normal, ou double-clic sur le fichier —, le script
agit comme d'habitude.

**2. En mode analyse, le script ne modifie RIEN.** Ni fichier, ni registre, ni service, ni
tâche planifiée, ni réglage réseau. Il mesure, décrit, et se termine par `exit 0`. Un code
de sortie non nul signifie « l'analyse a échoué » : ce qu'elle a écrit reste visible en
Expert, mais rien n'en est retenu, et en mode Simple l'action n'est pas lancée.

**3. Chaque constat s'écrit sur une ligne**, qui vise une case que le script déclare — une
option, un choix — ou un mot de son bloc `REPORT`. Une ligne qui vise autre chose est
ignorée, et relevée en mode Expert : un script ne parle que de ses propres cases.

**4. La sélection revient par `$CONFIG`, comme n'importe quel réglage.** C'est le point qui
rend le mode analyse presque gratuit à écrire : il n'y a **pas de mode « action » à
implémenter**. Après l'analyse, Sindry lance le script normalement, avec dans `$CONFIG`
ce qui a été coché — voir « Ce qui revient au script ». Votre code d'action existe déjà :
il lit `$CONFIG`, et fait ce qu'on lui dit.

### Les lignes de l'analyse

| Marqueur | Forme | Ce que c'est |
|---|---|---|
| `[FIND]` | `[FIND] Option champs…` ou `[FIND] Option.choix champs…` | Un constat sur une case déclarée : une option `[bool]`, un choix d'un `[multi]` ou d'un `[select]` |
| `[ITEM]` | `[ITEM] Option id=… champs…` | Un élément trouvé, rangé dans une liste `[items]` |
| `[METRIC]` | `[METRIC] Libellé value=… champs…` | Une mesure sans case : espace libre, température, durée de démarrage |
| `[NOTE]` | `[NOTE] Note [Option ou Option.choix] [show=…]` | Une phrase du bloc `REPORT`, sous une case ou, sans cible, en tête |
| `[LOG]` | `[LOG] Canal texte` | Une ligne de journal, rangée dans un canal que l'Expert peut filtrer. Jamais traduite. |
| `[PROGRESS]` | `[PROGRESS] 0-100` | Un pourcentage, quand `[STEP] n/m` ne convient pas |
| `[STEP]` | `[STEP] n/m texte` | Comme pendant l'action : la progression |

Les champs s'écrivent `nom=valeur`, séparés par des espaces. Une valeur qui contient des
espaces se met entre guillemets : `name="Google Chrome"`. Une valeur illisible — du texte
là où il faut un nombre, un jeton hors liste — est **ignorée et relevée, jamais
interprétée au mieux**.

**Une seconde ligne `[FIND]` sur la même case la complète** (depuis la 1.5) : ses champs
s'ajoutent, et sur un même champ la dernière valeur l'emporte. C'est ce qui permet de ne
recommander un choix qu'une fois tous mesurés :

```
[FIND] DnsProvider.cloudflare ms=24 current=false
[FIND] DnsProvider.google ms=26 current=true
[FIND] DnsProvider.cloudflare recommended=true
```

### Les champs de `[FIND]`

| Champ | Valeur | Sens |
|---|---|---|
| `size=` | entier, en **octets** | Espace récupérable. Sindry l'affiche dans l'unité et la langue de l'utilisateur. |
| `count=` | entier | Nombre d'éléments trouvés : fichiers, entrées de démarrage… |
| `state=` | `todo` / `ok` | Un réglage : `todo` = pas encore en place, `ok` = déjà fait — la case est alors grisée |
| `checked=` | `true` / `false` | L'avis du script : cocher d'office ou non, quoi que disent les mesures |
| `ms=` | entier | Une durée en millisecondes, pour comparer les choix d'un `[select]` (vue `chart`) |
| `current=` | `true` / `false` | Sur un choix de `[select]` : le réglage en place aujourd'hui |
| `recommended=` | `true` / `false` | Sur un choix de `[select]` : celui que le script conseille, présélectionné |
| `show=` | `simple` / `expert` | Ne s'affiche que dans ce mode |

### Les champs de `[ITEM]`

| Champ | Valeur | Sens |
|---|---|---|
| `id=` | lettres, chiffres et `- _ . : @ +`, 128 au plus | **Obligatoire.** Ce que Sindry renverra au script si l'élément est coché |
| `parent=` | l'`id` d'un autre élément de la même liste | Range l'élément sous ce parent : une arborescence |
| `name=` | texte | Lequel : « Default », « Adobe Reader » |
| `label=` | une clé du bloc `REPORT` | Ce que c'est, traduit : `label=Cache name=Default` s'affiche « Fichiers en cache — Default » |
| `path=` | texte | Où il se trouve. Affiché, **jamais renvoyé**. |
| `publisher=`, `version=`, `to=` | texte | Éditeur, version en place, version proposée |
| `date=` | `AAAA-MM-JJ` | Une date : dernière utilisation, installation |
| `kind=` | `folder` `file` `registry` `app` `startup` `service` `task` `driver` `browser` | L'icône de la ligne |
| `exe=` | chemin absolu local vers un `.exe`, un `.dll` ou un `.ico` | **La vraie icône** du programme (2.0.3), avec `kind=app`, `startup`, `service` ou `task`. Sindry la lit lui-même dans le fichier. Comme `path=`, **jamais renvoyé** — voir ci-dessous |
| `appicon=` | un nom : minuscules, chiffres, tirets | **L'icône officielle** d'une application (2.0.4), téléchargée par Sindry depuis le dépôt `burnout293/Sindry-Media-Library` — voir ci-dessous |
| `icon=` | un nom d'icône Lucide | Une icône **générique** pour l'élément, une application comme une catégorie (2.0.4) : `globe`, `music`, `code`… |
| `confidence=` | `high` / `medium` / `low` | Sûr, Probable, À vérifier — **`low` n'est jamais coché d'office** |
| `impact=` | `high` / `medium` / `low` | Impact élevé, moyen, faible |
| `risk=` | `high` / `medium` / `low` | Suspect, Inhabituel — `low` n'affiche rien |
| `locked=` | `open` `system` `protected` `inuse` | Visible mais **pas cochable**, avec la raison : programme ouvert, nécessaire à Windows… |
| `about=` | une clé `[note:…]` du bloc `REPORT` | **De quoi il s'agit**, en une phrase (2.0.4) — dans la fiche de l'élément |
| `why=` | une clé `[note:…]` du bloc `REPORT` | **Pourquoi c'est proposé** (2.0.4) |
| `warn=` | une clé `[note:…]` du bloc `REPORT` | **Attention** : ce que l'utilisateur perd ou risque (2.0.4) — repris au récapitulatif |
| `advice=` | `caution` / `discouraged` | **À double tranchant**, **Déconseillé** (2.0.4) — **jamais coché d'office**, même avec `checked=true` ; `discouraged` ne se coche qu'après une confirmation |
| `reboot=` | `true` / `false` | Le retirer demande de redémarrer (2.0.4) : annoncé dans la fiche, au récapitulatif et avant de lancer |
| `group=`, `keep=` | identifiant ; `true` / `false` | Pour `[items:keep-one]` : les exemplaires d'un même fichier, et celui à garder |
| `size=` `count=` `state=` `checked=` `show=` | comme pour `[FIND]` | |

### Les champs de `[METRIC]`

| Champ | Valeur | Sens |
|---|---|---|
| `value=` | un nombre, ou un jeton court | **Obligatoire.** La mesure. |
| `unit=` | `celsius` `pct` `hours` `days` `s` `count` `cycles` | Son unité, écrite dans la langue de l'utilisateur |
| `health=` | `ok` / `warn` / `crit` | Bon, À surveiller, Critique |
| `max=` | nombre | Le maximum possible : de quoi dessiner une jauge |
| `group=` | une clé de groupe du bloc `REPORT` | Regroupe les mesures dans la vue `light` |
| `show=` | `simple` / `expert` | Ne s'affiche que dans ce mode |

### Le bloc `REPORT` : les mots de l'analyse

Le script n'écrit jamais de phrase dans sa sortie : elle est en anglais, et l'interface est
bilingue. Tout ce que l'analyse affiche et qui n'est pas déjà le libellé d'une option se
déclare dans un bloc `REPORT`, placé entre `OPTIONS` et `LANG` :

```
## SINDRY:REPORT
## Junk       : [group] Junk files — Temporary files and the recycle bin
## Cache      : Cache
## FreeSpace  : Free space on drive C:
## UpdateNote : [note:warn] If an update is waiting to be installed, Windows will download it again.
## SINDRY:END
```

| Forme | Sert à | Visé par |
|---|---|---|
| `Clé : Libellé` | Nommer une mesure, ou la nature d'un élément | `[METRIC] Clé`, `[ITEM] … label=Clé` |
| `Clé : [note:info] Phrase` ou `[note:warn]` | Une phrase neutre, ou un avertissement | `[NOTE] Clé` |
| `Clé : [group] Libellé — sous-titre` | Un **poste** du résumé Simple | `[group:Clé]` sur une option ou un choix |
| `E104 : [error] Description — solution` | Expliquer une erreur, analyse ou non (2.0.3) — voir « Les codes d'erreur » | `[ERR] E104 …`, `[WARN] E104 …` |

Le bloc `REPORT` n'est donc pas réservé au mode analyse : un script qui n'analyse rien peut
n'y déclarer que ses codes d'erreur.

Chaque clé se traduit dans le bloc `LANG`, comme une option, sans répéter son étiquette :
`## Junk : Fichiers inutiles — Fichiers temporaires et corbeille`. Une clé sans traduction
s'affiche dans la langue de base (`TRADUCTION_RAPPORT`, avertissement).

### Exemple complet — faire de la place

C'est l'exemple de référence de la norme. Il passe le validateur en `-Strict`, et il est
identique au fichier `docs/mockups/exemple-analyse.ps1` — un test y veille. Il montre des
choix mesurés, une case simple, un choix montré sans être coché avec sa note, une
arborescence d'`[items]`, des éléments réservés à l'Expert, une note réservée au Simple,
une mesure et des lignes de journal ; puis l'action, qui ne lit que ce que Sindry lui
renvoie.

```powershell
## SINDRY:START
## id            : 6c08ece0-709a-4210-aacb-eda594e8deee
## lang          : en
## title         : Free up disk space
## desc          : Removes temporary files, downloaded updates and browser caches
## category      : cleaning
## icon          : trash-2
## tags          : temp, cache, disk space, browsers
## version       : 1.0
## author        : Sindry
## created       : 2026-10-10
## updated       : 2026-10-17
## admin         : true
## risk          : low
## duration      : medium
## reversible    : false
## interruptible : true
## reboot        : false
## engine        : auto
## scan          : true
## view          : donut
## panels        : plan progress
## SINDRY:END

## SINDRY:OPTIONS
## Targets    : [multi] Temporary files
##   user     : [group:Junk] Your temporary files
##   windows  : [group:Junk] [show:expert] Windows temporary files
##   update   : [group:OldUpdates] Downloaded Windows updates
## RecycleBin : [bool] [group:Junk] Empty the recycle bin
## Browsers   : [items] [view:tree] Browser caches
## SafeTest   : [bool] Simulate — shows what would be done, changes nothing
## SINDRY:END

## SINDRY:REPORT
## Junk       : [group] Junk files — Temporary files and the recycle bin
## OldUpdates : [group] Old Windows updates — What Windows keeps after installing them
## Cache      : Cache
## FreeSpace  : Free space on drive C:
## UpdateNote : [note:warn] If an update is waiting to be installed, Windows will download it again.
## Untouched  : [note:info] Your passwords, bookmarks and history are not touched.
## SINDRY:END

## SINDRY:LANG fr
## title      : Faire de la place
## desc       : Supprime les fichiers temporaires, les mises à jour téléchargées et le cache des navigateurs
## Targets    : Fichiers temporaires
##   user     : Vos fichiers temporaires
##   windows  : Fichiers temporaires de Windows
##   update   : Mises à jour de Windows téléchargées
## RecycleBin : Vider la corbeille
## Browsers   : Cache des navigateurs
## SafeTest   : Simuler — montre ce qui serait fait, sans rien modifier
## Junk       : Fichiers inutiles — Fichiers temporaires et corbeille
## OldUpdates : Anciennes mises à jour de Windows — Ce que Windows garde après les avoir installées
## Cache      : Fichiers en cache
## FreeSpace  : Espace libre sur le disque C:
## UpdateNote : Si une mise à jour attend d'être installée, Windows la téléchargera de nouveau.
## Untouched  : Vos mots de passe, vos favoris et votre historique ne sont pas touchés.
## SINDRY:END

# ==============================================================================
# L'exemple de référence de la norme d'analyse : docs/FORMAT_SCRIPT.md,
# « Le mode analyse », le reproduit à l'identique, et un test y veille.
# Il passe le validateur en -Strict (tools/lint-scripts.ps1).
#
# Le script est lancé deux fois :
#   1. ANALYSE  — Sindry pose SINDRY_MODE=scan. Le script mesure, décrit ce
#      qu'il a trouvé, et ne modifie RIEN.
#   2. ACTION   — Sindry relance le script sans SINDRY_MODE, avec dans
#      SINDRY_CONFIG ce que l'utilisateur a coché. Le script ne fait que ça.
#
# Seul, en double-clic, il agit avec ses valeurs par défaut ($CONFIG ci-dessous).
# ==============================================================================

$CONFIG = @{
    Targets    = @("user", "windows")
    RecycleBin = $false
    Browsers   = @()
    SafeTest   = $false
}
# Browsers est une liste [items] : ses éléments n'existent qu'après l'analyse.
# Sans Sindry, elle reste vide et le script ne touche à aucun navigateur.

# --- Sindry override (ne pas supprimer) ---
if ($env:SINDRY_CONFIG) {
    ($env:SINDRY_CONFIG | ConvertFrom-Json).PSObject.Properties |
        ForEach-Object { $CONFIG[$_.Name] = $_.Value }
}

# ==============================================================================
# Ce que le script sait faire — utilisé par les deux passages
# ==============================================================================

$Folders = @{
    user    = $env:TEMP
    windows = Join-Path $env:SystemRoot 'Temp'
    update  = Join-Path $env:SystemRoot 'SoftwareDistribution\Download'
}

function Measure-Folder([string] $Path) {
    $m = Get-ChildItem -LiteralPath $Path -Recurse -File -Force -ErrorAction SilentlyContinue |
         Measure-Object -Property Length -Sum
    [pscustomobject]@{ Size = [long]$m.Sum; Count = [int]$m.Count }
}

function Get-RecycleBinItems {
    @((New-Object -ComObject Shell.Application).NameSpace(10).Items())
}

# Les caches de navigateurs. Chaque cache reçoit un Id STABLE, calculé à partir
# de ce qui ne bouge pas entre l'analyse et l'action : le navigateur et le profil.
# Sindry ne renverra que des Id ; c'est cette même fonction, rappelée à l'action,
# qui retrouvera les chemins. Un Id reçu ne sert jamais à fabriquer un chemin.
$Browsers = @(
    @{ Id = 'chrome';  Name = 'Google Chrome';   Process = 'chrome';  Cache = 'Cache';  Root = Join-Path $env:LOCALAPPDATA 'Google\Chrome\User Data' }
    @{ Id = 'edge';    Name = 'Microsoft Edge';  Process = 'msedge';  Cache = 'Cache';  Root = Join-Path $env:LOCALAPPDATA 'Microsoft\Edge\User Data' }
    @{ Id = 'brave';   Name = 'Brave';           Process = 'brave';   Cache = 'Cache';  Root = Join-Path $env:LOCALAPPDATA 'BraveSoftware\Brave-Browser\User Data' }
    @{ Id = 'firefox'; Name = 'Mozilla Firefox'; Process = 'firefox'; Cache = 'cache2'; Root = Join-Path $env:LOCALAPPDATA 'Mozilla\Firefox\Profiles' }
)

function Get-BrowserCaches {
    foreach ($b in $Browsers) {
        if (-not (Test-Path -LiteralPath $b.Root)) { continue }
        $open = [bool](Get-Process -Name $b.Process -ErrorAction SilentlyContinue)
        foreach ($dir in Get-ChildItem -LiteralPath $b.Root -Directory -ErrorAction SilentlyContinue) {
            $path = Join-Path $dir.FullName $b.Cache
            if (-not (Test-Path -LiteralPath $path)) { continue }
            # Firefox préfixe ses profils d'un code (« me5fmut4.default-release ») :
            # on n'affiche que la fin. Un vrai script lirait le nom choisi par
            # l'utilisateur (Local State pour Chrome et Edge, profiles.ini pour Firefox).
            $shown = if ($b.Id -eq 'firefox') { ($dir.Name -split '\.', 2)[-1] } else { $dir.Name }
            [pscustomobject]@{
                Id      = "$($b.Id)-$($dir.Name -replace '[^\w.-]', '-')"
                Browser = $b
                Profile = $shown
                Path    = $path
                Open    = $open
            }
        }
    }
}

# ==============================================================================
# 1. ANALYSE — on mesure, on décrit, on ne modifie RIEN
# ==============================================================================
# Tout ce bloc est en lecture seule. Sindry ne peut pas le vérifier : c'est une
# promesse de l'auteur, et l'utilisateur coche sur la foi de ce qu'elle rapporte.
# Le script n'écrit jamais de phrase : des nombres, des noms, des jetons. Les
# mots viennent de l'entête (REPORT, LANG), déjà traduite.

if ($env:SINDRY_MODE -eq 'scan') {

    # --- Des choix mesurés : une ligne [FIND] par choix de Targets ---
    Write-Output "[STEP] 1/3 Measuring temporary folders"
    foreach ($t in 'user', 'windows') {
        $m = Measure-Folder $Folders[$t]
        Write-Output "[FIND] Targets.$t size=$($m.Size) count=$($m.Count)"
    }

    # --- Une case simple, mesurée ---
    $bin = Get-RecycleBinItems
    $binSize = [long]($bin | Measure-Object -Property Size -Sum).Sum
    Write-Output "[FIND] RecycleBin size=$binSize count=$($bin.Count)"

    # --- Gros, mais peut encore servir : montré, PAS coché, avec sa note ---
    Write-Output "[STEP] 2/3 Measuring downloaded Windows updates"
    $m = Measure-Folder $Folders.update
    Write-Output "[FIND] Targets.update size=$($m.Size) count=$($m.Count) checked=false"
    Write-Output "[NOTE] UpdateNote Targets.update"

    # --- Une arborescence : un parent par navigateur, un enfant par profil ---
    Write-Output "[STEP] 3/3 Looking for browser caches"
    $seen = @{}
    foreach ($c in Get-BrowserCaches) {
        # Navigateur ouvert : visible, mais pas cochable, avec la raison.
        $lock = if ($c.Open) { ' locked=open' } else { '' }
        if (-not $seen[$c.Browser.Id]) {
            $seen[$c.Browser.Id] = $true
            Write-Output "[ITEM] Browsers id=$($c.Browser.Id) name=""$($c.Browser.Name)"" kind=browser$lock"
        }
        $m = Measure-Folder $c.Path
        # Un cache minuscule encombre le résumé d'un débutant : Expert seulement.
        # Il garde sa case, et suit celle de son navigateur.
        $show = if ($m.Size -lt 1MB) { ' show=expert' } else { '' }
        # label= dit ce que c'est (traduit), name= lequel : « Fichiers en cache — Default ».
        Write-Output "[ITEM] Browsers id=$($c.Id) parent=$($c.Browser.Id) kind=folder label=Cache name=""$($c.Profile)"" path=""$($c.Path)"" size=$($m.Size)$lock$show"
        # Une ligne de journal, rangée dans le canal « Browsers » (Expert seulement).
        Write-Output "[LOG] Browsers $($c.Browser.Name) / $($c.Profile): $($m.Count) files"
    }
    # Une note pour le débutant, inutile à l'Expert qui voit la liste exacte.
    Write-Output "[NOTE] Untouched Browsers show=simple"

    # --- Une mesure sans case : l'espace libre, pour situer le reste ---
    $drive = Get-PSDrive -Name C
    $pct = [math]::Round(100 * $drive.Free / ($drive.Used + $drive.Free))
    $health = if ($pct -lt 10) { 'crit' } elseif ($pct -lt 20) { 'warn' } else { 'ok' }
    Write-Output "[METRIC] FreeSpace value=$pct unit=pct health=$health max=100"

    exit 0
}

# ==============================================================================
# 2. ACTION — uniquement ce que l'utilisateur a coché
# ==============================================================================
# Sindry a posé dans SINDRY_CONFIG, et l'override a versé dans $CONFIG :
#   Targets    = les choix cochés, par exemple @("user", "update")
#   RecycleBin = $true ou $false
#   Browsers   = les Id cochés, par exemple @("chrome-Default", "firefox-abcd.default")
# Une valeur reçue ne sert que de CLÉ dans les tables du script, et seulement si
# elle y figure : $Folders['inconnu'] vaut $null, et "$null\*" viserait la racine.

$SafeTest = ("$($CONFIG.SafeTest)" -eq 'True')
$targets  = @($CONFIG.Targets | Where-Object { $Folders.ContainsKey("$_") })
$doBin    = ("$($CONFIG.RecycleBin)" -eq 'True')
$wanted   = @($CONFIG.Browsers | ForEach-Object { "$_" })
$caches   = @(Get-BrowserCaches | Where-Object { $wanted -contains $_.Id })
$total    = $targets.Count + [int]$doBin + [int]($caches.Count -gt 0)
$step     = 0
$freed    = [long]0

function Remove-FolderContent([string] $Path) {
    $n = [long]0
    foreach ($f in Get-ChildItem -LiteralPath $Path -Recurse -File -Force -ErrorAction SilentlyContinue) {
        if ($SafeTest) { $n += $f.Length; continue }
        try {
            Remove-Item -LiteralPath $f.FullName -Force -ErrorAction Stop
            $n += $f.Length
        } catch {
            # Fichier ouvert par un programme : on le laisse, c'est normal.
        }
    }
    $n
}

foreach ($t in $targets) {
    $step++
    Write-Output "[STEP] $step/$total Cleaning $t"
    $stopped = $false
    if ($t -eq 'update' -and -not $SafeTest) {
        # Windows Update garde ses fichiers ouverts : on l'arrête le temps du ménage.
        Stop-Service -Name wuauserv -Force -ErrorAction SilentlyContinue
        $stopped = $true
    }
    $freed += Remove-FolderContent $Folders[$t]
    if ($stopped) { Start-Service -Name wuauserv -ErrorAction SilentlyContinue }
}

if ($doBin) {
    $step++
    Write-Output "[STEP] $step/$total Emptying the recycle bin"
    $freed += [long](Get-RecycleBinItems | Measure-Object -Property Size -Sum).Sum
    if (-not $SafeTest) { Clear-RecycleBin -Force -ErrorAction SilentlyContinue }
}

if ($caches.Count) {
    $step++
    Write-Output "[STEP] $step/$total Clearing browser caches"
    foreach ($c in $caches) {
        # Le navigateur a pu être ouvert depuis l'analyse : on revérifie.
        if (Get-Process -Name $c.Browser.Process -ErrorAction SilentlyContinue) {
            Write-Output "[WARN] $($c.Browser.Name) is open - cache left in place"
            continue
        }
        $freed += Remove-FolderContent $c.Path
        Write-Output "[LOG] Browsers Cleared $($c.Browser.Name) / $($c.Profile)"
    }
}

# Ce qui a vraiment été libéré : le bilan l'affiche, l'historique le garde.
Write-Output "[FREED] $freed"
Write-Output "[DONE] Disk space freed"
exit 0
```

Remarquez que les clés `Targets.$t` sont calculées. Le validateur ne peut vérifier que les
clés écrites en toutes lettres ; une clé calculée lui échappe, par construction. À vous de
garantir qu'elle ne produit que des choix déclarés — Sindry, lui, ignorera les autres.

---

## 7. La vérification finale

Avant de livrer, vérifie chaque point. Un « non » se corrige avant la livraison.

### Forme

- [ ] Le fichier commence par `## SINDRY:START` et chaque bloc se ferme par `## SINDRY:END`.
- [ ] Les quatorze champs obligatoires de l'entête sont présents, une seule fois chacun.
- [ ] `id` est un GUID neuf ; `category` vient de la section 5.1 ; `icon` de la section 5.4.
- [ ] `author`, `created`, `updated` et `version` sont présents et à jour ; dates au format
      `AAAA-MM-JJ`, `updated` ≥ `created`, aucune dans le futur.

### Options

- [ ] Chaque clé est identique dans `OPTIONS`, `LANG fr` et `$CONFIG`, sans espace.
- [ ] Chaque type est en minuscules ; chaque `select` ou `multi` a au moins deux choix.
- [ ] Chaque valeur par défaut est du bon type : un choix déclaré pour un `select`, un
      tableau pour un `multi` ou un `items`, `$true`/`$false` pour un `bool` ou un `hidden`.
- [ ] `$CONFIG = @{` tient sur une ligne et `}` est seule sur la sienne.
- [ ] La ligne d'override est recopiée à l'identique, hors commentaire.

### Traduction

- [ ] `## SINDRY:LANG fr` traduit `title`, `desc`, chaque option, chaque choix et chaque clé
      de `REPORT`, et rien d'autre.

### Sortie

- [ ] Chaque message affiché commence par un marqueur en majuscules et ne contient aucun
      accent.
- [ ] Le script se termine par `exit 0`, et chaque chemin d'échec par `exit 1`.
- [ ] Chaque `[ERR] X123` ou `[WARN] X123` vise un code déclaré `[error]` dans `REPORT`, et
      chaque code déclaré est utilisé.
- [ ] Si `SafeTest` existe, rien ne modifie le système quand il vaut `$true`.

### Analyse (si `scan : true`)

- [ ] `$env:SINDRY_MODE` est testé juste après l'override, avant toute modification, et la
      branche se termine par `exit 0`.
- [ ] Au moins une ligne `[FIND]`, `[ITEM]` ou `[METRIC]` est émise, et chacune vise ce qui
      est déclaré.
- [ ] Les `id=` sont stables ; à l'action, ils ne servent que de clés.

### Sécurité et style

- [ ] Aucune valeur n'est exécutée ; les outils du système sont appelés par leur chemin
      complet ; aucun chemin de suppression ne peut viser un emplacement protégé.
- [ ] Commentaires en français (chaque variable sur sa ligne, chaque fonction à son début),
      code et sortie en anglais ; chapitres au-delà de 200 lignes.
- [ ] Les drapeaux et listes par défaut sont regroupés avec les variables en tête.
- [ ] Le journal est aéré, en anglais seulement ; chaque erreur prévisible a son code.
- [ ] Le fichier est à enregistrer en **UTF-8 avec BOM** — rappelle-le à l'utilisateur.
