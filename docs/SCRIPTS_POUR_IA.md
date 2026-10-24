# Sindry — écrire un script PowerShell conforme

> **Référence complète, pour une IA.** Ce fichier est généré par
> `tools/build-script-guides.mjs` (Sindry 2.1.0) à partir de `docs/FORMAT_SCRIPT.md`,
> `tools/categories.json`, `tools/lots.json` et `tools/lucide-icon-names.txt`. Ne le modifiez
> pas à la main : modifiez ses sources, puis relancez le générateur.

Ce document contient **tout** ce qu'il faut pour écrire un script que Sindry accepte : la
mission, le contexte, les règles impératives résumées, les données de référence, puis le
contrat complet, et une liste de vérification finale. En cas de doute entre le résumé
(sections 1 à 5) et le contrat (section 6), **le contrat fait foi**.

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
    incohérent ; `reboot : true` s'il faut redémarrer **à la fin** ; `resume : true` s'il
    peut demander un redémarrage **au milieu**, puis reprendre (règle 42).
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
    `[FREED] <octets>`, `[LOG] Canal texte`, `[RESTART] <phase>` (règle 42).
23. **Le seul verdict est `exit`** : termine par `exit 0` en cas de réussite, `exit 1` en
    cas d'échec, `exit 3010` seulement juste après un `[RESTART]` (règle 42). Un `[ERR]`
    seul ne fait pas échouer le script.
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

### Le redémarrage au milieu d'un entretien (si le script en a besoin)

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

Les 2112 noms admis, et aucun autre :

a-arrow-down, a-arrow-up, accessibility, activity, activity-square, ad, airplay, air-vent, a-large-small, alarm-check, alarm-clock, alarm-clock-check, alarm-clock-minus, alarm-clock-off, alarm-clock-plus, alarm-minus, alarm-plus, alarm-smoke, album, alert-circle, alert-octagon, alert-triangle, align-center, align-center-horizontal, align-center-vertical, align-end-horizontal, align-end-vertical, align-horizontal-distribute-center, align-horizontal-distribute-end, align-horizontal-distribute-start, align-horizontal-justify-center, align-horizontal-justify-end, align-horizontal-justify-start, align-horizontal-space-around, align-horizontal-space-between, align-justify, align-left, align-right, align-start-horizontal, align-start-vertical, align-vertical-distribute-center, align-vertical-distribute-end, align-vertical-distribute-start, align-vertical-justify-center, align-vertical-justify-end, align-vertical-justify-start, align-vertical-space-around, align-vertical-space-between, ambulance, ampersand, ampersands, amphora, anchor, angle, angry, annoyed, antenna, anvil, aperture, apple, app-window, app-window-mac, archive, archive-restore, archive-x, area-chart, armchair, arrow-big-down, arrow-big-down-dash, arrow-big-left, arrow-big-left-dash, arrow-big-right, arrow-big-right-dash, arrow-big-up, arrow-big-up-dash, arrow-down, arrow-down-01, arrow-down-0-1, arrow-down-10, arrow-down-1-0, arrow-down-az, arrow-down-a-z, arrow-down-circle, arrow-down-from-line, arrow-down-left, arrow-down-left-from-circle, arrow-down-left-from-square, arrow-down-left-square, arrow-down-narrow-wide, arrow-down-right, arrow-down-right-from-circle, arrow-down-right-from-square, arrow-down-right-square, arrow-down-square, arrow-down-to-dot, arrow-down-to-line, arrow-down-up, arrow-down-wide-narrow, arrow-down-za, arrow-down-z-a, arrow-left, arrow-left-circle, arrow-left-from-line, arrow-left-right, arrow-left-square, arrow-left-to-line, arrow-right, arrow-right-circle, arrow-right-from-line, arrow-right-left, arrow-right-square, arrow-right-to-line, arrows-up-from-line, arrow-up, arrow-up-01, arrow-up-0-1, arrow-up-10, arrow-up-1-0, arrow-up-az, arrow-up-a-z, arrow-up-circle, arrow-up-down, arrow-up-from-dot, arrow-up-from-line, arrow-up-left, arrow-up-left-from-circle, arrow-up-left-from-square, arrow-up-left-square, arrow-up-narrow-wide, arrow-up-right, arrow-up-right-from-circle, arrow-up-right-from-square, arrow-up-right-square, arrow-up-square, arrow-up-to-line, arrow-up-wide-narrow, arrow-up-za, arrow-up-z-a, asterisk, asterisk-square, astroid, atom, at-sign, audio-lines, audio-lines-off, audio-lines-x, audio-waveform, award, axe, axis-3d, axis-3-d, baby, backpack, badge, badge-alert, badge-cent, badge-check, badge-dollar-sign, badge-euro, badge-help, badge-indian-rupee, badge-info, badge-japanese-yen, badge-minus, badge-percent, badge-plus, badge-pound-sterling, badge-question-mark, badge-russian-ruble, badge-swiss-franc, badge-turkish-lira, badge-x, baggage-claim, balloon, ban, banana, bandage, banknote, banknote-arrow-down, banknote-arrow-up, banknote-check, banknote-x, bar-chart, bar-chart-2, bar-chart-3, bar-chart-4, bar-chart-big, bar-chart-horizontal, bar-chart-horizontal-big, barcode, barrel, baseline, bath, battery, battery-charging, battery-full, battery-low, battery-medium, battery-plus, battery-warning, beaker, bean, bean-off, bed, bed-double, bed-single, beef, beef-off, beer, beer-off, bell, bell-check, bell-dot, bell-electric, bell-minus, bell-off, bell-plus, bell-ring, between-horizonal-end, between-horizonal-start, between-horizontal-end, between-horizontal-start, between-vertical-end, between-vertical-start, biceps-flexed, bike, binary, binoculars, biohazard, bird, birdhouse, bitcoin, blend, blender, blinds, blocks, bluetooth, bluetooth-connected, bluetooth-off, bluetooth-searching, bold, bolt, bomb, bone, bone-fracture, book, book-a, book-alert, book-audio, book-bookmark, book-check, book-copy, book-dashed, book-down, book-headphones, book-heart, book-image, book-key, book-lock, bookmark, bookmark-check, book-marked, bookmark-minus, bookmark-off, bookmark-plus, bookmark-x, book-minus, book-open, book-open-check, book-open-text, book-plus, book-search, book-template, book-text, book-type, book-up, book-up-2, book-user, book-x, boom-box, bot, bot-message-square, bot-off, bottle-wine, bow-arrow, box, boxes, box-select, braces, brackets, brain, brain-circuit, brain-cog, brick-wall, brick-wall-fire, brick-wall-shield, bridge, briefcase, briefcase-business, briefcase-conveyor-belt, briefcase-medical, bring-to-front, broccoli, broom, broom-sparkles, brush, brush-cleaning, bubbles, bug, bug-off, bug-play, building, building-2, building-complex, building-complex-plus, bus, bus-front, cable, cable-car, cake, cake-slice, calculator, calendar, calendar-1, calendar-arrow-down, calendar-arrow-up, calendar-check, calendar-check-2, calendar-chevrons-right, calendar-clock, calendar-cog, calendar-days, calendar-fold, calendar-heart, calendar-minus, calendar-minus-2, calendar-off, calendar-plus, calendar-plus-2, calendar-range, calendars, calendar-search, calendar-sync, calendar-x, calendar-x-2, camera, camera-off, can, candlestick-chart, candy, candy-cane, candy-off, cannabis, cannabis-off, can-soda, captions, captions-off, car, caravan, car-battery, card-sim, car-front, carrot, car-taxi-front, carton, carton-off, case-lower, case-sensitive, case-upper, cassette-tape, cast, castle, cat, cctv, cctv-off, chart-area, chart-bar, chart-bar-big, chart-bar-decreasing, chart-bar-increasing, chart-bar-stacked, chart-candlestick, chart-column, chart-column-big, chart-column-decreasing, chart-column-increasing, chart-column-stacked, chart-gantt, chart-line, chart-network, chart-no-axes-column, chart-no-axes-column-decreasing, chart-no-axes-column-increasing, chart-no-axes-combined, chart-no-axes-gantt, chart-pie, chart-scatter, chart-spline, check, check-check, check-circle, check-circle-2, check-line, check-square, check-square-2, chef-hat, cherry, chess-bishop, chess-king, chess-knight, chess-pawn, chess-queen, chess-rook, chevron-down, chevron-down-circle, chevron-down-square, chevron-first, chevron-last, chevron-left, chevron-left-circle, chevron-left-square, chevron-right, chevron-right-circle, chevron-right-square, chevrons-down, chevrons-down-up, chevrons-left, chevrons-left-right, chevrons-left-right-ellipsis, chevrons-right, chevrons-right-left, chevrons-up, chevrons-up-down, chevron-up, chevron-up-circle, chevron-up-square, church, cigarette, cigarette-off, circle, circle-alert, circle-arrow-down, circle-arrow-left, circle-arrow-out-down-left, circle-arrow-out-down-right, circle-arrow-out-up-left, circle-arrow-out-up-right, circle-arrow-right, circle-arrow-up, circle-check, circle-check-big, circle-chevron-down, circle-chevron-left, circle-chevron-right, circle-chevron-up, circle-dashed, circle-dashed-check, circle-divide, circle-dollar-sign, circle-dot, circle-dot-dashed, circle-ellipsis, circle-equal, circle-euro, circle-fading-arrow-up, circle-fading-plus, circle-gauge, circle-help, circle-minus, circle-off, circle-parking, circle-parking-off, circle-pause, circle-percent, circle-pile, circle-play, circle-plus, circle-pound-sterling, circle-power, circle-question-mark, circle-slash, circle-slash-2, circle-slashed, circle-small, circle-star, circle-stop, circle-user, circle-user-round, circle-x, circuit-board, citrus, clapperboard, clef-alto, clef-bass, clef-treble, clipboard, clipboard-check, clipboard-clock, clipboard-copy, clipboard-edit, clipboard-list, clipboard-minus, clipboard-paste, clipboard-pen, clipboard-pen-line, clipboard-plus, clipboard-signature, clipboard-type, clipboard-x, clock, clock-1, clock-10, clock-11, clock-12, clock-2, clock-3, clock-4, clock-5, clock-6, clock-7, clock-8, clock-9, clock-alert, clock-arrow-down, clock-arrow-left, clock-arrow-right, clock-arrow-up, clock-check, clock-fading, clock-plus, closed-caption, cloud, cloud-alert, cloud-backup, cloud-check, cloud-cog, cloud-download, cloud-drizzle, cloud-fog, cloud-hail, cloud-lightning, cloud-moon, cloud-moon-rain, cloud-off, cloud-rain, cloud-rain-wind, cloud-snow, cloud-sun, cloud-sun-rain, cloud-sync, cloud-upload, cloudy, clover, club, code, code-2, code-square, code-xml, coffee, cog, coins, columns, columns-2, columns-3, columns-3-cog, columns-4, columns-settings, combine, command, compass, component, computer, concierge-bell, cone, construction, contact, contact-2, contact-round, container, contrast, cookie, cooking-pot, copy, copy-check, copyleft, copy-minus, copy-plus, copyright, copy-slash, copy-x, corner-down-left, corner-down-right, corner-left-down, corner-left-up, corner-right-down, corner-right-up, corner-up-left, corner-up-right, cpu, creative-commons, credit-card, credit-card-check, credit-card-minus, credit-card-plus, credit-card-reader, credit-card-x, croissant, crop, cross, crosshair, crown, cuboid, cupcake, cup-soda, curly-braces, currency, cylinder, dam, database, database-arrow-down, database-arrow-up, database-backup, database-check, database-minus, database-plus, database-search, database-x, database-zap, decimals-arrow-left, decimals-arrow-right, delete, dessert, diameter, diamond, diamond-minus, diamond-percent, diamond-plus, dice-1, dice-2, dice-3, dice-4, dice-5, dice-6, dices, diff, disc, disc-2, disc-3, disc-album, divide, divide-circle, divide-square, dna, dna-off, dock, dog, dollar-sign, dome, donut, door-closed, door-closed-locked, door-closed-package, door-open, door-stairwell, dot, dot-square, download, download-cloud, drafting-compass, drama, drill, drone, droplet, droplet-off, droplets, drum, drumstick, dumbbell, ear, ear-off, earth, earth-lock, eclipse, edit, edit-2, edit-3, egg, egg-fried, egg-off, eject, ellipse, ellipsis, ellipsis-vertical, engine, equal, equal-approximately, equal-approximately-not, equal-not, equal-square, eraser, ethernet-port, euro, ev-charger, expand, external-link, eye, eye-closed, eye-dashed, eye-off, face-angry, face-expressionless, face-grinning, face-neutral, face-slightly-frowning, face-slightly-smiling, face-slightly-smiling-plus, factory, fan, fast-forward, faucet, feather, fence, ferris-wheel, file, file-archive, file-audio, file-audio-2, file-axis-3d, file-axis-3-d, file-badge, file-badge-2, file-bar-chart, file-bar-chart-2, file-box, file-braces, file-braces-corner, file-chart-column, file-chart-column-increasing, file-chart-line, file-chart-pie, file-check, file-check-2, file-check-corner, file-clock, file-code, file-code-2, file-code-corner, file-cog, file-cog-2, file-diff, file-digit, file-down, file-edit, file-exclamation-point, file-headphone, file-heart, file-image, file-input, file-json, file-json-2, file-key, file-key-2, file-line-chart, file-lock, file-lock-2, file-minus, file-minus-2, file-minus-corner, file-music, file-output, file-pen, file-pen-line, file-pie-chart, file-play, file-plus, file-plus-2, file-plus-corner, file-question, file-question-mark, files, file-scan, file-search, file-search-2, file-search-corner, file-signal, file-signature, file-sliders, file-spreadsheet, file-stack, file-symlink, file-terminal, file-text, file-type, file-type-2, file-type-corner, file-up, file-user, file-video, file-video-2, file-video-camera, file-volume, file-volume-2, file-warning, file-x, file-x-2, file-x-corner, film, filter, filter-x, fingerprint, fingerprint-pattern, fire-extinguisher, fish, fishing-hook, fishing-rod, fish-off, fish-symbol, flag, flag-off, flag-triangle-left, flag-triangle-right, flame, flame-kindling, flashlight, flashlight-off, flask-conical, flask-conical-off, flask-round, flip-horizontal, flip-horizontal-2, flip-vertical, flip-vertical-2, flower, flower-2, focus, folder, folder-archive, folder-bookmark, folder-check, folder-clock, folder-closed, folder-code, folder-cog, folder-cog-2, folder-dot, folder-down, folder-edit, folder-git, folder-git-2, folder-heart, folder-input, folder-kanban, folder-key, folder-lock, folder-minus, folder-open, folder-open-dot, folder-output, folder-pen, folder-plus, folder-root, folders, folder-search, folder-search-2, folder-symlink, folder-sync, folder-tree, folder-up, folder-x, fold-horizontal, fold-vertical, footprints, fork-knife, fork-knife-crossed, forklift, form, form-input, forward, frame, frown, fuel, fullscreen, function-square, funnel, funnel-plus, funnel-x, galaxy, gallery-horizontal, gallery-horizontal-end, gallery-thumbnails, gallery-vertical, gallery-vertical-end, gamepad, gamepad-2, gamepad-directional, gantt-chart, gantt-chart-square, gap-horizontal, gap-vertical, gauge, gauge-circle, gavel, gem, georgian-lari, germ, germ-off, ghost, gift, git-branch, git-branch-minus, git-branch-plus, git-commit, git-commit-horizontal, git-commit-vertical, git-compare, git-compare-arrows, git-fork, git-graph, git-merge, git-merge-conflict, git-pull-request, git-pull-request-arrow, git-pull-request-closed, git-pull-request-create, git-pull-request-create-arrow, git-pull-request-draft, glasses, glass-water, globe, globe-2, globe-check, globe-code, globe-lock, globe-off, globe-x, goal, gpu, grab, graduation-cap, grape, grid, grid-2x2, grid-2-x-2, grid-2x2-check, grid-2-x-2-check, grid-2x2-plus, grid-2-x-2-plus, grid-2x2-x, grid-2-x-2-x, grid-3x2, grid-3x3, grid-3-x-3, grip, grip-horizontal, grip-vertical, group, guitar, ham, hamburger, hammer, hand, handbag, hand-coins, hand-fist, hand-grab, hand-heart, hand-helping, hand-metal, hand-platter, handshake, hard-drive, hard-drive-download, hard-drive-upload, hard-hat, hash, hat-glasses, haze, hd, hdmi-port, heading, heading-1, heading-2, heading-3, heading-4, heading-5, heading-6, headphone-off, headphones, headset, heart, heart-crack, heart-handshake, heart-minus, heart-off, heart-plus, heart-pulse, heart-x, heater, helicopter, help-circle, helping-hand, hexagon, highlighter, history, home, hop, hop-off, hospital, hotel, hourglass, hourglass-cog, house, house-heart, house-plug, house-plus, houses, house-wifi, ice-cream, ice-cream-2, ice-cream-bowl, ice-cream-cone, id-card, id-card-lanyard, image, image-down, image-minus, image-off, image-play, image-plus, images, image-up, image-upscale, import, inbox, indent, indent-decrease, indent-increase, indian-rupee, infinity, info, inspect, inspection-panel, italic, iteration-ccw, iteration-cw, iv-bag, japanese-yen, joystick, kanban, kanban-square, kanban-square-dashed, kayak, key, keyboard, keyboard-music, keyboard-off, key-round, key-square, lambda, lamp, lamp-ceiling, lamp-desk, lamp-floor, lamp-wall-down, lamp-wall-up, landmark, land-plot, languages, laptop, laptop-2, laptop-minimal, laptop-minimal-check, lasso, lasso-select, laugh, layer-arrow-down, layer-arrow-up, layers, layers-2, layers-3, layers-arrow-down, layers-arrow-up, layers-minus, layers-plus, layout, layout-arrow-down, layout-arrow-right, layout-dashboard, layout-freeform, layout-grid, layout-list, layout-panel-left, layout-panel-top, layout-template, leaf, leafy-green, lectern, lens-concave, lens-convex, letter-text, library, library-big, library-square, life-buoy, ligature, lightbulb, lightbulb-off, lighthouse, line-chart, line-dot-right-horizontal, line-squiggle, line-style, link, link-2, link-2-off, list, list-check, list-checks, list-chevrons-down-up, list-chevrons-up-down, list-clock, list-collapse, list-end, list-filter, list-filter-plus, list-indent-decrease, list-indent-increase, list-minus, list-music, list-ordered, list-plus, list-restart, list-sort-ascending, list-sort-descending, list-start, list-todo, list-tree, list-video, list-x, loader, loader-2, loader-circle, loader-pinwheel, locate, locate-fixed, locate-off, location-edit, lock, lock-keyhole, lock-keyhole-open, lock-open, log-in, log-out, logs, lollipop, luggage, magnet, mail, mail-badge, mailbox, mail-check, mail-clock, mail-minus, mail-open, mail-pen, mail-plus, mail-question, mail-question-mark, mails, mail-search, mail-warning, mail-x, map, map-minus, map-pin, map-pin-check, map-pin-check-inside, map-pin-house, map-pin-minus, map-pin-minus-inside, map-pinned, map-pin-off, map-pin-pen, map-pin-plus, map-pin-plus-inside, map-pin-search, map-pin-x, map-pin-x-inside, map-plus, mars, mars-stroke, martini, maximize, maximize-2, medal, megaphone, megaphone-off, meh, memory-stick, menu, menu-square, merge, message-circle, message-circle-check, message-circle-code, message-circle-dashed, message-circle-dashed-check, message-circle-heart, message-circle-more, message-circle-off, message-circle-plus, message-circle-question, message-circle-question-mark, message-circle-reply, message-circle-warning, message-circle-x, messages-circle, message-square, message-square-check, message-square-code, message-square-dashed, message-square-diff, message-square-dot, message-square-heart, message-square-lock, message-square-more, message-square-off, message-square-plus, message-square-quote, message-square-reply, message-square-share, message-square-text, message-square-warning, message-square-x, messages-square, metronome, mic, mic-2, mic-audio-lines, mic-off, microchip, microscope, microwave, mic-signal, mic-vocal, midi-port, milestone, milk, milk-off, minimize, minimize-2, minus, minus-circle, minus-square, mirror-rectangular, mirror-round, monitor, monitor-check, monitor-cloud, monitor-cog, monitor-dot, monitor-down, monitor-off, monitor-pause, monitor-pc, monitor-play, monitor-smartphone, monitor-speaker, monitor-stop, monitor-up, monitor-x, moon, moon-star, mop, mop-sparkles, more-horizontal, more-vertical, mosque, motorbike, mountain, mountain-snow, mouse, mouse-left, mouse-off, mouse-pointer, mouse-pointer-2, mouse-pointer-2-off, mouse-pointer-ban, mouse-pointer-click, mouse-pointer-square-dashed, mouse-right, mouth, mouth-off, move, move-3d, move-3-d, move-diagonal, move-diagonal-2, move-down, move-down-left, move-down-right, move-horizontal, move-left, move-right, move-up, move-up-left, move-up-right, move-vertical, m-square, music, music-2, music-3, music-4, navigation, navigation-2, navigation-2-off, navigation-off, nepali-rupee, network, newspaper, nfc, non-binary, notebook, notebook-dot, notebook-pen, notebook-tabs, notebook-text, notepad-text, notepad-text-dashed, nut, nut-off, octagon, octagon-alert, octagon-minus, octagon-pause, octagon-x, omega, option, orbit, origami, outdent, package, package-2, package-check, package-minus, package-open, package-plus, package-search, package-x, paintbrush, paintbrush-2, paintbrush-vertical, paint-bucket, paint-roller, palette, palmtree, panda, panel-bottom, panel-bottom-close, panel-bottom-dashed, panel-bottom-inactive, panel-bottom-open, panel-left, panel-left-close, panel-left-dashed, panel-left-inactive, panel-left-open, panel-left-right-dashed, panel-right, panel-right-close, panel-right-dashed, panel-right-inactive, panel-right-open, panels-left-bottom, panels-left-right, panels-right-bottom, panels-top-bottom, panels-top-left, panel-top, panel-top-bottom-dashed, panel-top-close, panel-top-dashed, panel-top-inactive, panel-top-open, paper-bag, paperclip, parasol, parentheses, park, parking-circle, parking-circle-off, parking-meter, parking-square, parking-square-off, party-popper, pause, pause-circle, pause-octagon, paw-print, pc-case, pen, pen-box, pencil, pencil-line, pencil-off, pencil-ruler, pencil-sparkles, pen-line, pen-off, pen-square, pentagon, pen-tool, percent, percent-circle, percent-diamond, percent-square, person-standing, phi, philippine-peso, phone, phone-call, phone-forwarded, phone-incoming, phone-missed, phone-off, phone-outgoing, pi, piano, pickaxe, picture-in-picture, picture-in-picture-2, pie-chart, piggy-bank, pilcrow, pilcrow-left, pilcrow-right, pilcrow-square, pill, pill-bottle, pin, pin-off, pipette, pi-square, pizza, plane, plane-landing, plane-takeoff, plant-pot, play, play-circle, playing-card, playing-cards, playing-cards-fan, play-off, play-square, plug, plug-2, plug-zap, plug-zap-2, plus, plus-circle, plus-square, pocket-knife, podcast, podium, pointer, pointer-off, popcorn, popsicle, pound-sterling, power, power-circle, power-off, power-square, presentation, printer, printer-check, printer-x, projector, proportions, puzzle, pyramid, qr-code, quote, rabbit, radar, radiation, radical, radio, radio-off, radio-receiver, radio-tower, radius, rainbow, rat, ratio, receipt, receipt-cent, receipt-euro, receipt-indian-rupee, receipt-japanese-yen, receipt-pound-sterling, receipt-russian-ruble, receipt-swiss-franc, receipt-text, receipt-turkish-lira, rectangle-circle, rectangle-ellipsis, rectangle-goggles, rectangle-horizontal, rectangle-vertical, recycle, redo, redo-2, redo-dot, refresh-ccw, refresh-ccw-dot, refresh-cw, refresh-cw-off, refrigerator, regex, remove-formatting, repeat, repeat-1, repeat-2, repeat-off, replace, replace-all, reply, reply-all, rewind, ribbon, road, robot-arm, robot-vacuum, rocket, rocking-chair, roller-coaster, rose, rotate-3d, rotate-3-d, rotate-ccw, rotate-ccw-clock, rotate-ccw-key, rotate-ccw-square, rotate-cw, rotate-cw-clock, rotate-cw-fading-clock, rotate-cw-square, route, route-off, router, rows, rows-2, rows-3, rows-4, rss, ruler, ruler-dimension-line, russian-ruble, sailboat, salad, sandwich, satellite, satellite-dish, saudi-riyal, save, save-all, save-check, save-off, save-pen, save-plus, scale, scale-3d, scale-3-d, scaling, scan, scan-barcode, scan-box, scan-eye, scan-face, scan-heart, scan-line, scan-qr-code, scan-search, scan-square, scan-text, scatter-chart, school, school-2, scissors, scissors-line-dashed, scissors-square, scissors-square-dashed-bottom, scooter, screen-share, screen-share-off, scroll, scroll-text, search, search-alert, search-check, search-code, search-slash, search-x, section, send, send-horizonal, send-horizontal, send-to-back, separator-horizontal, separator-vertical, server, server-cog, server-crash, server-off, server-plus, settings, settings-2, shapes, share, share-2, sheet, shell, shelving-unit, shield, shield-alert, shield-ban, shield-check, shield-close, shield-cog, shield-cog-corner, shield-ellipsis, shield-half, shield-keyhole, shield-lock, shield-minus, shield-off, shield-plus, shield-question, shield-question-mark, shield-user, shield-x, ship, ship-cargo, ship-wheel, shirt, shopping-bag, shopping-basket, shopping-cart, shopping-cart-minus, shopping-cart-plus, shovel, shower-head, shredder, shrimp, shrimp-off, shrink, shrub, shuffle, sidebar, sidebar-close, sidebar-open, sigma, sigma-square, signal, signal-high, signal-low, signal-medium, signal-zero, signature, signpost, signpost-big, siren, skip-back, skip-forward, skull, slash, slash-square, slice, sliders, sliders-horizontal, sliders-vertical, smartphone, smartphone-charging, smartphone-nfc, smile, smile-plus, snail, snowflake, soap-dispenser-droplet, sofa, solar-panel, sort-asc, sort-desc, soup, space, spade, sparkle, sparkles, speaker, speech, spell-check, spell-check-2, spline, spline-pointer, split, split-square-horizontal, split-square-vertical, spool, sport-shoe, spotlight, spray-can, sprout, square, square-activity, square-arrow-down, square-arrow-down-left, square-arrow-down-right, square-arrow-left, square-arrow-out-down-left, square-arrow-out-down-right, square-arrow-out-up-left, square-arrow-out-up-right, square-arrow-right, square-arrow-right-enter, square-arrow-right-exit, square-arrow-up, square-arrow-up-left, square-arrow-up-right, square-asterisk, square-bookmark, square-bottom-dashed-scissors, square-centerline-dashed-horizontal, square-centerline-dashed-vertical, square-chart-gantt, square-check, square-check-big, square-chevron-down, square-chevron-left, square-chevron-right, square-chevron-up, square-code, square-dashed, square-dashed-bottom, square-dashed-bottom-code, square-dashed-kanban, square-dashed-mouse-pointer, square-dashed-plus, square-dashed-text, square-dashed-top-solid, square-dashed-x, square-dashed-x-corner, square-dimensions, square-divide, square-dot, square-equal, square-exclamation-point, square-function, square-gantt-chart, square-kanban, square-library, square-m, square-menu, square-minus, square-mouse-pointer, square-off, square-parking, square-parking-off, square-pause, square-pen, square-percent, square-pi, square-pilcrow, square-play, square-plus, square-power, square-radical, square-round-corner, square-scissors, squares-exclude, square-sigma, squares-intersect, square-slash, square-split-horizontal, square-split-vertical, square-square, squares-subtract, square-stack, square-star, square-stop, squares-unite, square-terminal, square-text, square-user, square-user-round, square-x, squircle, squircle-dashed, squirrel, stamp, star, star-check, star-half, star-minus, star-off, star-plus, stars, star-x, step-back, step-forward, stethoscope, sticker, sticky-note, sticky-note-check, sticky-note-minus, sticky-note-off, sticky-note-plus, sticky-notes, sticky-note-x, stone, stop-circle, store, stretch-horizontal, stretch-vertical, strikethrough, subscript, subtitles, summary, sun, sun-dim, sun-medium, sun-moon, sunrise, sunset, sun-snow, superscript, swatch-book, swiss-franc, switch-camera, sword, swords, syringe, table, table-2, table-cells-merge, table-cells-split, table-columns-split, table-config, table-of-contents, table-properties, table-rows-split, tablet, tablets, tablet-smartphone, tag, tag-plus, tags, tag-x, tally-1, tally-2, tally-3, tally-4, tally-5, tangent, target, telescope, tent, tent-tree, terminal, terminal-square, test-tube, test-tube-2, test-tube-diagonal, test-tubes, text, text-align-center, text-align-end, text-align-justify, text-align-start, text-cursor, text-cursor-input, text-initial, text-quote, text-search, text-select, text-selection, text-wrap, theater, thermometer, thermometer-snowflake, thermometer-sun, thumbs-down, thumbs-up, ticket, ticket-check, ticket-minus, ticket-percent, ticket-plus, tickets, ticket-slash, tickets-plane, ticket-x, tic-tac-toe, timeline, timer, timer-off, timer-reset, toggle-left, toggle-right, toilet, toolbox, tool-case, toothbrush, toothbrush-sparkles, tornado, torus, touchpad, touchpad-off, towel-rack, tower-control, toy-brick, tractor, traffic-cone, trailer, train, train-front, train-front-tunnel, train-track, tram-front, transgender, trash, trash-2, trash-off, tree-deciduous, tree-palm, tree-pine, trees, trending-down, trending-up, trending-up-down, triangle, triangle-alert, triangle-dashed, triangle-right, triangles-centerline-dashed-horizontal, triangles-centerline-dashed-vertical, trophy, truck, truck-electric, tube-lotion, turkish-lira, turntable, turtle, tv, tv-2, tv-minimal, tv-minimal-play, type, type-outline, umbrella, umbrella-off, underline, undo, undo-2, undo-dot, unfold-horizontal, unfold-vertical, ungroup, university, unlink, unlink-2, unlock, unlock-keyhole, unplug, upload, upload-cloud, usb, usb-c-port, user, user-2, user-check, user-check-2, user-circle, user-circle-2, user-cog, user-cog-2, user-group, user-key, user-lock, user-minus, user-minus-2, user-pen, user-plus, user-plus-2, user-round, user-round-arrow-left, user-round-check, user-round-cog, user-round-group, user-round-key, user-round-minus, user-round-pen, user-round-plus, user-round-search, user-round-x, users, users-2, user-search, user-shield, user-square, user-square-2, users-round, user-star, user-x, user-x-2, utensils, utensils-crossed, utility-pole, van, variable, vault, vector-polygon, vector-square, vegan, venetian-mask, venus, venus-and-mars, verified, vibrate, vibrate-off, video, video-off, videotape, view, virus, virus-off, voicemail, volleyball, volume, volume-1, volume-2, volume-off, volume-x, vote, wallet, wallet-2, wallet-cards, wallet-minimal, wallpaper, wand, wand-2, wand-sparkles, warehouse, washing-machine, watch, waves, waves-arrow-down, waves-arrow-up, waves-horizontal, waves-ladder, waves-vertical, waypoints, webcam, webcam-off, webhook, webhook-off, weight, weight-tilde, wheat, wheat-off, whistle, whole-word, wifi, wifi-cog, wifi-high, wifi-low, wifi-off, wifi-pen, wifi-sync, wifi-zero, wind, wind-arrow-down, wine, wine-off, workflow, worm, wrap-text, wrench, wrench-off, x, x-circle, x-line-top, x-octagon, x-square, zap, zap-off, zodiac-aquarius, zodiac-aries, zodiac-cancer, zodiac-capricorn, zodiac-gemini, zodiac-leo, zodiac-libra, zodiac-ophiuchus, zodiac-pisces, zodiac-sagittarius, zodiac-scorpio, zodiac-taurus, zodiac-virgo, zoom-in, zoom-out

---

## 6. Le contrat complet — docs/FORMAT_SCRIPT.md

Reproduit en entier. Les titres sont décalés de deux niveaux ; rien d'autre ne change.

### Écrire un script Sindry

Remplace `CREER_UN_SCRIPT.txt`, dont la convention n'était respectée par **aucun** des
13 scripts livrés. Pour que cela ne se reproduise pas, tout ce qui est décrit ici est
**vérifié par `tools/lint-scripts.ps1`** — et la dernière section liste les contrôles un
par un. Une règle ne peut plus changer d'un côté sans l'autre.

Référence normative : `docs/SPECIFICATION.md` §5.

**Trois versions de ce guide, générées depuis ce fichier** (`node tools/build-script-guides.mjs`) :
`docs/GUIDE_SCRIPTS.html`, à ouvrir dans un navigateur — recherche, générateur de squelette ;
`docs/SCRIPTS_POUR_IA.md`, à donner à une IA pour qu'elle écrive un script conforme ;
`docs/SCRIPTS_POUR_IA_COURT.md`, trois fois plus léger, pour un modèle local à petite mémoire.

---

#### En deux minutes

1. Copiez le squelette ci-dessous dans un fichier `.ps1`.
2. Générez un identifiant : `New-Guid`, collez-le dans `id`.
3. **Enregistrez en UTF-8 *avec BOM*** (voir Encodage — c'est le piège n°1).
4. Déposez-le dans `%LOCALAPPDATA%\Sindry\scripts\` (sous-dossiers libres).
5. Vérifiez : `.\tools\lint-scripts.ps1`

Un script non conforme **s'exécute quand même** : Sindry signale, il ne bloque pas
(§5.4). **Mais sept situations font refuser le lancement** — approbation manquante,
ancienne ligne d'override, réglage refusé par la garde, script non simulable alors que la
simulation est activée, sélection ou fichier qui ne correspond plus à la dernière analyse,
interpréteur absent, configuration trop volumineuse. Elles sont toutes listées dans
**« Ce qui empêche un script de se lancer »**, plus bas : lisez cette section avant
d'écrire votre premier script.

---

#### Le principe : trois blocs, un rôle chacun

```
## SINDRY:START     ce qu'est le script          ─┐
## SINDRY:OPTIONS   ses réglages, typés et nommés ├─ pour les humains et l'interface
## SINDRY:LANG fr   les mêmes, traduits          ─┘

$CONFIG = @{ ... }   les valeurs par défaut        ─── du PowerShell pur
```

**Tout ce qui s'adresse à un humain vit dans les blocs `##`.** `$CONFIG` ne contient que
des clés et des valeurs : pas d'annotation, pas de libellé, pas de traduction. Les deux
langues se déclarent donc **exactement de la même façon**, en bloc — c'est cette symétrie
qui rend la dérive détectable.

Le nom d'une clé apparaît dans trois endroits. Le validateur **croise les trois dans les
deux sens** : une option sans valeur par défaut, une valeur sans option déclarée, une
traduction qui ne correspond à rien — tout est signalé, avec le numéro de ligne.

---

#### Nommer le fichier

**Cette norme ne s'applique qu'aux scripts officiels**, ceux du catalogue
([Sindry-Catalogue](https://github.com/burnout293/Sindry-Catalogue)).
Vos scripts personnels se nomment comme vous voulez : l'application ne regarde jamais le
nom de fichier, elle ne connaît que le champ `id`.

Format : `NNN_NOM_EN_ANGLAIS.ps1` — trois chiffres, tiret bas, nom en majuscules.

```
126_CLEAR_REGISTRY.ps1
200_DISABLE_SLEEP.ps1
501_CHECK_DISK_HEALTH.ps1
```

| Plage | Domaine |
|---|---|
| `1xx` | Nettoyage |
| `2xx` | Performance |
| `3xx` | Vie privée |
| `4xx` | Applications |
| `5xx` | Santé |
| `6xx` | Outillage |
| `7xx` | Personnalisation |
| `8xx` | Réservé |
| `9xx` | Diagnostics internes |

**Le numéro fixe l'ordre par défaut, rien d'autre.** Dans un lot que personne n'a réordonné
à la main, les scripts se présentent et se lancent dans l'ordre de leurs fichiers, donc de
leurs numéros ; « Entretien complet » suit l'ordre des lots, puis celui de chaque lot.
Dès que l'utilisateur réordonne un lot dans le mode Expert, son ordre l'emporte. L'identité
du script, elle, vient de son `id` : renuméroter un fichier ne change que l'ordre par
défaut — jamais la configuration, le classement ou l'historique.

Tous les scripts officiels vivent à plat dans `scripts/` du dépôt du catalogue, sans
sous-dossiers : son index signé est une liste plate.

---

#### Le squelette

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

#### La langue : le code en anglais, les commentaires en français

**Tout ce que lit une machine est en anglais** : noms de fichiers, entête de base, clés,
libellés d'options, noms de variables et de fonctions, sortie d'exécution. Les
identifiants de catégorie s'écrivent aussi en anglais (`cleaning`, pas `nettoyage`).

**Les commentaires sont en français**, et le restent : ils s'adressent à qui relit le
script, pas à une machine. La conversion de la v3 n'y touchera pas.

Dans ce que lit l'interface, le français n'a qu'une place : le bloc `SINDRY:LANG fr`, qui
traduit ce qu'elle affiche.

---

#### L'entête

Tous ces champs sont **obligatoires**, sauf `tags`, `author`, `created`, `updated`, `scan`,
`view`, `chart`, `panels` et `show`. `author`, `created` et `updated` sont conseillés : le
validateur les rappelle par une note, qui ne fait jamais échouer `-Strict`.

| Champ | Valeurs | Rôle |
|---|---|---|
| `id` | GUID (`New-Guid`) | **Identifie le script pour toujours.** Un chemin change au moindre renommage ; l'`id` survit, et avec lui la configuration, le classement et l'historique. |
| `lang` | `en`, `fr`… | Langue dans laquelle cet entête est rédigé |
| `title` | texte court | Nom affiché |
| `desc` | une phrase | Description affichée |
| `category` | voir ci-dessous | Sa catégorie dans l'Expert, et une **suggestion** de lot |
| `icon` | nom Lucide | Voir la section Icônes |
| `tags` | liste, virgules | Facultatif, alimente la recherche |
| `version` | `2.0` | Version du script lui-même |
| `author` | un nom | Qui a écrit le script (2.0.3). Affiché dans la fiche de l'Expert |
| `created` | `2026-10-17` | Date de création, `AAAA-MM-JJ`, écrite une fois pour toutes (2.0.3) |
| `updated` | `2026-10-17` | Date de la dernière modification, `AAAA-MM-JJ` : à changer avec `version`, jamais avant `created`, jamais dans le futur (2.0.3) |
| `admin` | `true` / `false` | Nécessite les droits administrateur |
| `risk` | `low` / `medium` / `high` | `high` déclenche une confirmation |
| `duration` | `fast` / `medium` / `slow` | `slow` est annoncé à l'avance |
| `reversible` | `true` / `false` | `false` **pré-coche** « nécessite un point de restauration » |
| `interruptible` | `true` / `false` | `false` = ne peut pas être tué sans risque |
| `reboot` | `true` / `false` | `true` annonce un redémarrage **à la fin** : dans les faits de la fiche, avant le lancement et au bilan, qui le propose. Il ne se règle pas dans Sindry (1.7) : c'est le script qui le sait, et `[REBOOT]` le confirme |
| `resume` | `true` / `false` | Facultatif (2.1.0). `true` = le script peut demander un redémarrage **au milieu** de l'entretien, puis reprendre — voir « Redémarrer au milieu d'un entretien » |
| `engine` | `auto` / `winps` / `pwsh` | Interpréteur requis |
| `scan` | `true` / `false` | Facultatif. `true` = le script sait analyser avant d'agir — voir « Le mode analyse » |
| `view` | une vue | Facultatif. La présentation de l'analyse — voir « Choisir sa vue » |
| `chart` | un graphique | Facultatif. Le graphique du résumé Simple quand le lot rassemble plusieurs analyses — voir « Choisir sa vue » |
| `panels` | des panneaux | Facultatif. Les panneaux de l'Expert ouverts d'office — voir « Les panneaux de l'Expert » |
| `show` | `expert` | Facultatif. L'action n'existe qu'en mode Expert : ni montrée ni lancée en Simple |

##### Catégories et lots : deux choses distinctes

Depuis la 1.4, Sindry range les scripts de deux façons, qui ne se confondent plus :

| | La **catégorie** | Le **lot** |
|---|---|---|
| Ce que c'est | Le domaine du script : nettoyage, vie privée… | Un groupe de scripts lancés ensemble |
| Qui décide | **Le script**, par `category` | **L'utilisateur**, en mode Expert |
| Où elle sert | L'onglet **Scripts** du mode Expert, pour trier | Les boutons du mode **Simple**, et l'onglet **Lots** de l'Expert |
| Combien | Une par script | Autant que l'utilisateur veut : un script peut être dans plusieurs lots, ou dans aucun |

##### Les tokens acceptés par `category`

Chaque catégorie a **un id anglais et un id français**, tous deux acceptés — écrivez celui
qui correspond à la langue de votre `lang`, sans avoir à traduire. La liste vit dans
**`tools/categories.json`** (`id` = token anglais, `id_fr` = token français) — c'est la seule
source, lue à la fois par `tools/lint-scripts.ps1` (`CATEGORIE_INCONNUE` si vous en sortez) et
par l'application (`src-tauri/src/settings.rs`) pour éviter que la liste ne diverge d'un côté
comme c'est arrivé avec `CREER_UN_SCRIPT.txt`. Au 8/10/2026 : `cleaning`/`nettoyage`,
`performance`/`performance`, `privacy`/`vieprivee`, `apps`/`applications`, `health`/`sante`,
`tools`/`outillage`, `customize`/`personnalisation`.

Une catégorie inconnue ne bloque rien : le script est rangé dans **« Autres »**, en fin de
liste.

##### Ce que « suggestion » veut dire pour les lots

Les lots d'usine vivent dans **`tools/lots.json`**, et chacun dit quelles catégories il
accueille : « Faire le ménage » accueille `cleaning`, « Vie privée » accueille `privacy`…
Sindry place un script dans le lot qui accueille sa catégorie ; s'il n'y en a aucun, le
script est **« Dans aucun lot »** (il reste dans l'onglet Scripts de l'Expert, mais aucun
bouton du mode Simple ne le lance). Tant que personne ne l'a rangé à la main, il **suit**
`category` : changer la catégorie d'un script le déplace, chez tous ceux qui ne l'ont pas
rangé eux-mêmes. **Dès que l'utilisateur le range lui-même, c'est terminé** : son
rangement devient figé. Le script propose, l'humain dispose.

`tools/lots.json` (dans le dépôt de l'application) ne vaut qu'au **premier lancement**, ou
après « Réinitialiser » : un lot d'usine ajouté ou renommé n'apparaît pas chez ceux qui ont
déjà leurs lots.

Un lot **créé par l'utilisateur** n'accueille aucune catégorie : seul un rangement manuel y
place un script. Un script ne peut donc viser par `category` qu'un lot d'usine, jamais un
lot personnel d'une installation particulière.

La même règle vaut pour `reversible` et `reboot` : ils **pré-cochent** une case que
l'utilisateur peut décocher, et son choix l'emporte ensuite définitivement.

##### Le cas de `icon`

**Aucun emoji, jamais.** Un emoji est rendu par la police du système : il change d'aspect
d'une machine à l'autre, ne peut pas hériter de la couleur du texte, et ne s'aligne pas sur
la grille. Le validateur le refuse.

`icon` prend un **nom d'icône [Lucide](https://lucide.dev)**, en minuscules avec tirets.
Les 2 112 icônes sont embarquées, donc disponibles hors ligne. Le validateur vérifie le nom
et suggère la bonne orthographe : `mon` → *vouliez-vous dire `moon` ?*

Ces icônes sont en `stroke="currentColor"` : **elles héritent de la couleur du texte**.
Une même icône devient donc sombre sur l'aplat accent, claire en thème sombre et neutre au
repos, sans aucun réglage.

| Domaine | Icônes |
|---|---|
| Nettoyage | `broom` · `sparkles` · `trash-2` · `eraser` · `recycle` |
| Performance | `zap` · `gauge` · `rocket` · `timer` · `cpu` |
| Vie privée | `shield` · `shield-check` · `eye-off` · `lock` · `user-x` |
| Applications | `app-window` · `package` · `package-minus` · `layout-grid` · `box` |
| Santé | `activity` · `heart-pulse` · `stethoscope` · `hard-drive` · `scan-line` |
| Outillage | `wrench` · `settings` · `calendar-clock` · `download` · `hammer` |
| Divers | `moon` · `sun` · `wifi` · `network` · `globe` · `folder` · `file-text` · `database` · `monitor` · `power` · `refresh-cw` · `bell-off` |

N'importe lequel des 2 112 noms fonctionne ; cette table sert à choisir vite. En revanche
**un script ne peut pas fournir sa propre image** — c'est ce qui garantit l'uniformité.

##### Le cas de `engine`

Un `.ps1` ne peut pas choisir son interpréteur — sous Windows c'est le processus appelant
qui décide, il n'y a pas d'équivalent du shebang Unix. `auto` prend le plus récent
disponible. Ne forcez `pwsh` que si vous utilisez vraiment de la syntaxe PowerShell 7
(`??`, `?:`) : si PowerShell 7 est absent, Sindry **ne se replie jamais en silence** sur
5.1 — le script est marqué indisponible.

---

#### Le bloc `OPTIONS`

Une ligne par réglage : `## Clé : [type] Libellé — Description`.

```powershell
## SINDRY:OPTIONS
## TimeoutSeconds : [number] Operation timeout — in seconds
## CreateBackup   : [bool]   Create a backup first — recommended
## SINDRY:END
```

| Type | Contrôle affiché |
|---|---|
| `bool` | interrupteur |
| `number` | champ numérique |
| `string` | champ texte libre |
| `select` | **liste déroulante, un seul choix** |
| `multi` | **cases à cocher, plusieurs choix** |
| `hidden` | **interrupteur, comme `bool`** — mais visible en mode Expert seulement |

> **`hidden` n'est pas un modificateur de visibilité.** C'est un `bool` caché : sa valeur
> par défaut doit être `$true` ou `$false`, et l'interface lui injectera **toujours** un
> booléen. Un `[hidden]` posé sur une chaîne ou un nombre passe le validateur **sans un
> seul constat**, puis casse à l'exécution — le script reçoit `false` là où il attend son
> texte. Pour cacher un réglage non booléen, il n'existe aucun moyen aujourd'hui.

Le séparateur entre libellé et description est un tiret cadratin `—`, **entouré d'un espace
de chaque côté**. Sans ces espaces, toute la chaîne devient le libellé et la description
disparaît. La description est facultative ; le libellé ne l'est pas.

##### Un espace pour une option, deux pour un choix

C'est l'indentation **après `##`**, et elle seule, qui distingue une option d'un choix :

```
## MaCle      : [bool] Une option          ← UN espace  : c'est une option
##   monchoix : Un choix                   ← DEUX espaces : c'est un choix
```

**Alignez les deux-points en garnissant APRÈS le nom de la clé, jamais avant.** Une option
écrite `##  MaCle : [bool] X` (deux espaces) n'est plus une option : elle devient un choix
de l'option précédente. La clé disparaît de l'interface, et vous récoltez
`CHOIX_INATTENDU` puis `OPTION_NON_DECLAREE` sans comprendre pourquoi.

##### Listes et multi-listes

`select` et `multi` déclarent leurs choix en **sous-lignes indentées d'au moins deux
espaces**. C'est l'indentation, et elle seule, qui distingue un choix d'une option.

```powershell
## SINDRY:OPTIONS
## DnsProvider  : [select] DNS provider — the service that resolves website addresses
##   cloudflare : Cloudflare — 1.1.1.1, fastest on most connections
##   google     : Google — 8.8.8.8, very reliable
##   quad9      : Quad9 — 9.9.9.9, blocks known malicious domains
## CleanTargets : [multi]  What to clean — pick one or more
##   temp       : Temporary files
##   cache      : Browser caches
##   logs       : Old log files
## SINDRY:END
```

Chaque choix reçoit un libellé et une description, traduits dans le bloc `LANG` de la même
façon. Et dans `$CONFIG` :

```powershell
$CONFIG = @{
    DnsProvider  = "cloudflare"          # select : UN des choix déclarés
    CleanTargets = @("temp", "cache")    # multi  : un TABLEAU de choix déclarés
}
```

Le validateur vérifie que chaque valeur par défaut fait bien partie des choix, qu'un
`select` n'est pas un tableau, qu'un `multi` en est un, et qu'une liste déclare au moins
deux choix. `@()` est accepté pour un `multi` sans sélection initiale.

Côté script, un `multi` se parcourt comme n'importe quel tableau :

```powershell
foreach ($target in $CONFIG.CleanTargets) {
    Write-Host "[OK]   Cleaned: $target"
}
```

---

#### Le bloc de traduction

L'application est bilingue (§10). Le bloc traduit l'entête, les options **et les choix**,
vers une langue **différente** de celle déclarée par `lang` :

```powershell
## SINDRY:LANG fr
## title        : Utiliser un Internet plus rapide
## desc         : Accélère la navigation avec un résolveur plus rapide
## DnsProvider  : Fournisseur DNS — le service qui traduit les adresses des sites
##   cloudflare : Cloudflare — 1.1.1.1, le plus rapide
## SINDRY:END
```

`title` et `desc` sont obligatoires. Une traduction absente retombe sur la langue de base —
rien ne casse, le validateur émet un avertissement.

**En revanche, une traduction qui ne correspond à aucune option ou à aucun choix est une
erreur.** C'est le contrôle qui manquait à la v0.3 : renommez une clé et laissez sa
traduction derrière vous, et plus rien ne le signalait.

Les scripts officiels sont rédigés en anglais et traduits vers le français. Un script
personnel écrit en français fera l'inverse : le validateur accepte les deux sens, il exige
seulement qu'une seconde langue existe.

---

#### La ligne d'override — ne la supprimez pas

```powershell
# --- Sindry override (ne pas supprimer) ---
if ($env:SINDRY_CONFIG) {
    ($env:SINDRY_CONFIG | ConvertFrom-Json).PSObject.Properties |
        ForEach-Object { $CONFIG[$_.Name] = $_.Value }
}
```

**Sans elle, les réglages choisis dans l'interface sont purement et simplement ignorés** :
le script tourne toujours avec ses valeurs par défaut, sans que rien ne le signale.

Elle remplace un mécanisme bien pire. La v0.3 reconstruisait le bloc `$CONFIG` à coups
d'expression régulière, écrivait un `.ps1` temporaire et exécutait *celui-là* — fragile,
et déjà cassé au moment de la refonte. Désormais Sindry pose vos réglages dans la
variable `SINDRY_CONFIG` et exécute **votre fichier, tel quel**.

> **La variable contient le JSON, pas un chemin de fichier.** Une version antérieure de ce
> guide écrivait `Get-Content $env:SINDRY_CONFIG -Raw | ConvertFrom-Json` : cette forme
> ne fonctionne plus, et le validateur la refuse (`OVERRIDE_OBSOLETE`). Le fichier
> intermédiaire a été supprimé parce qu'il vivait dans un dossier inscriptible sans
> élévation : un autre programme pouvait le remplacer entre l'écriture par Sindry et la
> lecture par votre script, et faire ainsi entrer ses propres valeurs dans `$CONFIG` —
> en administrateur.

Bénéfice secondaire : sans cette variable, votre script reste parfaitement exécutable seul,
en double-clic, avec ses valeurs par défaut.

---

#### Un script écrit pour WinTool (avant la 2.0)

Jusqu'à la 1.8, l'application s'appelait WinTool, et le contrat aussi : `## WINTOOL:START`,
`## WINTOOL:OPTIONS`, `$env:WINTOOL_CONFIG`, `$env:WINTOOL_MODE`. **Sindry 2.0 lit encore
ces scripts, jusqu'à la v3** — le temps que les catalogues migrent :

- le script est ramené au contrat de Sindry ligne à ligne, avant toute lecture ; les
  numéros de ligne ne bougent pas, les autres constats tombent au bon endroit ;
- au lancement, Sindry pose les deux variables, `SINDRY_CONFIG` et `WINTOOL_CONFIG`
  (et `_MODE`) : l'ancienne ligne d'override reçoit toujours vos réglages ;
- l'application le signale en Expert (`CONTRAT_ANCIEN`), le validateur aussi, en
  **note** : une note ne fait jamais échouer `-Strict`, un catalogue en CI reste vert.

La conversion tient en un remplacement : `WINTOOL` par `SINDRY`, dans les marqueurs et les
deux variables. Rien d'autre ne change. **Un script modifié doit être approuvé à nouveau**
(son empreinte change) : pour un catalogue signé, c'est une nouvelle version de l'index.

En v3, l'ancien contrat ne sera plus lu : un script resté à `WINTOOL:START` n'aura plus
d'entête (`ENTETE_ABSENT`), et ses réglages ne lui parviendront plus.

---

#### Les marqueurs de sortie

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
[REBOOT]   message          un redémarrage est réellement nécessaire, à la fin de l'entretien
[RESTART]  phase            redémarrer maintenant, puis reprendre à cette phase (2.1.0), suivi de exit 3010
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

##### Les codes d'erreur : `[ERR] E104` (2.0.3)

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

##### `[CKPT]`, le marqueur qui rend l'annulation sûre

Au premier clic sur « Arrêter », Sindry laisse le script en cours aller au bout. Au
second, il ne le tue que si le script est `interruptible`, **ou** si un `[CKPT]` a été
franchi. Émettez-en un dès que l'état du système redevient cohérent :

```powershell
Write-Host "[CKPT] Backup complete - safe to interrupt from here"
```

#### Redémarrer au milieu d'un entretien (2.1.0)

Certaines actions ne se terminent qu'après un redémarrage : un pilote qu'on désinscrit,
dont le fichier reste chargé jusqu'au prochain démarrage ; une installation qui en réclame
un avant la suivante. Le script ne redémarre **jamais** le PC lui-même — Windows tuerait
Sindry au milieu de l'entretien. Il le **demande**, et Sindry s'occupe du reste :

1. le script émet `[RESTART] <phase>`, puis se termine par **`exit 3010`** ;
2. Sindry garde de quoi reprendre (dans HKLM), prépare sa relance, et propose le
   redémarrage avec un compte à rebours de 60 secondes — l'entretien va au bout même si
   personne n'est devant le PC ;
3. après le redémarrage, Windows rouvre la session (s'il le permet, sinon à la connexion
   de l'utilisateur) ; Sindry se relance seul, en administrateur, dans une petite fenêtre
   « Action en cours » ;
4. il relance **le même script**, avec la même configuration et **`SINDRY_RESUME=<phase>`**,
   puis enchaîne les actions restantes. À la fin, le bilan habituel.

| Règle | Détail |
|---|---|
| `## resume : true` | Obligatoire pour qu'un `[RESTART]` soit suivi. Sans lui, Sindry ne redémarre pas et l'action finit en échec |
| La phase | Minuscules, chiffres et tirets, 32 caractères au plus : `[RESTART] delete-file` |
| `exit 3010` | Juste après `[RESTART]` : ni succès, ni échec — la suite vient après le redémarrage. Sans `[RESTART]`, 3010 est un échec |
| `$env:SINDRY_RESUME` | Absent au premier passage ; vaut la phase au retour. Testez-le **juste après l'override** (et la branche d'analyse), avant toute modification |
| Une phase se rejoue | Si le PC redémarre pendant une phase, elle recommence : écrivez-la pour qu'un second passage ne fasse pas de dégâts |
| Ni analyse, ni simulation | `[RESTART]` est ignoré en analyse (`SINDRY_MODE=scan`) et en simulation : en `SafeTest`, dites ce que vous feriez |
| Un plafond | Trois redémarrages par entretien par défaut (réglage Expert 4.5) ; au-delà, l'entretien s'arrête |
| Pas d'option secrète | La configuration est gardée pour la reprise : aucune option `Password`, `Token`… dans un script `resume` |
| Installeurs tiers | Passez-leur `/norestart` (MSI : `REBOOT=ReallySuppress`), puis traduisez leur 3010 ou 1641 en `[REBOOT]` (à la fin) ou en `[RESTART]` |

`Restart-Computer`, `Stop-Computer`, `shutdown /r`, `/s`, `/g` ou `/p` sont des **erreurs**
du validateur (`REDEMARRAGE_DIRECT`), et Sindry les signale avant d'approuver un script.

`[REBOOT]` garde son sens : un redémarrage nécessaire **à la fin**, proposé au bilan,
jamais imposé. `[RESTART]`, c'est maintenant, au milieu, avec reprise.

**Exemple complet.** Un pilote se désinscrit, le PC redémarre, puis son fichier — enfin
libéré — se supprime :

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

En ligne de commande (`sindry-cli`), il n'y a ni redémarrage ni reprise : l'entretien
s'arrête au `[RESTART]` et sort en 3010, avec la liste des actions restantes.

---

#### Le mode analyse

> **Exploité par Sindry depuis la 1.4.** Un script qui ne déclare rien de ce qui suit
> fonctionne exactement comme avant : le mode analyse est une capacité en plus, jamais
> une obligation.

C'est le fonctionnement de CCleaner ou de Malwarebytes, en trois temps :

1. **Analyser** — Sindry demande au script ce qu'il ferait, sans rien modifier.
2. **Cocher** — l'utilisateur voit ce que le script a trouvé et choisit ce qu'il veut traiter.
3. **Agir** — Sindry relance le script en ne lui transmettant que les cases cochées.

En mode Simple, c'est l'étape 2 de l'assistant, « Voici ce que j'ai trouvé ». En mode
Expert, c'est le bouton **Analyser** de la fiche d'un script (§17 de la spécification).

**Le script ne compose pas l'écran : il le décrit.** Il n'écrit que des nombres, des noms
et des jetons. Les mots viennent de son entête, déjà traduite, et Sindry les met en page
dans la vue que le script a choisie. C'est ce qui permet au même script de s'afficher en
résumé pour un débutant et en détail pour l'Expert, en français comme en anglais.

##### Déclarer : l'entête

| Champ | Valeurs | Rôle |
|---|---|---|
| `scan` | `true` / `false` | `true` = le script sait analyser. Sans lui, rien de ce qui suit n'est lu. |
| `view` | une vue, puis `expert=<vue>` si l'Expert doit en voir une autre | La présentation de l'analyse — voir « Choisir sa vue ». Absente : la liste à cocher. |
| `chart` | `donut`, `crown`, `gauge`, `bar`, `rows`, `treemap`, `waffle` | Le graphique du résumé Simple, quand le lot rassemble plusieurs analyses. Absent : l'anneau-engrenage (`donut`). |
| `panels` | des panneaux, séparés par des espaces | Les panneaux de l'Expert ouverts d'office — voir « Les panneaux de l'Expert » |
| `show` | `expert` | L'action n'existe qu'en Expert — voir « Ce qui ne s'affiche que dans un mode » |

```
## scan          : true
## view          : gauge expert=table
## panels        : plan progress
```

##### Le contrat, en quatre règles

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

##### Les lignes de l'analyse

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

###### Les champs de `[FIND]`

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

###### Les champs de `[ITEM]`

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

**`exe=` : montrer la vraie icône d'un programme (2.0.3).** Le script désigne un fichier du
système, il n'envoie jamais d'image ; Sindry lit l'icône **sur place**, rien n'est
téléchargé — demander une icône à un service en ligne lui livrerait la liste des
programmes de l'utilisateur. Sindry tourne en administrateur et le fichier peut avoir été
piégé, d'où ces règles :

- **un chemin absolu sur un lecteur local** : `C:\Program Files\App\app.exe`. Un chemin
  réseau (`\\serveur\partage\…`) est refusé — Sindry ouvrirait une connexion vers ce
  serveur, et y laisserait l'empreinte du mot de passe du compte —, comme `\\?\`, `\\.\`,
  `..`, un flux (`app.exe:x`) ou un nom de périphérique (`CON.exe`) ;
- **un `.exe`, un `.dll` ou un `.ico`**, jamais un raccourci `.lnk` : sa cible est écrite par
  l'utilisateur. Sindry ne charge pas le fichier, il en lit les ressources octet par
  octet ; aucun code du fichier ne s'exécute ;
- **rien que pour ce qui s'exécute** : `kind=app`, `startup`, `service` ou `task` ;
- un fichier absent, illisible ou sans icône garde l'icône du `kind`, sans erreur. Les
  fichiers de Windows dont l'icône vit à part (`SystemResources\*.mun`) sont trouvés ;
- n'écrivez pas `exe=` pour un service hébergé par `svchost.exe`, ni pour une tâche qui
  lance `rundll32.exe` ou `cmd.exe` : leur icône n'apprend rien. Désignez le vrai programme
  s'il est connu, sinon rien.

**Quelle icône s'affiche.** De la plus précise à la plus générale : l'icône lue dans le
fichier d'`exe=` (une application installée), puis l'icône téléchargée d'`appicon=`, puis
l'icône Lucide d'`icon=`, puis celle du `kind`. Écrivez-en plusieurs : chacune sert de repli
à la précédente.

**`appicon=` : l'icône officielle d'une application qui n'est pas encore là (2.0.4).** Le
script ne donne qu'un **nom** ; Sindry télécharge `icons/<nom>.png` depuis **un seul endroit**,
le dépôt GitHub `burnout293/Sindry-Media-Library`. Un script ne peut donc ni faire contacter à Sindry
un serveur de son choix, ni envoyer une image :

- en HTTPS seulement, sans suivre de redirection ni envoyer d'identifiant ;
- **toutes** les icônes que l'analyse nomme sont demandées, quel que soit l'état de leur
  élément : le serveur n'apprend pas ce qui est installé ;
- un PNG ou un ICO, reconnu à ses premiers octets, 128 Ko au plus — jamais un SVG ;
- gardée 30 jours sur le disque, et revérifiée à chaque lecture ;
- l'écran s'affiche d'abord avec l'icône de repli ; l'icône officielle prend sa place en
  arrivant ;
- l'utilisateur peut couper ce téléchargement (Réglages, 3.3) : rien ne quitte alors le PC.

Une icône absente du dépôt n'est pas une erreur : l'élément garde son repli. Pour en ajouter
une, c'est au dépôt `Sindry-Media-Library` qu'il faut la proposer, pas au script.

###### Les champs de `[METRIC]`

| Champ | Valeur | Sens |
|---|---|---|
| `value=` | un nombre, ou un jeton court | **Obligatoire.** La mesure. |
| `unit=` | `celsius` `pct` `hours` `days` `s` `count` `cycles` | Son unité, écrite dans la langue de l'utilisateur |
| `health=` | `ok` / `warn` / `crit` | Bon, À surveiller, Critique |
| `max=` | nombre | Le maximum possible : de quoi dessiner une jauge |
| `group=` | une clé de groupe du bloc `REPORT` | Regroupe les mesures dans la vue `light` |
| `show=` | `simple` / `expert` | Ne s'affiche que dans ce mode |

##### Le bloc `REPORT` : les mots de l'analyse

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

##### `[items]` : une liste que seule l'analyse connaît

Un `[multi]` déclare ses choix dans l'entête. Mais les caches de navigateurs, les restes de
programmes désinstallés, les doublons n'existent qu'une fois la machine examinée. `[items]`
est une liste **vide dans l'entête**, que l'analyse remplit de lignes `[ITEM]` :

```
## Browsers   : [items] Browser caches
```

Sa valeur par défaut dans `$CONFIG` est toujours un tableau, en général `@()`. Sans
Sindry, la liste reste vide et le script ne touche à rien : c'est le comportement sûr
du double-clic.

**L'`id` est une clé, pas un chemin.** Calculez-le à partir de ce qui ne bouge pas entre
l'analyse et l'action — le navigateur et le profil, l'identifiant d'un programme. À
l'action, **retrouvez les éléments en rappelant la même fonction** que pendant l'analyse,
et ne vous servez de l'id reçu que pour choisir parmi ce que vous avez vous-même retrouvé.
Un id ne sert jamais à fabriquer un chemin.

Sindry y veille de son côté : il retient les ids de la dernière analyse de chaque script,
avec l'empreinte du fichier analysé, et **refuse de lancer** l'action si `$CONFIG` contient
un id que cette analyse n'a pas annoncé — voir « Ce qui empêche un script de se lancer ».

**Une arborescence.** `parent=` range un élément sous un autre de la même liste. Seules les
**feuilles** reviennent au script : cocher un navigateur coche ses profils, et ce sont les
ids des profils qui reviennent. Un parent jamais annoncé n'efface pas l'élément : il
s'affiche à la racine, et l'Expert voit l'anomalie.

**`[items:keep-one]`, pour les doublons.** Les éléments qui partagent un `group=` sont des
exemplaires d'un même fichier ; l'utilisateur choisit celui qu'il garde — `keep=true`
désigne celui que conseille le script, sinon c'est le premier. **Ce qui revient au script,
ce sont les ids à supprimer.**

Limites : 5 000 éléments par liste, 20 000 lignes par analyse, 400 caractères par texte.
Au-delà, la suite est ignorée, et l'Expert le voit.

**L'écran de liste (2.0.4).** Une action dont les listes `[items]` comptent plus de
**12 éléments** en tout, ou dont un élément est **expliqué** (`about=`, `why=`, `warn=`,
`advice=`), présente toutes ses listes dans un seul écran, sans rien à déclarer de plus :

| Ce que voit l'utilisateur | Détail |
|---|---|
| Deux onglets : **Choisir** et **Récapitulatif** | Le récapitulatif liste ce qui est coché, ses avertissements (`warn=`) et ses redémarrages (`reboot=true`) en tête |
| Une **recherche** | Sur le nom, l'éditeur, le chemin et la catégorie ; elle surligne ce qu'elle trouve |
| Les **catégories**, une à la fois | Les parents des éléments s'ils en ont (Navigateurs, Bureautique…), sinon les listes elles-mêmes (Programmes, Démarrage…), dans l'ordre du script |
| **Filtres et tri** | Trier par catégorie, nom, recommandation ou éditeur ; ne montrer qu'une recommandation (ou une liste) ; afficher ou masquer les étiquettes — recommandation, version, redémarrage, éditeur |
| La **fiche** de l'élément choisi, à droite | Seulement si le script explique ses éléments : De quoi s'agit-il, Pourquoi c'est proposé, Attention, Redémarrage |

La **recommandation** se lit dans ce que le script dit déjà : `advice=discouraged`
Déconseillé, `advice=caution` À double tranchant, `risk=high` Suspect, puis `confidence=` —
`low` À vérifier, `medium` Probable, `high` Sûr.

**Expliquer un élément.** `about=`, `why=` et `warn=` nomment une **note** du bloc `REPORT`,
jamais une phrase de la sortie, qui reste en anglais ; chaque note se traduit dans `LANG`.
Une clé inconnue est ignorée (`ITEM_TEXTE`).

```
## SINDRY:REPORT
## AvWhat : [note:info] Your antivirus.
## AvWhy  : [note:info] Two antiviruses get in each other's way: Microsoft Defender is there too.
## AvWarn : [note:warn] Removing it leaves you unprotected until Defender takes over.
## SINDRY:END

[ITEM] Programs id=avast kind=app name="Avast Free Antivirus" about=AvWhat why=AvWhy warn=AvWarn advice=discouraged reboot=true
```

- **`advice=`** marque ce qui coupe dans les deux sens — un programme utile à certains, un
  service dont dépend une fonction — ou ce qu'il vaut mieux garder, comme un antivirus.
  Rien de tout cela n'est coché d'office ; un élément `discouraged` ne se coche que depuis
  sa fiche, après une confirmation.
- **`reboot=true`** annonce le redémarrage d'avance. C'est toujours au script de le
  confirmer à la fin par `[REBOOT]`, s'il a réellement retiré l'élément.

Pour un lot d'une seule action, le mode Simple montre le résumé de Sindry, en postes. Pour
qu'il montre l'écran de liste, déclarez une vue : `## view : tree`.

**« Ce qui sera fait »** liste tout ce qui est coché, regroupé par option — une liste
`[items]`, un `[multi]` — avec l'icône de chaque élément et sa version proposée (`to=`). En
Expert, c'est un panneau ; en Simple, il s'affiche aussi, sous l'analyse, replié s'il dépasse
quinze lignes — sauf quand l'écran de liste, qui a son récapitulatif, est affiché.

##### Les étiquettes : `[group:]`, `[show:]`, `[view:]`, `[scan]`

Elles se placent entre le type et le libellé d'une option ; `[group:]` et `[show:]` se
posent aussi sur un choix, avant son libellé :

```
## Targets    : [multi] Temporary files
##   user     : [group:Junk] Your temporary files
##   windows  : [group:Junk] [show:expert] Windows temporary files
## RecycleBin : [bool] [group:Junk] Empty the recycle bin
## Browsers   : [items] [view:tree] Browser caches
## MinAgeDays : [number] [scan] Only files older than — in days
```

| Étiquette | Sur | Effet |
|---|---|---|
| `[group:Clé]` | option, choix | Range la case dans un **poste** du résumé Simple : le groupe `Clé`, déclaré dans `REPORT`. Sans groupe, une option fait un poste à elle seule. |
| `[show:simple]`, `[show:expert]` | option, choix | La case ne s'affiche que dans ce mode |
| `[view:x]` | option | La présentation de cette seule option : `bars` ou `donut` pour un `[multi]`, `chart` pour un `[select]`, `tree` `table` `tiles` `treemap` pour un `[items]` |
| `[scan]` | option | Un réglage **de l'analyse elle-même** — une ancienneté minimale, un dossier à examiner. Ce n'est pas une case : Sindry le transmet tel qu'il est configuré, à l'analyse comme à l'action. |

Les étiquettes ne se traduisent pas : le bloc `LANG` ne reprend que le libellé.

##### Ce que Sindry coche d'office

Dans cet ordre :

1. **`checked=`** — l'avis du script l'emporte, sauf sur un élément `locked=` (jamais
   cochable) ou `advice=` (jamais coché d'office, 2.0.4) et sur une option `[bool]` en
   `state=ok` (toujours décochée et grisée).
2. Sinon, **`state=todo`** coche, et **`state=ok`** décoche et grise la case (« Déjà en place »).
3. Sinon, pour un constat **`[FIND]`** : une **`size` ou un `count` supérieur à zéro** coche ;
   zéro, ou aucune mesure, décoche.
4. Sinon, pour un élément **`[ITEM]`** : il est **coché**, sauf `locked=` (jamais cochable),
   **`confidence=low`** ou **`advice=`** (jamais cochés d'office). Sa `size` n'entre pas en compte : un
   élément de taille 0 est coché — écrivez `checked=false` si ce n'est pas votre avis.
5. Un `[select]` présélectionne le choix `recommended=true`, sinon `current=true`, sinon le
   premier rapporté.

Une nouvelle analyse remet toutes les cases à l'avis du script.

**Montrer sans cocher.** `checked=false` sert ce qui est gros mais peut encore servir : les
mises à jour de Windows téléchargées, par exemple. Accompagnez-le d'une `[NOTE]` qui dit
pourquoi la case n'est pas cochée.

##### Ce qui ne s'affiche que dans un mode

Un seul affichage, réglé par le script — pas deux rendus à écrire :

| Où | Forme | Effet |
|---|---|---|
| Entête | `## show : expert` | L'action entière n'existe qu'en Expert. Dans un lot lancé en Simple, elle n'est **ni montrée ni lancée**, et l'écran d'analyse le dit. |
| Option, choix | `[show:expert]`, `[show:simple]` | La case n'apparaît que dans ce mode |
| Ligne | `show=expert`, `show=simple` | Ce constat, cet élément, cette mesure ou cette note n'apparaît que dans ce mode |

**Un élément caché garde sa case, et suit celle qui le contient — en revenant à l'avis du
script.** Le cache de 300 Ko d'un profil, marqué `show=expert`, n'encombre pas le résumé
d'un débutant ; si celui-ci coche « Google Chrome », ce petit cache est traité avec les
autres s'il était coché d'office, et laissé s'il ne l'était pas. Le résumé annonce ce qu'il
cache (« Un élément de plus ne s'affiche qu'en mode Expert »), jamais en silence.

##### Choisir sa vue

`## view : <vue>` choisit la présentation de l'analyse, pour les deux modes ; `expert=<vue>`
en donne une autre à l'Expert : `## view : gauge expert=table`.

| Vue | Ce qu'il lui faut | Pour |
|---|---|---|
| `checklist` | rien : c'est la vue par défaut | Tout : chaque option devient une liste à cocher |
| `minimal` | une case | Une phrase et un interrupteur : « Libérer 1,2 Go — fichiers inutiles » |
| `bars` | un `[multi]` dont les choix ont une `size` | Les choix en barres proportionnelles |
| `donut` | un `[multi]` dont les choix ont une `size` | L'anneau, ses choix en légende |
| `tiles` | un `[items]` | Une tuile par élément |
| `table` | un `[items]` | Un tableau triable, avec recherche |
| `treemap` | un `[items]` avec des `size` | Des rectangles proportionnels, à explorer |
| `tree` | un `[items]` avec des `parent=` | L'arborescence |
| `timeline` | un `[items]` avec `impact=`, et une `[METRIC]` en `unit=s` | Le démarrage : ce qui se lance, et le temps gagné |
| `compare` | des `[bool]` avec `state=` | Aujourd'hui → après, réglage par réglage |
| `chart` | un `[select]` dont les choix ont un `ms=` | Des colonnes à comparer : plus bas, plus rapide |
| `light` | des `[METRIC]` avec `health=` | Un feu : tout va bien, un point à surveiller, un problème |
| `gauge` | une `[METRIC]` avec `max=` | Une jauge à dents, à la couleur de son état |
| `history` | rien : Sindry garde le `[FREED]` de chaque action | Ce que l'action a libéré, passage après passage |

**Une vue ne fait jamais disparaître une case.** Ce qu'elle ne montre pas suit dessous, en
liste. Et une vue qui ne trouve pas de quoi s'afficher — un `donut` sans aucune taille —
retombe sur la liste à cocher : jamais un écran vide.

En mode Simple, la vue du script ne s'applique que si le lot ne contient **qu'une** action
analysable. Dès qu'il y en a plusieurs, Sindry les rassemble dans son propre résumé : un
poste par groupe, un graphique, et des chevrons pour déplier le détail.

**Le graphique du résumé se déclare** (`## chart`, 1.7 — c'était un réglage de l'utilisateur
jusqu'à la 1.6) : c'est le script qui sait ce qui montre le mieux ce qu'il trouve. La première
action du lot, dans l'ordre du lot, qui en déclare un décide ; aucune : l'anneau-engrenage.

| `chart` | Le graphique | Pour |
|---|---|---|
| `donut` | L'anneau-engrenage : un arc par poste | Des tailles qui s'additionnent (défaut) |
| `crown` | La double couronne : les postes au centre, ce qui les compose autour | Des postes faits de plusieurs éléments |
| `gauge` | La jauge à dents seule, et « Voir le détail » | Un seul total qui compte |
| `bar` | La barre d'espace du disque | Ce qui sera libéré, rapporté au disque |
| `rows` | Le fond de chaque ligne, rempli à sa part | Des postes nombreux, à comparer |
| `treemap` | Les rectangles | Quelques gros postes et beaucoup de petits |
| `waffle` | La gaufre : cent cases | Des proportions, plutôt que des tailles |

##### Les panneaux de l'Expert

`## panels : plan progress` ouvre d'office, à côté de l'analyse, des panneaux que
l'utilisateur peut aussi ouvrir et fermer lui-même :

| Panneau | Contenu |
|---|---|
| `config` | Les réglages appliqués |
| `progress` | Le journal de l'analyse, filtrable par étapes, constats et canaux `[LOG]` |
| `plan` | Ce qui sera fait, d'après les cases cochées |
| `payload` | Ce que recevra le script : le JSON exact de `SINDRY_CONFIG` |
| `attention` | Les points d'attention que Sindry repère dans le texte du script |
| `history` | Ce que l'action a libéré, passage après passage |
| `origin` | La provenance : catalogue, fichier, empreinte, accord |
| `disk` | L'espace disque, aujourd'hui et après l'action |

##### Ce qui revient au script

Après l'analyse, Sindry lance le script **sans** `SINDRY_MODE`, avec dans `$CONFIG` :

| Option | Ce que reçoit le script |
|---|---|
| `[bool]` visée par un `[FIND]` | `$true` si sa case est cochée, `$false` sinon |
| `[multi]` | La liste des choix cochés **parmi ceux que l'analyse a rapportés** : un choix qu'elle n'a pas montré n'est jamais traité |
| `[select]` visée par des `[FIND]` | Le choix retenu |
| `[items]` | La liste des ids cochés — les feuilles seulement |
| `[items:keep-one]` | La liste des ids **à supprimer** |
| `[scan]`, et toute option sans constat | Sa valeur configurée, inchangée |

**Rien de coché, rien de lancé.** Une action dont aucune case n'est cochée n'est pas
relancée. Une action qui n'a rapporté que des mesures non plus : elle a fait son travail en
analysant. La lancer pour rien l'inscrirait « fait » dans l'historique.

##### `[FREED]` : ce qui a réellement été libéré

L'analyse et l'action sont deux exécutions distinctes. Entre les deux, des fichiers
temporaires apparaissent, d'autres sont verrouillés et ne pourront pas être supprimés. Le
chiffre de l'analyse est donc une **estimation**.

Un script qui libère de l'espace peut le dire, en fin d'action :

```powershell
Write-Output "[FREED] $freed"      # en octets
```

Le bilan affiche alors le chiffre réel, l'historique le garde, et la vue `history` le
montre. Sans `[FREED]`, le bilan affiche l'estimation de l'analyse, précédée de
« environ ».

##### Exemple complet — faire de la place

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

##### Exemple court — un réglage

Pour l'optimisation ou la vie privée, on ne mesure pas une taille : on constate un **état**.

```powershell
if ($env:SINDRY_MODE -eq 'scan') {
    $key   = 'HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection'
    $value = (Get-ItemProperty -Path $key -Name AllowTelemetry -ErrorAction SilentlyContinue).AllowTelemetry
    $state = if ($value -eq 0) { 'ok' } else { 'todo' }
    Write-Output "[FIND] DisableTelemetry state=$state"
    exit 0
}
```

`DisableTelemetry` est ici une option `[bool]`. Si la télémétrie est déjà coupée, la case
arrive grisée avec la mention « Déjà en place » ; sinon elle arrive cochée. Avec
`## view : compare`, plusieurs réglages de ce genre s'affichent en « aujourd'hui → après ».

##### Ce qu'il faut respecter

- **Une option ciblée signifie « fais-le » quand elle vaut `$true`.** La case cochée
  transmet `$true`. Une option comme `KeepCookies`, dont `$true` veut dire « ne touche à
  rien », ne peut pas porter un constat : la case cochée dirait l'inverse de ce qu'elle
  montre. Nommez vos options par l'action.
- **L'analyse se place après la ligne d'override et avant tout code qui modifie.** Après,
  pour disposer de `$CONFIG` — et de vos options `[scan]`. Avant, parce qu'une
  modification qui s'exécuterait avant le test de `SINDRY_MODE` aurait lieu pendant
  l'analyse.
- **L'analyse rapporte tout ce qu'elle pourrait faire**, pas seulement ce que la
  configuration actuelle sélectionne. C'est l'utilisateur qui choisit, après.
- **À l'action, ne faites confiance qu'à ce que vous retrouvez.** Une valeur reçue ne sert
  que de clé dans les tables du script, et seulement si elle y figure : `$Folders['x']`
  vaut `$null` pour une clé inconnue, et `"$null\*"` viserait la racine du disque.
- **Analyser, c'est exécuter.** Sindry lance le script en administrateur pour l'analyser,
  exactement comme pour agir. Un script non approuvé ne sera pas plus analysé qu'exécuté
  (§12.1 de la spécification).
- **L'analyse doit être rapide.** Elle précède chaque action et l'utilisateur l'attend
  devant l'écran. Mesurez des tailles, ne calculez pas d'empreintes — sauf si c'est le
  métier du script, comme pour les doublons.
- **Rien à faire, rien à annoncer.** Quand toutes les cases d'un lot arrivent en
  `state=ok`, Sindry dit « Tout est déjà en place » et ne lance rien. Une `[NOTE]`
  s'affiche telle quelle : n'émettez pas « un redémarrage est nécessaire » si l'analyse
  n'a rien trouvé à faire. Le redémarrage s'annonce d'ailleurs par `reboot : true` et
  `[REBOOT]`, pas par une note.

---

#### Encodage : UTF-8 **avec BOM**

C'est le piège n°1, et il est silencieux.

PowerShell 5.1 — celui livré nativement avec Windows, donc celui qui exécutera la plupart
des scripts — lit un `.ps1` **sans BOM** comme de l'ANSI, pas de l'UTF-8. Le moindre accent
est alors mal décodé : au mieux l'affichage est du charabia, au pire le script ne s'analyse
plus du tout.

Ce validateur lui-même a cassé à sa première exécution pour cette raison exacte.

```powershell
# Convertir un fichier existant
$c = (Resolve-Path .\mon_script.ps1).Path
$t = [System.IO.File]::ReadAllText($c, [System.Text.UTF8Encoding]::new($false))
[System.IO.File]::WriteAllText($c, $t, [System.Text.UTF8Encoding]::new($true))
```

Dans VS Code : *Sélectionner l'encodage → Enregistrer avec l'encodage → UTF-8 with BOM*.

---

#### Valider

```powershell
.\tools\lint-scripts.ps1                                               # vos scripts
.\tools\lint-scripts.ps1 -Path ..\Sindry-Catalogue\scripts -Strict   # catalogue officiel
```

`-Strict` traite les avertissements comme des erreurs. Les scripts du catalogue officiel
doivent passer en `-Strict`. Les **notes** n'échouent jamais, même en `-Strict` : elles
annoncent un changement à venir, pas une faute.

##### Les contrôles, un par un

Cette table est le contrat entre la documentation et le code. **Ajouter un contrôle au
validateur sans l'ajouter ici est un défaut.**

| Code | Gravité | Déclencheur |
|---|---|---|
| `BOM_ABSENT` | erreur | Fichier non-ASCII sans BOM UTF-8 |
| `CONTRAT_ANCIEN` | note | Contrat d'avant la 2.0 (`WINTOOL:START`, `$env:WINTOOL_CONFIG`) : lu jusqu'à la v3 |
| `ENTETE_ABSENT` | erreur | Pas de bloc `SINDRY:START` |
| `ENTETE_NON_FERME` | erreur | `SINDRY:START` sans `SINDRY:END` |
| `CHAMP_MANQUANT` | erreur | Champ d'entête obligatoire absent |
| `CHAMP_DOUBLON` | avertissement | Champ déclaré deux fois |
| `VALEUR_INVALIDE` | erreur | `risk`, `duration`, `engine`, `show` ou un booléen hors valeurs admises ; `[show:]` hors `simple`/`expert` |
| `ID_INVALIDE` | erreur | `id` n'est pas un GUID |
| `ID_COLLISION` | erreur | Deux scripts portent le même `id` |
| `CHAMP_CONSEILLE` | note | `author`, `created` ou `updated` absent : conseillés, jamais exigés (2.0.3) |
| `DATE_INVALIDE` | erreur | `created` ou `updated` hors de la forme `AAAA-MM-JJ`, ou date qui n'existe pas |
| `DATES_INVERSEES` | erreur | `updated` précède `created` |
| `DATE_FUTURE` | avertissement | `created` ou `updated` à plus d'un jour dans le futur |
| `CATEGORIE_INCONNUE` | avertissement | Catégorie hors `tools/categories.json` → « Autres », et aucun lot |
| `ICONE_EMOJI` | erreur | `icon` contient un emoji |
| `ICONE_INCONNUE` | erreur | Nom d'icône absent de Lucide — suggestion si proche |
| `OPTIONS_ABSENT` | erreur | Pas de bloc `SINDRY:OPTIONS` |
| `OPTIONS_NON_FERME` | erreur | Bloc `OPTIONS` non refermé |
| `TYPE_ABSENT` | erreur | Option déclarée sans `[type]` |
| `TYPE_INVALIDE` | erreur | Type inconnu, ou variante autre que `[items:keep-one]` |
| `LIBELLE_VIDE` | erreur | Option, choix ou entrée de `REPORT` sans texte |
| `CHOIX_MANQUANT` | erreur | `select` ou `multi` avec moins de deux choix |
| `CHOIX_INATTENDU` | erreur | Choix déclarés sur un type qui n'en accepte pas |
| `CONFIG_ABSENT` | erreur | Pas de bloc `$CONFIG = @{ }` |
| `CONFIG_NON_FERME` | erreur | Bloc `$CONFIG` non refermé |
| `CONFIG_LIGNE_ILLISIBLE` | avertissement | Ligne non reconnue comme « Clé = valeur » |
| `OPTION_NON_DECLAREE` | erreur | Clé dans `$CONFIG` absente du bloc `OPTIONS` |
| `OPTION_ORPHELINE` | erreur | Option déclarée sans valeur par défaut dans `$CONFIG` |
| `TYPE_INCOHERENT` | avertissement | `[bool]` sur une valeur non booléenne, `[number]` sur du texte |
| `DEFAUT_INVALIDE` | erreur | Défaut hors des choix, `select` en tableau, `multi` ou `items` qui n'en est pas un |
| `TRADUCTION_ABSENTE` | erreur | Aucun bloc `SINDRY:LANG` vers une langue autre que `lang` |
| `TRADUCTION_NON_FERMEE` | erreur | Bloc de traduction non refermé |
| `TRADUCTION_INCOMPLETE` | erreur | `title` ou `desc` non traduit |
| `TRADUCTION_OPTION` | avertissement | Une option sans libellé traduit |
| `TRADUCTION_CHOIX` | avertissement | Un choix sans libellé traduit |
| `TRADUCTION_ORPHELINE` | erreur | Traduction d'une option, d'un choix ou d'une entrée de `REPORT` qui n'existe pas |
| `TRADUCTION_RAPPORT` | avertissement | Une entrée de `REPORT` sans texte traduit |
| `OVERRIDE_ABSENT` | erreur | Ligne d'override manquante |
| `OVERRIDE_OBSOLETE` | erreur | Ancienne ligne d'override lisant un fichier — `SINDRY_CONFIG` contient le JSON |
| `OVERRIDE_FICHIER` | erreur | Repli vers un fichier de configuration — **Sindry refuse de lancer le script** |
| `MARQUEUR_INCONNU` | erreur | Balise proche d'un marqueur connu — `[REBBOT]` → `[REBOOT]` |
| `MARQUEUR_CASSE` | avertissement | Marqueur pas en majuscules |
| `SORTIE_NON_ANGLAISE` | avertissement | Message affiché contenant des accents |
| `SCAN_NON_GERE` | erreur | `scan : true` mais `$env:SINDRY_MODE` n'est jamais lu : le script agirait au lieu d'analyser |
| `SCAN_SANS_FIND` | erreur | `scan : true` mais aucun `[FIND]`, `[ITEM]` ni `[METRIC]` n'est émis |
| `FIND_SANS_SCAN` | avertissement | Le script sait analyser mais ne déclare pas `scan : true` : il ne sera jamais interrogé |
| `FIND_CLE_INCONNUE` | erreur | `[FIND]` vise une option absente, d'un type autre que `[bool]`/`[multi]`/`[select]`, ou un choix non déclaré |
| `FIND_MESURE` | erreur | `[FIND]` sans mesure (`size`, `count`, `state` ou `ms`), avec un champ inconnu ou une valeur hors liste |
| `ITEM_CLE_INCONNUE` | erreur | `[ITEM]` vise une option absente, ou qui n'est pas un `[items]` |
| `ITEM_SANS_ID` | erreur | `[ITEM]` sans `id=` |
| `ITEM_CHAMP` | erreur | `[ITEM]` avec un champ inconnu, une valeur hors liste, ou `exe=` sur un `kind` qui ne s'exécute pas |
| `ITEM_EXE` | avertissement | `exe=` qui n'est pas un chemin local absolu vers un `.exe`, un `.dll` ou un `.ico` (2.0.3) |
| `ITEM_ICONE` | avertissement | `icon=` qui n'est pas une icône Lucide, ou `appicon=` qui n'est pas un nom (minuscules, chiffres, tirets) (2.0.4) |
| `LIBELLE_INCONNU` | erreur | `[ITEM] … label=` vise une clé absente du bloc `REPORT` |
| `ITEM_TEXTE` | erreur | `[ITEM] … about=`, `why=` ou `warn=` ne vise pas une note (`[note:info]`, `[note:warn]`) du bloc `REPORT` (2.0.4) |
| `METRIC_CLE_INCONNUE` | erreur | `[METRIC]` vise une clé qui n'est pas un libellé du bloc `REPORT` |
| `METRIC_CHAMP` | erreur | `[METRIC]` sans `value=`, avec un champ inconnu ou une valeur hors liste |
| `NOTE_INCONNUE` | erreur | `[NOTE]` vise une note absente de `REPORT` ou une cible non déclarée, ou porte un champ autre que `show=` |
| `LOG_CANAL` | erreur | `[LOG]` sans canal : un mot, puis le texte |
| `PROGRESS_VALEUR` | erreur | `[PROGRESS]` hors d'un entier de 0 à 100 |
| `RAPPORT_NON_FERME` | erreur | Bloc `SINDRY:REPORT` non refermé |
| `NOTE_NIVEAU` | erreur | `[note:…]` hors `info` / `warn` |
| `ETIQUETTE_INCONNUE` | erreur | Étiquette inconnue sur une option, un choix ou une entrée de `REPORT` |
| `GROUPE_INCONNU` | erreur | `[group:Clé]` vise un groupe que `REPORT` ne déclare pas |
| `GROUPE_ORPHELIN` | avertissement | Groupe déclaré dans `REPORT` où aucune option ni aucun choix ne se range |
| `CODE_ERREUR_FORME` | erreur | Clé `[error]` qui n'est pas un code : une à trois majuscules, puis deux à quatre chiffres (2.0.3) |
| `CODE_ERREUR_INCONNU` | avertissement | `[ERR] X123 …` ou `[WARN] X123 …` littéral dont le code n'est pas déclaré `[error]` dans `REPORT` |
| `CODE_ERREUR_INUTILISE` | avertissement | Code déclaré `[error]` qui n'apparaît nulle part ailleurs dans le script |
| `VUE_INCONNUE` | erreur | `view` ou `[view:]` hors des vues admises, ou `view` d'une autre forme que `<vue> expert=<vue>` |
| `GRAPHIQUE_INCONNU` | erreur | `chart` hors des graphiques admis |
| `PANNEAU_INCONNU` | erreur | `panels` cite un panneau inconnu |
| `REDEMARRAGE_DIRECT` | erreur | Le script redémarre le PC lui-même (`Restart-Computer`, `shutdown /r`…) au lieu de le demander (2.1.0) |
| `RESTART_SANS_RESUME` | erreur | `[RESTART]` sans `## resume : true` dans l'entête |
| `RESTART_PHASE` | erreur | Phase de `[RESTART]` mal formée : minuscules, chiffres et tirets, 32 au plus |
| `RESTART_PHASE_NON_TRAITEE` | erreur | Phase émise par `[RESTART]` qu'aucun test de `SINDRY_RESUME` ne reprend |
| `RESTART_SANS_3010` | erreur | `[RESTART]` sans `exit 3010` dans le script |
| `RESUME_NON_LU` | erreur | `resume : true` mais `$env:SINDRY_RESUME` n'est jamais lu |
| `CODE_3010` | avertissement | `exit 3010` sans `[RESTART]` : Sindry le lirait comme un échec |

Les contrôles des lignes d'analyse ne voient que ce qui est écrit **en toutes lettres** :
une clé ou une valeur calculée (`Targets.$t`, `size=$($m.Size)`) leur échappe, par
construction. Sindry, lui, lit la vraie sortie, et ignore ce qui ne correspond pas.

Le contrôle des marqueurs ne regarde que les balises **en tête de chaîne affichée**.
Sans cette restriction, les transtypages PowerShell deviennent des faux positifs :
`[long]` ressemble à `[DONE]` à deux caractères près, et `[int]` à `[INFO]`.

---

#### Le style du catalogue officiel

Le validateur vérifie le contrat ; ces règles-ci ne se vérifient qu'à la relecture. Elles
sont **exigées pour le catalogue officiel**, et conseillées pour vos propres scripts : un
script lisible et réglable sans relire tout son code, un journal qu'un débutant comprend.

| Règle | Ce qu'elle demande |
|---|---|
| **Langue** | Code, noms, clés et sortie en anglais ; commentaires en français |
| **Entête du fichier** | `author`, `created`, `updated` et `version` toujours à jour : `updated` et `version` changent à chaque modification |
| **Variables** | Chacune commentée **sur sa ligne** |
| **Fonctions** | Chacune commentée **à son début** : ce qu'elle fait, ce qu'elle reçoit, ce qu'elle renvoie |
| **Chapitres** | Au-delà de 200 lignes, des chapitres balisés : Sommaire, Résumé, Variables, Fonctions, Préparation, Exécution, Journal, Annexes… — adaptés au script |
| **Interrupteurs** | Un maximum de drapeaux (booléens, listes) pour activer, désactiver ou adapter chaque fonctionnalité, **regroupés avec les variables en tête du script**. Les listes de comportement par défaut (ce qui est coché d'office, ce qui est exclu) s'y modifient sans toucher au reste |
| **Codes d'erreur** | Un maximum, chacun déclaré `[error]` avec sa description et, si possible, sa solution, traduites en français dans `LANG fr` — voir « Les codes d'erreur » |
| **Journal** | Clair pour un débutant, aéré, sans pavé ; des tableaux et des listes quand ça s'y prête. Le journal de Sindry est en chasse fixe, lignes conservées : des colonnes alignées par des espaces y tiennent |
| **Une seule langue au journal** | La sortie est en anglais, jamais en deux langues : Sindry ne traduit pas une ligne de journal. Ce que l'utilisateur doit lire dans sa langue passe par les codes d'erreur (description et solution traduites) et, en analyse, par les mots du bloc `REPORT` |

---

#### Votre modèle : le squelette de ce document

Le dépôt [Sindry-Catalogue](https://github.com/burnout293/Sindry-Catalogue) contient le catalogue officiel, et chacun de ses scripts passe le
validateur en `-Strict`. Ils sont de bons exemples de ce qu'on peut faire — mais ils
évoluent, et aucun n'est garanti représentatif de toutes les règles.

**Le squelette donné plus haut reste le modèle de référence**, et pour un script
analysable, l'exemple complet de « Le mode analyse » — aussi disponible tel quel dans
`docs/mockups/exemple-analyse.ps1`. Il est vérifié : on l'extrait de
ce fichier et on le passe au validateur en `-Strict` à chaque relecture de la documentation.

---

#### Pièges silencieux — ce que le validateur ne vous dira pas

Ces règles sont appliquées par le code mais **ne produisent aucun constat**. Un script qui
les enfreint passe `-Strict` sans un mot, puis se comporte mal.

##### En analyse, un refus d'accès se lit comme un zéro

`-ErrorAction SilentlyContinue` transforme un dossier interdit en dossier vide. Lancée sans
droits d'administrateur, l'analyse de l'exemple rapporte `size=0` pour
`C:\Windows\Temp` — non parce qu'il est vide, mais parce qu'elle n'a pas pu y entrer.

Dans Sindry, un script `admin : true` est analysé en administrateur et voit juste. Le piège
guette **l'auteur qui teste à la main** : lancez vos analyses dans une console
administrateur, sinon vous validerez des zéros qui n'en sont pas.

##### Le type s'écrit en minuscules

`[Bool]` ou `[Select]` **passent le validateur** — la comparaison PowerShell y est
insensible à la casse. Mais l'interface compare en respectant la casse : elle ne reconnaît
pas le type et affiche un champ de texte libre à la place de l'interrupteur ou de la liste.

Écrivez toujours `[bool]`, `[number]`, `[string]`, `[select]`, `[multi]`, `[hidden]`.

##### Le code de langue fait exactement deux lettres

`## SINDRY:LANG fr` — pas `fr-FR`, pas `french`. Le validateur n'accepte que deux lettres
et signalerait `TRADUCTION_ABSENTE` sans expliquer pourquoi.

##### L'accolade fermante de `$CONFIG` est seule sur sa ligne

```powershell
$CONFIG = @{        # l'ouverture aussi tient sur une seule ligne
    MaCle = $true
}                   # rien d'autre sur cette ligne, pas même un commentaire
```

Un `}  # fin de config` empêche le parseur de trouver la fin du bloc.

##### Le bloc d'override ne peut pas être en commentaire

Il doit contenir littéralement `$env:SINDRY_CONFIG`, sur une ligne non commentée. Un
exemple mis en commentaire dans un en-tête `<# … #>` ne compte pas, et vous récoltez
`OVERRIDE_ABSENT`.

##### Une clé d'option ne contient pas d'espace

Et elle doit être **identique aux trois endroits** : `SINDRY:OPTIONS`, `SINDRY:LANG` et
`$CONFIG`. Le validateur croise les trois dans les deux sens.

##### `category` ne peut pas viser un lot

`category` désigne une **catégorie** (`tools/categories.json`), jamais un lot. « Entretien
complet », en particulier, est un lot agrégat : il rassemble automatiquement ce qui est
rangé ailleurs, et aucun script ne peut le désigner. Écrire `category : maintenance` range
le script dans « Autres ».

##### Une valeur d'`engine` inconnue se comporte comme `auto`

`engine : powershell` ou `engine : ps7` ne produisent aucune erreur : Sindry prend
simplement l'interpréteur le plus récent disponible. Les trois seules valeurs qui ont un
sens sont `auto`, `winps` et `pwsh`.

##### La configuration injectée est plafonnée

Au-delà de **30 000 caractères** de JSON, Sindry refuse le lancement (limite Windows sur
une variable d'environnement). Un `[multi]` à très nombreux choix ou une `[string]` très
longue peuvent y conduire.

##### Ce que l'écran d'analyse fait vraiment d'un `[ITEM]`

Relevé dans l'interface (2.0.3). Rien de tout cela ne produit de constat :

| Vous écrivez | L'écran en fait |
|---|---|
| `size=` | **De l'espace à libérer** : il entre dans « trouvé » et « coché », et sans `[FREED]` le bilan affiche « environ X libérés ». Un script qui installe, télécharge ou ajoute n'écrit **jamais** `size=` |
| pas de `size=` | L'élément coché compte en « N éléments » |
| `state=ok` | Badge « À jour », ligne **inactive**, case non cochable. Pour une chose déjà en place qu'on peut refaire (réinstaller), écrivez plutôt `checked=false` et `version=` |
| `version=` seul | Un badge neutre, qui ne dit pas « installé » |
| `version=` et `to=` | Un badge accentué `version → to` |
| `name=` | Jamais traduit. `label=<clé de REPORT>` l'est, et s'affiche « Libellé — name » |
| vue `tiles` | L'initiale du nom sur une pastille de couleur, et le nom : **ni l'icône d'`exe=`, ni la version** |
| vue `table` | Colonnes fixes — parent, élément, confiance, taille — avec recherche : **ni version, ni éditeur** |
| `## chart` | Tous les graphiques du résumé pèsent des **tailles** : sans `size=`, un poste quitte le graphique et passe sous « Aussi ». Et le premier script du lot qui déclare un `chart` l'impose à tout le résumé |

---

#### Sécurité : ce que votre script reçoit n'est pas fiable

Un script officiel s'exécute en administrateur, souvent sans que personne relise son code
(§16.4 de la spécification). Or une partie de ce qu'il lit vient de fichiers et de
variables que **n'importe quel programme tournant sous le compte de l'utilisateur peut
modifier**, sans élévation. Tout ce qu'un tel programme peut écrire, il peut le faire viser
par votre script.

| Ce que vous lisez | D'où ça vient | Ce que Sindry garantit | Ce qui vous reste |
|---|---|---|---|
| `$CONFIG` | `settings.json`, inscriptible par l'utilisateur | Type et choix conformes à votre bloc `OPTIONS` ; aucun chemin d'un texte libre ne vise un emplacement protégé | Bornes des nombres, sens des textes |
| `$env:SystemRoot`, `$env:ProgramData`, `$env:SystemDrive`, `$env:ProgramFiles` | Rétablies par Sindry depuis la base de registre `HKLM` | Justes sous Sindry | **Rien hors de Sindry** : en double-clic, elles viennent du profil |
| `$env:TEMP`, `$env:TMP`, `$env:LOCALAPPDATA`, `$env:APPDATA`, `$env:USERPROFILE` | Le profil de l'utilisateur | Ramenées dans son profil si elles visent le système | À vérifier avant toute suppression |
| Le dossier courant | System32 sous Sindry | — | **Ne vous en servez jamais** : `$PSScriptRoot` pour vos fichiers |

Les règles qui en découlent :

1. **Avant de supprimer ou d'écraser, vérifiez où vous êtes.** Un chemin issu de `$CONFIG`
   ou d'une variable de l'utilisateur ne doit jamais désigner Windows, les Program Files,
   ProgramData, la racine d'un lecteur ou un autre profil. Sindry le refuse déjà pour les
   textes libres, mais votre script peut être lancé seul — et un double-clic n'a pas de
   garde.
2. **Bornez vos nombres.** Sindry garantit qu'un `[number]` est un nombre, pas qu'il est
   raisonnable : une ancienneté de `-1` heure, un délai de `100000` jours se refusent dans
   le script.
3. **N'exécutez jamais une valeur.** Pas d'`Invoke-Expression`, pas de
   `[scriptblock]::Create`, pas de commande bâtie par concaténation avec une valeur de
   `$CONFIG`. Passez les valeurs en paramètres (`-LiteralPath $chemin`).
4. **Appelez les outils du système par leur chemin complet**
   (`"$env:SystemRoot\System32\ipconfig.exe"`), jamais par leur seul nom.
5. **Pas de secret dans une option.** Une option `[string]` est enregistrée en clair dans
   `settings.json`. Sindry masque dans le journal les valeurs des clés dont le nom contient
   `Password`, `Secret` ou `Token` — nommez-les ainsi si vous ne pouvez pas faire autrement.

Exemple, pour une liste de dossiers à vider reçue en texte libre :

```powershell
$interdits = @($env:SystemRoot, $env:ProgramFiles, ${env:ProgramFiles(x86)}, $env:ProgramData) |
    Where-Object { $_ } | ForEach-Object { [IO.Path]::GetFullPath($_).TrimEnd('\') }

foreach ($dossier in ("$($CONFIG.CustomPaths)" -split ';' | ForEach-Object { $_.Trim() } | Where-Object { $_ })) {
    $plein = [IO.Path]::GetFullPath($dossier).TrimEnd('\')
    $racine = [IO.Path]::GetPathRoot($plein).TrimEnd('\')
    $refuse = ($plein -eq $racine) -or ($interdits | Where-Object { $plein -eq $_ -or $plein.StartsWith("$_\", 'OrdinalIgnoreCase') })
    if ($refuse) {
        Write-Host "[WARN] Protected location skipped: $dossier"
        continue
    }
    # ... nettoyage de $plein
}
```

---

#### Ce que Sindry relève autour de votre script

Avant et après chaque script, Sindry relève ce qu'un logiciel malveillant modifie pour
s'installer durablement — tâches planifiées, services, démarrage, variables du système,
exclusions de l'antivirus, fichier `hosts`, certificats racine, règles du pare-feu — et le
dit dans le bilan (§12.5 de la spécification). Avant l'approbation, il repère aussi ces
gestes dans le texte du script (points d'attention, §12.2).

Rien ne vous est interdit : un script qui désactive des services ou crée une tâche
planifiée le fait, et le bilan le montre. **Dites-le dans `desc`** : l'utilisateur qui lit
« Programmes de fond de Windows : 2 modifiés » doit pouvoir le rapprocher de ce que
l'action annonçait. Et une action simulée, ou une analyse, qui modifie l'une de ces
familles est signalée comme ne tenant pas sa promesse.

---

#### Ce qui empêche un script de se lancer

Sindry applique partout le principe « **constater, jamais bloquer** » : un script
non conforme s'exécute quand même, ses anomalies sont simplement affichées. Il
existe exactement **sept exceptions**, et aucune ne porte sur la forme du fichier :
toutes protègent ce qui s'exécute avec les droits administrateur, ou ce que
l'interface promet à l'utilisateur.

Si vous écrivez des scripts pour Sindry, ce sont les seules choses qui peuvent
faire refuser le lancement :

##### 1. Le script n'est pas approuvé

Tout script qui ne vient pas du catalogue officiel — ou qui en vient mais a été modifié
depuis — doit être approuvé une fois, par son empreinte exacte. Toute modification du
fichier invalide l'approbation. Voir §12.1 et §16.4 de la spécification.

##### 2. La ligne d'override lit un fichier

**C'est le refus qui surprend le plus, donc lisez-le en entier.**

`SINDRY_CONFIG` **contient le JSON** de vos réglages. Elle ne contient pas, et
n'a jamais à contenir, le chemin d'un fichier. La bonne forme est :

```powershell
# --- Sindry override (ne pas supprimer) ---
if ($env:SINDRY_CONFIG) {
    ($env:SINDRY_CONFIG | ConvertFrom-Json).PSObject.Properties |
        ForEach-Object { $CONFIG[$_.Name] = $_.Value }
}
```

Sindry **refuse de lancer** un script dont le bloc d'override passe
`SINDRY_CONFIG` — ou une variable qui la porte — à `Get-Content` :

```powershell
# REFUSÉ : mécanisme retiré pour raison de sécurité
$wtJson = $env:SINDRY_CONFIG
if (-not $wtJson.TrimStart().StartsWith('{')) { $wtJson = Get-Content -LiteralPath $wtJson -Raw }
```

**Pourquoi.** Ce repli vient d'une version antérieure où la variable donnait le
chemin d'un fichier JSON. Ce fichier vivait dans un dossier inscriptible sans
élévation : un autre programme pouvait le remplacer entre le moment où Sindry
l'écrivait et celui où PowerShell le lisait, et faire entrer ses propres valeurs
dans votre `$CONFIG` — **avec les droits administrateur**. Le fichier a disparu,
mais tant que le repli existe dans un script, la porte peut être rouverte.

**Ce contrôle ne regarde que votre bloc d'override.** Un script qui lit des
fichiers pour son propre compte n'est pas concerné, même s'il utilise
`Get-Content` partout ailleurs :

```powershell
# Parfaitement accepté : rien à voir avec la configuration injectée
$hosts = Get-Content -LiteralPath "$env:SystemRoot\System32\drivers\etc\hosts" -Raw
```

Le validateur signale le même problème sous le code `OVERRIDE_FICHIER`, avec le
numéro de ligne, avant même que vous n'essayiez de lancer le script.

##### 3. Un réglage refusé par la garde

Juste avant le lancement, Sindry contrôle chaque valeur de `$CONFIG` contre votre bloc
`OPTIONS` (section « Sécurité » ci-dessus). Le script **n'est pas lancé** si une valeur n'a
pas le type déclaré, si un choix ne fait pas partie de la liste, ou si un texte libre
désigne un emplacement protégé. L'utilisateur voit quel réglage est en cause, et pourquoi.

Conséquence pour vous : **déclarez précisément.** Une option qui reçoit un chemin doit être
un `[string]`, une liste fermée un `[select]` ou un `[multi]`. Les clés de `$CONFIG` absentes
de `OPTIONS` ne sont pas transmises.

##### 4. La simulation, pour un script qui ne sait pas se simuler

Un script qui déclare l'option `SafeTest` sait **se simuler** : il montre ce qu'il ferait,
sans rien modifier. L'utilisateur règle la simulation script par script, et une pastille
« Simulation » permet de tout simuler d'un coup (§6.9 de la spécification).

Quand la simulation est **activée** — tous les scripts simulables simulés —, un script
qui ne déclare pas `SafeTest` est **refusé**, pas exécuté. C'est volontaire : injecter une
clé qu'un script n'utilise pas ajouterait une entrée inerte à sa table, et il modifierait
la machine pendant que l'interface annonce une simulation. Un refus visible vaut mieux
qu'une garantie fausse.

**Le libellé de l'option ne vous appartient pas.** Sindry affiche partout « Simuler »,
quel que soit celui que vous écrivez : le terme doit être le même pour tous les scripts.
Le bloc `OPTIONS` exige néanmoins un libellé, et la traduction, une ligne ; écrivez donc
celui-ci, pour que votre script se lise comme l'interface le montrera.

Pour qu'un script sache se simuler, déclarez l'option et honorez-la :

```powershell
## SafeTest      : [bool]   Simulate — shows what would be done, changes nothing
```

```powershell
$SafeTest = ("$($CONFIG.SafeTest)" -eq 'True')

if ($SafeTest) {
    Write-Host "[INFO] SafeTest - would delete $($files.Count) file(s)"
} else {
    Remove-Item @files -Force
}
```

**Deux pièges à éviter**, observés dans des scripts existants :

- **Ne simulez pas à moitié.** Une commande qui contacte le réseau, accepte un
  contrat de licence ou modifie un réglage global doit être derrière le garde,
  elle aussi — pas seulement la suppression finale.
- **N'inventez jamais un résultat.** Écrire `[OK] simulated: no corruption found`
  sans avoir rien vérifié fait mentir le journal. Dites ce que vous *auriez* fait,
  jamais ce que vous *auriez trouvé*.

##### 5. Une sélection ou un fichier qui ne correspond plus à la dernière analyse

Pour un script analysable, ce qui revient dans une liste `[items]` est une liste d'ids
(voir « Le mode analyse »). Sindry retient les ids que la **dernière analyse** de chaque
script a annoncés, avec l'empreinte exacte du fichier analysé, et **refuse de lancer**
l'action si `$CONFIG` contient un id que cette analyse n'a pas annoncé — ou si le fichier a
changé depuis. L'utilisateur voit « Relancez l'analyse ».

Ce souvenir ne vit qu'en mémoire : une analyse ne vaut que pour la session qui l'a vue.
Conséquence pour vous : **un id doit être stable** entre l'analyse et l'action, et il ne doit
désigner que ce que l'analyse a montré. C'est ce qui empêche une configuration modifiée à la
main de faire traiter au script un élément que personne n'a vu.

##### 6. L'interpréteur demandé est absent

Un script `engine : pwsh` n'est pas lancé si PowerShell 7 n'est pas installé : Sindry ne se
replie jamais en silence sur PowerShell 5.1 (voir « Le cas de `engine` »). Il est marqué
indisponible, et l'utilisateur voit pourquoi.

##### 7. Une configuration trop volumineuse

Au-delà de 30 000 caractères de JSON, `SINDRY_CONFIG` ne tiendrait plus dans une variable
d'environnement : Sindry refuse le lancement plutôt que de transmettre une configuration
tronquée (voir « La configuration injectée est plafonnée »).

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
- [ ] Aucun redémarrage direct. Un `[RESTART] <phase>` a `## resume : true`, est suivi de
      `exit 3010`, et sa phase est reprise par un test de `$env:SINDRY_RESUME`.

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
