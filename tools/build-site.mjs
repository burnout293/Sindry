// Builds the Sindry site for GitHub Pages (2.2.0), into _site/:
//
//   index.html             what Sindry is, the download, screenshots (site/accueil.html)
//   catalogue.html         the official catalogue, read from its SIGNED index
//   guide-scripts.html     docs/GUIDE_SCRIPTS.html, as it is
//   ia.html, ia/*.md       the guides for an AI, and the changelog of their rules
//   feuille-de-route.html  docs/ROADMAP.md
//   versions.html          the release notes, from GitHub
//
// The catalogue and the releases are read from GitHub when the site is built:
// the pages themselves fetch nothing and carry no tracker. The catalogue index
// is checked against the public key compiled into Sindry before it is read,
// and every script against the index, as Sindry itself does (catalogue.rs).
//
//   node tools/build-site.mjs               build, with the data from GitHub
//   node tools/build-site.mjs --hors-ligne  build without network (CI, tests)
//   node tools/build-site.mjs --out <dir>   elsewhere than _site/
//   node tools/build-site.mjs --strict      fail if GitHub cannot be read, or
//                                           a signature does not hold (Pages
//                                           workflow: the site online stays)
//
// GH_TOKEN, when set (the Pages workflow), raises the limits of the GitHub API.

import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync, copyFileSync, rmSync, existsSync, readdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { render, inline, cells, escapeHtml, escapeAttr } from './markdown.mjs';
import { verifyMinisign } from './minisign.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const read = (p) => readFileSync(join(ROOT, p), 'utf8').replace(/^﻿/, '').replace(/\r\n/g, '\n');
const args = process.argv.slice(2);
const OFFLINE = args.includes('--hors-ligne');
const STRICT = args.includes('--strict');
const OUT = resolve(ROOT, args.includes('--out') ? args[args.indexOf('--out') + 1] : '_site');

const REPO = 'burnout293/Sindry';
const CATALOGUE = 'burnout293/Sindry-Catalogue';
const version = JSON.parse(read('package.json')).version;
const categories = JSON.parse(read('tools/categories.json'));

// ---------------------------------------------------------------------------
// GitHub
// ---------------------------------------------------------------------------

async function get(url, { json = false } = {}) {
  const headers = { 'User-Agent': 'sindry-site' };
  if (url.startsWith('https://api.github.com/')) {
    headers.Accept = 'application/vnd.github+json';
    headers['X-GitHub-Api-Version'] = '2022-11-28';
    if (process.env.GH_TOKEN) headers.Authorization = `Bearer ${process.env.GH_TOKEN}`;
  }
  const r = await fetch(url, { headers, redirect: 'follow' });
  if (!r.ok) throw new Error(`${url} : HTTP ${r.status}`);
  return json ? r.json() : Buffer.from(await r.arrayBuffer());
}

// ---------------------------------------------------------------------------
// The catalogue
// ---------------------------------------------------------------------------

/** The header of a script, and the titles of its translations. */
function scriptHeader(text) {
  const meta = {};
  const tr = {};
  let block = '';
  for (const raw of text.replace(/^﻿/, '').split(/\r?\n/)) {
    const l = raw.trim();
    let m;
    // SINDRY:, or WINTOOL: for a script written before 2.0 (read until v3).
    if ((m = l.match(/^##\s*(?:SINDRY|WINTOOL):([A-Z]+)(?:\s+([A-Za-z-]+))?\s*$/))) {
      block = m[1] === 'START' ? 'start' : m[1] === 'LANG' && m[2] ? `lang:${m[2].toLowerCase()}` : '';
      continue;
    }
    if (!block || !(m = l.match(/^##\s([A-Za-z_]+)\s*:\s*(.*)$/))) continue;
    const key = m[1].toLowerCase();
    if (block === 'start') meta[key] = m[2].trim();
    else if (key === 'title' || key === 'desc') (tr[block.slice(5)] ??= {})[key] = m[2].trim();
  }
  const fr = (meta.lang || 'en') === 'fr' ? meta : { ...meta, ...tr.fr };
  return { ...meta, title_fr: fr.title || meta.title || '', desc_fr: fr.desc || meta.desc || '' };
}

async function loadCatalogue() {
  const base = `https://github.com/${CATALOGUE}/releases/latest/download`;
  const [index, sig] = await Promise.all([get(`${base}/index.json`), get(`${base}/index.json.sig`)]);
  // The signature first: an index that is not checked is hostile data.
  verifyMinisign(read('src-tauri/catalogue.pub'), index, sig.toString('utf8'));
  const idx = JSON.parse(index.toString('utf8').replace(/^﻿/, ''));
  const scripts = await Promise.all(idx.scripts.map(async (s) => {
    const bytes = await get(`https://github.com/${CATALOGUE}/releases/download/${encodeURIComponent(idx.tag)}/${encodeURIComponent(s.file)}`);
    const sha = createHash('sha256').update(bytes).digest('hex');
    if (sha !== s.sha256 || bytes.length !== s.size) throw new Error(`${s.file} ne correspond pas a l'index signe`);
    return { ...s, meta: scriptHeader(bytes.toString('utf8')) };
  }));
  return { version: idx.version, published: idx.published, tag: idx.tag, scripts };
}

// ---------------------------------------------------------------------------
// The pages
// ---------------------------------------------------------------------------

const lucide = join(ROOT, 'node_modules', 'lucide-static', 'icons');
/** A Lucide icon, inline (the pages load nothing else), or nothing if it is unknown. */
function icon(name) {
  const p = join(lucide, `${name}.svg`);
  if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(name) || !existsSync(p)) return '';
  return readFileSync(p, 'utf8').replace(/<!--[\s\S]*?-->/g, '').trim()
    .replace(/<svg[^>]*?(viewBox="[^"]*")[^>]*>/, '<svg class="ico" $1 fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">')
    .replace(/\s*\n\s*/g, '');
}

const frDate = (d) => new Date(d).toLocaleDateString('fr-FR', { day: 'numeric', month: 'long', year: 'numeric', timeZone: 'Europe/Paris' });
const mo = (n) => `${(n / 1048576).toFixed(1).replace('.', ',')} Mo`;
const plain = (s) => s.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase();

const NAV = [
  ['index.html', 'Accueil'],
  ['catalogue.html', 'Catalogue'],
  ['guide-scripts.html', 'Écrire un script'],
  ['ia.html', 'Pour une IA'],
  ['feuille-de-route.html', 'Feuille de route'],
  ['versions.html', 'Versions'],
];

function page(file, title, description, body) {
  const nav = NAV.map(([href, label]) => `<a href="${href}"${href === file ? ' aria-current="page"' : ''}>${label}</a>`).join('');
  return `<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<!-- Icônes : Lucide (ISC ; MIT pour celles issues de Feather). Polices : Inter, Sindry Mono (SIL OFL 1.1). Textes : licences/THIRD-PARTY-NOTICES.txt -->
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'self'; img-src 'self'; style-src 'self'; script-src 'self'; font-src 'self'; connect-src 'none'; base-uri 'none'; form-action 'none'">
<meta name="referrer" content="no-referrer">
<title>${escapeHtml(title)}</title>
<meta name="description" content="${escapeAttr(description)}">
<link rel="icon" href="images/icone-32.png">
<link rel="stylesheet" href="style.css">
<script src="site.js"></script>
</head>
<body>
<header class="top"><div class="top-in">
<a class="brand" href="index.html"><img src="images/icone.png" alt="" width="30" height="30">Sindry</a>
<nav class="nav" aria-label="Pages">${nav}<a href="https://github.com/${REPO}">GitHub</a></nav>
<button class="theme-btn" id="themeBtn" type="button">Thème : système</button>
</div></header>
<main>
${body}
</main>
<footer><div class="in">
<span>Sindry ${escapeHtml(version)} : gratuit, sous licence Apache 2.0 avec des conditions propres à Sindry (<a href="licences/LICENSE.txt">LICENSE</a>). Le catalogue officiel est sous licence MIT.</span>
<span>Ce site ne dépose aucun cookie et ne contient aucun traceur. Polices Inter et Sindry Mono, dérivée d'IBM Plex Mono (SIL OFL 1.1) ; icônes Lucide (ISC, et MIT pour celles qui viennent de Feather) : <a href="licences/THIRD-PARTY-NOTICES.txt">mentions et textes des licences</a>.</span>
</div></footer>
</body>
</html>
`;
}

/** The download block of the home page, from the latest stable release. */
function downloadBlock(release) {
  const asset = release?.assets?.find((a) => /_x64-setup\.exe$/.test(a.name));
  if (!asset) {
    return `<div class="actions"><a class="btn primary" href="https://github.com/${REPO}/releases/latest">${icon('download')}Télécharger Sindry</a></div>
<p class="meta">Windows 10 et 11, 64 bits.</p>`;
  }
  return `<div class="actions">
<a class="btn primary" href="${escapeAttr(asset.browser_download_url)}">${icon('download')}Télécharger Sindry ${escapeHtml(release.tag_name.replace(/^v/, ''))}</a>
<a class="btn" href="versions.html">Les nouveautés</a>
</div>
<p class="meta">Publiée le ${frDate(release.published_at)} · ${escapeHtml(asset.name)} · ${mo(asset.size)} · Windows 10 et 11, 64 bits</p>`;
}

function fingerprintBlock(release) {
  const asset = release?.assets?.find((a) => /_x64-setup\.exe$/.test(a.name));
  const sha = asset?.digest?.match(/^sha256:([0-9a-f]{64})$/)?.[1];
  if (!sha) return '<p>Pour vérifier l\'installeur avant de le lancer, chaque version publie son empreinte SHA-256 sur la <a href="versions.html">page des versions</a>.</p>';
  return `<p>Pour vérifier l'installeur avant de le lancer, comparez son empreinte SHA-256 :</p>
<p class="empreinte">${sha}</p>
<div class="md"><pre><code>Get-FileHash .\\${escapeHtml(asset.name)} -Algorithm SHA256</code></pre></div>`;
}

function homePage(releases) {
  const stable = releases?.find((r) => !r.prerelease) || null;
  const body = read('site/accueil.html')
    .replace(/<!--[\s\S]*?-->\s*/, '')
    .replace('{{telechargement}}', () => downloadBlock(stable))
    .replace('{{empreinte}}', () => fingerprintBlock(stable))
    .replace(/\{\{icone:([a-z0-9-]+)\}\}/g, (_, n) => icon(n));
  return page('index.html', 'Sindry : l\'entretien de Windows, expliqué avant d\'être fait',
    'Outil gratuit de maintenance Windows : des scripts PowerShell lisibles, une analyse avant chaque action, un catalogue signé.', body);
}

const RISK = { low: ['Risque faible', 'ok'], medium: ['Risque moyen', 'warn'], high: ['Risque élevé', 'err'] };

function cataloguePage(cat) {
  let body;
  if (!cat) {
    body = `<div class="page-h"><h1>Le catalogue officiel</h1>
<p>Le catalogue n'a pas pu être lu quand ce site a été construit. Il reste consultable sur <a href="https://github.com/${CATALOGUE}">son dépôt</a>.</p></div>`;
  } else {
    const byId = new Map(categories.flatMap((c) => [[c.id, c], [c.id_fr, c]]));
    const groups = new Map(categories.map((c) => [c.id, { c, scripts: [] }]));
    groups.set('autres', { c: { id: 'autres', name_fr: 'Autres', icon: 'shapes' }, scripts: [] });
    for (const s of [...cat.scripts].sort((a, b) => a.file.localeCompare(b.file))) {
      groups.get(byId.get(s.meta.category)?.id || 'autres').scripts.push(s);
    }
    const card = (s) => {
      const m = s.meta;
      const risk = RISK[m.risk];
      const badges = [
        risk ? `<span class="badge ${risk[1]}">${risk[0]}</span>` : '',
        m.admin === 'true' ? '<span class="badge">Administrateur</span>' : '',
        m.reversible === 'false' ? '<span class="badge warn">Irréversible</span>' : '',
        m.scan === 'true' ? '<span class="badge">Analyse d\'abord</span>' : '',
        m.reboot === 'true' || m.resume === 'true' ? '<span class="badge">Redémarrage</span>' : '',
        m.show === 'expert' ? '<span class="badge">Mode Expert</span>' : '',
        m.version ? `<span class="badge">v${escapeHtml(m.version)}</span>` : '',
        `<a href="https://github.com/${CATALOGUE}/blob/${encodeURIComponent(cat.tag)}/scripts/${encodeURIComponent(s.file)}">Lire le script</a>`,
      ].join('');
      const num = s.file.match(/^(\d{3})_/)?.[1] || '';
      return `<article class="script" data-cherche="${escapeAttr(plain(`${m.title_fr} ${m.desc_fr} ${m.title || ''} ${s.file} ${num}`))}">
<span class="pastille">${icon(m.icon || '') || icon('file-code')}</span>
<div><h3><span class="num">${num}</span>${escapeHtml(m.title_fr)}</h3><p>${escapeHtml(m.desc_fr)}</p><div class="badges">${badges}</div></div>
</article>`;
    };
    const sections = [...groups.values()].filter((g) => g.scripts.length).map((g) =>
      `<section class="cat"><h2>${icon(g.c.icon || '')}${escapeHtml(g.c.name_fr)}</h2><div class="scripts">${g.scripts.map(card).join('\n')}</div></section>`).join('\n');
    body = `<div class="page-h"><h1>Le catalogue officiel</h1>
<p>Version ${escapeHtml(cat.version)}, publiée le ${frDate(cat.published)} : ${cat.scripts.length} actions, sous licence MIT. Cette liste est lue dans l'index <b>signé</b> du catalogue, vérifié avec la clé que Sindry porte : la même vérification que celle de Sindry avant d'installer un script.</p></div>
<div class="cat-tools"><input id="catCherche" type="search" placeholder="Chercher une action" aria-label="Chercher une action"><span class="count" id="catCompte"></span></div>
${sections}`;
  }
  return page('catalogue.html', 'Le catalogue officiel de Sindry', 'Les actions du catalogue officiel de Sindry, lues dans son index signé.', body);
}

const AI_FILES = [
  ['SCRIPTS_POUR_IA.md', 'Le guide complet', 'Pour un modèle à grand contexte : ChatGPT, Claude, Gemini… Les règles, les données de référence et le contrat entier.'],
  ['SCRIPTS_POUR_IA_COURT.md', 'La version courte', 'Pour un modèle local, au contexte court : les mêmes règles, et seulement les parties du contrat dont chaque script se sert.'],
  ['SCRIPTS_POUR_IA_NOUVEAUTES.md', 'Le journal des consignes', 'Pour une IA qui a déjà un guide plus ancien : seules les consignes nouvelles ou modifiées, version par version.'],
];

function aiPage() {
  const files = AI_FILES.map(([f, title, text]) => {
    const size = Buffer.byteLength(read(`docs/${f}`));
    return `<div class="dl">${icon('file-text')}<div><b>${title}</b><span>${text}</span></div><a class="btn" href="ia/${f}" download>${icon('download')}${f} <small>${Math.max(1, Math.round(size / 1024))} Ko</small></a></div>`;
  }).join('\n');
  // The changelog, rendered below: without its title and its fingerprint.
  const changelog = read('docs/SCRIPTS_POUR_IA_NOUVEAUTES.md')
    .replace(/^# .*\n/, '').replace(/<!--[\s\S]*?-->\n?/g, '');
  const ctx = { ids: new Set(), toc: [] };
  const body = `<div class="page-h"><h1>Faire écrire un script par une IA</h1>
<p>Donnez l'un de ces fichiers à une IA, puis décrivez ce que le script doit faire. Le guide lui apprend le contrat de Sindry, ses règles de sécurité et le style du catalogue officiel : le script qu'elle écrit passe le validateur. Relisez-le toujours avant de le lancer.</p></div>
<div class="dl-list">${files}</div>
<h2 id="journal">Ce qui change, version par version</h2>
<div class="md">${render(changelog, ctx)}</div>`;
  return page('ia.html', 'Sindry : faire écrire un script par une IA', 'Les guides à donner à une IA pour qu\'elle écrive des scripts Sindry, et le journal de leurs consignes.', body);
}

function roadmapPage() {
  const ctx = { ids: new Set(), toc: [] };
  return page('feuille-de-route.html', 'La feuille de route de Sindry', 'Ce qui est fait, ce qui vient, et le cap de la version 3.',
    `<div class="md large">${render(read('docs/ROADMAP.md'), ctx)}</div>`);
}

/**
 * What each version brought, from ROADMAP.md: the row `| **2.1.0** | summary |`
 * and the `| | — item |` rows under it. The text of a GitHub release is only
 * the installation note, the same for all; the roadmap is the real record.
 */
function roadmapVersions() {
  const found = new Map();
  let current = null;
  for (const line of read('docs/ROADMAP.md').split('\n')) {
    const c = line.startsWith('|') ? cells(line) : null;
    const v = c?.[0].match(/^\*\*v?(\d+\.\d+(?:\.\d+)?)\*\*$/);
    if (v) found.set(v[1], (current = { summary: c[1], items: [] }));
    else if (current && c && c[0] === '' && c[1].startsWith('— ')) current.items.push(c[1].slice(2));
    else current = null;
  }
  return found;
}

function releasesPage(releases) {
  let body;
  if (!releases) {
    body = `<div class="page-h"><h1>Les versions</h1><p>Les versions n'ont pas pu être lues quand ce site a été construit : elles sont sur <a href="https://github.com/${REPO}/releases">GitHub</a>.</p></div>`;
  } else {
    const notes = roadmapVersions();
    const items = releases.map((r) => {
      const v = r.tag_name.replace(/^v/, '');
      const n = notes.get(v) || notes.get(v.replace(/\.0$/, ''));
      const asset = r.assets?.find((a) => /_x64-setup\.exe$/.test(a.name));
      const sha = asset?.digest?.match(/^sha256:([0-9a-f]{64})$/)?.[1];
      const content = n
        ? `<p>${inline(n.summary)}</p>${n.items.length ? `<ul>${n.items.map((x) => `<li>${inline(x)}</li>`).join('')}</ul>` : ''}`
        : `<p>Le détail de cette version est sur <a href="${escapeAttr(r.html_url)}">GitHub</a>.</p>`;
      const dl = asset
        ? `<div class="dl">${icon('download')}<div><b>${escapeHtml(asset.name)}</b><span>${mo(asset.size)}</span>${sha ? `<span class="empreinte">SHA-256 ${sha}</span>` : ''}</div><a class="btn" href="${escapeAttr(asset.browser_download_url)}">Télécharger</a></div>`
        : '';
      return `<section class="release"><header><h2 id="${escapeAttr(r.tag_name)}">Sindry ${escapeHtml(v)}</h2><span>${frDate(r.published_at)}${r.prerelease ? ' · bêta' : ''}</span><a href="${escapeAttr(r.html_url)}">Sur GitHub</a></header>
<div class="md">${content}</div>${dl}</section>`;
    }).join('\n');
    body = `<div class="page-h"><h1>Les versions</h1><p>Chaque version publiée, de la plus récente à la plus ancienne, avec ce qu'elle a apporté. Sindry vous propose lui-même la dernière, et vérifie sa signature avant de l'installer.</p></div>\n${items}`;
  }
  return page('versions.html', 'Les versions de Sindry', 'Ce que chaque version de Sindry a apporté, et ses installeurs.', body);
}

// ---------------------------------------------------------------------------

async function attempt(what, fn) {
  if (OFFLINE) return null;
  try {
    return await fn();
  } catch (e) {
    console.log(`::${STRICT ? 'error' : 'warning'}::${what} : ${e.message}`);
    if (STRICT) process.exit(1);
    return null;
  }
}

const releases = await attempt('Versions', async () =>
  (await get(`https://api.github.com/repos/${REPO}/releases?per_page=50`, { json: true })).filter((r) => !r.draft));
const catalogue = await attempt('Catalogue', loadCatalogue);

rmSync(OUT, { recursive: true, force: true });
for (const d of ['', 'fonts', 'images', 'ia', 'licences']) mkdirSync(join(OUT, d), { recursive: true });
const write = (p, text) => writeFileSync(join(OUT, p), text, 'utf8');
const copy = (from, to) => copyFileSync(join(ROOT, from), join(OUT, to));

write('index.html', homePage(releases));
write('catalogue.html', cataloguePage(catalogue));
write('ia.html', aiPage());
write('feuille-de-route.html', roadmapPage());
write('versions.html', releasesPage(releases));
write('404.html', page('404.html', 'Page introuvable · Sindry', 'Cette page n\'existe pas.',
  '<div class="page-h"><h1>Page introuvable</h1><p>Cette page n\'existe pas. Retour à <a href="index.html">l\'accueil</a>.</p></div>'));
write('.nojekyll', '');
copy('docs/GUIDE_SCRIPTS.html', 'guide-scripts.html');
for (const [f] of AI_FILES) copy(`docs/${f}`, `ia/${f}`);
copy('site/style.css', 'style.css');
copy('site/site.js', 'site.js');
for (const f of readdirSync(join(ROOT, 'site/images'))) copy(`site/images/${f}`, `images/${f}`);
for (const f of readdirSync(join(ROOT, 'src/fonts'))) copy(`src/fonts/${f}`, `fonts/${f}`);
copy('src-tauri/icons/128x128@2x.png', 'images/icone.png');
copy('src-tauri/icons/32x32.png', 'images/icone-32.png');
// The icons and the fonts travel with their licences.
copy('THIRD-PARTY-NOTICES.md', 'licences/THIRD-PARTY-NOTICES.txt');
copy('LICENSE', 'licences/LICENSE.txt');

console.log(`Site construit dans ${OUT}${OFFLINE ? ' (hors ligne)' : ''} : versions ${releases ? releases.length : 'absentes'}, catalogue ${catalogue ? `${catalogue.version}, ${catalogue.scripts.length} actions` : 'absent'}.`);
