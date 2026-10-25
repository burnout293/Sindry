# SindryOS — soigner Windows sans le démarrer

> Étude demandée le 25/10/2026. Elle propose une architecture, en dit les limites et
> liste ce qui reste à décider. Rien n'est encore construit. Les points marqués
> **à vérifier** le seront par un prototype.

---

## En bref

- **SindryOS** : une clé USB qui démarre le PC sur un petit Windows de dépannage (WinPE),
  où Sindry soigne le **Windows éteint** du disque : pilote qui empêche de démarrer,
  fichiers verrouillés, mise à jour qui bloque, disque plein, fichiers à sauver.
- **Base retenue : WinPE**, construit **sur le PC de l'utilisateur** avec le kit de
  Microsoft (ADK). Distribuer une image toute faite est restreint par la licence.
- **Interface native**, sans WebView2 : WebView2 n'est pas pris en charge dans WinPE. Le
  mode Simple d'abord.
- **Deux produits, un cœur** : `sindry-core` (contrat, analyse, moteur, signatures)
  partagé par **SindryApp** (l'application actuelle), **sindry-cli** et **SindryOS**.
- **Des actions écrites pour le hors-ligne** (`## offline : true`) : un script actuel ne
  tourne pas tel quel, il vise le Windows qui tourne.
- **Trois limites dures** : BitLocker (la clé de récupération, ou une préparation depuis
  Windows), le démarrage rapide (le disque est « en cours d'utilisation »), et les
  contrôleurs de disque qui demandent un pilote.

---

## 1. Ce que SindryOS permettrait

| Situation | Ce que fait SindryOS | Pourquoi Windows démarré ne peut pas |
|---|---|---|
| Windows ne démarre plus (écran bleu en boucle) | Désactive le pilote ou le service fautif (`Start = 4` dans le registre hors ligne) | Le pilote se charge avant tout le reste |
| Un pilote ou un fichier verrouillé (restes de Webroot : `WRkrn`, script 417) | Le retire directement | En marche, il est chargé : il faut aujourd'hui désinscrire, redémarrer, supprimer |
| Une mise à jour empêche de démarrer | La désinstalle (`DISM /Image /Remove-Package`) | — |
| Disque plein, Windows ne démarre plus | Libère de la place (temporaires, caches, anciennes mises à jour) | — |
| Fichiers système abîmés | `SFC /offbootdir`, `DISM /Image /RestoreHealth` | La réparation en marche bute sur les fichiers ouverts |
| Démarrage cassé | Reconstruit le démarrage (`bcdboot`) | — |
| Logiciel malveillant qui se défend | Liste et retire ses démarrages automatiques (registre, tâches, services) quand il ne tourne pas | Il se relance, ou bloque l'outil |
| Avant de réinstaller | Sauve les fichiers personnels sur la clé ou un disque externe | — |
| Comprendre un écran bleu | Lit les rapports d'incident (`Minidump`) et les journaux `.evtx` hors ligne, et dit le pilote en cause | — |
| Revenir en arrière | Restaure un point de restauration (`rstrui /offline`) | — |

L'analyse garde son rôle : **montrer avant d'agir**. SindryOS examine le Windows éteint,
affiche ce qu'il a trouvé, et l'utilisateur coche.

## 2. Ce que SindryOS ne fera jamais

Démarrer sur une clé donne un accès complet au disque, sans les protections de Windows.
C'est la force de l'outil, et ce qui le rendrait dangereux entre de mauvaises mains. Il ne
proposera donc jamais :
- de réinitialiser un mot de passe, ni d'activer le compte Administrateur intégré ;
- de contourner BitLocker : sans la clé, le disque reste fermé, et c'est voulu ;
- de profiter de l'absence des protections de Windows (Defender, UCPD…) pour imposer des
  réglages qu'elles refusent en marche.

**La trace.** Chaque session hors ligne écrit son journal sur la clé **et** sur le
Windows soigné. Au démarrage suivant, SindryApp le montre : « Une session SindryOS a eu
lieu le … : 3 actions ». Rien ne se fait en cachette.

## 3. Sur quoi le construire

| | **WinPE (ADK)** | **WinRE du PC** | **Linux autonome** | **Outil dans WinRE** |
|---|---|---|---|---|
| Ce que c'est | Le Windows de déploiement de Microsoft | L'environnement de récupération déjà sur le PC | Une distribution Linux sur clé | Sindry ajouté aux « Options avancées » du PC |
| Les scripts PowerShell | Oui (composant PowerShell) | **à vérifier** : PowerShell absent par défaut | Non : PowerShell 7 sous Linux n'a ni registre ni outils Windows | Comme WinRE |
| Registre hors ligne | `reg load`, natif | Natif | `hivex` : d'autres outils, un autre langage | Natif |
| DISM, SFC, bcdboot | Oui | Oui | Non | Oui |
| Secure Boot | Signé Microsoft : passe partout | Idem | Dépend de `shim` ; refusé d'office sur certains PC (Secured-core) | Idem |
| BitLocker | Composant `WinPE-SecureStartup` | Inclus | `dislocker`, fragile en écriture | Inclus |
| Wi-Fi | **Absent** | Présent (composant WinPE-WiFi) | Oui | Présent |
| Distribution | Image **non redistribuable** telle quelle : à construire chez l'utilisateur | Propre à chaque PC | Libre | Modifie le PC lui-même |
| L'interface de Sindry (WebView2) | Non prise en charge | Idem | Tauri tourne (WebKitGTK) | Idem |
| Taille | ≈ 500 à 700 Mo | ≈ 500 Mo | ≈ 1 Go | Limitée par la partition de récupération |

**Recommandation : WinPE, construit sur le PC de l'utilisateur.** C'est le seul qui
garde les scripts PowerShell, les outils de réparation de Windows et Secure Boot sans
réécrire le moteur. Linux libérerait la licence et l'interface, mais il faudrait réécrire
toutes les actions dans un autre langage. Ce serait un autre produit. La variante WinRE,
qui apporte le Wi-Fi et un outil directement dans « Options avancées », vient plus tard :
la partition de récupération est petite, et la modifier a déjà posé problème chez
Microsoft (mise à jour KB5034441).

## 4. L'interface

WebView2, qui affiche l'interface de SindryApp, n'est pas pris en charge dans WinPE. Trois
voies :

| Voie | Pour | Contre |
|---|---|---|
| **Interface native Rust, rendu logiciel** (Iced avec `tiny-skia`, licence MIT) | Pas de GPU requis, un seul exe, le style de Sindry | Une interface à écrire ; **à vérifier** dans WinPE (affichage de base) |
| **Ligne de commande** (`sindry-cli`, déjà amorcée dans `cli.rs`) | Prête vite ; sert aussi le pilotage à distance de la v3 | Pas pour un débutant |
| Faire tourner WebView2 dans WinPE | Toute l'interface actuelle | Non pris en charge ; des dizaines de fichiers à ajouter ; fragile |

**Recommandation** : la ligne de commande pour le prototype, puis une interface native
réduite au **mode Simple** : choisir le Windows à soigner, analyser, cocher, faire, bilan.

## 5. Architecture : SindryApp et SindryOS

**Oui, les détacher.** Leurs environnements diffèrent (WebView2 ou non), leurs rythmes de
publication aussi (WinPE suit les versions de l'ADK), et leurs risques encore plus. Mais ils
partagent ce qui fait Sindry :

```
sindry-core     contrat des scripts, analyse, moteur, catalogue et signatures, garde
 ├─ SindryApp   l'application actuelle (Tauri)
 │              + « Créer une clé SindryOS », « Préparer un passage hors ligne »
 ├─ sindry-cli  la ligne de commande : WinPE, version portable, pilotage à distance (v3)
 └─ SindryOS    le constructeur de la clé, et l'interface native dans WinPE
```

- **Prérequis** : sortir `sindry-core` de `src-tauri`. Le moteur s'y prête déjà :
  `runner::lancer` ne connaît pas Tauri, il publie par le trait `Sortie`. Il faut encore
  détacher la découverte et les réglages de `AppHandle`. Ce travail sert aussi la version
  portable (2.4.0) et le pilotage à distance (v3).
- **Dépôts** : `sindry-core` et `sindry-cli` dans le dépôt Sindry (un espace de travail
  Cargo), SindryOS dans un dépôt `Sindry-OS` (constructeur, interface, publication propre).
- **Un seul catalogue signé** : les actions hors ligne vont dans Sindry-Catalogue, avec
  `## offline : true`, vérifiées par la même clé.

## 6. Le contrat des scripts hors ligne

Un script actuel vise le Windows **qui tourne** : sous SindryOS, ce serait WinPE. Les
actions hors ligne se déclarent donc à part.

| Élément | Règle proposée |
|---|---|
| `## offline : true` | L'action est écrite pour un Windows éteint ; seule celle-là apparaît dans SindryOS |
| `SINDRY_TARGET` | La racine du Windows soigné (`D:\`) ; `SINDRY_WINDIR` = `D:\Windows` |
| Les ruches | **SindryOS les charge**, le script ne fait jamais `reg load` : `HKLM:\SINDRY_SOFTWARE`, `HKLM:\SINDRY_SYSTEM`, une ruche par profil. Déchargées après, même en cas d'échec |
| Le jeu de contrôle | Lu dans `SINDRY_SYSTEM\Select` (`ControlSet001`…) : jamais `CurrentControlSet`, qui serait celui de WinPE |
| Interdits | Tout ce qui interroge le système qui tourne : `Get-Service`, `Get-AppxPackage`, `Get-CimInstance`, `HKLM:\SOFTWARE`… Le validateur les relèverait (`HORS_LIGNE_EN_LIGNE`) |
| L'analyse | Identique : `SINDRY_MODE=scan`, rien n'est modifié |

## 7. Contraintes et limites techniques

| Contrainte | Effet | Réponse |
|---|---|---|
| **BitLocker**, ou le chiffrement d'office de Windows 11 | Disque illisible. Le TPM ne déverrouille pas hors de la chaîne de démarrage de Windows | **Préparer depuis Windows** : SindryApp suspend BitLocker pour un seul redémarrage (`manage-bde -protectors -disable C: -RebootCount 1`, avec accord). Sinon, SindryOS demande la clé de récupération (48 chiffres), jamais gardée |
| **Démarrage rapide, veille prolongée** | Le disque est « en cours d'utilisation » : y écrire perd des données au réveil de Windows | SindryOS le détecte et **refuse d'écrire** ; « Préparer » fait un arrêt complet |
| **Contrôleurs de disque** (Intel VMD/RST, RAID) | Disque invisible dans WinPE | Le constructeur ajoute les pilotes de stockage **du PC qui construit** (`pnputil /export-driver`) : une clé est au mieux pour son PC |
| **Secure Boot** | Aucun problème avec WinPE, signé Microsoft | — |
| **ARM64** (PC Snapdragon) | WinPE et Sindry ARM64 nécessaires | Plus tard ; x64 d'abord |
| **Wi-Fi** | Absent de WinPE | Câble, ou partage de connexion USB du téléphone ; le dépannage hors ligne n'en a presque jamais besoin |
| **Clavier** | QWERTY par défaut | `wpeutil SetKeyboardLayout` en AZERTY |
| **Plusieurs Windows** sur le PC | Lequel soigner ? | Lister les volumes qui portent `\Windows\System32\config\SYSTEM`, faire choisir |
| Disques dynamiques, Espaces de stockage, VeraCrypt | Non lus, ou mal | Non pris en charge, et l'écran le dit |
| **72 heures** | WinPE redémarre seul au bout de 72 h | Sans effet pour un dépannage |
| **Mémoire** | WinPE vit en mémoire (lecteur `X:`) | 1 Go au minimum, 2 Go pour être à l'aise |
| **Taille** | ≈ 500 à 700 Mo | Clé de 2 Go ou plus, en FAT32 pour l'UEFI ; une ISO aussi, pour Ventoy et les VM |
| **Licence** | L'image WinPE ne se redistribue pas telle quelle | Construite chez l'utilisateur, avec l'ADK qu'il télécharge ; **à faire confirmer** sur les termes de l'ADK en vigueur |
| Les protections de Windows ne tournent pas | Defender, UCPD, les droits dynamiques sont absents | Une responsabilité : voir §2 |

## 8. Sécurité

- **Tout tourne en SYSTEM**, sans UAC, sur un disque sans protection active. Les
  vérifications de SindryApp s'appliquent donc toutes : un script du catalogue est
  vérifié contre l'index signé, avec la clé compilée dans l'exe ; un script personnel est
  montré et **approuvé à chaque session**. Rien ne peut être retenu sur une clé USB
  inscriptible par n'importe qui.
- **La clé elle-même** peut être modifiée par quiconque la tient. SindryApp retient
  l'empreinte des fichiers de la clé qu'il a construite (dans HKLM) et sait la
  **vérifier** avant usage. La règle reste : une clé SindryOS se range comme une clé de
  maison.
- **BitLocker reste le garde-fou** : sans la clé de récupération ou une préparation
  depuis Windows, le disque reste fermé. SindryOS ne cherche jamais à le contourner.

## 9. Comment SindryApp et SindryOS se parlent

1. **Créer la clé** (SindryApp, en administrateur) : télécharger l'ADK et son module
   WinPE, construire l'image (composants PowerShell, WMI, BitLocker, stockage), y ajouter
   SindryOS, les pilotes de disque du PC et le catalogue hors ligne vérifié, puis l'écrire
   sur la clé (et en ISO).
2. **Préparer un passage**, quand Windows démarre encore : coupe le démarrage rapide pour
   ce redémarrage, suspend BitLocker pour un redémarrage, puis dit comment démarrer sur la
   clé (touche du menu de démarrage, F12 ou Échap selon le PC).
3. **Choisir d'avance**, en idée : les actions hors ligne cochées dans SindryApp sont
   déposées sur la clé ; SindryOS les propose d'office.
4. **Le retour** : au démarrage suivant de Windows, SindryApp lit le journal de session
   et l'ajoute à l'historique.

**Avec la version portable (2.4.0)**, une seule clé peut porter les deux : Sindry portable
dans un dossier, SindryOS pour démarrer.

## 10. Plan proposé

| Étape | Contenu | Taille |
|---|---|---|
| 0. Prototype | ADK, WinPE et PowerShell ; `sindry-cli` ; deux actions hors ligne (désactiver un pilote, lire le dernier écran bleu) ; essai dans la VM, avec et sans BitLocker | Petite |
| 1. Le cœur | `sindry-core` et `sindry-cli` sortis de `src-tauri` ; sert aussi la 2.4.0 et la v3 | Moyenne |
| 2. Le constructeur | Un script PowerShell (`Sindry-OS/construire.ps1`), puis « Créer une clé SindryOS » dans SindryApp | Moyenne |
| 3. L'interface | Native, mode Simple ; maquettes d'abord | Grande |
| 4. Le catalogue hors ligne | Cinq à huit actions : pilotes et services, restes verrouillés, mises à jour, place, démarrage, sauvegarde | Moyenne |
| 5. La variante WinRE | Wi-Fi, outil dans « Options avancées » | Moyenne, plus tard |

## 11. À décider

| Question | Recommandation |
|---|---|
| Construire chez l'utilisateur, ou publier une image | **Chez l'utilisateur** (licence) |
| Un dépôt `Sindry-OS` à part | **Oui**, avec `sindry-core` partagé |
| L'interface du début | **Ligne de commande** pour le prototype, puis native en mode Simple |
| BitLocker | **Préparer depuis Windows**, clé de récupération en secours |
| Le nom | SindryOS, Sindry Secours, Sindry Rescue… |
| La place dans la feuille de route | Après la 2.4.0 (portable), dont elle partage le cœur ; avant ou pendant la v3 |

---

## Sources

- Licence de WinPE : un éditeur indique ne plus pouvoir fournir d'image WinPE
  personnalisée, à construire chez le client — [SEP sesam BSR Pro for Windows](https://wiki.sep.de/wiki/index.php/SEP_sesam_BSR_Pro_for_Windows).
  Le texte qui fait foi est celui des termes livrés avec l'ADK.
- Composants de WinPE — [WinPE Optional Components Reference (Microsoft)](https://learn.microsoft.com/en-au/windows-hardware/manufacture/desktop/winpe-add-packages--optional-components-reference?view=windows-11).
- Wi-Fi absent de WinPE, présent dans WinRE — [MSEndpointMgr](https://msendpointmgr.com/2018/03/06/build-a-winpe-with-wireless-support/),
  [Broadcom](https://knowledge.broadcom.com/external/article/268140).
- UCPD, qui bloque `TaskbarDa` — [oofhours](https://oofhours.com/2025/05/02/what-is-windows-11s-new-ucpd-feature/),
  [ElevenForum](https://www.elevenforum.com/t/add-or-remove-widgets-button-on-taskbar-in-windows-11.32/latest).
- WebView2 : aucune prise en charge documentée dans WinPE (recherche du 25/10/2026).
