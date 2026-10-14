/**
 * Archive les maquettes de docs/mockups, numerotees et datees.
 *
 *   node tools/archiver-maquettes.mjs <dossier d'archives>
 *
 * Pour chaque maquette du registre (docs/mockups/registre.json) :
 *   - une copie nommee M17_2026-10-11_themes-ambiances.html ;
 *   - un badge en bas a gauche (« M17 · 11/10/2026 07:50 · Themes d'ambiance »)
 *     et le numero en tete du titre de l'onglet : un onglet oublie se
 *     reconnait d'un coup d'oeil ;
 *   - ses ressources (police, icone) dans _ressources/, chemins reecrits : la
 *     copie s'ouvre seule, sans le depot.
 * Et un index.html, la plus recente en tete.
 *
 * Rejouable : les copies sont reecrites a l'identique, rien n'est jamais
 * supprime du dossier d'archives. Une maquette de docs/mockups absente du
 * registre est signalee : elle n'a pas encore de numero.
 */
import { readFileSync, writeFileSync, mkdirSync, copyFileSync, readdirSync, existsSync } from 'node:fs';
import { join, dirname, basename } from 'node:path';
import { fileURLToPath } from 'node:url';

const racine = join(dirname(fileURLToPath(import.meta.url)), '..');
const source = join(racine, 'docs', 'mockups');
const dest = process.argv[2];
if (!dest) {
  console.error('Usage : node tools/archiver-maquettes.mjs <dossier d\'archives>');
  process.exit(1);
}

const RESSOURCES = {
  '../../src/fonts/inter-latin-wght-normal.woff2': join(racine, 'src', 'fonts', 'inter-latin-wght-normal.woff2'),
  '../../src/fonts/sindry-mono-400.woff2': join(racine, 'src', 'fonts', 'sindry-mono-400.woff2'),
  '../../src/marque.png': join(racine, 'src', 'marque.png'),
  'sindry-256.png': join(source, 'sindry-256.png'),
};

const registre = JSON.parse(readFileSync(join(source, 'registre.json'), 'utf8')).maquettes;
const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
const num = (n) => `M${String(n).padStart(2, '0')}`;
const dateFr = (d) => { const [j, h] = d.split(' '); const [a, m, jj] = j.split('-'); return `${jj}/${m}/${a} ${h}`; };
const nomArchive = (e) => `${num(e.n)}_${e.date.slice(0, 10)}_${basename(e.fichier, '.html')}.html`;

mkdirSync(join(dest, '_ressources'), { recursive: true });
for (const chemin of Object.values(RESSOURCES)) {
  if (existsSync(chemin)) copyFileSync(chemin, join(dest, '_ressources', basename(chemin)));
}

for (const e of registre) {
  const f = join(source, e.fichier);
  if (!existsSync(f)) { console.warn(`[WARN] ${num(e.n)} : ${e.fichier} introuvable`); continue; }
  let html = readFileSync(f, 'utf8');
  for (const [avant, chemin] of Object.entries(RESSOURCES)) {
    html = html.split(avant).join(`_ressources/${basename(chemin)}`);
  }
  const libelle = `${num(e.n)} · ${dateFr(e.date)} · ${e.titre}`;
  html = html.replace(/<title>/, `<title>${esc(num(e.n))} · `);
  const badge = `<div data-maquette-badge style="position:fixed;left:10px;bottom:10px;z-index:2147483647;pointer-events:none;`
    + `font:650 12px/1.2 'Segoe UI',system-ui,sans-serif;color:#fff;background:rgba(18,12,26,.9);`
    + `border:1px solid rgba(255,255,255,.22);padding:6px 11px;border-radius:999px;box-shadow:0 4px 14px rgba(0,0,0,.35)">${esc(libelle)}</div>`;
  if (!html.includes('data-maquette-badge')) html = html.replace(/(<body[^>]*>)/, `$1\n${badge}`);
  writeFileSync(join(dest, nomArchive(e)), html);
  console.log(`[OK]   ${nomArchive(e)}`);
}

const connus = new Set(registre.map((e) => e.fichier));
for (const f of readdirSync(source).filter((x) => x.endsWith('.html') && !connus.has(x))) {
  console.warn(`[WARN] ${f} n'est pas au registre : pas de numero, pas archivee`);
}

const lignes = [...registre].reverse().map((e) => `
  <a class="m" href="${esc(nomArchive(e))}">
    <span class="n">${esc(num(e.n))}</span>
    <span class="t"><b>${esc(e.titre)}</b><small>${esc(e.resume || '')}</small></span>
    <span class="d">${esc(dateFr(e.date))}</span>
  </a>`).join('');
writeFileSync(join(dest, 'index.html'), `<!doctype html>
<html lang="fr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Maquettes Sindry</title>
<style>
  body { margin: 0; padding: 28px 16px 60px; background: #0E0A13; color: #F2ECF7; font-family: 'Segoe UI', system-ui, sans-serif; }
  main { max-width: 900px; margin: 0 auto; }
  h1 { font-size: 24px; margin: 0 0 4px; } p { color: #A99DB8; margin: 0 0 20px; font-size: 14px; }
  .m { display: grid; grid-template-columns: 56px 1fr auto; gap: 14px; align-items: center; padding: 12px 14px; margin-bottom: 8px;
       border-radius: 14px; background: #18111F; border: 1px solid #2C2236; color: inherit; text-decoration: none; }
  .m:hover { border-color: #FD9E21; }
  .n { font-weight: 800; font-size: 15px; color: #FFB35C; }
  .t b { display: block; font-size: 15px; } .t small { color: #A99DB8; font-size: 12.5px; }
  .d { color: #8C7B95; font-size: 12.5px; white-space: nowrap; }
  @media (max-width: 560px) { .m { grid-template-columns: 44px 1fr; } .d { grid-column: 2; } }
</style></head>
<body><main><h1>Maquettes</h1><p>${registre.length} maquettes, la plus récente en tête. Chacune porte son numéro dans l'onglet et un badge en bas à gauche.</p>${lignes}
</main></body></html>
`);
console.log(`[DONE] ${registre.length} maquettes dans ${dest}`);
