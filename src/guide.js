/**
 * Le guide (1.7) : mettre un element en avant quand c'est necessaire.
 *
 * Forme retenue (maquette M16) : la LAMPE TORCHE. Tout s'assombrit sauf un
 * cercle de lumiere aux bords doux, centre sur l'element, et une bulle explique
 * a quoi il sert. Le reste de l'ecran ne repond plus tant que la bulle est
 * ouverte ; l'element eclaire, lui, reste cliquable — dans une alerte, c'est
 * souvent lui qu'il faut toucher.
 *
 * Deux usages :
 *   - montrerGuide(etape) : une bulle seule, avec ses propres boutons (une
 *     alerte : « Relancer en administrateur », « Plus tard ») ;
 *   - lancerVisite(etapes) : une suite de bulles, Suivant / Precedent / Passer,
 *     les fleches du clavier. Une etape dont l'element est absent ou cache est
 *     sautee : une visite ne s'arrete pas sur du vide.
 *
 * Le guide ne s'ouvre jamais seul en plein travail : il repond a un geste.
 * Echap le ferme. Le contenu des visites vit dans src/guide.json.
 *
 * Une etape : { cible: Element, kick, titre, texte, alerte?: bool,
 *               actions?: [{ libelle, primaire?, icone?, faire? }] }
 */

let ouvert = null; // { etapes, i, libelles, calque, bulle, capte, torche }

/** L'echelle de rendu (`zoom` sur la racine, reglage 1.3) : getBoundingClientRect
 *  mesure a l'echelle de l'ecran, le calque est en pixels CSS. */
function echelle() {
  const e = Number(getComputedStyle(document.documentElement).getPropertyValue('--echelle'));
  return Number.isFinite(e) && e > 0 ? e : 1;
}

const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);

/** Un element qu'on peut eclairer : present, et affiche. */
const visible = (el) => !!el?.isConnected && el.getClientRects().length > 0 && el.getBoundingClientRect().width > 0;

/** Tout le calque sauf un rectangle arrondi : ce qui doit intercepter les clics. */
function cheminEvide(t, W, H) {
  const r = Math.min(14, t.h / 2), { x, y, w, h } = t;
  return `path(evenodd, "M0 0H${W}V${H}H0Z M${x + r} ${y}H${x + w - r}A${r} ${r} 0 0 1 ${x + w} ${y + r}V${y + h - r}A${r} ${r} 0 0 1 ${x + w - r} ${y + h}H${x + r}A${r} ${r} 0 0 1 ${x} ${y + h - r}V${y + r}A${r} ${r} 0 0 1 ${x + r} ${y}Z")`;
}

const icone = (nom) => `<svg class="ico i14" aria-hidden="true"><use href="#${nom}"/></svg>`;

/** Le contenu de la bulle pour l'etape courante. */
function rendreBulle() {
  const { etapes, i, libelles, bulle } = ouvert;
  const e = etapes[i];
  const n = etapes.length;
  const visite = !e.actions;
  const boutons = visite
    ? `${n > 1 ? `<span class="guide-points" aria-hidden="true">${etapes.map((_, k) => `<i class="${k === i ? 'on' : ''}"></i>`).join('')}</span>` : ''}
       ${i < n - 1 ? `<button class="btn lien" type="button" data-guide-nav="passer">${esc(libelles.passer)}</button>` : ''}
       ${i > 0 ? `<button class="btn" type="button" data-guide-nav="precedent">${esc(libelles.precedent)}</button>` : ''}
       <button class="btn primary" type="button" data-guide-nav="suivant">${esc(i === n - 1 ? libelles.terminer : libelles.suivant)}</button>`
    : e.actions.map((a, k) => `<button class="btn${a.primaire ? ' primary' : ''}" type="button" data-guide="${k}">${
      a.icone ? icone(a.icone) : ''}${esc(a.libelle)}</button>`).join('');
  bulle.classList.toggle('alerte', !!e.alerte);
  bulle.innerHTML = `<span class="guide-fleche"></span>
    <div class="guide-kick">${icone(e.alerte ? 'warn' : 'sparkles')}${esc(e.kick)}${visite && n > 1 ? ` · ${esc(libelles.etape(i + 1, n))}` : ''}</div>
    <h2 id="guideTitre">${esc(e.titre)}</h2>
    <p>${esc(e.texte)}</p>
    <div class="guide-bas">${boutons}</div>`;
}

function placer() {
  if (!ouvert) return;
  const { etapes, i, calque, bulle, capte, torche } = ouvert;
  const cible = etapes[i].cible;
  if (!visible(cible)) return fermerGuide();
  const z = echelle();
  const r = cible.getBoundingClientRect();
  const m = 6;
  const t = { x: r.left / z - m, y: r.top / z - m, w: r.width / z + 2 * m, h: r.height / z + 2 * m };
  const W = calque.clientWidth, H = calque.clientHeight;

  const cx = t.x + t.w / 2, cy = t.y + t.h / 2;
  // Un cercle pour un bouton, une ellipse pour une zone allongée (le champ de
  // bouées, une colonne) : un cercle qui l'engloberait éclairerait tout l'écran.
  const rx = t.w / 2 + 10, ry = t.h / 2 + 10;
  torche.style.background = rx > 2 * ry || ry > 2 * rx
    ? `radial-gradient(ellipse ${rx * 1.25}px ${ry * 1.25}px at ${cx}px ${cy}px, transparent 75%, var(--guide-voile) 100%)`
    : `radial-gradient(circle at ${cx}px ${cy}px, transparent ${Math.max(rx, ry)}px, var(--guide-voile) ${Math.max(rx, ry) + 70}px)`;
  capte.style.clipPath = cheminEvide(t, W, H);

  const bw = bulle.offsetWidth, bh = bulle.offsetHeight;
  // Sous l'element s'il y a la place, sinon au-dessus ; une zone trop haute
  // pour l'un et l'autre garde la bulle en son centre.
  let haut, fleche = true;
  if (t.y + t.h + 16 + bh < H - 8) { haut = t.y + t.h + 16; bulle.className = 'guide-bulle dessous'; }
  else if (t.y - 16 - bh > 8) { haut = t.y - 16 - bh; bulle.className = 'guide-bulle dessus'; }
  else { haut = Math.max(8, Math.min(H - bh - 8, cy - bh / 2)); bulle.className = 'guide-bulle'; fleche = false; }
  if (etapes[i].alerte) bulle.classList.add('alerte');
  const left = Math.max(12, Math.min(W - bw - 12, cx - bw / 2));
  bulle.style.left = `${left}px`;
  bulle.style.top = `${haut}px`;
  const f = bulle.querySelector('.guide-fleche');
  f.hidden = !fleche;
  f.style.left = `${Math.max(16, Math.min(bw - 30, cx - left - 7))}px`;
}

function aller(i) {
  if (!ouvert) return;
  if (i < 0 || i >= ouvert.etapes.length) return fermerGuide();
  ouvert.i = i;
  rendreBulle();
  // L'element peut etre hors de vue (une liste qui defile) : on l'y amene.
  ouvert.etapes[i].cible.scrollIntoView?.({ block: 'nearest', inline: 'nearest' });
  placer();
  (ouvert.bulle.querySelector('.guide-bas .primary') || ouvert.bulle.querySelector('.guide-bas button'))?.focus();
}

function surTouche(ev) {
  if (!ouvert) return;
  const visite = !ouvert.etapes[ouvert.i].actions;
  if (ev.key === 'Escape') { ev.preventDefault(); fermerGuide(); }
  else if (visite && ev.key === 'ArrowRight') { ev.preventDefault(); aller(ouvert.i + 1); }
  else if (visite && ev.key === 'ArrowLeft' && ouvert.i > 0) { ev.preventDefault(); aller(ouvert.i - 1); }
}

function ouvrir(etapes, libelles) {
  fermerGuide();
  const calque = document.createElement('div');
  calque.className = 'guide';
  calque.innerHTML = `<div class="guide-torche"></div><div class="guide-capte"></div>
    <div class="guide-bulle" role="dialog" aria-modal="true" aria-labelledby="guideTitre"></div>`;
  document.body.appendChild(calque);
  ouvert = {
    etapes, i: 0, libelles, calque,
    bulle: calque.querySelector('.guide-bulle'),
    capte: calque.querySelector('.guide-capte'),
    torche: calque.querySelector('.guide-torche'),
  };
  ouvert.bulle.addEventListener('click', (ev) => {
    const nav = ev.target.closest('[data-guide-nav]')?.dataset.guideNav;
    if (nav === 'passer') return fermerGuide();
    if (nav === 'suivant') return aller(ouvert.i + 1);
    if (nav === 'precedent') return aller(ouvert.i - 1);
    const b = ev.target.closest('[data-guide]');
    if (!b) return;
    const action = ouvert.etapes[ouvert.i].actions?.[Number(b.dataset.guide)];
    fermerGuide();
    action?.faire?.();
  });
  addEventListener('resize', placer);
  addEventListener('keydown', surTouche, true);
  aller(0);
}

/** Une bulle seule, avec ses propres boutons. Toucher l'element eclaire, c'est
 *  ce qu'on attendait : le guide s'efface. */
export function montrerGuide(etape) {
  if (!visible(etape?.cible)) return;
  ouvrir([etape], {});
  ouvert.ecoute = etape.cible;
  etape.cible.addEventListener('click', fermerGuide, { once: true });
}

/** Une visite. `libelles` : { passer, precedent, suivant, terminer,
 *  etape: (n, total) => texte } — le guide ne connait pas les langues. Rend
 *  le nombre d'etapes reellement montrees. */
export function lancerVisite(etapes, libelles) {
  const montrees = etapes.filter((e) => visible(e.cible));
  if (montrees.length) ouvrir(montrees, libelles);
  return montrees.length;
}

export function fermerGuide() {
  if (!ouvert) return;
  ouvert.ecoute?.removeEventListener('click', fermerGuide);
  ouvert.calque.remove();
  ouvert = null;
  removeEventListener('resize', placer);
  removeEventListener('keydown', surTouche, true);
}

export const guideOuvert = () => !!ouvert;
