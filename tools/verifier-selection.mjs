// Verifie que l'interface coche d'office ce que tools/selection-cas.json
// attend (2.0).
//
// Ce que Sindry coche apres une analyse est calcule deux fois : par
// l'interface (src/analyse.js : selection, configAnalyse, aFaire) et par la
// ligne de commande (analyse::selection_office, en Rust), qui lance ce que le
// mode Simple lancerait sans qu'on decoche rien. Deux calculs qui divergent en
// silence, c'est une action lancee d'un cote avec d'autres elements que de
// l'autre. Ce fichier de cas est leur contrat : ce script y passe le
// JavaScript, le test `la_selection_d_office_est_celle_de_l_interface` le Rust.
//
// Usage : node tools/verifier-selection.mjs            verifier
//         node tools/verifier-selection.mjs --ecrire   recalculer « attendu »
//         (a relire a la main avant de le garder : c'est la reference)

import { readFileSync, writeFileSync } from 'node:fs';

const racine = new URL('..', import.meta.url);
const chemin = new URL('tools/selection-cas.json', racine);
const A = await import(new URL('src/analyse.js', racine));
const doc = JSON.parse(readFileSync(chemin, 'utf8'));
const ecrire = process.argv.includes('--ecrire');

let erreurs = 0;
doc.cas.forEach((c, i) => {
  // Une entree minimale, comme celle de la decouverte : seules les options comptent.
  const entree = { id: `cas-${i}-${c.nom}`, meta: { options: c.options.map((o) => ({ choices: [], scan: false, keep_one: false, ...o })) } };
  A.oublier(entree.id);
  const m = A.modele(entree, c.analyse, true);
  const obtenu = { config: A.configAnalyse(m), a_faire: A.aFaire(m) };
  if (ecrire) {
    c.attendu = obtenu;
    return;
  }
  if (JSON.stringify(obtenu) !== JSON.stringify(c.attendu)) {
    erreurs += 1;
    console.log(`::error title=Selection::cas « ${c.nom} » : obtenu ${JSON.stringify(obtenu)}, attendu ${JSON.stringify(c.attendu)}`);
  }
});

if (ecrire) {
  writeFileSync(chemin, `${JSON.stringify(doc, null, 2)}\n`);
  console.log(`${doc.cas.length} cas recalcules : relisez « attendu » avant de les garder.`);
} else if (erreurs) {
  process.exit(1);
} else {
  console.log(`Selection d'office : ${doc.cas.length} cas, conformes.`);
}
