// The « Nouveautés » of Sindry (2.4.0): the notes of each version, as the user
// reads them, shown in the application — after an update, once, and from the
// help menu.
//
//   docs/notes/<version>.md   written by hand, one file per published version
//   src/nouveautes.json       generated: the versions of the current major
//                             (2.0.2 to today for 2.x), newest first, in HTML
//
// A version without its notes fails here, in CI: what nothing checks dies
// (the lesson of the doc that drifted from the code).
//
//   node tools/build-nouveautes.mjs           write src/nouveautes.json
//   node tools/build-nouveautes.mjs --check   fail if it is out of date

import { readFileSync, writeFileSync, readdirSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { render } from './markdown.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const read = (p) => readFileSync(join(ROOT, p), 'utf8').replace(/^﻿/, '').replace(/\r\n/g, '\n');
const SORTIE = 'src/nouveautes.json';

const version = JSON.parse(read('package.json')).version;
const majeure = version.split('.')[0];
const partie = (v) => v.split('.').map(Number);
const plusRecente = (a, b) => {
  const [x, y] = [partie(a), partie(b)];
  for (let i = 0; i < 3; i++) if (x[i] !== y[i]) return y[i] - x[i];
  return 0;
};

const versions = readdirSync(join(ROOT, 'docs/notes'))
  .map((f) => f.match(/^(\d+\.\d+\.\d+)\.md$/)?.[1])
  .filter((v) => v && v.split('.')[0] === majeure)
  .sort(plusRecente);

if (!versions.includes(version)) {
  console.log(`::error::docs/notes/${version}.md manque : chaque version publiee a ses notes, que Sindry montre apres la mise a jour.`);
  process.exit(1);
}

const contenu = {
  format: 1,
  majeure: Number(majeure),
  versions: versions.map((v) => {
    // The « Nouveautés » heading of each file is the title of the whole window.
    const md = read(`docs/notes/${v}.md`).replace(/^## Nouveautés\s*\n/, '');
    return { version: v, html: render(md, { ids: new Set(), toc: [] }) };
  }),
};
const texte = `${JSON.stringify(contenu, null, 1)}\n`;

if (process.argv.includes('--check')) {
  if (!existsSync(join(ROOT, SORTIE)) || read(SORTIE) !== texte) {
    console.log(`::error::${SORTIE} n'est plus a jour : lancez node tools/build-nouveautes.mjs, puis commitez.`);
    process.exit(1);
  }
  console.log(`Nouveautes a jour : ${versions.length} versions de la ${majeure}.x.`);
} else {
  writeFileSync(join(ROOT, SORTIE), texte, 'utf8');
  console.log(`${SORTIE} : ${versions.length} versions (${versions[0]} a ${versions.at(-1)}).`);
}
