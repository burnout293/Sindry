# Sindry — feuille de route

Versions mineures successives jusqu'à ce que le gros du travail soit fait ; la 2.0
viendra ensuite. Numérotation sémantique : une version qui apporte des fonctions monte
d'un cran (1.1, 1.2…), une version de corrections seules prend un troisième chiffre
(1.1.1, 1.1.2…).

| Version | Contenu | État |
|---|---|---|
| **1.0.0** | Première version publiée | Publiée le 27/09/2026 |
| **1.1** | Mise à jour automatique de Sindry | Publiée le 06/10/2026 |
| **1.1.1** | Simulation réglable script par script, bouées posées sur la vague, rangées multiples — et premier test réel de la mise à jour automatique | Publiée le 06/10/2026 |
| **1.2** | Les scripts quittent Sindry : catalogue officiel dans son propre dépôt (MIT), index signé, installation et mises à jour depuis l'application (§16) ; durcissement (§12.4) ; visite du premier démarrage (§13) ; désinstallation au choix | Publiée le 06/10/2026 |
| **1.2.1** | Réglages en page entière, une section par page, recherche, Simple / Expert sans les quitter (§8) ; emplacements protégés : résumé et plan du disque ; écran « Choisir » : le champ de bouées défile au lieu de se défaire, « Continuer » collé en bas (§15.2) | Publiée le 07/10/2026 |
| **1.4** | Publiée d'un seul tenant, avec ce que prévoyait la 1.3 : | Prête |
| | — catalogues multiples : ajouter, modifier, retirer, activer ; consulter et choisir ses actions (§16) ; présentation modifiable sans code (`src/presentation.json`, §13) ; « action » au lieu d'« entretien » (§3) | Fait |
| | — analyser → cocher → nettoyer (§17), avec une norme qui laisse les scripts composer l'écran : listes d'éléments, arborescences, confiance, mesures, notes ; résumé à déplier en Simple (anneau par défaut, graphique au choix), vues et panneaux en Expert, un seul affichage réglé par `show` (`docs/FORMAT_SCRIPT.md`, exemple `docs/mockups/exemple-analyse.ps1`) | Fait |
| | — catégories distinctes des lots, onglets Scripts / Lots en Expert, réglages de la 1.2 migrés sans perte (§4.1) | Fait |
| **1.5** | Témoin des changements sensibles (§12.5) : relevé avant et après chaque script, bilan en langage courant, points d'attention étendus aux mêmes gestes | Fait |
| | — icône de l'application, aux couleurs de l'accent orange (remplacée en 1.6) | Fait |
| | — compatibilité du catalogue vérifiée par un banc : chaque analyse lue par le vrai moteur et affichée par la vraie interface (`?catalogue=reel`) ; un second `[FIND]` complète le premier ; sortie de PowerShell 5.1 décodée dans la page de codes de la console | Fait |
| | — couleur d'accent au choix, orange par défaut (§15.4) | Fait |
| | — mode Simple en quatre étapes : Choisir, Vérifier (une case par action), Analyser, Entretien ; depuis l'Expert, « Exécuter » et « Lancer ce lot » passent par l'analyse (§2) | Fait |
| | — journal : filtre par famille de lignes (infos, étapes, réussites, avertissements, erreurs, analyse, canaux, texte brut) | Fait |
| | — point de restauration : verdict indépendant de la langue de Windows, accents lisibles, message simple en Simple et détail au journal, aucun point pendant une simulation | Fait |
| **1.6** | Pré-version (tag `v1.6.0-beta`) : à installer à la main, les installations ne la reçoivent pas d'elles-mêmes | Pré-version |
| | — thème Braise, aux couleurs de l'icône de Sindry : Ardente par défaut, Crépuscule au choix, en clair et en sombre (§15.4) | Fait |
| | — graphiques à engrenages : l'anneau-engrenage par défaut, la jauge à dents, la double couronne (§17.2) | Fait |
| | — faits des fiches en icônes et valeurs, provenance à côté du titre (§2) | Fait |
| | — journal toujours ouvrable, même vide, en Simple comme en Expert (§2) | Fait |
| | — nouvelle icône ; l'ancien nom reste jusqu'à la 2.0 | Fait |
| **1.7** | Pré-version (tag `v1.7.0-beta`) | Pré-version, publiée le 11/10/2026 |
| | — vagues : la pause suit la fenêtre réduite, plus le focus ; l'eau et les bouées s'arrêtent ensemble (§15.3) | Fait |
| | — réglages par script : « Actif » et « Nécessite un redémarrage » retirés (§4.2) | Fait |
| | — la balise sur « Droits limités » ; encart et guide à la lampe torche quand une action a besoin des droits administrateur (§15.6) | Fait |
| | — ambiances Synthwave, Terminal et Aurore, avec leur décor (§15.4, maquette M17) | Fait |
| | — bienvenue en cinq pages (réglages d'abord, visite proposée à la fin), visites de l'accueil et de l'Expert, centre d'aide « ? », contenu dans `src/guide.json` vérifié en CI (§13, maquette M18) | Fait |
| | — ambiance **Spatial** (maquette M19, d'abord pensée comme un thème Star Wars) | Fait |
| | — le graphique du résumé se déclare dans le script (`## chart`) ; le réglage 1.7 disparaît (§17.2) | Fait |
| | — icône de l'installeur et du désinstalleur (celle de NSIS jusqu'ici) | Fait |
| **1.8** | Bouées de la deuxième rangée recalées sur leur vague ; installeur : « Mettre à jour » et « Réparer » au lieu de désinstaller ; versions bêta au choix (réglage 1.4 « Mises à jour »), réglages rangés par numéro ; plus de vitesse à l'étape « Vérifier », le nombre d'actions au lieu des minutes | Fait |
| | — catalogues : ajouter, modifier, retirer dans les deux modes ; retirer efface la copie téléchargée, l'officiel compris, qui se réinstalle de zéro | Fait |
| | — choix multiples : une liste à cocher lisible, compteur, « Tout / Aucun » (maquette M22 B) | Fait |
| | — **Plan** devient l'ambiance par défaut : grille, roues au trait, cotes (maquette M23 B) | Fait |
| | Publiée comme dernière version stable (tag `v1.8.0`) | Prête |
| **2.0** | Pré-version (tag `v2.0.0-beta`) | En cours |
| | — l'application s'appelle **Sindry** : nom, exécutable, dossiers, registre, dépôts, police, contrat des scripts (`SINDRY:`) ; une installation existante est reprise d'elle-même, et l'ancien contrat est lu jusqu'à la 2.1 (§18) | Fait |
| | — plus aucune fenêtre de console : point de restauration, relance en administrateur | Fait |
| | — réglage « Ambiance » renuméroté 1.7 ; le style Plan s'étend au mode Expert | Fait |
| | — emplacements protégés pour une seule action, dans sa configuration (§12.4) | Fait |
| | — ambiances partageables : un fichier `.sindry-ambiance` documenté (`docs/FORMAT_AMBIANCE.md`, exemple « Forêt de nuit »), couleurs et décor libre, décor isolé, contrastes vérifiés, import, export, retrait (§15.7) | Fait |
| | — texte atténué du journal de Braise sombre, Ardente et Crépuscule, porté à 4,6:1 (3,5:1) : trouvé en passant les ambiances livrées au validateur | Fait |
| | — ligne de commande complète, `sindry.com` : actions, lots, analyse, réglages, catalogues, ambiances ; `--json`, codes de sortie, Ctrl+C selon le §6.3 ; ce qui est coché d'office tenu d'accord avec l'interface par un fichier de cas commun (§19, `docs/LIGNE_DE_COMMANDE.md`) | Fait |
| | — installation silencieuse avec options : langue, thème, ambiance, sans bienvenue, configuration, `PATH` — appliquées au premier lancement de chaque compte (§19.3) | Fait |
| | — nouvelle version de l'icône : goutte de flammes et engrenage gris, alpha nettoyé (voile du fond retiré, corps opaque), dessin inchangé | Fait |
| **2.0.1** | Corrections et ambiances en fichiers — pré-version | En cours |
| | — un lot dont tout est déjà en place ne reste plus bloqué à l'analyse ; raccourcis recréés après la mise à jour depuis la 1.8 ; anciens scripts livrés effacés de Program Files | Fait |
| | — Réglages en cinq sections : Général, Apparence, Catalogues, Actions, Avancé (l'historique y entre) (§8) | Fait |
| | — toute ambiance est un fichier ; trois de base (Plan, Forêt de nuit, Aurore australe), exportables, retirables, réinstallables ; les sept autres publiées à part (`ambiances/depot/`) ; qui en utilisait une la garde ; la forme de l'interface en listes fermées (§15.4, §15.7) | Fait |
| | — section Catalogues en tableau : quatre styles proposés (maquette M24) | Choisi : M29 (2.0.2) |
| **2.0.2** | Progression par ambiance, catalogues en tableau — pré-version | En cours |
| | — la montée d'eau de l'étape 3 remplacée par le graphique de progression de chaque ambiance, dans un cadre isolé ; la barre de Sindry par défaut ; « Entretien terminé » à la fin (M26, M28) | Fait |
| | — les ambiances livrées et restées intactes suivent la version de Sindry ; Plan 1.1, plus discrète (M25) | Fait |
| | — « Entretien complet » suit l'ordre des lots, puis celui de chaque lot (il suivait l'ordre des identifiants) | Fait |
| | — migration depuis WinTool : les sous-dossiers se fusionnent, la clé de l'ancien installeur s'efface | Fait |
| | — Catalogues : barre de commandes, tableau, détail et liste des actions en bas, clic droit (M29) | En cours |
| après la 2.0 | Ambiance Star Wars, style Nouvelle République — une fois que la communauté peut construire et partager les siennes | Plus tard |
| en continu | Améliorations graphiques et d'organisation | Glissées dans chaque version |

## Pourquoi cet ordre

**La mise à jour passe en premier**, alors que l'analyse est la priorité fonctionnelle.
La 1.0.0 installée ne sait pas se mettre à jour : chaque version publiée avant l'updater
oblige chaque utilisateur à retélécharger l'installeur à la main. Livrer l'updater en 1.1,
c'est faire de la 1.1 la **dernière** installation manuelle.

**Le contrat d'analyse est écrit avant son interface.** Il change la façon dont les
scripts sont écrits, et c'est un autre agent qui les écrit. Le publier tôt — accepté par le
validateur dès la 1.1 — lui laisse le temps d'adapter le catalogue pendant que l'interface
se construit.

**Catégories et lots viennent après l'updater** parce qu'ils imposent une migration des
réglages : les utilisateurs de la 1.0.0 ont déjà composé leurs lots, et une migration se
livre mieux automatiquement.

**Le catalogue est passé devant l'analyse** (1.2, d'abord prévu en 1.4). Tant que les
scripts voyageaient dans l'installeur, la séparation entre Sindry et ses scripts — sur
laquelle repose la licence — restait difficile à défendre. Elle devait devenir réelle avant
toute nouvelle version.

**L'ordre de publication de la 1.2 est imposé.** Une installation qui se met à jour en 1.2
perd les scripts que l'installeur posait : le catalogue officiel doit donc être **publié
avant** Sindry 1.2, sans quoi la mise à jour automatique viderait les installations
existantes. Et la clé publique du catalogue est compilée dans Sindry : elle doit exister
avant la construction de la 1.2.

**Les sources tierces sont venues ensuite** (1.4) : c'est la surface de sécurité la plus
large — du PowerShell téléchargé auprès de n'importe qui, exécuté en administrateur. Elles
reprennent le chemin éprouvé par la source officielle (index signé, vérifié avant d'être
lu), avec une décision en plus : qui les ajoute, et avec quelle clé (§16.2).

## Ce qui attend une décision

- **Le catalogue officiel et la norme d'analyse.** Sindry 1.4 sait analyser ; les scripts
  du catalogue, eux, doivent l'adopter un par un — c'est le travail de l'agent qui les
  écrit, avec `docs/FORMAT_SCRIPT.md` pour modèle. Tant qu'un script ne déclare pas
  `scan : true`, il s'applique tel quel, comme en 1.2.

## Idées pour plus tard

- **Exceptions par script aux emplacements protégés.** La liste reste la même pour tous,
  mais tel script — « vérifier les fichiers de Windows » — a le droit de recevoir
  `C:\Windows`, sans ouvrir la porte aux autres. Même rangement que la liste (`HKLM`,
  administrateur), rattaché à l'identifiant du script.
- **Témoin des changements sensibles — fait en 1.5 (§12.5).** Aucun bac à sable n'est
  possible pour un script administrateur (§12) : Sindry ne peut pas l'*empêcher* de
  toucher aux tâches planifiées ou aux variables d'environnement. Il peut en revanche
  **relever, avant et après chaque script**, ce qu'un logiciel malveillant modifie pour
  s'installer durablement — tâches planifiées, services, clés de démarrage automatique,
  variables d'environnement du système, exclusions de l'antivirus, fichier `hosts`,
  certificats racine, règles du pare-feu — et **le dire dans le bilan** : « ce script a
  créé la tâche planifiée X ». Presque tout se lit dans `HKLM` ou sur le disque, sans
  PowerShell, donc vite. En simulation, tout changement relevé trahirait un script qui ne
  simule pas vraiment. Les points d'attention de l'écran d'approbation (§12.2)
  s'étendraient aux mêmes gestes (`Register-ScheduledTask`,
  `Add-MpPreference -ExclusionPath`…).
- **Signer l'exécutable.** C'est ce qui fait taire SmartScreen avec le temps et réduit les
  faux positifs. Ni Microsoft (Azure Artifact Signing) ni les certificats « open source »
  ne s'offrent simplement à un particulier français au moment d'écrire : à trancher
  (statut d'indépendant, certificat OV, signature hors de GitHub Actions).
- **Faux positifs des antivirus.** Soumettre chaque version à l'analyse de Microsoft ;
  envisager `RemoteSigned` comme politique par défaut, `Bypass` étant un signal que les
  antivirus d'entreprise surveillent. Le script d'export des données de navigateur
  ressemble, par construction, à un voleur d'identifiants : le plus exposé.
- **Signaler à Tauri** que son modèle NSIS efface les données de l'interface avec
  `RmDir /r`, qui suit les jonctions, depuis un désinstalleur administrateur.
