# Créer une ambiance Sindry

Une ambiance habille Sindry : ses couleurs, en clair, en sombre ou les deux, l'allure de
l'interface (sa forme), et un décor derrière la houle de l'accueil. Elle tient dans **un seul fichier** `.sindry-ambiance`, qui
se partage comme une image : on l'envoie, on l'importe, c'est fait.

Tout ce qui est décrit ici est **vérifié par Sindry** à l'import, à chaque lecture du dossier
des ambiances, et par la ligne de commande. Un fichier qui ne respecte pas une règle est
refusé avec **la liste complète** de ce qui ne va pas — vous corrigez tout en une fois.

Référence : `docs/SPECIFICATION.md` §15.7. **Exemples complets** : les ambiances que Sindry
livre sont elles-mêmes des fichiers de ce format, vérifiés à chaque compilation —
[`ambiances/base/`](../ambiances/base) (Plan, Forêt de nuit, Aurore australe) et
[`ambiances/depot/`](../ambiances/depot) (Braise, Synthwave, Spatial, Terminal, Aurore
boréale).

---

## En deux minutes

1. **Partez d'un fichier qui marche.** Copiez une ambiance des exemples ci-dessus, ou, dans
   Sindry, choisissez-en une (Réglages → 2 « Apparence ») et cliquez **Exporter** :
   l'Explorateur s'ouvre sur son fichier. Copiez-le avant de le modifier.
2. Changez `id` (minuscules, chiffres, tirets) et `nom`.
3. Changez les couleurs, ajoutez un décor si vous voulez.
4. **Vérifiez** : `sindry ambiance verifier mon-ambiance.sindry-ambiance`, ou importez-la —
   Sindry dit tout ce qui ne va pas.
5. Partagez le fichier.

Ouvrez-le avec n'importe quel éditeur de texte, et enregistrez-le en **UTF-8**.

---

## Le fichier

Du JSON. Les clés qui commencent par `_` sont ignorées partout : le JSON n'a pas de
commentaires, elles en tiennent lieu (`"_note": "à revoir"`). Toute autre clé inconnue est
refusée — une faute de frappe se voit, avec la bonne clé suggérée.

```json
{
  "format": 1,
  "id": "foret-de-nuit",
  "nom": { "fr": "Forêt de nuit", "en": "Night forest" },
  "description": { "fr": "Des sapins sous la lune.", "en": "Pines under the moon." },
  "auteur": "Camille",
  "version": "1.0",
  "police": "normale",
  "clair":  { "bg": "#F3F6F1", "…": "…" },
  "sombre": { "bg": "#07110C", "…": "…" },
  "decor":  { "balisage": "<svg …>…</svg>", "css": "…" }
}
```

| Clé | | Ce qu'elle contient |
|---|---|---|
| `format` | obligatoire | `1` |
| `id` | obligatoire | Minuscules, chiffres et tirets, 40 caractères au plus. C'est lui qui identifie l'ambiance : un fichier du même `id` la **remplace** (Sindry demande avant) — même une ambiance de base. |
| `nom` | obligatoire | 40 caractères au plus. Un texte, ou `{ "fr": …, "en": … }` ; une seule langue sert pour les deux. |
| `description` | facultatif | 200 caractères au plus, même forme que `nom`. |
| `auteur` | facultatif | 60 caractères au plus. |
| `version` | facultatif | 20 caractères au plus. Montrée quand une ambiance en remplace une autre. |
| `police` | facultatif | `normale` (par défaut) ou `chasse-fixe` : toute l'interface dans la police du journal, comme l'ambiance Terminal. |
| `forme` | facultatif | L'allure de l'interface (plus bas). |
| `clair`, `sombre` | **l'un au moins** | Les couleurs de chaque mode (ci-dessous). Avec un seul des deux, l'ambiance n'existe que dans ce mode : le bouton clair / sombre le dit et ne fait rien. |
| `decor` | facultatif | Le décor (plus bas). |
| `progression` | facultatif | Le graphique qui montre où en est un entretien (plus bas). Sans lui, la barre de Sindry. |

---

## Les couleurs

Chaque mode donne **toutes** les clés ci-dessous : ce sont les jetons de l'interface. Une
couleur s'écrit `#RRGGBB` (ou `#RGB`), **sans transparence** — le contraste ne se mesure
que sur des couleurs pleines.

| Jeton | Où il sert |
|---|---|
| `bg` | Le fond de la fenêtre |
| `surface` | Les cartes, les panneaux, les listes |
| `surface-2` | Une surface en retrait : en-têtes, champs, survol |
| `surface-3` | Une surface plus marquée : pastilles, pistes |
| `surface-soft` | Le fond d'une ligne de réglage — **opaque**, jamais translucide |
| `border` | Les filets |
| `border-2` | Les filets marqués, les contours au survol |
| `text` | Le texte |
| `text-2` | Le texte secondaire : descriptions, légendes |
| `text-3` | Le texte tertiaire : numéros, indications |
| `acc` | L'accent : boutons principaux, sélection, ce qui agit |
| `acc-2` | Le second accent : dégradés, reflets, décors |
| `acc-on` | Le texte couleur d'accent posé sur le fond |
| `acc-soft` | Le fond d'un élément sélectionné — **opaque** |
| `ink` | Le texte posé **sur** `acc` et `acc-2` (l'étiquette d'un bouton principal) |
| `ok` | Réussite |
| `warn` | Avertissement |
| `err` | Erreur |
| `ok-s` | Le fond d'un message de réussite |
| `warn-s` | Le fond d'un avertissement |
| `err-s` | Le fond d'une erreur |
| `knob-off` | La pastille d'un interrupteur éteint |
| `serie-1` | Les graphiques de l'analyse : la première série… |
| `serie-2` | … la deuxième… |
| `serie-3` | … |
| `serie-4` | … |
| `serie-5` | … |
| `serie-6` | … la sixième |
| `rouage-a` | Le contour en engrenage des graphiques, d'une teinte… |
| `rouage-b` | … à l'autre |
| `eau-a` | La houle de l'accueil, d'une teinte… |
| `eau-b` | … à l'autre |
| `eau-fond` | L'opacité de la nappe du fond, de `0` (invisible) à `1` — un **nombre**, pas un texte |
| `eau-surface` | L'opacité de la nappe de surface, de `0` à `1` |
| `term-bg` | Le fond du journal technique |
| `term-text` | Le texte du journal |
| `term-dim` | Le texte atténué du journal (heures, détails) |

`eau-fond` et `eau-surface` à `0` effacent la houle : les bouées restent, posées sur une
ligne invisible. C'est le choix des ambiances Terminal.

### Ce qui doit se lire

Sindry mesure le contraste de chaque texte sur chacun de ses fonds, comme pour les
ambiances livrées : **4,5:1 au moins** partout (le seuil du texte courant du WCAG). En
dessous, le fichier est refusé, et le refus dit quelle paire, de combien, et ce qu'il faut.

| Texte | Sur |
|---|---|
| `text` | `bg`, `surface`, `surface-2`, `surface-soft`, `acc-soft` |
| `text-2` | `bg`, `surface`, `surface-2` |
| `text-3` | `bg`, `surface` |
| `acc-on` | `bg`, `surface`, `acc-soft` |
| `ok`, `warn`, `err` | `bg`, `surface`, et leur fond (`ok-s`, `warn-s`, `err-s`) |
| `ink` | `acc`, `acc-2` |
| `term-text`, `term-dim` | `term-bg` |
| Les marqueurs du journal (bleu, turquoise, vert, ambre, rouge — communs à toutes les ambiances) | `term-bg` |

Ce n'est pas une préférence de style : une ambiance illisible ferait manquer un
avertissement à quelqu'un qui s'apprête à lancer une action en administrateur.

---

## La forme

Au-delà des couleurs, une ambiance peut changer l'allure de l'interface. Elle ne le fait
**jamais en écrivant du CSS pour l'interface** — un fichier venu de n'importe où pourrait
masquer un avertissement ou maquiller un bouton —, mais en choisissant, réglage par
réglage, dans des **listes fermées** ; Sindry traduit ces choix en règles qu'il écrit
lui-même. Tout est facultatif : ce qui n'est pas dit garde la première valeur.

```json
"forme": { "bouees": "losanges", "titres": "espaces", "coins": "droits", "ligne_eau": "points", "equerres": true, "lueur": true }
```

| Réglage | Valeurs | |
|---|---|---|
| `bouees` | `rondes`, `arrondies`, `losanges`, `neon`, `pointillees`, `verre` | Les bouées de l'accueil et leurs étiquettes : cartouches (Plan), balises de carte d'un trait froid — `rouage-a` — (Spatial), néon (Synthwave), au trait pointillé (Terminal), verre dépoli (Aurore) |
| `titres` | `normaux`, `nets`, `neon`, `espaces`, `invite` | Le titre de l'accueil, ceux des bouées, la carte recommandée : serré et gras, en italique lumineux, en capitales fines et espacées, en invite de commande avec un curseur |
| `coins` | `doux`, `serres`, `droits`, `aucun` | Les arrondis des cartes, boutons et pastilles |
| `ligne_eau` | `vague`, `trait`, `neon`, `pointilles`, `points` | La ligne qui court sur la houle |
| `chiffres` | `normaux`, `chasse-fixe` | Les comptes des bouées, numéros et chemins, dans la police du journal |
| `expert` | `normal`, `plan` | L'Expert sur une grille fine (couleur `surface-3`), la sélection d'un trait |
| `equerres` | `true`, `false` | Des équerres autour de la carte choisie |
| `lueur` | `true`, `false` | Une lueur d'accent sous les boutons principaux |
| `balayage` | `true`, `false` | Des lignes de balayage sur tout l'écran et la lueur du phosphore sur les textes |

Avec `"police": "chasse-fixe"`, toute l'interface passe dans la police du journal.

---

## Le décor

```json
"decor": {
  "balisage": "<svg viewBox=\"0 0 1600 760\" preserveAspectRatio=\"xMidYMax slice\">…</svg>",
  "css": "svg { position: absolute; inset: 0; width: 100%; height: 100%; } …"
}
```

- **`balisage`** : du SVG ou du HTML — formes, dégradés, images embarquées, texte.
- **`css`** : sa feuille de style. Les animations CSS sont permises.

**Où il s'affiche.** À l'accueil du mode Simple, **derrière la houle et les bouées**, sur toute la largeur de la fenêtre et 760 px de
haut, ancré en bas. Il suit la marée quand le champ de bouées défile. Le mode Expert n'a pas
de décor : on y lit des listes et des journaux.

Pour un dessin qui tient à toutes les largeurs : un `viewBox` de 1600 × 760 et
`preserveAspectRatio="xMidYMax slice"` — le bas reste en place, les côtés se coupent.

**Les couleurs de l'ambiance y sont disponibles**, celles du mode affiché :
`var(--acc)`, `var(--surface-3)`, `var(--eau-fond)`… toutes les clés de la table, avec `--`
devant. Et la racine porte `data-mode="clair"` ou `data-mode="sombre"` :

```css
.lune { fill: var(--acc-2); }
[data-mode="clair"] .luciole { display: none; }
```

Quand l'utilisateur passe du clair au sombre, le décor est redessiné avec les autres
couleurs. Quand Windows demande moins d'animations, Sindry les arrête lui-même : rien à
prévoir.

### Un décor isolé

Un décor vient de n'importe qui. Sindry l'affiche dans un **cadre isolé** : aucun script ne
s'y exécute, rien ne s'y charge depuis le disque ni le réseau, il ne reçoit aucun clic et ne
voit rien du reste de l'interface. Sindry vérifie en plus le fichier, pour vous dire tout de
suite ce qui ne s'afficherait pas :

- **Balises refusées** : `script`, `iframe`, `frame`, `object`, `embed`, `link`, `meta`,
  `base`, `form` et ses champs, `foreignObject`, `audio`, `video`, `style` (le style va dans
  `css`), `a`, `template`, et la structure du document (`html`, `head`, `body`, `title`).
- **Aucun gestionnaire d'événement** (`onload=`, `onclick=`…).
- **Liens** — `href`, `xlink:href`, `src`, `srcset` et `url()` : seulement une ancre du décor
  (`#degrade`) ou une image embarquée en base64, `data:image/png;base64,…` (ou `jpeg`,
  `gif`, `webp`). Pas de SVG en `data:`.
- **CSS** : ni `@import`, ni `image-set()`, ni échappement `\XX` — écrivez le caractère
  lui-même.
- Le décor pèse **512 Ko au plus**, le fichier **1 Mo**.

---

## La progression

Pendant un entretien, l'écran du mode Simple montre où en est le travail. Une ambiance qui
ne dit rien reçoit la barre de Sindry. Une ambiance peut aussi **dessiner son propre
graphique** — l'engrenage de Plan, la lune de Forêt de nuit, le voile d'Aurore australe —
dans une section `progression`, construite comme le décor :

```json
"progression": {
  "place": "pastille",
  "balisage": "<svg viewBox=\"0 0 220 220\"><circle class=\"piste\" cx=\"110\" cy=\"110\" r=\"70\"/><circle class=\"arc\" cx=\"110\" cy=\"110\" r=\"70\" pathLength=\"100\"/></svg><p class=\"pct\">{{pourcentage}}</p>",
  "css": ".arc { fill: none; stroke: var(--acc); stroke-width: 15; stroke-dasharray: calc(var(--pct) * 1px) 100px; } …"
}
```

| Clé | | |
|---|---|---|
| `place` | obligatoire | `pastille` : 88 × 88, dans l'en-tête de la carte de l'entretien, à gauche du bilan. `bandeau` : toute la largeur de la carte, 150 de haut, au-dessus du bilan. |
| `balisage` | obligatoire | SVG ou HTML, comme le décor. |
| `css` | facultatif | Sa feuille de style. |

**Ce que Sindry donne au graphique**

| | |
|---|---|
| `{{pourcentage}}` | Remplacé dans le balisage par le pourcentage, écrit comme il faut dans la langue de l'interface (« 50 % », « 50% »). **C'est la seule information à montrer** : le nombre d'actions et l'état sont déjà écrits à côté. |
| `--p`, `--pct` | La progression, de 0 à 1 et de 0 à 100 — des nombres, à utiliser dans des `calc()`. Elles **glissent d'elles-mêmes** de l'ancienne valeur à la nouvelle, en 0,8 s : rien à animer. |
| `--depuis` | Les secondes écoulées depuis le début de l'entretien. Sindry recompose le cadre à chaque action terminée ; une animation en boucle continue sans à-coup avec `animation-delay: calc(var(--depuis) * -1s)`. |
| `data-etat` | Sur la racine : `en-cours`, `fini` ou `arrete`. Pour arrêter une roue à la fin, par exemple : `:root[data-etat="fini"] .roue { animation-play-state: paused; }`. |
| `data-mode`, les couleurs | Comme pour le décor : `data-mode="clair"` ou `"sombre"`, et toutes les couleurs du mode affiché. |

**Exemples complets** : chaque ambiance livrée a la sienne — Plan (un engrenage autour
d'une jauge, en `pastille`), Forêt de nuit (une lune qui se remplit), Aurore australe (un
voile qui gagne le ciel), et dans le dépôt des ambiances : Braise (la jauge à dents),
Synthwave (un soleil qui se lève), Spatial (une orbite), Terminal (une barre en
caractères).

**Les règles du décor, à l'identique** : le même cadre isolé, sans script ni réseau ; les
mêmes contrôles, sous les mêmes codes, dont le message commence par « progression » ;
**256 Ko au plus**. Les polices de l'interface n'y sont pas chargées : écrivez
`Consolas, monospace` ou `'Segoe UI', system-ui, sans-serif`. Quand Windows demande moins
d'animations, Sindry les arrête lui-même : la progression avance, sans glisser.

---

## Installer, partager, retirer

Les ambiances installées vivent dans **`%LOCALAPPDATA%\Sindry\ambiances\`**. Déposer un
fichier `.sindry-ambiance` dans ce dossier suffit : il apparaît dans le réglage 2.3 à la
prochaine ouverture des réglages. Un fichier refusé n'empêche pas les autres : le réglage
liste ceux qu'il ignore, et pourquoi.

Dans **Réglages → 2 « Apparence »**, sous les ambiances (2.4) :

- **Importer…** — choisit un fichier, le vérifie, l'installe et le choisit. Une ambiance du
  même `id` existe déjà ? Sindry montre les deux versions et demande avant de remplacer.
- **Exporter** — l'Explorateur s'ouvre sur le fichier de l'ambiance choisie : c'est lui
  qu'on partage, décor compris.
- **Retirer** — efface le fichier de l'ambiance choisie, après confirmation. Sindry revient
  à Plan.
- **Ouvrir le dossier**.
- **Réinstaller les ambiances de base** — remet Plan, Forêt de nuit et Aurore australe, si
  elles manquent. Une ambiance de base retirée ne revient que sur demande.

Sans aucune ambiance installée, Sindry garde l'allure de Plan, sans décor.

Une ambiance n'a aucun pouvoir sur le PC : elle ne s'exécute pas, et Sindry ne l'approuve
donc pas comme un script. Le dossier est inscriptible sans élévation ; un programme qui y
déposerait un fichier n'obtiendrait qu'un décor, isolé comme les autres.

---

## Les contrôles, un par un

| Code | Déclencheur |
|---|---|
| `TAILLE` | Le fichier dépasse 1 Mo |
| `JSON_INVALIDE` | Pas du JSON, ou pas un objet |
| `CLE_INCONNUE` | Une clé que le format ne connaît pas (et qui ne commence pas par `_`) — suggestion si proche |
| `FORMAT_INCONNU` | `format` absent, ou autre que `1` |
| `ID_INVALIDE` | `id` absent, ou hors minuscules, chiffres et tirets, ou plus de 40 caractères |
| `TEXTE_INVALIDE` | `nom` absent ou vide ; un texte trop long, ou qui contient un caractère de contrôle |
| `POLICE_INCONNUE` | `police` hors `normale` et `chasse-fixe` |
| `FORME_INCONNUE` | Un réglage de `forme` hors de sa liste, un interrupteur qui n'est pas `true` ou `false`, ou `forme` qui n'est pas un objet |
| `MODE_ABSENT` | Ni `clair` ni `sombre` |
| `MODE_INVALIDE` | `clair` ou `sombre` n'est pas un objet |
| `JETON_MANQUANT` | Une couleur ou une opacité manque dans un mode |
| `COULEUR_INVALIDE` | Une couleur qui n'est pas `#RRGGBB` ou `#RGB` |
| `NOMBRE_INVALIDE` | `eau-fond` ou `eau-surface` hors d'un nombre de 0 à 1 |
| `CONTRASTE` | Une paire de la table « Ce qui doit se lire » sous 4,5:1 |
| `DECOR_INVALIDE` | `decor` n'est pas un objet, ou `balisage` / `css` pas un texte |
| `DECOR_TROP_GRAND` | Le décor dépasse 512 Ko |
| `PROGRESSION_INVALIDE` | `progression` n'est pas un objet ; `place` hors `pastille` et `bandeau` ; `balisage` vide ; une valeur qui n'est pas un texte |
| `PROGRESSION_TROP_GRANDE` | La progression dépasse 256 Ko |
| `DECOR_BALISE` | Une balise refusée — dans le décor ou dans la progression |
| `DECOR_EVENEMENT` | Un gestionnaire d'événement — dans le décor ou dans la progression |
| `DECOR_LIEN` | Un lien, une source ou un `url()` qui ne vise ni une ancre ni une image embarquée ; une animation d'un attribut de lien |
| `DECOR_CSS` | Dans la feuille ou le balisage : `@import`, `image-set()`, `cross-fade()`, `element()`, `expression()`, `-moz-binding`, `behavior:`, `javascript:`, `vbscript:`, un échappement `\XX` ; `</` dans la feuille |

Et, à la lecture du dossier :

| Code | Déclencheur |
|---|---|
| `ID_DOUBLON` | Un fichier précédent (ordre alphabétique) porte déjà cet `id` |
| `ILLISIBLE` | Le fichier ne se lit pas (droits, encodage autre que UTF-8) |
| `TROP_NOMBREUSES` | Plus de 200 ambiances : les suivantes sont ignorées |
