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
// build.
//
//   node tools/build-script-guides.mjs           write the three files
//   node tools/build-script-guides.mjs --check   fail if one is out of date (CI)

import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const read = (p) => readFileSync(join(ROOT, p), 'utf8').replace(/^﻿/, '').replace(/\r\n/g, '\n');

const version = JSON.parse(read('package.json')).version;
const contract = read('docs/FORMAT_SCRIPT.md');
const categories = JSON.parse(read('tools/categories.json'));
const lots = JSON.parse(read('tools/lots.json'));
const icons = read('tools/lucide-icon-names.txt').split('\n').map((l) => l.trim()).filter((l) => l && !l.startsWith('#'));

// ---------------------------------------------------------------------------
// Markdown → HTML, for the subset FORMAT_SCRIPT.md uses: headings, paragraphs,
// fenced code, tables, lists, block quotes, rules, and inline code, bold,
// italic and links. Anything else stays text, escaped.
// ---------------------------------------------------------------------------

const escapeHtml = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
const escapeAttr = (s) => escapeHtml(s).replace(/"/g, '&quot;');

function slugify(text, used) {
  const base = text.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase()
    .replace(/[`*]/g, '').replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'section';
  let id = base;
  for (let n = 2; used.has(id); n++) id = `${base}-${n}`;
  used.add(id);
  return id;
}

/** Inline markdown. Code spans are set aside first: nothing inside them is markup. */
function inline(md) {
  const codes = [];
  let s = md.replace(/`([^`]+)`/g, (_, c) => `\u0000${codes.push(c) - 1}\u0000`);
  s = escapeHtml(s);
  s = s.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_, t, u) => `<a href="${u.replace(/"/g, '&quot;')}">${t}</a>`);
  s = s.replace(/\*\*((?:[^*]|\*[^*]+\*)+?)\*\*/g, '<strong>$1</strong>');
  s = s.replace(/(^|[^*\w])\*([^*\s](?:[^*]*?[^*\s])?)\*(?![*\w])/g, '$1<em>$2</em>');
  return s.replace(/\u0000(\d+)\u0000/g, (_, i) => `<code>${escapeHtml(codes[i])}</code>`);
}

const MARKERS = 'INFO|OK|WARN|ERR|STEP|PROGRESS|CKPT|REBOOT|DONE|FREED|LOG|FIND|ITEM|METRIC|NOTE';

/** A light colouring of code: `##` metadata, comments, strings, output markers. */
function highlight(code) {
  return code.split('\n').map((line) => {
    const t = line.trimStart();
    if (t.startsWith('##')) return `<span class="k-meta">${escapeHtml(line)}</span>`;
    if (t.startsWith('#') || t.startsWith('<#')) return `<span class="k-com">${escapeHtml(line)}</span>`;
    let s = escapeHtml(line);
    s = s.replace(/"(?:[^"\\`]|`.|\\.)*"|'[^']*'/g, (m) => `<span class="k-str">${m}</span>`);
    return s.replace(new RegExp(`\\[(${MARKERS})\\]`, 'g'), '<span class="k-mark">[$1]</span>');
  }).join('\n');
}

/** Splits a table row on `|`, but never inside a code span. */
function cells(row) {
  const r = row.trim().replace(/^\|/, '').replace(/\|$/, '');
  const out = [];
  let cur = '', inCode = false;
  for (const ch of r) {
    if (ch === '`') inCode = !inCode;
    if (ch === '|' && !inCode) { out.push(cur.trim()); cur = ''; } else cur += ch;
  }
  out.push(cur.trim());
  return out;
}

const SEVERITIES = new Set(['erreur', 'avertissement', 'note']);

function render(src, ctx, { inQuote = false } = {}) {
  const lines = src.split('\n');
  const out = [];
  let para = [];
  const flush = () => {
    if (para.length) out.push(`<p>${inline(para.join(' '))}</p>`);
    para = [];
  };
  const marker = /^(\s*)([-*]|\d+\.)\s+(.*)$/;
  let i = 0;
  while (i < lines.length) {
    const l = lines[i];
    if (/^\s*```/.test(l)) {
      flush();
      const indent = l.length - l.trimStart().length;
      const buf = [];
      for (i++; i < lines.length && !/^\s*```\s*$/.test(lines[i]); i++) buf.push(lines[i].slice(Math.min(indent, lines[i].length - lines[i].trimStart().length)));
      i++;
      out.push(`<pre><code>${highlight(buf.join('\n'))}</code></pre>`);
      continue;
    }
    const h = /^(#{1,6})\s+(.*)$/.exec(l);
    if (h) {
      flush();
      const level = h[1].length;
      const text = h[2].trim();
      const id = slugify(text, ctx.ids);
      out.push(`<h${level} id="${id}">${inline(text)}</h${level}>`);
      if (!inQuote && (level === 2 || level === 3)) ctx.toc.push({ level, id, html: inline(text) });
      i++;
      continue;
    }
    if (/^---+\s*$/.test(l)) { flush(); out.push('<hr>'); i++; continue; }
    if (/^\s*\|/.test(l) && /^\s*\|?\s*:?-{3,}/.test(lines[i + 1] || '')) {
      flush();
      const head = cells(l);
      const rows = [];
      for (i += 2; i < lines.length && /^\s*\|/.test(lines[i]); i++) rows.push(cells(lines[i]));
      const td = (c) => (SEVERITIES.has(c) ? `<span class="sev ${c}">${c}</span>` : inline(c));
      out.push(`<div class="tbl-wrap" data-rows="${rows.length}"><table><thead><tr>${head.map((c) => `<th>${inline(c)}</th>`).join('')}</tr></thead>`
        + `<tbody>${rows.map((r) => `<tr>${r.map((c) => `<td>${td(c)}</td>`).join('')}</tr>`).join('')}</tbody></table></div>`);
      continue;
    }
    if (/^>/.test(l)) {
      flush();
      const buf = [];
      for (; i < lines.length && /^>/.test(lines[i]); i++) buf.push(lines[i].replace(/^> ?/, ''));
      out.push(`<blockquote>${render(buf.join('\n'), ctx, { inQuote: true })}</blockquote>`);
      continue;
    }
    const m = marker.exec(l);
    if (m && m[1].length === 0) {
      flush();
      const ordered = /\d/.test(m[2]);
      const items = [];
      while (i < lines.length) {
        const mm = marker.exec(lines[i]);
        if (!mm || mm[1].length !== 0 || /\d/.test(mm[2]) !== ordered) break;
        const body = [mm[3]];
        for (i++; i < lines.length; i++) {
          const n = lines[i];
          if (n.trim() === '') {
            // A blank line inside an item continues it only if indented text follows.
            if (/^\s{2,}\S/.test(lines[i + 1] || '')) { body.push(''); continue; }
            break;
          }
          if (!/^\s/.test(n)) break;
          body.push(n.replace(/^\s{2,4}/, ''));
        }
        items.push(body.join('\n'));
        if ((lines[i] || '').trim() === '' && marker.exec(lines[i + 1] || '')?.[1] === '') i++;
      }
      const tag = ordered ? 'ol' : 'ul';
      out.push(`<${tag}>${items.map((b) => {
        const html = render(b, ctx, { inQuote: true });
        const single = /^<p>([\s\S]*)<\/p>$/.exec(html);
        return `<li>${single && !html.includes('</p><') ? single[1] : html}</li>`;
      }).join('')}</${tag}>`);
      continue;
    }
    if (l.trim() === '') { flush(); i++; continue; }
    para.push(l.trim());
    i++;
  }
  flush();
  return out.join('\n');
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
const outputs = [
  ['docs/SCRIPTS_POUR_IA.md', ai],
  ['docs/SCRIPTS_POUR_IA_COURT.md', aiShort],
  ['docs/GUIDE_SCRIPTS.html', buildHtml(ai, aiShort)],
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
