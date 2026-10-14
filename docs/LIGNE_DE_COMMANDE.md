# Sindry en ligne de commande

Depuis la 2.0, tout ce que fait Sindry se fait aussi depuis une console : lister et lancer
des actions et des lots, analyser, régler, gérer les catalogues et les ambiances — et
l'installer sans un clic. Pour un script de déploiement, une tâche planifiée, ou qui
préfère le clavier.

Référence : `docs/SPECIFICATION.md` §19.

---

## Où elle se trouve

L'installeur pose **`sindry.com`** à côté de `sindry.exe`, dans
`C:\Program Files\Sindry\`. C'est un programme console à part entière : la console attend
sa fin, reçoit sa sortie et son code de sortie. Dans une console ouverte dans ce dossier,
ou si le dossier est dans le `PATH` (option `/PATH` de l'installeur), **`sindry`** suffit :
Windows trouve `sindry.com` avant `sindry.exe`.

```powershell
& "C:\Program Files\Sindry\sindry.com" scripts
```

Aucune fenêtre ne s'ouvre. La ligne de commande passe par **les mêmes contrôles que
l'interface** : approbation (§12.1), garde des réglages et emplacements protégés (§12.4),
simulation (§6.9), analyse et sa mémoire (§17.4), historique. Elle n'ouvre aucune porte que
l'interface tient fermée.

---

## Les commandes

Les commandes s'écrivent en français ou en anglais (`lancer` ou `run`). Les messages
suivent la langue des réglages, ou la variable `SINDRY_LANG=fr|en`.

### Actions

| Commande | Anglais | Ce qu'elle fait |
|---|---|---|
| `scripts` | `list` | Les actions installées : id, titre, origine, approuvée, analysable, simulable |
| `lots` | `batches` | Les lots, et leurs actions dans l'ordre d'exécution |
| `montrer <id> [--source]` | `show` | Une action : fichier, empreinte, approbation, options, points d'attention ; `--source` affiche le script |
| `verifier <id>` | `check` | Contrat et syntaxe, sans rien exécuter. Code 1 s'il y a une erreur |
| `analyser <id>` | `scan` | Lance l'analyse (§17) : ce que l'action ferait, et ce que Sindry cocherait d'office |
| `lancer <cible>…` | `run` | Lance des actions (`<id>`) et des lots (`lot:<id>`), dans l'ordre donné |
| `approuver <id> --empreinte <h>` | `approve … --hash` | Approuve **ce fichier exact** — en administrateur |
| `historique [--nombre N]` | `history [--count N]` | Les dernières exécutions |

**`lancer` fait ce que fait l'entretien du mode Simple** :

1. un seul point de restauration avant toute la série, si une action le demande et ne sera
   pas simulée (`--sans-restauration` pour s'en passer) ;
2. pour une action qui sait analyser : l'analyse d'abord, puis l'action avec **ce que
   l'interface cocherait d'office** (« Ce que Sindry coche d'office », `FORMAT_SCRIPT.md`).
   Rien à faire : l'action n'est pas lancée. `--sans-analyse` lance l'action avec ses seuls
   réglages ; une liste `[items]` n'y reçoit alors rien ;
3. une action partagée par deux lots ne tourne qu'une fois ;
4. en cas d'échec, le réglage 3.1 décide : continuer, ou arrêter la série ;
5. chaque exécution entre dans l'historique, comme depuis l'interface.

`--simuler` simule ce lancement, quels que soient les réglages : une action qui ne sait pas
se simuler est refusée, pas exécutée (§6.9).

**Ctrl+C suit la règle du bouton « Arrêter » (§6.3).** Une première fois : l'action en cours
va au bout, rien d'autre n'est lancé. Une deuxième : elle est interrompue si elle peut
l'être sans risque. Une troisième : elle est arrêtée quand même.

**Approuver depuis la console.** Un script qui ne vient pas du catalogue officiel doit être
approuvé, une fois, pour son contenu exact. `sindry montrer <id> --source` affiche le
script et son empreinte ; après l'avoir lu, en administrateur :

```powershell
sindry approuver 3f2b1a9c-7e4d-4c6a-9b0e-1d5f6a8c2e0b --empreinte 5ae974bf…51aa3
```

L'empreinte donnée doit être celle du fichier **tel qu'il est maintenant** : modifié entre
la lecture et l'approbation, il est refusé.

### Réglages

| Commande | Anglais | |
|---|---|---|
| `reglages` | `settings` | Les réglages |
| `regler <réglage> <valeur>` | `set` | Voir la table ci-dessous |
| `config exporter <fichier>` | `config export` | Toute la configuration dans un fichier (lots, réglages par action, préférences) |
| `config importer <fichier>` | `config import` | La reprendre ; un fichier invalide est refusé sans rien changer |
| `config reinitialiser --oui` | `config reset --yes` | Revenir aux réglages d'usine |

| Réglage | Valeurs |
|---|---|
| `theme` | `clair`, `sombre`, `systeme` (`light`, `dark`, `system`) |
| `langue` | `fr`, `en` |
| `ambiance` | une ambiance livrée (`plan`, `synthwave`…) ou installée |
| `mises-a-jour` | `proposer`, `beta`, `jamais` |
| `echec` | `continuer`, `arreter` |
| `journaux` | une taille en Mo ; `0` : pas de limite |
| `politique` | `bypass`, `remotesigned`, `unrestricted` |
| `simulation` | `oui`, `non` — toutes les actions qui savent se simuler |
| `echelle` | `1`, `1.15`, `1.3`, `1.5`, `1.75` |

### Catalogues

| Commande | Anglais | |
|---|---|---|
| `catalogues` | `catalogs` | Les catalogues, leur version installée, leur état |
| `catalogue verifier [<id>]` | `catalog check` | Ce qu'une mise à jour changerait — rien n'est écrit |
| `catalogue installer [<id>]` | `catalog install` | Installe ou met à jour ; l'officiel sans `<id>` |
| `catalogue activer <id>` | `catalog enable` | |
| `catalogue desactiver <id>` | `catalog disable` | |

Ajouter, modifier ou retirer un catalogue tiers reste dans l'interface : c'est une décision
de confiance, qu'on prend en voyant la clé et le dépôt.

### Ambiances

| Commande | Anglais | |
|---|---|---|
| `ambiances` | `looks` | Les ambiances installées, et les fichiers ignorés |
| `ambiance verifier <fichier>` | `look check` | Vérifie un fichier `.sindry-ambiance`, sans l'installer — de quoi travailler à une ambiance (`FORMAT_AMBIANCE.md`) |
| `ambiance importer <fichier> [--remplacer]` | `look import [--replace]` | L'installe ; une ambiance du même `id` n'est remplacée qu'avec `--remplacer` |
| `ambiance exporter <id> <fichier>` | `look export` | Copie son fichier |
| `ambiance retirer <id> --oui` | `look remove --yes` | Efface son fichier |
| `ambiance choisir <id>` | `look use` | La choisir |

---

## Pour un script

**`--json`** : la sortie est un document JSON, en **ASCII pur** (les accents deviennent
`\u00e9`). PowerShell 5.1 lit la sortie d'un programme dans la page de codes de la console,
pas en UTF-8 : en ASCII, `ConvertFrom-Json` la reçoit intacte.

```powershell
$actions = sindry scripts --json | ConvertFrom-Json
$bilan   = sindry lancer lot:cleaning --json | ConvertFrom-Json
$bilan.resultats | Where-Object statut -ne 'reussi'
```

Sans `--json`, les lignes des scripts défilent comme dans le journal ; en `--json`, elles
ne sont pas affichées (chaque résultat donne le chemin de son journal complet). Une erreur,
en `--json`, est aussi un document : `{ "erreur": "…", "code": 3 }`.

**Codes de sortie**

| Code | Signification |
|---|---|
| `0` | Réussi |
| `1` | Échec : une action a échoué, ou une opération n'a pas abouti |
| `2` | Usage : commande, option ou valeur inconnue |
| `3` | Refusé par une règle de Sindry : script non approuvé, réglage refusé par la garde, simulation impossible, fichier invalide |
| `4` | Il faut les droits administrateur |
| `5` | Introuvable : action, lot, ambiance, catalogue |
| `3010` | Terminé, un redémarrage est nécessaire — la convention de Windows Installer |

Pour `lancer`, le code résume la série : `1` si une action a échoué, sinon `3` si une a été
refusée, sinon `3010` si l'une demande un redémarrage, sinon `0`.

**Une exécution à la fois** est la règle de l'interface (§6.6). Elle vaut dans un même
processus : ne lancez pas une action depuis la console pendant que l'interface en exécute
une autre.

---

## Installation silencieuse

```powershell
.\Sindry_2.0.0_x64-setup.exe /S
.\Sindry_2.0.0_x64-setup.exe /S /LANG=en /THEME=sombre /SANSBIENVENUE /PATH
.\Sindry_2.0.0_x64-setup.exe /S /CONFIG="\\serveur\partage\sindry-config.json"
```

| Option | |
|---|---|
| `/S` | Silencieuse : aucune fenêtre (NSIS). `/P` : passive, une barre de progression |
| `/LANG=fr\|en` | La langue de l'interface |
| `/THEME=clair\|sombre\|systeme` | |
| `/AMBIANCE=<id>` | Une ambiance livrée, ou installée ensuite (`sindry ambiance importer`) |
| `/SANSBIENVENUE` | Pas de bienvenue au premier lancement (`/NOWELCOME`) |
| `/CONFIG=<fichier>` | Une configuration exportée (`config exporter`, Réglages 5.1) : lots, réglages par action, préférences. Lue et vérifiée à l'installation, puis copiée — le fichier d'origine n'est plus nécessaire |
| `/PATH` | Ajoute le dossier d'installation au `PATH` du système : `sindry` dans toute console. Retiré à la désinstallation |
| `/NS` | Pas de raccourcis (NSIS) |
| `/D=<dossier>` | Le dossier d'installation (NSIS) — **en dernier**, sans guillemets |

**Comment les options arrivent chez chacun.** L'installeur tourne souvent sous un autre
compte que celui qui utilisera Sindry — un administrateur, le compte SYSTEM d'un outil de
déploiement — et les réglages vivent dans le profil de chacun. Les options sont donc
écrites dans `preconfiguration.json`, à côté de l'exécutable ; Sindry les applique **au
premier lancement de chaque compte**, quand il n'a pas encore de réglages. Un compte qui a
déjà les siens les garde. Une mise à jour n'y touche pas. Le fichier part avec Sindry à la
désinstallation.

**Depuis Git Bash**, les arguments qui commencent par `/` sont réécrits en chemins
(`/S` devient `S:/`) : préfixez la commande de `MSYS_NO_PATHCONV=1`. PowerShell et l'invite
de commandes les passent tels quels.

Une option invalide (`/LANG=de`, un `/CONFIG` illisible) n'empêche pas l'installation :
elle est écrite dans le journal de l'installeur, et rien n'est préconfiguré.

Après une installation silencieuse, le catalogue officiel s'installe pour un compte depuis
ce compte : `sindry catalogue installer`.

**Désinstaller sans un clic** : `"C:\Program Files\Sindry\uninstall.exe" /S`. Une
désinstallation silencieuse n'efface aucune donnée (§12.4).
