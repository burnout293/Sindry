// Builds the two script-writing guides from their single sources (2.0.3):
//
//   docs/GUIDE_SCRIPTS.html   for people: the whole contract, searchable, with a
//                             skeleton generator and the AI version one click away
//   docs/SCRIPTS_POUR_IA.md   for an AI: mission, context, the rules digested,
//                             reference data, the whole contract, a final checklist
//   docs/SCRIPTS_POUR_IA_COURT.md
//                             the same for a model with a small context (2.0.4):
//                             the same rules, a choice of icons, and only the
//                             parts of the contract every script needs
//
// Sources: docs/FORMAT_SCRIPT.md (the contract), tools/categories.json,
// tools/lots.json, tools/lucide-icon-names.txt, and the hand-written parts in
// tools/script-guides/ (the icons of the short version among them). Nothing
// else: a rule changes in FORMAT_SCRIPT.md, and the guides follow at the next
// build. It also keeps the fingerprint of the rules in their changelog,
// docs/SCRIPTS_POUR_IA_NOUVEAUTES.md, written by hand (2.2.0, see below).
//
//   node tools/build-script-guides.mjs           write the three files, and the fingerprint
//   node tools/build-script-guides.mjs --check   fail if one is out of date (CI)
//   node tools/build-script-guides.mjs --sans-nouveaute
//                                                accept a change of the rules
//                                                without a changelog entry (a typo)

import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { render } from './markdown.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const read = (p) => readFileSync(join(ROOT, p), 'utf8').replace(/^﻿/, '').replace(/\r\n/g, '\n');

const version = JSON.parse(read('package.json')).version;
const contract = read('docs/FORMAT_SCRIPT.md');
const categories = JSON.parse(read('tools/categories.json'));
const lots = JSON.parse(read('tools/lots.json'));
const icons = read('tools/lucide-icon-names.txt').split('\n').map((l) => l.trim()).filter((l) => l && !l.startsWith('#'));

// ---------------------------------------------------------------------------
// The changelog of the rules for an AI (2.2.0): docs/SCRIPTS_POUR_IA_NOUVEAUTES.md,
// what changes from one version to the next, for a model that already has an
// older guide. Nothing generates it, so nothing would remind anyone to write
// it: it keeps a fingerprint of the rules (ai-intro.md, ai-checklist.md), and
// the fingerprint only follows a change when the changelog has an entry for
// the current version. The rules that entry quotes must also read exactly as
// in ai-intro.md: a rule cannot change without its entry.
// ---------------------------------------------------------------------------

const CHANGELOG = 'docs/SCRIPTS_POUR_IA_NOUVEAUTES.md';
const FINGERPRINT = /<!-- empreinte des consignes : ([0-9a-f]{16}) /;
const ruleSources = read('tools/script-guides/ai-intro.md') + '\n' + read('tools/script-guides/ai-checklist.md');
const fingerprint = createHash('sha256').update(ruleSources).digest('hex').slice(0, 16);

/** The numbered rules of a text: each first line, and the indented lines after it. */
function numberedRules(md) {
  const found = new Map();
  let current = null;
  for (const line of md.split('\n')) {
    const n = line.match(/^(\d+)\. /);
    if (n) found.set(n[1], (current = []));
    else if (current && line.trim() && !/^\s/.test(line)) current = null;
    if (current) current.push(line);
  }
  return new Map([...found].map(([k, lines]) => [k, lines.join('\n').trimEnd()]));
}

/** The changelog with the fingerprint of today's rules, or an error that says what to do. */
function buildChangelog() {
  const log = read(CHANGELOG);
  const recorded = log.match(FINGERPRINT);
  if (!recorded) throw new Error(`${CHANGELOG} : la ligne « <!-- empreinte des consignes : … --> » a disparu ; remettez-la sous l'introduction.`);
  const heading = `## ${version} — `;
  const start = log.indexOf(`\n${heading}`);
  if (recorded[1] !== fingerprint && start < 0 && !process.argv.includes('--sans-nouveaute')) {
    throw new Error(`Les consignes pour l'IA ont change (ai-intro.md, ai-checklist.md) sans entree « ${heading}» dans ${CHANGELOG}. `
      + 'Ajoutez-la ; pour une retouche qui ne change aucune consigne (une coquille), lancez node tools/build-script-guides.mjs --sans-nouveaute.');
  }
  if (start >= 0) {
    const end = log.indexOf('\n## ', start + 1);
    const quoted = numberedRules(log.slice(start, end < 0 ? log.length : end));
    const intro = read('tools/script-guides/ai-intro.md');
    const current = numberedRules(intro.slice(intro.indexOf('\n## 3. '), intro.indexOf('\n## 4. ')));
    for (const [n, text] of quoted) {
      if (current.get(n) !== text) throw new Error(`${CHANGELOG}, entree ${version} : la regle ${n} n'est plus celle de ai-intro.md ; recopiez-la.`);
    }
  }
  return log.replace(FINGERPRINT, `<!-- empreinte des consignes : ${fingerprint} `);
}

// ---------------------------------------------------------------------------
// The AI version
// ---------------------------------------------------------------------------

/** Shifts the headings of a markdown text down by `n` levels, outside code. */
function demote(md, n) {
  let fence = false;
  return md.split('\n').map((l) => {
    if (/^\s*```/.test(l)) fence = !fence;
    if (fence || !/^#{1,6}\s/.test(l)) return l;
    return '#'.repeat(n) + l;
  }).join('\n');
}

/** The reference data; `iconsPart` is the text of 5.4: the whole list, or a choice. */
function referenceData(iconsPart = allIcons()) {
  const catRows = categories.map((c, i) =>
    `| \`${c.id}\` | \`${c.id_fr}\` | ${c.name_fr} / ${c.name_en} | \`${i + 1}xx\` |`).join('\n');
  const lotRows = lots.map((l) =>
    `| ${l.name_fr} | ${l.aggregate ? 'tout ce qui est rangé dans les autres lots (aucun script ne le vise)' : (l.categories || []).map((c) => `\`${c}\``).join(', ')} |`).join('\n');
  return `### 5.1 Les catégories — champ \`category\`

Écris l'identifiant anglais avec \`lang : en\` (le cas des scripts officiels), l'identifiant
français avec \`lang : fr\`. Rien d'autre n'est admis.

| Anglais | Français | Nom | Numéros de fichier |
|---|---|---|---|
${catRows}

### 5.2 Les lots d'usine

Un script entre dans le lot qui accueille sa catégorie. Il ne vise jamais un lot.

| Lot | Accueille |
|---|---|
${lotRows}

### 5.3 Le nom du fichier

\`NNN_NOM_EN_ANGLAIS.ps1\` : trois chiffres dans la plage de la catégorie (5.1), un tiret
bas, le nom en majuscules, mots séparés par des tirets bas — par exemple
\`105_CLEAN_THUMBNAIL_CACHE.ps1\`. Le numéro fixe l'ordre par défaut dans le lot.

${iconsPart}`;
}

function allIcons() {
  return `### 5.4 Les icônes Lucide — champ \`icon\`

Les ${icons.length} noms admis, et aucun autre :

${icons.join(', ')}
`;
}

function buildAi() {
  const intro = read('tools/script-guides/ai-intro.md')
    .replace('{{VERSION}}', version)
    .replace('{{DATA}}', referenceData().trimEnd());
  const checklist = read('tools/script-guides/ai-checklist.md');
  return [
    intro.trimEnd(),
    '',
    '---',
    '',
    '## 6. Le contrat complet — docs/FORMAT_SCRIPT.md',
    '',
    'Reproduit en entier. Les titres sont décalés de deux niveaux ; rien d\'autre ne change.',
    '',
    demote(contract.trimEnd(), 2),
    '',
    '---',
    '',
    checklist.trimEnd(),
    '',
  ].join('\n');
}

// ---------------------------------------------------------------------------
// The short AI version (2.0.4). Its sections are picked by their title in the
// contract: a title renamed there stops the build, and CI with it, instead of
// leaving the short version behind.
// ---------------------------------------------------------------------------

/** What the short version keeps of the contract. `own`: the text under the
 *  title, without its sub-sections; `table`: up to the end of its first table. */
const EXTRACTS = [
  ['## Le squelette'],
  ['## Les marqueurs de sortie'],
  ["## Redémarrer au milieu d'un entretien (2.1.0)"],
  ['### Le contrat, en quatre règles'],
  ["### Les lignes de l'analyse", 'own'],
  ['#### Les champs de `[FIND]`'],
  ['#### Les champs de `[ITEM]`', 'table'],
  ['#### Les champs de `[METRIC]`'],
  ["### Le bloc `REPORT` : les mots de l'analyse"],
  ['### Exemple complet — faire de la place'],
];

/** Moves every heading outside code by `delta` levels (at least 1). */
function relevel(md, delta) {
  let fence = false;
  return md.split('\n').map((l) => {
    if (/^\s*```/.test(l)) fence = !fence;
    const h = !fence && l.match(/^(#{1,6})(\s.*)$/);
    return h ? '#'.repeat(Math.max(1, h[1].length + delta)) + h[2] : l;
  }).join('\n');
}

/** One section of the contract, its title brought to level 3. */
function extract(title, mode) {
  const lines = contract.split('\n');
  const level = title.match(/^#+/)[0].length;
  let fence = false, start = -1, end = lines.length;
  for (let i = 0; i < lines.length; i++) {
    if (/^\s*```/.test(lines[i])) fence = !fence;
    const h = !fence && lines[i].match(/^(#{1,6})\s/);
    if (!h) continue;
    if (start < 0) {
      if (lines[i] === title) start = i;
    } else if (mode === 'own' || h[1].length <= level) {
      end = i;
      break;
    }
  }
  if (start < 0) throw new Error(`FORMAT_SCRIPT.md : section « ${title} » introuvable ; mettez a jour EXTRACTS dans tools/build-script-guides.mjs`);
  let body = lines.slice(start, end);
  if (mode === 'table') {
    let last = body.findIndex((l) => l.startsWith('|'));
    while (last + 1 < body.length && body[last + 1].startsWith('|')) last++;
    body = body.slice(0, last + 1);
  }
  return relevel(body.join('\n').trimEnd(), 3 - level);
}

/** 5.4 in short: a choice of icons by theme, each checked against Lucide. */
function shortIcons() {
  const known = new Set(icons);
  const rows = read('tools/script-guides/ai-court-icones.txt').split('\n')
    .filter((l) => l.trim() && !l.startsWith('#'))
    .map((l) => {
      const [theme, names] = l.split(' : ');
      const list = names.trim().split(/\s+/);
      const unknown = list.filter((n) => !known.has(n));
      if (unknown.length) throw new Error(`ai-court-icones.txt : pas des icones Lucide : ${unknown.join(', ')}`);
      return `| ${theme.trim()} | ${list.join(', ')} |`;
    });
  return `### 5.4 Les icônes Lucide — champ \`icon\`

Une sélection pour un outil de maintenance. Le validateur admet les ${icons.length} noms de
Lucide (liste complète dans \`SCRIPTS_POUR_IA.md\`) : prends ici celui qui convient.

| Thème | Noms |
|---|---|
${rows.join('\n')}
`;
}

function buildAiShort() {
  const rules = read('tools/script-guides/ai-intro.md');
  const head = read('tools/script-guides/ai-court-entete.md').replace('{{VERSION}}', version);
  // Everything after the long version's own header: mission, context, rules, method.
  const body = rules.slice(rules.indexOf('\n---\n')).replace('{{DATA}}', referenceData(shortIcons()).trimEnd());
  return [
    head.trimEnd() + body.trimEnd(),
    '',
    '---',
    '',
    '## 6. Extraits du contrat — docs/FORMAT_SCRIPT.md',
    '',
    'Les parties du contrat dont chaque script se sert, telles quelles. Les autres titres cités',
    'entre « » dans les règles sont dans la version complète, `SCRIPTS_POUR_IA.md`.',
    '',
    EXTRACTS.map(([title, mode]) => extract(title, mode)).join('\n\n'),
    '',
    '---',
    '',
    read('tools/script-guides/ai-checklist.md').trimEnd(),
    '',
  ].join('\n');
}

// ---------------------------------------------------------------------------
// The human version
// ---------------------------------------------------------------------------

function buildHtml(ai, aiShort) {
  const ctx = { ids: new Set(['creer', 'reference']), toc: [] };
  const content = render(contract, ctx);
  const toc = ctx.toc.map((t) => `      <li class="l${t.level}"><a href="#${t.id}">${t.html.replace(/<\/?(code|strong|em)>/g, '')}</a></li>`).join('\n');
  const data = JSON.stringify({ version, categories, lots, icons, ai, aiShort }).replace(/</g, '\\u003c');
  return read('tools/script-guides/guide.html')
    .replaceAll('{{VERSION}}', version)
    .replace('<!--TOC-->', toc)
    .replace('<!--CONTENT-->', content)
    .replace('/*DATA*/', () => data);
}

// ---------------------------------------------------------------------------

const ai = buildAi();
const aiShort = buildAiShort();
let changelog;
try {
  changelog = buildChangelog();
} catch (e) {
  console.log(`::error::${e.message}`);
  process.exit(1);
}
const outputs = [
  ['docs/SCRIPTS_POUR_IA.md', ai],
  ['docs/SCRIPTS_POUR_IA_COURT.md', aiShort],
  ['docs/GUIDE_SCRIPTS.html', buildHtml(ai, aiShort)],
  [CHANGELOG, changelog],
];

if (process.argv.includes('--check')) {
  const stale = outputs.filter(([p, text]) => !existsSync(join(ROOT, p)) || read(p) !== text);
  if (stale.length) {
    for (const [p] of stale) console.log(`::error::${p} n'est plus a jour : lancez node tools/build-script-guides.mjs, puis commitez.`);
    process.exit(1);
  }
  console.log('Guides des scripts a jour.');
} else {
  for (const [p, text] of outputs) {
    writeFileSync(join(ROOT, p), text, 'utf8');
    console.log(`${p} : ${Math.round(Buffer.byteLength(text) / 1024)} Ko`);
  }
}
