// Verifie src/guide.json, le contenu du guide : visites et alertes (§13, §15.6,
// docs/PRESENTATION.md).
//
// Ce fichier est fait pour etre modifie a la main. Une faute de frappe ne doit
// pas finir en visite qui s'arrete au milieu : chaque regle ci-dessous echoue
// ici, en CI, avec un message qui dit quoi corriger et ou.
//
// La regle qui compte le plus : chaque etape vise un element par son attribut
// `data-guide`, et cet attribut doit exister dans l'interface (src/index.html,
// ou un gabarit de src/main.js). Sans cette verification, renommer un element
// casserait une visite sans que personne ne le voie — une convention que rien
// ne verifie finit toujours morte.
//
// Usage : node tools/verifier-guide.mjs

import { readFileSync } from 'node:fs';

const racine = new URL('..', import.meta.url);
const LANGUES = ['fr', 'en'];
const ECRANS = ['simple', 'expert'];
const MAX_ETAPES = 10;
const erreurs = [];
const err = (ou, message) => erreurs.push(`${ou} : ${message}`);

let doc;
try {
  doc = JSON.parse(readFileSync(new URL('src/guide.json', racine), 'utf8'));
} catch (e) {
  console.log(`::error title=Guide::src/guide.json illisible : ${e.message}`);
  process.exit(1);
}

// Les cibles connues : tous les `data-guide="…"` de l'interface.
const sources = ['src/index.html', 'src/main.js'].map((f) => readFileSync(new URL(f, racine), 'utf8')).join('\n');
const cibles = new Set([...sources.matchAll(/data-guide="([a-z0-9-]+)"/g)].map((m) => m[1]));

function texte(ou, valeur) {
  if (!valeur || typeof valeur !== 'object' || Array.isArray(valeur)) {
    return err(ou, 'attendu un objet { "fr": "…", "en": "…" }');
  }
  for (const l of LANGUES) {
    if (typeof valeur[l] !== 'string' || !valeur[l].trim()) err(ou, `texte « ${l} » manquant ou vide`);
  }
  for (const l of Object.keys(valeur)) {
    if (!LANGUES.includes(l)) err(ou, `langue « ${l} » inconnue (attendu : ${LANGUES.join(', ')})`);
  }
}

function cible(ou, nom) {
  if (typeof nom !== 'string' || !cibles.has(nom)) {
    err(ou, `cible « ${nom} » introuvable dans l'interface (connues : ${[...cibles].sort().join(', ')})`);
  }
}

const id = (ou, valeur, deja) => {
  if (typeof valeur !== 'string' || !/^[a-z0-9-]+$/.test(valeur)) return err(ou, '« id » : minuscules, chiffres et tirets');
  if (deja.has(valeur)) err(ou, `« id » « ${valeur} » deja utilise`);
  deja.add(valeur);
};

if (doc.format !== 1) err('format', `vaut ${JSON.stringify(doc.format)}, cette version de WinTool lit le format 1`);

const idsVisites = new Set();
if (!Array.isArray(doc.visites)) err('visites', 'attendu une liste');
(doc.visites || []).forEach((v, i) => {
  const ou = `visites[${i}]${v?.id ? ` (${v.id})` : ''}`;
  id(ou, v?.id, idsVisites);
  if (!ECRANS.includes(v?.ecran)) err(`${ou}.ecran`, `« ${v?.ecran} » inconnu (attendu : ${ECRANS.join(', ')})`);
  texte(`${ou}.titre`, v?.titre);
  texte(`${ou}.resume`, v?.resume);
  if (!Array.isArray(v?.etapes) || !v.etapes.length) return err(`${ou}.etapes`, 'au moins une etape est attendue');
  // Une visite se suit d'un trait : au-dela, on decroche.
  if (v.etapes.length > MAX_ETAPES) err(`${ou}.etapes`, `${v.etapes.length} etapes, ${MAX_ETAPES} au plus`);
  v.etapes.forEach((e, k) => {
    cible(`${ou}.etapes[${k}].cible`, e?.cible);
    texte(`${ou}.etapes[${k}].titre`, e?.titre);
    texte(`${ou}.etapes[${k}].texte`, e?.texte);
  });
});
// Deux visites que WinTool lance lui-meme : la bienvenue propose la premiere.
if (!idsVisites.has('accueil')) err('visites', 'la visite « accueil » est attendue : la bienvenue la propose');

const idsAlertes = new Set();
if (!Array.isArray(doc.alertes)) err('alertes', 'attendu une liste');
(doc.alertes || []).forEach((a, i) => {
  const ou = `alertes[${i}]${a?.id ? ` (${a.id})` : ''}`;
  id(ou, a?.id, idsAlertes);
  cible(`${ou}.cible`, a?.cible);
  for (const k of ['titre_aide', 'resume', 'encart_action', 'encart_lot', 'kick', 'titre', 'texte']) texte(`${ou}.${k}`, a?.[k]);
});
if (!idsAlertes.has('droits')) err('alertes', 'l\'alerte « droits » est attendue : les encarts des droits administrateur la lisent');

for (const e of erreurs) console.log(`::error title=Guide::${e}`);
if (erreurs.length) process.exit(1);
const etapes = (doc.visites || []).reduce((n, v) => n + (v.etapes?.length || 0), 0);
console.log(`Guide : ${doc.visites.length} visites (${etapes} etapes), ${doc.alertes.length} alerte(s), cibles presentes.`);
