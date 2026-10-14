/**
 * Banc d'essai de l'interface.
 *
 * Sindry impose `requireAdministrator` (§12.4) et ne peut donc pas etre lance
 * depuis un shell non eleve. Ce serveur sert `src/` tel quel et injecte
 * `fixtures.js` juste avant `main.js` : l'interface tourne alors dans un
 * navigateur ordinaire, avec un faux pont vers Rust.
 *
 *   node tools/bench/serve.mjs [port]
 *
 * Rien de ceci n'est embarque dans l'application : le banc vit a cote.
 */

import { createServer } from 'node:http';
import { readFile, readdir } from 'node:fs/promises';
import { extname, join, normalize, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const ICI = fileURLToPath(new URL('.', import.meta.url));
const RACINE = normalize(join(ICI, '..', '..', 'src'));
const PORT = Number(process.argv[2] || 8123);

const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.json': 'application/json; charset=utf-8',
  '.woff2': 'font/woff2',
  '.png': 'image/png',
};

const server = createServer(async (req, res) => {
  const url = new URL(req.url, `http://localhost:${PORT}`);
  let chemin = decodeURIComponent(url.pathname);
  if (chemin === '/') chemin = '/index.html';

  // Le faux pont vit hors de src/ pour ne jamais partir dans le bundle.
  if (chemin === '/__bench.js') {
    const corps = await readFile(join(ICI, 'fixtures.js'));
    res.writeHead(200, { 'content-type': TYPES['.js'] });
    return res.end(corps);
  }

  // Le vrai catalogue, tel que le moteur l'a lu et analyse (voir le test
  // `catalogue_reel_analyses`) : `?catalogue=reel`. Absent tant qu'on ne l'a
  // pas produit.
  if (chemin === '/__catalogue-reel.json') {
    try {
      const corps = await readFile(join(ICI, 'catalogue-reel.json'));
      res.writeHead(200, { 'content-type': TYPES['.json'] });
      return res.end(corps);
    } catch {
      res.writeHead(404);
      return res.end('[]');
    }
  }

  // Les ambiances du depot (§15.7) : `list_ambiances` les rend comme Rust le
  // ferait. Les trois de base d'office ; `?ambiances=toutes`, les autres aussi.
  if (chemin === '/__ambiances.json') {
    const toutes = url.searchParams.get('toutes') === '1';
    const contenus = [];
    for (const sous of toutes ? ['base', 'depot'] : ['base']) {
      const dossier = join(ICI, '..', '..', 'ambiances', sous);
      for (const f of (await readdir(dossier)).filter((x) => x.endsWith('.sindry-ambiance')).sort()) {
        contenus.push({ fichier: f, texte: await readFile(join(dossier, f), 'utf8') });
      }
    }
    res.writeHead(200, { 'content-type': TYPES['.json'] });
    return res.end(JSON.stringify(contenus));
  }

  const cible = normalize(join(RACINE, chemin));
  // Un chemin qui remonte hors de src/ est refuse : meme un banc local ne
  // sert pas le disque entier.
  if (!cible.startsWith(RACINE + sep) && cible !== RACINE) {
    res.writeHead(403);
    return res.end('hors racine');
  }

  try {
    let corps = await readFile(cible);
    if (chemin === '/index.html') {
      corps = String(corps).replace(
        '<script type="module" src="main.js">',
        '<script src="/__bench.js"></script>\n  <script type="module" src="main.js">'
      );
    }
    res.writeHead(200, { 'content-type': TYPES[extname(cible)] || 'application/octet-stream' });
    res.end(corps);
  } catch {
    res.writeHead(404);
    res.end('introuvable');
  }
});

server.listen(PORT, () => {
  console.log(`banc pret : http://localhost:${PORT}/  (racine ${RACINE})`);
});
