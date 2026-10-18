## 7. La vérification finale

Avant de livrer, vérifie chaque point. Un « non » se corrige avant la livraison.

### Forme

- [ ] Le fichier commence par `## SINDRY:START` et chaque bloc se ferme par `## SINDRY:END`.
- [ ] Les quatorze champs obligatoires de l'entête sont présents, une seule fois chacun.
- [ ] `id` est un GUID neuf ; `category` vient de la section 5.1 ; `icon` de la section 5.4.
- [ ] Les dates, s'il y en a, sont au format `AAAA-MM-JJ`, `updated` ≥ `created`, aucune
      dans le futur.

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
- [ ] Commentaires en français, code et sortie en anglais ; drapeaux regroupés en tête.
- [ ] Le fichier est à enregistrer en **UTF-8 avec BOM** — rappelle-le à l'utilisateur.
