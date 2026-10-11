/**
 * Le guide (1.7) : mettre un element en avant quand c'est necessaire.
 *
 * Forme retenue : la LAMPE TORCHE. Tout s'assombrit sauf un cercle de lumiere
 * aux bords doux, centre sur l'element, et une bulle explique a quoi il sert.
 * Le reste de l'ecran ne repond plus tant que la bulle est ouverte ; l'element
 * eclaire, lui, reste cliquable — c'est souvent lui qu'il faut toucher.
 *
 * Le guide ne s'ouvre jamais seul en plein travail : il repond a un geste
 * (« Me montrer » dans un encart, une visite demandee). Echap le ferme.
 *
 * Une etape :
 *   { cible: Element, kick, titre, texte, alerte?: bool,
 *     actions: [{ libelle, primaire?: bool, icone?, faire?: () => void }] }
 * Une action sans `faire` ferme simplement le guide.
 */

let ouvert = null; // { etape, calque, bulle, capte, torche, fermer }

/** L'echelle de rendu (`zoom` sur la racine, reglage 1.6) : les mesures de
 *  getBoundingClientRect sont a l'echelle de l'ecran, le calque est en pixels
 *  CSS. */
function echelle() {
  const e = Number(getComputedStyle(document.documentElement).getPropertyValue('--echelle'));
  return Number.isFinite(e) && e > 0 ? e : 1;
}

const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);

/** Tout le calque sauf un rectangle arrondi : ce qui doit intercepter les clics. */
function cheminEvide(t, W, H) {
  const r = Math.min(14, t.h / 2), { x, y, w, h } = t;
  return `path(evenodd, "M0 0H${W}V${H}H0Z M${x + r} ${y}H${x + w - r}A${r} ${r} 0 0 1 ${x + w} ${y + r}V${y + h - r}A${r} ${r} 0 0 1 ${x + w - r} ${y + h}H${x + r}A${r} ${r} 0 0 1 ${x} ${y + h - r}V${y + r}A${r} ${r} 0 0 1 ${x + r} ${y}Z")`;
}

function placer() {
  if (!ouvert) return;
  const { etape, calque, bulle, capte, torche } = ouvert;
  if (!etape.cible.isConnected) return fermerGuide();
  const z = echelle();
  const r = etape.cible.getBoundingClientRect();
  const m = 6;
  const t = { x: r.left / z - m, y: r.top / z - m, w: r.width / z + 2 * m, h: r.height / z + 2 * m };
  const W = calque.clientWidth, H = calque.clientHeight;

  const cx = t.x + t.w / 2, cy = t.y + t.h / 2, rayon = Math.max(t.w, t.h) / 2 + 8;
  torche.style.background = `radial-gradient(circle at ${cx}px ${cy}px, transparent ${rayon}px, var(--guide-voile) ${rayon + 70}px)`;
  capte.style.clipPath = cheminEvide(t, W, H);

  const bw = bulle.offsetWidth, bh = bulle.offsetHeight;
  const dessous = t.y + t.h + 16 + bh < H - 8;
  bulle.classList.toggle('dessous', dessous);
  bulle.classList.toggle('dessus', !dessous);
  const left = Math.max(12, Math.min(W - bw - 12, cx - bw / 2));
  bulle.style.left = `${left}px`;
  bulle.style.top = `${dessous ? t.y + t.h + 16 : t.y - 16 - bh}px`;
  bulle.querySelector('.guide-fleche').style.left = `${Math.max(16, Math.min(bw - 30, cx - left - 7))}px`;
}

function surTouche(ev) {
  if (ev.key === 'Escape' && ouvert) {
    ev.preventDefault();
    fermerGuide();
  }
}

/** Eclaire `etape.cible` et ouvre sa bulle. Remplace un guide deja ouvert. */
export function montrerGuide(etape) {
  fermerGuide();
  if (!etape?.cible) return;
  const calque = document.createElement('div');
  calque.className = 'guide';
  calque.innerHTML = `<div class="guide-torche"></div><div class="guide-capte"></div>
    <div class="guide-bulle${etape.alerte ? ' alerte' : ''}" role="dialog" aria-modal="true" aria-labelledby="guideTitre">
      <span class="guide-fleche"></span>
      <div class="guide-kick"><svg class="ico i14" aria-hidden="true"><use href="#${etape.alerte ? 'warn' : 'sparkles'}"/></svg>${esc(etape.kick)}</div>
      <h2 id="guideTitre">${esc(etape.titre)}</h2>
      <p>${esc(etape.texte)}</p>
      <div class="guide-bas">${(etape.actions || []).map((a, i) => `<button class="btn${a.primaire ? ' primary' : ''}" type="button" data-guide="${i}">${
        a.icone ? `<svg class="ico i14" aria-hidden="true"><use href="#${a.icone}"/></svg>` : ''}${esc(a.libelle)}</button>`).join('')}</div>
    </div>`;
  document.body.appendChild(calque);
  ouvert = {
    etape, calque,
    bulle: calque.querySelector('.guide-bulle'),
    capte: calque.querySelector('.guide-capte'),
    torche: calque.querySelector('.guide-torche'),
  };
  calque.querySelector('.guide-bas').addEventListener('click', (ev) => {
    const b = ev.target.closest('[data-guide]');
    if (!b) return;
    const action = etape.actions[Number(b.dataset.guide)];
    fermerGuide();
    action?.faire?.();
  });
  // Toucher l'element eclaire, c'est ce qu'on attendait : le guide s'efface.
  etape.cible.addEventListener('click', fermerGuide, { once: true });
  addEventListener('resize', placer);
  addEventListener('keydown', surTouche, true);
  placer();
  (calque.querySelector('.guide-bas .primary') || calque.querySelector('.guide-bas button'))?.focus();
}

export function fermerGuide() {
  if (!ouvert) return;
  ouvert.etape.cible.removeEventListener('click', fermerGuide);
  ouvert.calque.remove();
  ouvert = null;
  removeEventListener('resize', placer);
  removeEventListener('keydown', surTouche, true);
}

export const guideOuvert = () => !!ouvert;
