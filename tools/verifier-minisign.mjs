// The minisign check of the site (tools/minisign.mjs, 2.2.0) accepts what
// catalogue.rs accepts, and refuses what it refuses. Same fixtures as the Rust
// tests: indexes signed with a throwaway key, never the real one.
//
//   node tools/verifier-minisign.mjs

import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { verifyMinisign } from './minisign.mjs';

const F = join(dirname(fileURLToPath(import.meta.url)), '..', 'src-tauri', 'fixtures', 'catalogue');
const lire = (f) => readFileSync(join(F, f));
const cle = lire('cle-essai.pub').toString('utf8');
const index = lire('index-1.0.0.json');
const sig = lire('index-1.0.0.json.sig').toString('utf8');

/** The signature file with its trusted comment changed, wrapped as tauri writes it. */
function commentaireChange() {
  const texte = Buffer.from(sig.trim(), 'base64').toString('utf8').replace(/file:index-1\.0\.0\.json/, 'file:index-9.9.9.json');
  return Buffer.from(texte, 'utf8').toString('base64');
}

const CAS = [
  ['signature valide', () => verifyMinisign(cle, index, sig), null],
  ['autre cle', () => verifyMinisign(lire('cle-autre.pub').toString('utf8'), index, sig), /autre cle/],
  ['index modifie', () => verifyMinisign(cle, Buffer.concat([index, Buffer.from(' ')]), sig), /signature invalide/],
  ['signature d\'un autre index', () => verifyMinisign(cle, index, lire('index-0.9.0.json.sig').toString('utf8')), /signature invalide/],
  ['commentaire de confiance modifie', () => verifyMinisign(cle, index, commentaireChange()), /commentaire de confiance/],
  ['signature illisible', () => verifyMinisign(cle, index, 'pas une signature'), /illisible/],
];

let echecs = 0;
for (const [nom, essai, attendu] of CAS) {
  let erreur = null;
  try { essai(); } catch (e) { erreur = e.message; }
  const ok = attendu ? erreur !== null && attendu.test(erreur) : erreur === null;
  if (!ok) {
    echecs++;
    console.log(`::error::minisign, ${nom} : attendu ${attendu ? `un refus ${attendu}` : 'un accord'}, obtenu ${erreur ? `« ${erreur} »` : 'un accord'}`);
  }
}
if (echecs) process.exit(1);
console.log(`Verification minisign du site : ${CAS.length} cas conformes.`);
