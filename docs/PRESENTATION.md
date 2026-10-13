# Modifier la présentation et les guides de Sindry

La présentation — les pages qui s'ouvrent au premier démarrage, et que l'on revoit depuis
**Réglages → Général → Présentation de Sindry** ou le bouton **?** — lit tout son contenu dans
[`src/presentation.json`](../src/presentation.json). Titres, phrases, icônes, illustrations,
ordre et nombre de pages : tout se change dans ce fichier, sans toucher au code.

Les **guides** (1.7) — les visites à la lampe torche et les alertes comme celle des droits
administrateur — lisent le leur dans [`src/guide.json`](../src/guide.json) : voir
« Les guides », plus bas.

## Le circuit

1. Modifier `src/presentation.json`.
2. Vérifier : `node tools/verifier-presentation.mjs`. Il dit ce qui ne va pas, et où
   (`pages[2] (actions).essentiel[1] : texte « en » vide`). La CI le lance aussi : une
   erreur bloque la construction plutôt que d'afficher une page blanche.
3. Regarder le résultat dans le banc d'essai, page `http://localhost:8123/?onboarding`
   (voir `tools/bench/serve.mjs`), ou dans l'application.

Le fichier est **embarqué dans l'exécutable** : modifié après l'installation, il serait
ignoré. C'est voulu. Un fichier modifiable sur le disque laisserait un programme malveillant
faire dire à Sindry, dans une page de confiance, « désactivez votre antivirus ».

## Le fichier

```json
{
  "format": 1,
  "pages": [ … ]
}
```

`format` vaut 1. Les pages s'affichent dans l'ordre de la liste.

### Les textes

Tout texte existe dans les deux langues :

```json
{ "fr": "Le mode Simple : un bouton, et c’est fait.", "en": "Simple mode: one button, and it’s done." }
```

Les apostrophes typographiques (’) et les guillemets français (« ») passent tels quels.

### La page de bienvenue

Facultative ; si elle existe, c'est la première. C'est elle qui fait régler Sindry à son
goût : la langue, l'ambiance, clair ou sombre, la taille (1.7). Relancée depuis les Réglages,
la présentation commence après elle.

```json
{
  "id": "bienvenue",
  "type": "bienvenue",
  "titre": { "fr": "…", "en": "…" },
  "texte": { "fr": "…", "en": "…" }
}
```

### La page « prêt »

Facultative ; si elle existe, c'est la dernière (1.7). Elle propose la visite de l'accueil
(`accueil` dans `src/guide.json`) : « Me faire visiter » la lance, « Je découvre seul » ferme
la présentation. `icone` : une icône Lucide, `map` par défaut.

```json
{
  "id": "pret",
  "type": "pret",
  "icone": "map",
  "titre": { "fr": "…", "en": "…" },
  "texte": { "fr": "…", "en": "…" }
}
```

### Une page ordinaire

```json
{
  "id": "securite",
  "illustration": "securite",
  "titre": { "fr": "Votre sécurité d’abord", "en": "Your safety first" },
  "essentiel": [
    { "icone": "eye", "texte": { "fr": "…", "en": "…" } }
  ],
  "detail": [
    { "fr": "Un paragraphe, replié derrière « En savoir plus ».", "en": "…" }
  ]
}
```

| Champ | Rôle |
|---|---|
| `id` | nom court, unique : minuscules, chiffres, tirets |
| `titre` | le titre de la page |
| `illustration` | `principe`, `catalogue`, `securite` (les trois illustrations animées), `icone`, ou `aucune` |
| `icone` | avec `"illustration": "icone"` : une grande icône à la place d'une illustration |
| `essentiel` | ce qui se voit d'emblée : **cinq phrases au plus**, une icône chacune |
| `detail` | les paragraphes repliés derrière « En savoir plus » (facultatif) |
| `action` | facultatif : un bouton sur la page (voir plus bas) |

Les icônes sont celles de [Lucide](https://lucide.dev/icons) : on écrit leur nom tel que le
site l'affiche (`shield-check`, `mouse-pointer-click`…). Le contrôleur refuse un nom inconnu.

**Lisible à 9 ans comme à 80** (spécification §13) : dans `essentiel`, une idée par phrase,
pas de terme technique. Le détail peut en dire plus, toujours sans jargon.

### Le bouton du catalogue

Une page peut proposer d'installer le catalogue officiel. Une fois installé, le bouton
laisse place à la phrase `installe`, où `{n}` est le nombre d'actions et `{s}` le pluriel.

```json
"action": {
  "type": "catalogue",
  "installer": { "fr": "Installer le catalogue officiel", "en": "Install the official catalogue" },
  "plus_tard": { "fr": "ou plus tard, depuis les Réglages", "en": "or later, from Settings" },
  "installe": { "fr": "Catalogue installé : {n} action{s} prête{s}.", "en": "Catalogue installed: {n} action{s} ready." }
}
```

## Ce qui reste dans le code

Les boutons de navigation (« Suivant », « Retour », « Passer la présentation »…), les libellés
« Langue » et « Apparence » de la page de bienvenue, et le contenu des trois illustrations
animées. Les boutons sont dans `src/i18n.js` (clés `onb.*`) ; une nouvelle illustration
animée demande du code (`illustrationVisite` dans `src/main.js`, styles dans
`src/styles.css`).

## Les guides

[`src/guide.json`](../src/guide.json) contient deux listes. Même circuit que la présentation :
modifier, puis `node tools/verifier-guide.mjs` (la CI le lance aussi), puis regarder dans le
banc d'essai — bouton **?** de la barre de titre.

```json
{
  "format": 1,
  "visites": [
    {
      "id": "accueil",
      "ecran": "simple",
      "titre": { "fr": "Visite de l’accueil", "en": "…" },
      "resume": { "fr": "6 étapes, une minute", "en": "…" },
      "etapes": [
        { "cible": "mode", "titre": { "fr": "…", "en": "…" }, "texte": { "fr": "…", "en": "…" } }
      ]
    }
  ],
  "alertes": [ { "id": "droits", "cible": "droits", "…": "…" } ]
}
```

- **Une visite** apparaît dans le menu **?**, dans l'ordre du fichier. `ecran` dit où elle se
  passe (`simple` : l'accueil du mode Simple ; `expert`) — Sindry y bascule avant de commencer.
  Dix étapes au plus. La visite `accueil` est obligatoire : la page « prêt » la propose.
- **Une étape** éclaire sa `cible` et affiche son titre et son texte. Une étape dont l'élément
  n'est pas à l'écran (le champ de bouées sans aucun lot, par exemple) est sautée.
- **Une cible** est le nom d'un attribut `data-guide="…"` de l'interface : `mode`,
  `recommande`, `bouees`, `simulation`, `journal`, `aide`, `droits`, `recherche`, `onglets`,
  `fiche`, `journal-apercu`. Le vérificateur donne la liste à jour. Pour viser un nouvel
  élément, lui ajouter un `data-guide` dans `src/index.html` (ou dans le gabarit de
  `src/main.js` qui le dessine) — jamais une classe CSS : elle changera au prochain remaniement.
- **L'alerte `droits`** porte les textes des encarts (fiche d'une action, étape « Vérifier »)
  et de la bulle qui éclaire « Droits limités ». Ses boutons (« Plus tard », « Relancer en
  administrateur ») restent dans `src/i18n.js`.

Les boutons de navigation des visites (« Suivant », « Précédent », « Passer », « Terminer »)
et du menu **?** sont dans `src/i18n.js` (clés `guide.*`, `aide.*`).
