# Sindry — écrire un script PowerShell conforme

> **Référence complète, pour une IA.** Ce fichier est généré par
> `tools/build-script-guides.mjs` (Sindry {{VERSION}}) à partir de `docs/FORMAT_SCRIPT.md`,
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

### Les réglages nombreux (plus de 10)

44. **Range-les en sections** : `[section:Cle]` entre le type et le libellé fait de chaque
    section un **onglet** de la fiche Expert ; `[section:Cle.Sous]` ajoute un sous-titre
    dans l'onglet. Déclare chaque section **et** chaque sous-titre dans `REPORT`
    (`## Cle : [section] Titre — sous-titre`, `## Cle.Sous : [section] Titre`) et
    traduis-les dans `LANG fr`. Des onglets courts (3 à 8 réglages), nommés par ce que
    l'utilisateur cherche (Affichage, Confidentialité…), jamais par la technique
    (Registre…). Un réglage sans section finit dans un dernier onglet « Autres réglages »
    (« Ranger les réglages en sections »).

### Les échecs et ce qui est déjà en place

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

{{DATA}}
