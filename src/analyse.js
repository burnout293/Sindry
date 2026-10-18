/**
 * L'analyse (specification §17) dans l'interface : ce que les scripts ont
 * trouve, ce qu'on coche, et ce qui repartira vers eux.
 *
 * Le moteur lit la sortie de chaque analyse contre l'entete du script
 * (src-tauri/src/analyse.rs) et la publie en `script:analysis`. Ce module en
 * fait l'ecran :
 *   - en mode Simple, un resume : des postes que le script compose
 *     ([group:…]), un graphique, et un chevron pour deplier chaque poste ;
 *   - en mode Expert, le detail d'une action, dans la vue qu'elle a choisie
 *     (## view), avec ses panneaux.
 * Il tient aussi la selection, et calcule ce que Sindry renverra au script
 * dans SINDRY_CONFIG.
 *
 * Le script n'ecrit jamais de phrase. Les mots viennent de son entete
 * (OPTIONS, REPORT, LANG) et des textes de Sindry (i18n). Un element marque
 * `show=expert` ne s'affiche pas en mode Simple, mais il garde sa case : il
 * suit celle qui le contient, et revient a l'avis du script.
 *
 * Maquette de reference : docs/mockups/analyse.html.
 */

import { t, currentLang } from './i18n.js';

const ECHAPPE = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' };
const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ECHAPPE[c]);
const ico = (nom, classe = '') => `<svg class="ico ${classe}" aria-hidden="true"><use href="#${nom}" /></svg>`;

/** Couleurs des series : les six du theme (`--serie-1` a `--serie-6`, tirees
 *  de l'icone), puis quelques autres au-dela, pour qu'un long resume ne
 *  repete pas ses couleurs trop tot. Relues quand la variante ou le theme
 *  changent. */
const SERIES_EN_PLUS = ['#2DD4BF', '#A3E635', '#F472B6', '#60A5FA', '#94A3B8', '#C084FC'];
let paletteMemo = { cle: null, couleurs: [] };
function palette() {
  const r = document.documentElement;
  // Une ambiance pose ses couleurs en ligne sur la racine (2.0.1) : la cle les
  // lit la, sans recalcul de style — un changement d'ambiance ou de mode la
  // change.
  const cle = `${r.style.getPropertyValue('--serie-1')}|${r.style.getPropertyValue('--serie-2')}|${r.dataset.theme || ''}|${matchMedia('(prefers-color-scheme: dark)').matches}`;
  if (paletteMemo.cle !== cle) {
    const cs = getComputedStyle(r);
    const lues = [1, 2, 3, 4, 5, 6].map((i) => cs.getPropertyValue(`--serie-${i}`).trim()).filter((c) => /^#[0-9a-f]{6}$/i.test(c));
    paletteMemo = { cle, couleurs: [...lues, ...SERIES_EN_PLUS] };
  }
  return paletteMemo.couleurs;
}
const serie = (i) => { const p = palette(); return p[i % p.length]; };

/** La couleur d'une flamme a la position t (0 : premiere serie, 1 : troisieme) :
 *  les dents de la jauge s'allument de l'or au magenta. */
function flamme(t) {
  const [a, b, c] = palette();
  const melange = (h1, h2, u) => {
    const v = (h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16));
    const [x, y] = [v(h1), v(h2)];
    return `#${x.map((k, i) => Math.round(k + (y[i] - k) * u).toString(16).padStart(2, '0')).join('')}`;
  };
  return t < 0.5 ? melange(a, b, t * 2) : melange(b, c, (t - 0.5) * 2);
}

/* --- Geometrie des graphiques a engrenages (1.6) -------------------------- */

/** Un engrenage : `dents` dents, rayon exterieur r1, interieur r2. */
function engrenage(cx, cy, r1, r2, dents) {
  const pas = (Math.PI * 2) / dents, pts = [];
  for (let i = 0; i < dents; i++) {
    const a = i * pas;
    for (const [ang, r] of [[a - pas * 0.22, r2], [a - pas * 0.14, r1], [a + pas * 0.14, r1], [a + pas * 0.22, r2]]) {
      pts.push(`${(cx + r * Math.cos(ang)).toFixed(1)},${(cy + r * Math.sin(ang)).toFixed(1)}`);
    }
  }
  return `M${pts.join('L')}Z`;
}
/** Une dent seule (trapeze), la i-eme sur n, en partant du haut. */
function dent(cx, cy, r1, r2, i, n) {
  const pas = (Math.PI * 2) / n, a = -Math.PI / 2 + i * pas;
  return `M${[[a - pas * 0.3, r2], [a - pas * 0.19, r1], [a + pas * 0.19, r1], [a + pas * 0.3, r2]]
    .map(([ang, r]) => `${(cx + r * Math.cos(ang)).toFixed(1)},${(cy + r * Math.sin(ang)).toFixed(1)}`).join('L')}Z`;
}
/** Un arc de couronne, de l'angle a a l'angle b (radians), d'epaisseur e. */
function arcCouronne(cx, cy, r, a, b, e) {
  const ri = r - e / 2, re = r + e / 2, grand = b - a > Math.PI ? 1 : 0;
  const P = (rr, u) => `${(cx + rr * Math.cos(u)).toFixed(1)},${(cy + rr * Math.sin(u)).toFixed(1)}`;
  return `M${P(re, a)}A${re},${re} 0 ${grand} 1 ${P(re, b)}L${P(ri, b)}A${ri},${ri} 0 ${grand} 0 ${P(ri, a)}Z`;
}

/** Graphiques du resume Simple : la cle du reglage, l'ordre de la liste. */
export const GRAPHIQUES = ['donut', 'bar', 'rows', 'treemap', 'waffle', 'gauge'];

/* -------------------------------------------------------------------------
   Formats, dans la langue de l'interface
   ------------------------------------------------------------------------- */

const locale = () => (currentLang() === 'en' ? 'en-US' : 'fr-FR');
const nb = (n) => Number(n).toLocaleString(locale());

/** « 1,1 Go », « 521 Mo » : jamais `size=1181116006` (§3). */
export function taille(octets) {
  const n = Number(octets) || 0;
  const u = currentLang() === 'en' ? ['bytes', 'KB', 'MB', 'GB', 'TB'] : ['octets', 'ko', 'Mo', 'Go', 'To'];
  if (n < 1024) return `${nb(n)} ${u[0]}`;
  let v = n / 1024;
  let i = 1;
  while (v >= 1024 && i < u.length - 1) { v /= 1024; i += 1; }
  return `${v.toLocaleString(locale(), { maximumFractionDigits: v < 10 && i >= 3 ? 1 : 0 })} ${u[i]}`;
}
const pl = (cle, n) => t(cle, { n: nb(n), s: n > 1 ? 's' : '' });
const dateCourte = (iso) => {
  const d = new Date(`${String(iso).slice(0, 10)}T00:00:00`);
  return Number.isNaN(d.getTime()) ? String(iso) : d.toLocaleDateString(locale(), { day: 'numeric', month: 'short', year: 'numeric' });
};
const UNITES = {
  celsius: (v) => `${v} °C`, pct: (v) => `${v} %`, hours: (v) => `${nb(v)} h`, days: (v) => pl('an.unite.jours', Number(v)),
  s: (v) => `${v} s`, count: (v) => nb(v), cycles: (v) => pl('an.unite.cycles', Number(v)),
};

/* -------------------------------------------------------------------------
   Les mots du script : son entete, traduite
   ------------------------------------------------------------------------- */

const trad = (entree) => entree.meta.translations?.[currentLang()];
export function titreAction(entree) {
  return trad(entree)?.title || entree.meta.title || entree.path;
}
function libOption(entree, opt) {
  return trad(entree)?.options?.[opt.key]?.[0] || opt.label || opt.key;
}
function descOption(entree, opt) {
  return trad(entree)?.options?.[opt.key]?.[1] || opt.desc || '';
}
function libChoix(entree, opt, valeur) {
  const c = opt.choices.find((x) => x.value === valeur);
  return trad(entree)?.choices?.[`${opt.key}/${valeur}`]?.[0] || c?.label || valeur;
}
/** Une entree du bloc REPORT, traduite : libelle, note ou groupe. */
function rapport(entree, cle) {
  const r = entree.meta.report?.[cle];
  if (!r) return null;
  const tr = trad(entree)?.report?.[cle];
  return { ...r, label: tr?.[0] || r.label, desc: (tr && tr[1]) || r.desc };
}

/* -------------------------------------------------------------------------
   Le modele : une analyse, rangee option par option
   ------------------------------------------------------------------------- */

/**
 * `analyse` : `analyse::Analyse`, tel que publie par `script:analysis`.
 * Le modele garde l'entree du script : c'est elle qui donne les mots.
 */
export function modele(entree, analyse, succes = true) {
  const m = {
    entree,
    succes,
    options: {},
    metriques: (analyse.metrics || []).map((x) => ({ cle: x.key, ...x.fields })),
    notes: (analyse.notes || []).map((n) => ({ cle: n.key, cible: n.target || null, show: n.show || '' })),
    etapes: analyse.steps || [],
    anomalies: analyse.anomalies || [],
    tronque: !!analyse.truncated,
  };
  for (const o of entree.meta.options || []) m.options[o.key] = { def: o, find: null, finds: {}, items: [] };
  for (const f of analyse.finds || []) {
    const o = m.options[f.option];
    if (!o) continue;
    if (f.choice) o.finds[f.choice] = f.fields;
    else o.find = f.fields;
  }
  for (const it of analyse.items || []) m.options[it.option]?.items.push(it.fields);
  return m;
}

/** Ce qu'un mode montre : `show=expert` se cache en Simple, `show=simple` en Expert. */
export const montre = (x, mode) => !x?.show || x.show === mode;
const choixDe = (o, valeur) => o.def.choices.find((c) => c.value === valeur);
/** Un choix ou une option qu'une etiquette [show:…] reserve a l'autre mode. */
const visibleDecl = (o, valeur, mode) => montre(o.def, mode) && (valeur == null || montre(choixDe(o, valeur), mode));
const enfantsDe = (o, id) => o.items.filter((it) => it.parent === id);
const feuilles = (o) => o.items.filter((it) => !enfantsDe(o, it.id).length && !it.locked && it.state !== 'ok');
const racines = (o) => o.items.filter((it) => !it.parent || !o.items.some((x) => x.id === it.parent));
/** The elements of a list: everything that is not a parent. */
const elementsDe = (o) => o.items.filter((it) => !enfantsDe(o, it.id).length);

/* -------------------------------------------------------------------------
   Long lists (2.0.4, mockups M31 to M33): beyond LISTE_LONGUE elements, the
   [items] lists of an action open folded, under a bar to search them, show
   them by category or A → Z, and sort them by name or publisher.
   ------------------------------------------------------------------------- */

const LISTE_LONGUE = 12;
/** The [items] lists drawn as a list — not a table, tiles, a treemap or
 *  duplicates — minus those a view already shows (`exclues`). */
const listesAffichees = (m, exclues = new Set()) => Object.entries(m.options).filter(([k, o]) =>
  o.def.kind === 'items' && !o.def.scan && !o.def.keep_one && !['table', 'tiles', 'treemap'].includes(o.def.view) && !exclues.has(k));
const estLongue = (m, exclues) => listesAffichees(m, exclues).reduce((n, [, o]) => n + elementsDe(o).length, 0) > LISTE_LONGUE;

/** What is searched, how it is shown and sorted, per action. Nothing here
 *  goes to the script. */
const LISTES = new Map();
function etatListes(m) {
  let e = LISTES.get(m.entree.id);
  if (!e) { e = { q: '', affichage: 'arbre', tri: 'nom' }; LISTES.set(m.entree.id, e); }
  return e;
}
/** Lowercase without accents, one character for one: a match found in the
 *  folded text sits at the same place in the original. */
const plier = (s) => [...String(s)].map((c) => c.normalize('NFD')[0].toLowerCase()).join('');
function surligne(texte, q) {
  const i = q ? plier(texte).indexOf(q) : -1;
  if (i < 0) return esc(texte);
  const chars = [...texte];
  return `${esc(chars.slice(0, i).join(''))}<mark>${esc(chars.slice(i, i + [...q].length).join(''))}</mark>${esc(chars.slice(i + [...q].length).join(''))}`;
}
/** Is this parent folded? Its user's choice, else folded in a long list. */
function estPlie(m, k, id) {
  return selection(m).plie[`${k}/${id}`] ?? estLongue(m);
}

/** The bar above long lists: search, display, sort. */
function barreListes(m, exclues) {
  if (!estLongue(m, exclues)) return '';
  const e = etatListes(m);
  const id = esc(m.entree.id);
  const listes = listesAffichees(m, exclues);
  const avecParents = listes.some(([, o]) => o.items.some((it) => it.parent));
  const avecEditeurs = listes.some(([, o]) => o.items.some((it) => it.publisher));
  const seg = (attr, val, lib, actif) => `<button type="button" data-a-script="${id}" ${attr}="${val}" aria-pressed="${actif}">${esc(lib)}</button>`;
  return `<div class="an-barre-listes">
      <span class="an-cherche-liste">${ico('search', 'i14')}<input type="search" class="an-cherche" data-a-script="${id}" data-a-cherche-liste
        placeholder="${esc(t('an.chercher_liste'))}" aria-label="${esc(t('an.chercher_liste'))}" value="${esc(e.q)}"></span>
      ${avecParents ? `<span class="an-seg"><span>${esc(t('an.afficher'))}</span>${seg('data-a-affichage', 'arbre', t('an.par_categories'), e.affichage === 'arbre')}${seg('data-a-affichage', 'plat', t('an.liste_az'), e.affichage === 'plat')}</span>` : ''}
      ${avecEditeurs ? `<span class="an-seg"><span>${esc(t('an.trier_par'))}</span>${seg('data-a-tri-liste', 'nom', t('an.tri_nom'), e.tri === 'nom')}${seg('data-a-tri-liste', 'editeur', t('an.tri_editeur'), e.tri === 'editeur')}</span>` : ''}
    </div>`;
}

/* -------------------------------------------------------------------------
   La selection : ce qui repartira vers le script
   ------------------------------------------------------------------------- */

const SEL = new Map();

/** L'avis du script, sinon la regle de Sindry : ce qui pese ou est a faire. */
const coche = (c) => (c.checked !== undefined ? c.checked === 'true'
  : c.state === 'todo' || (c.state !== 'ok' && (Number(c.size) > 0 || Number(c.count) > 0)));
/** Un element « A verifier » (confidence=low) n'est jamais coche d'office. */
const cocheItem = (it) => (it.checked !== undefined ? it.checked === 'true' && !it.locked
  : !it.locked && it.state !== 'ok' && it.confidence !== 'low');

/** Une nouvelle analyse remet la selection a l'avis du script. */
export function oublier(scriptId) {
  SEL.delete(scriptId);
}

export function selection(m) {
  const id = m.entree.id;
  let s = SEL.get(id);
  if (!s) { s = { bool: {}, choix: {}, select: {}, items: {}, garder: {}, supprimer: {}, filtre: {}, plie: {} }; SEL.set(id, s); }
  for (const [k, o] of Object.entries(m.options)) {
    const d = o.def;
    if (d.scan) continue;
    if (d.kind === 'bool' && o.find && s.bool[k] === undefined) s.bool[k] = coche(o.find) && o.find.state !== 'ok';
    if (d.kind === 'multi') {
      s.choix[k] ||= {};
      for (const [c, f] of Object.entries(o.finds)) if (s.choix[k][c] === undefined) s.choix[k][c] = coche(f);
    }
    if (d.kind === 'select' && s.select[k] === undefined && Object.keys(o.finds).length) {
      const e = Object.entries(o.finds);
      s.select[k] = (e.find(([, f]) => f.recommended === 'true') || e.find(([, f]) => f.current === 'true') || e[0])[0];
    }
    if (d.kind === 'items') {
      s.items[k] ||= {};
      for (const it of o.items) if (s.items[k][it.id] === undefined) s.items[k][it.id] = cocheItem(it);
      if (d.keep_one) {
        s.garder[k] ||= {};
        if (s.supprimer[k] === undefined) s.supprimer[k] = true;
        for (const g of new Set(o.items.map((it) => it.group || it.id))) {
          if (o.items.some((x) => x.id === s.garder[k][g])) continue;
          const membres = o.items.filter((it) => (it.group || it.id) === g);
          s.garder[k][g] = (membres.find((it) => it.keep === 'true') || membres[0]).id;
        }
      }
    }
  }
  return s;
}

/** Ce qu'un `[items:keep-one]` supprimera : tout sauf l'exemplaire garde. */
function aSupprimer(m, k) {
  const s = selection(m);
  const o = m.options[k];
  if (!s.supprimer[k]) return [];
  return o.items.filter((it) => s.garder[k][it.group || it.id] !== it.id && !it.locked);
}

/**
 * Ce que Sindry posera dans SINDRY_CONFIG pour l'action, d'apres les cases.
 * Seuls les choix que l'analyse a rapportes peuvent etre coches : un choix
 * qu'elle n'a pas montre n'est jamais traite.
 */
export function configAnalyse(m) {
  const s = selection(m);
  const cfg = {};
  for (const [k, o] of Object.entries(m.options)) {
    const d = o.def;
    if (d.scan) continue;
    if (d.kind === 'bool' && o.find) cfg[k] = !!s.bool[k];
    else if (d.kind === 'multi' && Object.keys(o.finds).length) cfg[k] = Object.keys(o.finds).filter((c) => s.choix[k][c]);
    else if (d.kind === 'select' && s.select[k]) cfg[k] = s.select[k];
    else if (d.kind === 'items') {
      cfg[k] = d.keep_one ? aSupprimer(m, k).map((it) => it.id) : feuilles(o).filter((it) => s.items[k][it.id]).map((it) => it.id);
    }
  }
  return cfg;
}

/**
 * Y a-t-il quelque chose a faire ? Sans rien de coche, Sindry ne relance pas
 * le script : le lancer pour rien ne ferait que l'inscrire « fait » dans
 * l'historique.
 */
export function aFaire(m) {
  const s = selection(m);
  for (const [k, o] of Object.entries(m.options)) {
    const d = o.def;
    if (d.scan) continue;
    if (d.kind === 'bool' && o.find && s.bool[k]) return true;
    if (d.kind === 'multi' && Object.keys(o.finds).some((c) => s.choix[k][c])) return true;
    if (d.kind === 'select' && s.select[k]) {
      const actuel = Object.entries(o.finds).find(([, f]) => f.current === 'true')?.[0];
      if (s.select[k] !== actuel) return true;
    }
    if (d.kind === 'items' && (d.keep_one ? aSupprimer(m, k).length : feuilles(o).some((it) => s.items[k][it.id]))) return true;
  }
  return false;
}

/** Trouve, coche (octets), elements sans taille, reglages. */
export function totaux(m) {
  const s = selection(m);
  let trouve = 0, choisi = 0, elements = 0, reglages = 0;
  for (const [k, o] of Object.entries(m.options)) {
    const d = o.def;
    if (d.scan) continue;
    if (d.kind === 'bool' && o.find) {
      const v = Number(o.find.size) || 0;
      if (o.find.state !== 'ok') trouve += v;
      if (s.bool[k]) { choisi += v; if (!v) { if (o.find.state === 'todo') reglages += 1; else elements += Number(o.find.count) || 1; } }
    }
    if (d.kind === 'multi') for (const [c, f] of Object.entries(o.finds)) { const v = Number(f.size) || 0; trouve += v; if (s.choix[k][c]) choisi += v; }
    if (d.kind === 'select' && Object.keys(o.finds).length) {
      const actuel = Object.entries(o.finds).find(([, f]) => f.current === 'true')?.[0];
      if (s.select[k] && s.select[k] !== actuel) reglages += 1;
    }
    if (d.kind === 'items') {
      if (d.keep_one) {
        for (const it of aSupprimer(m, k)) { const v = Number(it.size) || 0; trouve += v; choisi += v; }
      } else {
        for (const it of feuilles(o)) {
          const v = Number(it.size) || 0;
          trouve += v;
          if (s.items[k][it.id]) { if (v) choisi += v; else elements += 1; }
        }
      }
    }
  }
  return { trouve, choisi, elements, reglages };
}

export function resumeTotaux(tt, m) {
  const morceaux = [];
  if (tt.trouve) morceaux.push(t('an.coches_sur', { choisi: taille(tt.choisi), trouve: taille(tt.trouve) }));
  if (tt.reglages) morceaux.push(pl('an.reglages_a_appliquer', tt.reglages));
  if (tt.elements) morceaux.push(pl('an.elements_coches', tt.elements));
  if (!morceaux.length) return m.metriques.length ? t('an.pour_information') : t('an.rien_a_faire');
  return morceaux.join(' · ');
}

/* -------------------------------------------------------------------------
   Les briques : une option, rendue comme une liste a cocher
   ------------------------------------------------------------------------- */

const attrs = (m, k, extra = '') => `data-a-script="${esc(m.entree.id)}" data-a-opt="${esc(k)}" ${extra}`;

const JETONS = {
  confidence: { high: ['an.conf.high', 'low'], medium: ['an.conf.medium', 'neutre'], low: ['an.conf.low', 'med'] },
  impact: { high: ['an.impact.high', 'high'], medium: ['an.impact.medium', 'med'], low: ['an.impact.low', 'neutre'] },
  risk: { high: ['an.risk.high', 'high'], medium: ['an.risk.medium', 'med'] },
  health: { ok: ['an.health.ok', 'ok'], warn: ['an.health.warn', 'warn'], crit: ['an.health.crit', 'crit'] },
};
/** Application icons downloaded by Sindry from the Sindry-Icons repository
 *  (`appicon.rs`, 2.0.4), by name. They arrive after the screen is shown. */
const ICONES_APP = new Map();
const IMAGE_ICONE = /^data:image\/(png|x-icon);base64,[A-Za-z0-9+/=]+$/;
const NOM_LUCIDE = /^[a-z0-9]+(-[a-z0-9]+)*$/;

/** An icon arrived: kept for the next drawings, and put in place at once
 *  where its fallback is shown, without redrawing the screen. */
export function iconeApp(nom, url) {
  if (!url || !IMAGE_ICONE.test(url)) return;
  ICONES_APP.set(nom, url);
  document.querySelectorAll(`[data-appicon="${CSS.escape(nom)}"]`).forEach((el) => {
    el.outerHTML = `<img class="an-ico-app" src="${url}" alt="" width="20" height="20">`;
  });
}

/** Every `appicon=` an analysis names: all of them are fetched, whatever the
 *  state of their item, so that the server cannot tell what is installed. */
export function nomsIconesApp(modeles) {
  const noms = new Set();
  for (const m of modeles) for (const o of Object.values(m.options)) for (const it of o.items) if (it.appicon) noms.add(it.appicon);
  return [...noms];
}

/** The icon of an item, from the most to the least specific:
 *  - the real icon of the program, read by Sindry in the file `exe=` names
 *    (`icon.rs`, 2.0.3) — never an image from the script;
 *  - the official icon `appicon=` names, downloaded from the Sindry-Icons
 *    repository (2.0.4) — until it arrives, the fallback below waits in its
 *    place;
 *  - the Lucide icon `icon=` names (2.0.4), drawn as a mask so that it takes
 *    the colour of the text;
 *  - the icon of its kind. */
function iconeElement(it) {
  if (typeof it.image === 'string' && IMAGE_ICONE.test(it.image)) {
    return `<img class="an-ico-app" src="${it.image}" alt="" width="20" height="20">`;
  }
  const telechargee = it.appicon && ICONES_APP.get(it.appicon);
  if (telechargee) return `<img class="an-ico-app" src="${telechargee}" alt="" width="20" height="20">`;
  const repli = typeof it.icon === 'string' && NOM_LUCIDE.test(it.icon)
    ? `<span class="an-ico-luc" style="-webkit-mask-image:url(icons/${it.icon}.svg);mask-image:url(icons/${it.icon}.svg)"></span>`
    : ico(ICONE_GENRE[it.kind] || 'file', 'i14');
  return it.appicon ? `<span class="an-ico-attente" data-appicon="${esc(it.appicon)}">${repli}</span>` : repli;
}
const ICONE_GENRE = { folder: 'folder', file: 'file', registry: 'database', app: 'package', startup: 'power', service: 'settings', task: 'calendar-clock', driver: 'cpu', browser: 'globe' };
const badge = (type, v) => {
  const j = JETONS[type]?.[v];
  return j ? `<span class="badge ${j[1]}">${esc(t(j[0]))}</span>` : '';
};
const verrouTexte = (v) => t(`an.locked.${['open', 'system', 'protected', 'inuse'].includes(v) ? v : 'system'}`);

function noteHtml(m, n) {
  const r = rapport(m.entree, n.cle);
  if (!r) return '';
  return `<div class="an-note ${r.level === 'warn' ? 'warn' : 'info'}">${ico(r.level === 'warn' ? 'warn' : 'info', 'i14')}<span>${esc(r.label)}</span></div>`;
}
function notesPour(m, cible, mode) {
  return m.notes.filter((n) => n.cible === cible && montre(n, mode)).map((n) => noteHtml(m, n)).join('');
}

/** Le nom d'un element : label= dit ce que c'est (traduit), name= lequel. */
function nomElement(m, it) {
  const lib = it.label ? rapport(m.entree, it.label)?.label || it.label : '';
  return lib && it.name ? `${lib} — ${it.name}` : lib || it.name || it.id;
}

function brique(m, k, mode) {
  const o = m.options[k];
  const s = selection(m);
  const d = o.def;
  if (d.scan || !visibleDecl(o, null, mode)) return '';
  const lib = libOption(m.entree, d);
  if (d.kind === 'bool') {
    if (!o.find || !montre(o.find, mode)) return '';
    const f = o.find;
    const fait = f.state === 'ok';
    const etat = f.state === 'todo' ? `<span class="badge acc">${esc(t('an.a_faire'))}</span>` : fait ? `<span class="badge low">${esc(t('an.deja_en_place'))}</span>` : '';
    const meta = f.count && !f.state ? `<span class="muted">${esc(pl('an.n_elements', Number(f.count)))}</span>` : '';
    return `<label class="an-ligne${fait ? ' inactive' : ''}${s.bool[k] ? '' : ' decoche'}">
        <input type="checkbox" ${attrs(m, k, 'data-a-k="bool"')}${s.bool[k] ? ' checked' : ''}${fait ? ' disabled' : ''}>
        <span class="an-t"><b>${esc(lib)}</b><span class="an-badges">${etat}${meta}</span></span>
        <span class="an-v">${Number(f.size) > 0 ? taille(f.size) : ''}</span></label>${notesPour(m, k, mode)}`;
  }
  if (d.kind === 'multi') {
    if (d.view === 'bars' || d.view === 'donut') return d.view === 'bars' ? vueBarres(m, k, mode) : vueAnneau(m, k, mode);
    const entrees = Object.entries(o.finds).filter(([c, f]) => montre(f, mode) && visibleDecl(o, c, mode))
      .sort((a, b) => (Number(b[1].size) || 0) - (Number(a[1].size) || 0));
    if (!entrees.length) return '';
    const max = Math.max(1, ...entrees.map(([, f]) => Number(f.size) || 0));
    const total = Object.entries(o.finds).reduce((n, [c, f]) => n + (s.choix[k][c] ? Number(f.size) || 0 : 0), 0);
    const lignes = entrees.map(([c, f]) => {
      const vide = !(Number(f.size) > 0) && !(Number(f.count) > 0);
      return `<label class="an-ligne${vide ? ' inactive' : ''}${s.choix[k][c] ? '' : ' decoche'}">
          <input type="checkbox" ${attrs(m, k, `data-a-k="choix" data-a-choix="${esc(c)}"`)}${s.choix[k][c] ? ' checked' : ''}${vide ? ' disabled' : ''}>
          <span class="an-t"><b>${esc(libChoix(m.entree, d, c))}</b>
            ${vide ? `<span class="muted">${esc(t('an.rien_a_nettoyer'))}</span>` : `<span class="an-jauge"><i style="width:${Math.max(1.5, (Number(f.size) / max) * 100)}%"></i></span>`}</span>
          <span class="an-v">${vide ? '' : taille(f.size)}${f.count && !vide ? `<small>${esc(pl('an.n_fichiers', Number(f.count)))}</small>` : ''}</span>
        </label>${notesPour(m, `${k}.${c}`, mode)}`;
    }).join('');
    return `<div class="an-bloc"><div class="an-bloc-h"><span class="an-bh-t">${esc(lib)}</span>
        <button class="an-lien" type="button" ${attrs(m, k, 'data-a-tout="multi"')}>${esc(t('an.tout_cocher'))}</button>
        <span class="an-total">${esc(t('an.n_coches', { taille: taille(total) }))}</span></div>${lignes}${notesPour(m, k, mode)}</div>`;
  }
  if (d.kind === 'select') {
    if (d.view === 'chart') return vueColonnes(m, k);
    const entrees = Object.entries(o.finds).filter(([c, f]) => montre(f, mode) && visibleDecl(o, c, mode));
    if (!entrees.length) return '';
    const max = Math.max(1, ...entrees.map(([, f]) => Number(f.ms) || 0));
    const lignes = entrees.sort((a, b) => (Number(a[1].ms) || 0) - (Number(b[1].ms) || 0)).map(([c, f]) => `
        <label class="an-ligne${s.select[k] === c ? '' : ' decoche'}">
          <input type="radio" name="an-${esc(m.entree.id)}-${esc(k)}" ${attrs(m, k, `data-a-k="select" data-a-choix="${esc(c)}"`)}${s.select[k] === c ? ' checked' : ''}>
          <span class="an-t"><b>${esc(libChoix(m.entree, d, c))}</b>
            <span class="an-badges">${f.current === 'true' ? `<span class="badge neutre">${esc(t('an.actuel'))}</span>` : ''}${f.recommended === 'true' ? `<span class="badge acc">${esc(t('an.recommande'))}</span>` : ''}</span>
            ${f.ms ? `<span class="an-jauge"><i style="width:${(Number(f.ms) / max) * 100}%"></i></span>` : ''}</span>
          <span class="an-v">${f.ms ? `${nb(f.ms)} ms` : ''}</span></label>`).join('');
    return `<div class="an-bloc"><div class="an-bloc-h"><span class="an-bh-t">${esc(lib)}</span>${entrees.some(([, f]) => f.ms) ? `<span class="an-total">${esc(t('an.plus_court'))}</span>` : ''}</div>${lignes}${notesPour(m, k, mode)}</div>`;
  }
  if (d.kind === 'items') {
    if (d.keep_one) return briqueGarder(m, k, mode);
    if (d.view === 'table') return vueTableau(m, k);
    if (d.view === 'tiles') return vueTuiles(m, k, mode);
    if (d.view === 'treemap') return vueTreemap(m, k);
    return briqueListe(m, k, mode);
  }
  return '';
}

/**
 * One element of an [items] list, and its children. `vue` (2.0.4): `q`, the
 * search, highlighted in the name; `montres`, the children to show when a
 * search or a filter keeps only some; `deplie`, forced open by a search;
 * `trier`, the order of the children; `parent`, the category shown beside
 * the name in the A → Z list.
 */
function ligneItem(m, k, it, mode, vue = {}) {
  const o = m.options[k];
  const s = selection(m);
  const enfants = enfantsDe(o, it.id);
  const visibles = enfants.filter((e) => montre(e, mode) && (!vue.montres || vue.montres.has(e.id)));
  if (vue.trier) visibles.sort(vue.trier);
  const verrou = !!it.locked;
  const fait = it.state === 'ok';
  let cochee = !!s.items[k][it.id];
  let partiel = false;
  if (enfants.length) {
    // Ce que ce mode cache n'entre pas dans l'etat affiche du parent.
    const e = enfants.filter((x) => !x.locked && x.state !== 'ok' && montre(x, mode));
    const n = e.filter((x) => s.items[k][x.id]).length;
    cochee = e.length > 0 && n === e.length;
    partiel = n > 0 && n < e.length;
  }
  const sousTotal = enfants.length ? enfants.reduce((n, e) => n + (Number(e.size) || 0), 0) : Number(it.size) || 0;
  const badges = [
    badge('confidence', it.confidence), badge('impact', it.impact), badge('risk', it.risk),
    it.to ? `<span class="badge acc">${esc(it.version || '')} → ${esc(it.to)}</span>` : it.version ? `<span class="badge neutre">${esc(it.version)}</span>` : '',
    fait ? `<span class="badge low">${esc(t('an.a_jour'))}</span>` : '',
    verrou ? `<span class="badge neutre">${ico('lock', 'i12')}${esc(verrouTexte(it.locked))}</span>` : '',
    it.date ? `<span class="an-pub">${esc(dateCourte(it.date))}</span>` : '',
    it.publisher ? `<span class="an-pub">${surligne(it.publisher, vue.q)}</span>` : '',
    it.count && !(Number(it.size) > 0) ? `<span class="an-pub">${esc(pl('an.n_elements', Number(it.count)))}</span>` : '',
    vue.parent ? `<span class="an-pub an-categorie">${esc(nomElement(m, vue.parent))}</span>` : '',
  ].join('');
  const plie = !vue.deplie && estPlie(m, k, it.id);
  const sous = { ...vue, parent: null };
  return `<label class="an-ligne${verrou || fait ? ' inactive' : ''}${cochee || partiel ? '' : ' decoche'}">
      <input type="checkbox" ${attrs(m, k, `data-a-k="item" data-a-item="${esc(it.id)}" data-a-mode="${mode}"`)}${cochee ? ' checked' : ''}${partiel ? ' data-partiel="1"' : ''}${verrou || fait ? ' disabled' : ''}>
      <span class="an-ico">${iconeElement(it)}</span>
      <span class="an-t"><b>${surligne(nomElement(m, it), vue.q)}</b>${it.path ? `<span class="an-chemin">${surligne(it.path, vue.q)}</span>` : ''}<span class="an-badges">${badges}</span></span>
      <span class="an-v">${sousTotal ? taille(sousTotal) : ''}${visibles.length ? `<small>${esc(pl('an.n_elements', visibles.length))}</small>` : ''}</span>
      ${enfants.length ? `<button class="an-plier${plie ? ' plie' : ''}" type="button" ${attrs(m, k, `data-a-plier="${esc(it.id)}"`)} aria-label="${esc(t(plie ? 'an.deplier' : 'an.replier'))}">${ico('chevron', 'i14')}</button>` : ''}
    </label>${enfants.length && !plie ? `<div class="an-enfants">${visibles.map((e) => ligneItem(m, k, e, mode, sous)).join('')}</div>` : ''}`;
}

function briqueListe(m, k, mode) {
  const o = m.options[k];
  const s = selection(m);
  const longue = estLongue(m);
  const e = longue ? etatListes(m) : { q: '', affichage: 'arbre', tri: null };
  const q = plier(e.q.trim());
  const f = feuilles(o);
  const aVerifier = f.filter((it) => it.confidence === 'low' || it.risk).length;
  const filtre = s.filtre[k] || 'tout';
  const parentDe = (it) => o.items.find((x) => x.id === it.parent);
  const correspond = (it) => !q || plier(`${nomElement(m, it)} ${it.publisher || ''} ${it.path || ''}`).includes(q);
  const garde = (it) => filtre !== 'verifier' || it.confidence === 'low' || !!it.risk;
  const avant = (a, b) => String(a).localeCompare(String(b), undefined, { sensitivity: 'base', numeric: true });
  const trier = e.tri ? (a, b) => (e.tri === 'editeur' ? avant(a.publisher || '\uffff', b.publisher || '\uffff') : 0) || avant(nomElement(m, a), nomElement(m, b)) : null;

  let lignes;
  if (longue && e.affichage === 'plat') {
    // A → Z: every element, its category beside its name.
    lignes = elementsDe(o)
      .filter((it) => montre(it, mode) && (!it.parent || montre(parentDe(it), mode)) && garde(it)
        && (correspond(it) || (it.parent && correspond(parentDe(it)))))
      .sort(trier || (() => 0))
      .map((it) => ligneItem(m, k, it, mode, { q, parent: it.parent ? parentDe(it) : null }));
  } else {
    // By category: a category shows when it matches, then with all its
    // children, or when some of its children match, then with those only.
    const montres = new Set();
    const rac = racines(o).filter((r) => {
      if (!montre(r, mode)) return false;
      const enfants = enfantsDe(o, r.id).filter((x) => montre(x, mode));
      if (!enfants.length) return correspond(r) && garde(r);
      const elle = correspond(r);
      const gardes = enfants.filter((x) => (elle || correspond(x)) && garde(x));
      gardes.forEach((x) => montres.add(x.id));
      return gardes.length > 0;
    });
    if (trier) rac.sort((a, b) => avant(nomElement(m, a), nomElement(m, b)));
    const filtrant = !!q || filtre === 'verifier';
    lignes = rac.map((it) => ligneItem(m, k, it, mode, { q, montres: filtrant ? montres : null, deplie: !!q, trier }));
  }
  const n = f.filter((it) => s.items[k][it.id]).length;
  const filtres = aVerifier
    ? `<span class="an-filtres"><button type="button" ${attrs(m, k, 'data-a-filtre="tout"')} aria-pressed="${filtre === 'tout'}">${esc(t('an.filtre_tout'))}</button><button type="button" ${attrs(m, k, 'data-a-filtre="verifier"')} aria-pressed="${filtre === 'verifier'}">${esc(t('an.filtre_verifier', { n: aVerifier }))}</button></span>`
    : '';
  const vide = q ? t('an.rien_ne_correspond', { q: e.q.trim() }) : t('an.rien_trouve');
  return `<div class="an-bloc"><div class="an-bloc-h"><span class="an-bh-t">${esc(libOption(m.entree, o.def))}</span>
      <button class="an-lien" type="button" ${attrs(m, k, 'data-a-tout="items"')}>${esc(t('an.tout_cocher'))}</button>
      <button class="an-lien" type="button" ${attrs(m, k, 'data-a-rien="items"')}>${esc(t('an.tout_decocher'))}</button>
      ${filtres}<span class="an-total">${esc(t('an.n_sur_n', { n, total: f.length }))}</span></div>
    ${lignes.join('') || `<div class="an-vide">${esc(vide)}</div>`}
    ${notesPour(m, k, mode)}</div>`;
}

function briqueGarder(m, k, mode) {
  const o = m.options[k];
  const s = selection(m);
  const groupes = [...new Set(o.items.filter((it) => montre(it, mode)).map((it) => it.group || it.id))];
  const html = groupes.map((g) => {
    const membres = o.items.filter((it) => (it.group || it.id) === g);
    const t0 = Number(membres[0].size) || 0;
    return `<div class="an-groupe"><div class="an-groupe-h">${esc(t('an.exemplaires', { nom: membres[0].name || membres[0].id, n: membres.length, taille: taille(t0), libere: taille(t0 * (membres.length - 1)) }))}</div>
      ${membres.map((it) => {
        const garde = s.garder[k][g] === it.id;
        return `<div class="an-ligne${garde ? ' decoche' : ''}">
          <button class="an-garder${garde ? ' oui' : ''}" type="button" ${attrs(m, k, `data-a-garder="${esc(g)}" data-a-item="${esc(it.id)}"`)}>${esc(t(garde ? 'an.garde' : 'an.garder'))}</button>
          <span class="an-t"><b>${esc(it.name || it.id)}</b>${it.path ? `<span class="an-chemin">${esc(it.path)}</span>` : ''}<span class="an-badges">${it.date ? `<span class="an-pub">${esc(dateCourte(it.date))}</span>` : ''}${garde || !s.supprimer[k] ? '' : `<span class="badge med">${esc(t('an.sera_supprime'))}</span>`}</span></span>
          <span class="an-v">${Number(it.size) > 0 ? taille(it.size) : ''}</span></div>`;
      }).join('')}</div>`;
  }).join('');
  return `<div class="an-bloc"><div class="an-bloc-h"><span class="an-bh-t">${esc(libOption(m.entree, o.def))}</span>
      <span class="an-total">${esc(t('an.un_garde'))}</span>
      <button class="switch" type="button" role="switch" aria-checked="${!!s.supprimer[k]}" aria-label="${esc(t('an.supprimer_doublons'))}" ${attrs(m, k, 'data-a-k="supprimer"')}></button></div>
    ${html || `<div class="an-vide">${esc(t('an.rien_trouve'))}</div>`}${notesPour(m, k, mode)}</div>`;
}

function metriquesHtml(m, mode) {
  const ms = m.metriques.filter((x) => montre(x, mode));
  if (!ms.length) return '';
  const groupes = [...new Set(ms.map((x) => x.group || ''))];
  return groupes.map((g) => `<div class="an-bloc">${g ? `<div class="an-bloc-h">${ico('hard-drive', 'i14')}<span class="an-bh-t">${esc(g)}</span></div>` : `<div class="an-bloc-h"><span class="an-bh-t">${esc(t('an.mesures'))}</span></div>`}
    <div class="an-metriques">${ms.filter((x) => (x.group || '') === g).map((x) => {
      const h = JETONS.health[x.health];
      const lib = rapport(m.entree, x.cle)?.label || x.cle;
      const valeur = JETONS.health[x.value] ? t(JETONS.health[x.value][0]) : (UNITES[x.unit] || ((v) => (Number.isNaN(Number(v)) ? v : nb(v))))(x.value);
      const jauge = x.max ? `<span class="an-jauge"><i class="${esc(x.health || '')}" style="width:${Math.min(100, (Number(x.value) / Number(x.max)) * 100)}%"></i></span>` : '';
      return `<div class="an-metrique"><span>${esc(lib)}</span><b>${h ? `<i class="an-pastille ${h[1]}"></i>` : ''}${esc(valeur)}</b>${h && !JETONS.health[x.value] ? `<span>${esc(t(h[0]))}</span>` : ''}${jauge}</div>`;
    }).join('')}</div></div>`).join('');
}

/** Les lignes que ce mode ne montre pas (hors notes). */
function compteMasques(m, mode) {
  let n = m.metriques.filter((x) => !montre(x, mode)).length;
  for (const o of Object.values(m.options)) {
    if (o.find && !montre(o.find, mode)) n += 1;
    n += Object.entries(o.finds).filter(([c, f]) => !montre(f, mode) || !visibleDecl(o, c, mode)).length;
    n += o.items.filter((it) => !montre(it, mode)).length;
  }
  return n;
}

function anomaliesHtml(m) {
  if (!m.anomalies.length) return '';
  return `<details class="an-anomalies"><summary>${ico('warn', 'i14')} ${esc(pl('an.anomalies', m.anomalies.length))}</summary>
    <ul>${m.anomalies.map((a) => `<li>${esc(a)}</li>`).join('')}</ul></details>`;
}

/** La liste a cocher : les briques dans l'ordre de l'entete. */
export function corps(m, mode = 'expert') {
  const globales = m.notes.filter((n) => !n.cible && montre(n, mode)).map((n) => noteHtml(m, n)).join('');
  const briques = Object.keys(m.options).map((k) => brique(m, k, mode)).join('');
  const n = compteMasques(m, mode);
  const masques = n && mode === 'simple' ? `<div class="an-masques">${ico('info', 'i14')}<span>${esc(pl('an.lignes_expert', n))}</span></div>` : '';
  return `${metriquesHtml(m, mode)}${barreListes(m)}${briques}${globales}${masques}${mode === 'expert' ? anomaliesHtml(m) : ''}`
    || `<div class="an-vide">${esc(t('an.rien_trouve'))}</div>`;
}

/* -------------------------------------------------------------------------
   Les vues (## view) : la meme analyse, presentee comme le script l'a choisi
   ------------------------------------------------------------------------- */

/** Etat d'affichage partage entre les rendus : rien ici ne part au script. */
const ETAT = { tm: {}, tri: { col: 'taille', sens: -1 }, recherche: '' };

const premiere = (m, kinds, pred = () => true) =>
  Object.entries(m.options).find(([, o]) => kinds.includes(o.def.kind) && !o.def.scan && !o.def.keep_one && pred(o));

function vueBarres(m, k, mode) {
  const s = selection(m);
  const o = m.options[k];
  const e = Object.entries(o.finds).filter(([c, f]) => Number(f.size) > 0 && montre(f, mode) && visibleDecl(o, c, mode)).sort((a, b) => b[1].size - a[1].size);
  if (!e.length) return '';
  const max = Math.max(...e.map(([, f]) => Number(f.size)));
  return `<div class="an-postes">${e.map(([c, f]) => `<label class="an-poste${s.choix[k][c] ? '' : ' decoche'}">
      <i class="an-remplissage" style="width:${(f.size / max) * 100}%;background:var(--acc)"></i>
      <input type="checkbox" ${attrs(m, k, `data-a-k="choix" data-a-choix="${esc(c)}"`)}${s.choix[k][c] ? ' checked' : ''}>
      <span class="an-t"><b>${esc(libChoix(m.entree, o.def, c))}</b>${f.count ? `<span>${esc(pl('an.n_fichiers', Number(f.count)))}</span>` : ''}</span><span class="an-v">${taille(f.size)}</span></label>`).join('')}</div>`;
}

/** L'anneau-engrenage (1.6) : un arc a bouts arrondis par entree, sur une
 *  piste, dans un contour d'engrenage. Une entree decochee reste, eteinte. */
function anneau(entrees, couleurs, centre, taillePx = 220) {
  const total = entrees.reduce((a, e) => a + e.taille, 0) || 1;
  const r = 78, C = 2 * Math.PI * r;
  let cumul = 0;
  const arcs = entrees.map((e, i) => {
    const l = (e.taille / total) * C;
    const arc = `<circle r="${r}" cx="110" cy="110" stroke="${couleurs[i]}" stroke-dasharray="${Math.max(0.5, l - 3)} ${C}" stroke-dashoffset="${-cumul}" opacity="${e.coche || e.partiel ? 1 : 0.22}"></circle>`;
    cumul += l;
    return arc;
  }).join('');
  return `<div class="an-anneau" style="width:${taillePx}px;height:${taillePx}px"><svg viewBox="0 0 220 220">
      <path class="an-dents" d="${engrenage(110, 110, 108, 98, 14)}"/><circle class="an-piste" r="${r}" cx="110" cy="110"></circle>${arcs}</svg><div class="an-centre">${centre}</div></div>`;
}

/** La jauge a dents (1.6) : une seule valeur ; ses dents s'allument une a
 *  une, aux couleurs de la flamme — ou d'une seule, pour un etat de sante. */
function jaugeDents(fraction, centre, taillePx = 220, couleur = null) {
  const n = 28, allumees = Math.round(Math.max(0, Math.min(1, fraction)) * n);
  const dents = Array.from({ length: n }, (_, i) => `<path d="${dent(110, 110, 106, 86, i, n)}" style="fill:${i < allumees ? couleur || flamme(i / (n - 1)) : 'var(--surface-3)'}"/>`).join('');
  return `<div class="an-jd" style="width:${taillePx}px;height:${taillePx}px"><svg viewBox="0 0 220 220">${dents}
      <circle class="an-jd-cercle" cx="110" cy="110" r="79"></circle><circle class="an-jd-coeur" cx="110" cy="110" r="67"></circle></svg><div class="an-centre">${centre}</div></div>`;
}

/** La double couronne (1.6) : au centre les postes, autour ce qui compose
 *  chacun — on voit d'un coup un poste et son detail. */
function couronne(postes, couleurs, centre, taillePx = 240) {
  const total = postes.reduce((a, p) => a + p.taille, 0) || 1;
  let a = -Math.PI / 2, interieur = '', exterieur = '';
  postes.forEach((p, i) => {
    const da = (p.taille / total) * Math.PI * 2;
    const allume = p.coche || p.partiel;
    interieur += `<path d="${arcCouronne(110, 110, 54, a + 0.02, a + da - 0.02, 22)}" style="fill:${couleurs[i]}" opacity="${allume ? 1 : 0.22}"/>`;
    const parts = (p.parts || []).filter((x) => x.taille > 0 && !x.cache);
    const somme = parts.reduce((s2, x) => s2 + x.taille, 0);
    let b = a;
    (somme ? parts : [{ taille: 1, get: () => allume }]).forEach((x, j) => {
      const db = (x.taille / (somme || 1)) * da;
      exterieur += `<path d="${arcCouronne(110, 110, 81, b + 0.025, b + db - 0.025, 17)}" style="fill:${couleurs[i]}" opacity="${x.get() ? (j % 2 ? 0.62 : 0.88) : 0.16}"/>`;
      b += db;
    });
    a += da;
  });
  return `<div class="an-jd" style="width:${taillePx}px;height:${taillePx}px"><svg viewBox="0 0 220 220">
      <path class="an-dents" d="${engrenage(110, 110, 108, 99, 18)}"/>${exterieur}${interieur}</svg><div class="an-centre">${centre}</div></div>`;
}

function vueAnneau(m, k, mode) {
  const s = selection(m);
  const o = m.options[k];
  const e = Object.entries(o.finds).filter(([c, f]) => Number(f.size) > 0 && montre(f, mode) && visibleDecl(o, c, mode)).sort((a, b) => b[1].size - a[1].size)
    .map(([c, f]) => ({ attr: attrs(m, k, `data-a-k="choix" data-a-choix="${esc(c)}"`), label: libChoix(m.entree, o.def, c), taille: Number(f.size), choisi: s.choix[k][c] ? Number(f.size) : 0, coche: !!s.choix[k][c] }));
  if (!e.length) return '';
  const c = e.map((_, i) => serie(i));
  const choisi = e.reduce((a, x) => a + x.choisi, 0);
  return `<div class="an-anneau-zone compacte">${anneau(e, c, `<b>${taille(choisi)}</b><span>${esc(t('an.coches_court'))}</span>`, 180)}<div class="an-postes">${e.map((x, i) => lignePosteSimple(x, c[i])).join('')}</div></div>`;
}

function lignePosteSimple(e, couleur) {
  return `<label class="an-poste${e.coche ? '' : ' decoche'}"><input type="checkbox" ${e.attr}${e.coche ? ' checked' : ''}>
    <span class="an-pt" style="background:${couleur}"></span><span class="an-t"><b>${esc(e.label)}</b></span><span class="an-v">${taille(e.taille)}</span></label>`;
}

function vueTuiles(m, k, mode) {
  const s = selection(m);
  const o = m.options[k];
  const items = o.items.filter((it) => montre(it, mode) && !enfantsDe(o, it.id).length);
  if (!items.length) return '';
  return `<div class="an-tuiles">${items.map((it, i) => {
    const c = !!s.items[k][it.id];
    const nom = nomElement(m, it);
    return `<label class="an-tuile${c ? ' coche' : ''}${it.locked ? ' verrou' : ''}">
      <input type="checkbox" hidden ${attrs(m, k, `data-a-k="item" data-a-item="${esc(it.id)}" data-a-mode="${mode}"`)}${c ? ' checked' : ''}${it.locked ? ' disabled' : ''}>
      ${c ? `<span class="an-marque">${ico('check', 'i12')}</span>` : ''}<span class="an-av" style="background:${serie(i)}">${esc(nom.slice(0, 1).toUpperCase())}</span>
      <b>${esc(nom)}</b><span class="muted">${it.locked ? esc(verrouTexte(it.locked)) : Number(it.size) > 0 ? taille(it.size) : ''}</span></label>`;
  }).join('')}</div>${notesPour(m, k, mode)}`;
}

function vueAvantApres(m, mode) {
  const s = selection(m);
  const lignes = Object.entries(m.options).filter(([, o]) => o.def.kind === 'bool' && o.find && montre(o.find, mode) && visibleDecl(o, null, mode));
  if (!lignes.length) return '';
  return `<table class="an-aa"><tr><th></th><th>${esc(t('an.reglage'))}</th><th>${esc(t('an.aujourdhui'))}</th><th></th><th>${esc(t('an.apres'))}</th></tr>${lignes.map(([k, o]) => {
    const fait = o.find.state === 'ok';
    const apres = fait || s.bool[k];
    return `<tr><td><input type="checkbox" ${attrs(m, k, 'data-a-k="bool"')}${s.bool[k] ? ' checked' : ''}${fait ? ' disabled' : ''}></td><td>${esc(libOption(m.entree, o.def))}</td>
      <td>${fait ? `<span class="badge low">${esc(t('an.en_place'))}</span>` : `<span class="badge neutre">${esc(t('an.pas_en_place'))}</span>`}</td><td class="an-fl">→</td>
      <td>${apres ? `<span class="badge low">${esc(t('an.en_place'))}</span>` : `<span class="badge neutre">${esc(t('an.inchange'))}</span>`}</td></tr>`;
  }).join('')}</table>`;
}

function vueTableau(m, k) {
  const s = selection(m);
  const o = m.options[k];
  const q = ETAT.recherche.toLowerCase();
  const nomParent = (it) => (it.parent ? o.items.find((x) => x.id === it.parent)?.name || '' : '');
  const lignes = feuilles(o).map((it) => ({ it, prog: nomParent(it) }))
    .filter(({ it, prog }) => !q || `${prog} ${nomElement(m, it)} ${it.path || ''}`.toLowerCase().includes(q));
  const ordre = { high: 0, medium: 1, low: 2 };
  const v = (x) => (ETAT.tri.col === 'taille' ? Number(x.it.size) || 0 : ETAT.tri.col === 'prog' ? x.prog : ETAT.tri.col === 'confiance' ? ordre[x.it.confidence] ?? 3 : nomElement(m, x.it));
  lignes.sort((a, b) => (v(a) > v(b) ? 1 : v(a) < v(b) ? -1 : 0) * ETAT.tri.sens);
  const avecParent = lignes.some((x) => x.prog);
  const th = (col, lib) => `<th data-a-tri="${col}"${ETAT.tri.col === col ? ` aria-sort="${ETAT.tri.sens > 0 ? 'ascending' : 'descending'}"` : ''}>${esc(lib)}${ETAT.tri.col === col ? (ETAT.tri.sens > 0 ? ' ▲' : ' ▼') : ''}</th>`;
  return `<div class="an-bloc an-tableau"><div class="an-bloc-h"><span class="an-bh-t">${esc(libOption(m.entree, o.def))}</span>
      <input type="search" class="an-cherche" placeholder="${esc(t('an.chercher'))}" value="${esc(ETAT.recherche)}" data-a-cherche></div>
    <div class="an-defile-x"><table class="an-tab"><tr><th></th>${avecParent ? th('prog', t('an.col_parent')) : ''}${th('nom', t('an.col_element'))}${th('confiance', t('an.col_confiance'))}${th('taille', t('an.col_taille'))}</tr>
    ${lignes.map(({ it, prog }) => `<tr><td><input type="checkbox" ${attrs(m, k, `data-a-k="item" data-a-item="${esc(it.id)}"`)}${s.items[k][it.id] ? ' checked' : ''}></td>
      ${avecParent ? `<td>${esc(prog)}</td>` : ''}<td>${esc(nomElement(m, it))}${it.path ? `<div class="an-chemin">${esc(it.path)}</div>` : ''}</td><td>${badge('confidence', it.confidence)}</td><td class="an-n">${Number(it.size) > 0 ? taille(it.size) : '—'}</td></tr>`).join('')}
    ${lignes.length ? '' : `<tr><td colspan="5" class="an-tab-vide">${esc(t('an.rien_ne_correspond', { q: ETAT.recherche }))}</td></tr>`}</table></div></div>`;
}

function vueTreemap(m, k) {
  const s = selection(m);
  const o = m.options[k];
  const parents = racines(o).filter((p) => enfantsDe(o, p.id).length);
  const plat = !parents.length;
  const groupes = (plat ? feuilles(o).map((it) => ({ p: it, e: [it] })) : parents.map((p) => ({ p, e: enfantsDe(o, p.id) })))
    .map((x) => ({ ...x, taille: x.e.reduce((a, y) => a + (Number(y.size) || 0), 0), n: x.e.filter((y) => s.items[k][y.id]).length }))
    .sort((a, b) => b.taille - a.taille);
  const couleur = Object.fromEntries(groupes.map((x, i) => [x.p.id, serie(i)]));
  const cases = (tuiles, hauteur) => {
    const total = tuiles.reduce((a, x) => a + x.taille, 0) || 1;
    return `<div class="an-treemap" style="height:${hauteur}px"><div class="an-tm-rang">${tuiles.map((x) => `
      <button type="button" class="an-tm-case${x.on ? '' : ' off'}${x.taille / total < 0.04 ? ' petite' : ''}" style="flex-grow:${Math.max(x.taille, total * 0.02)};background:${x.fond}" ${x.attr} title="${esc(x.label)} — ${taille(x.taille)}">
        <b>${esc(x.label)}</b><span>${taille(x.taille)}${x.sous ? ` · ${esc(x.sous)}` : ''}</span></button>`).join('')}</div></div>`;
  };
  const ouvert = ETAT.tm[`${m.entree.id}/${k}`];
  const prog = !plat && groupes.find((x) => x.p.id === ouvert);
  if (!prog) {
    return `<div class="an-bloc"><div class="an-bloc-h"><span class="an-bh-t">${esc(libOption(m.entree, o.def))}</span><span class="an-total">${esc(t(plat ? 'an.tm_clic_coche' : 'an.tm_clic_ouvre'))}</span></div>
      <div class="an-tm-zone">${cases(groupes.map((x) => ({
        label: nomElement(m, x.p), taille: x.taille, on: x.n > 0, fond: couleur[x.p.id],
        sous: plat ? '' : t('an.n_sur_n', { n: x.n, total: x.e.length }),
        attr: plat ? attrs(m, k, `data-a-tm-item="${esc(x.p.id)}"`) : attrs(m, k, `data-a-tm="${esc(x.p.id)}"`),
      })), 200)}</div></div>`;
  }
  // Ouvert : le parent devient un bandeau, ses enfants se rangent dessous, en
  // retrait, le long d'un trait a sa couleur, dans des teintes de celle-ci.
  const c = couleur[prog.p.id];
  const enfants = [...prog.e].sort((a, b) => (Number(b.size) || 0) - (Number(a.size) || 0));
  const teinte = (i) => `color-mix(in srgb, ${c} ${100 - Math.min(60, i * 26)}%, white)`;
  return `<div class="an-bloc"><div class="an-bloc-h"><button class="an-lien" type="button" ${attrs(m, k, 'data-a-tm-retour')}>← ${esc(t('an.tm_retour'))}</button></div>
    <div class="an-tm-zone"><div class="an-tm-parent" style="background:${c}"><b>${esc(nomElement(m, prog.p))}</b><span>${taille(prog.taille)} · ${esc(t('an.n_sur_n', { n: prog.n, total: prog.e.length }))}</span></div>
    <div class="an-tm-dedans" style="--c:${c}">${cases(enfants.map((x, i) => ({
      label: nomElement(m, x), taille: Number(x.size) || 0, on: !!s.items[k][x.id], fond: teinte(i),
      sous: JETONS.confidence[x.confidence] ? t(JETONS.confidence[x.confidence][0]) : '',
      attr: attrs(m, k, `data-a-tm-item="${esc(x.id)}"`),
    })), 150)}</div></div></div>`;
}

function vueChronologie(m, mode) {
  const s = selection(m);
  const entree = premiere(m, ['items']);
  if (!entree) return '';
  const [k, o] = entree;
  const items = o.items.filter((it) => montre(it, mode) && !enfantsDe(o, it.id).length);
  const duree = { high: 14, medium: 8, low: 3 };
  const mesure = m.metriques.find((x) => x.unit === 's' && montre(x, mode));
  const fin = Math.max(Number(mesure?.value) || 0, 4 + items.length * 5 + 14);
  let gain = 0;
  const lignes = items.map((it, i) => {
    const debut = 4 + i * 5;
    const d = duree[it.impact] || 3;
    const coupe = !!s.items[k][it.id];
    if (coupe) gain += d * 0.55;
    return `<label class="an-chrono-ligne"><span><input type="checkbox" ${attrs(m, k, `data-a-k="item" data-a-item="${esc(it.id)}"`)}${coupe ? ' checked' : ''}${it.locked ? ' disabled' : ''}> ${esc(nomElement(m, it))}</span>
      <span class="an-chrono-piste"><i class="${it.locked ? 'sys' : coupe ? 'coupe' : ''}" style="left:${(debut / fin) * 100}%;width:${(d / fin) * 100}%"></i></span></label>`;
  }).join('');
  const entete = mesure
    ? `<div class="an-graphe-h"><b>${nb(mesure.value)} s → ≈ ${nb(Math.max(1, Math.round(Number(mesure.value) - gain)))} s</b><span>${esc(rapport(m.entree, mesure.cle)?.label || '')}</span></div>`
    : '';
  return `${entete}<div class="an-chrono">${lignes}</div><p class="an-legende">${esc(t('an.chrono_legende'))}</p>`;
}

function vueColonnes(m, k) {
  const s = selection(m);
  const o = m.options[k];
  const e = Object.entries(o.finds).filter(([, f]) => f.ms);
  if (!e.length) return '';
  const max = Math.max(...e.map(([, f]) => Number(f.ms)));
  return `<div class="an-bloc"><div class="an-bloc-h"><span class="an-bh-t">${esc(libOption(m.entree, o.def))}</span><span class="an-total">${esc(t('an.plus_bas'))}</span></div>
    <div class="an-colonnes">${e.map(([c, f]) => `<label class="an-colonne${s.select[k] === c ? ' choisi' : ''}">
      <input type="radio" hidden name="an-col-${esc(m.entree.id)}-${esc(k)}" ${attrs(m, k, `data-a-k="select" data-a-choix="${esc(c)}"`)}${s.select[k] === c ? ' checked' : ''}>
      <b>${nb(f.ms)} ms</b><i style="height:${(f.ms / max) * 100}%"></i><span>${esc(libChoix(m.entree, o.def, c))}${f.current === 'true' ? `<br>(${esc(t('an.actuel').toLowerCase())})` : ''}</span></label>`).join('')}</div></div>`;
}

function vueFeu(m, mode) {
  const ms = m.metriques.filter((x) => montre(x, mode));
  if (!ms.length) return '';
  const ordre = ['ok', 'warn', 'crit'];
  const pire = ms.reduce((a, x) => (ordre.indexOf(x.health) > ordre.indexOf(a) ? x.health : a), 'ok');
  const groupes = [...new Set(ms.map((x) => x.group).filter(Boolean))];
  const detail = groupes.length
    ? groupes.map((g) => {
      const h = ms.filter((x) => x.group === g).reduce((a, x) => (ordre.indexOf(x.health) > ordre.indexOf(a) ? x.health : a), 'ok');
      return `${esc(rapport(m.entree, g)?.label || g)} : ${esc(t(JETONS.health[h][0]))}`;
    }).join(' · ')
    : ms.filter((x) => x.health && x.health !== 'ok').map((x) => {
      const valeur = (UNITES[x.unit] || nb)(x.value);
      return `${esc(rapport(m.entree, x.cle)?.label || x.cle)} : ${esc(valeur)} — ${esc(t(JETONS.health[x.health]?.[0] || 'an.health.ok'))}`;
    }).join(' · ');
  return `<div class="an-feu"><span class="an-lampe ${pire}">${ico(pire === 'ok' ? 'shield-check' : 'warn', 'i20')}</span>
    <span><b>${esc(t(`an.feu.${pire}`))}</b><span>${detail}</span></span></div>`;
}

function vueJauge(m, mode) {
  const x = m.metriques.find((y) => y.max && montre(y, mode));
  if (!x) return '';
  const h = JETONS.health[x.health] || JETONS.health.ok;
  const couleur = { ok: 'var(--ok)', warn: 'var(--warn)', crit: 'var(--err)' }[h[1]];
  const valeur = (UNITES[x.unit] || nb)(x.value);
  return `<div class="an-jauge-seule">${jaugeDents(Number(x.value) / Number(x.max), `<b>${esc(valeur)}</b><span>${esc(rapport(m.entree, x.cle)?.label || x.cle)}</span>`, 220, couleur)}
    <span class="badge ${h[1] === 'ok' ? 'low' : h[1] === 'warn' ? 'med' : 'high'}">${esc(t(h[0]))}</span></div>`;
}

/** L'historique : ce que l'action a libere a chaque passage. Sindry le sait :
 *  il garde le [FREED] de chaque action. Le script n'a rien a fournir. */
function vueHistorique(historique = []) {
  const h = historique.filter((r) => r.freed != null && !r.simulated).slice(-12);
  if (!h.length) return `<p class="an-legende">${esc(t('an.histo_vide'))}</p>`;
  const max = Math.max(1, ...h.map((r) => r.freed));
  return `<div class="an-graphe-h"><b>${taille(h.reduce((a, r) => a + r.freed, 0))}</b><span>${esc(pl('an.histo_passages', h.length))}</span></div>
    <div class="an-histo-barres">${h.map((r) => `<i style="height:${(r.freed / max) * 100}%" title="${esc(dateCourte(r.at))} : ${esc(taille(r.freed))}"><b>${esc(taille(r.freed))}</b></i>`).join('')}</div>
    <div class="an-histo-dates">${h.map((r) => `<span>${esc(dateCourte(r.at))}</span>`).join('')}</div>
    <p class="an-legende">${esc(t('an.histo_legende'))}</p>`;
}

/** Pour le panneau « Historique » de l'Expert. */
export { vueHistorique as historique };

/** Une phrase et un interrupteur : la plus simple des vues. */
function vueMinimale(m, mode) {
  const p = postesAction(m, mode)[0];
  if (!p) return '';
  const quoi = p.taille ? t('an.liberer', { taille: taille(p.taille), quoi: p.label.toLowerCase() }) : p.label;
  return `<div class="an-min"><span class="an-t"><b>${esc(quoi)}</b><span>${esc(p.sous || '')}</span></span>
    <button class="switch" type="button" role="switch" aria-checked="${p.coche}" aria-label="${esc(p.label)}" data-a-poste="${esc(p.cle)}" data-a-switch></button></div>`;
}

/**
 * La vue d'une action, telle que le script l'a choisie. Une vue qui ne trouve
 * pas de quoi s'afficher retombe sur la liste a cocher : jamais un ecran vide
 * parce qu'un script a demande une vue qui ne convient pas a ce qu'il a trouve.
 */
export function vue(m, mode = 'expert', { historique = [] } = {}) {
  const meta = m.entree.meta;
  const nom = (mode === 'expert' && meta.view_expert) || meta.view || 'checklist';
  const multi = premiere(m, ['multi'], (o) => Object.values(o.finds).some((f) => Number(f.size) > 0));
  const items = premiere(m, ['items'], (o) => o.items.length);
  const select = premiere(m, ['select'], (o) => Object.values(o.finds).some((f) => f.ms));
  const rendus = {
    minimal: () => vueMinimale(m, mode),
    light: () => vueFeu(m, mode) && `${vueFeu(m, mode)}${m.notes.filter((n) => montre(n, mode)).map((n) => noteHtml(m, n)).join('')}`,
    gauge: () => vueJauge(m, mode),
    checklist: () => '',
    bars: () => multi && vueBarres(m, multi[0], mode),
    donut: () => multi && vueAnneau(m, multi[0], mode),
    tiles: () => items && vueTuiles(m, items[0], mode),
    compare: () => vueAvantApres(m, mode),
    tree: () => '',
    table: () => items && vueTableau(m, items[0]),
    treemap: () => items && vueTreemap(m, items[0]),
    timeline: () => vueChronologie(m, mode),
    chart: () => select && vueColonnes(m, select[0]),
    history: () => vueHistorique(historique),
  };
  const principal = (rendus[nom] || rendus.checklist)() || '';
  // Ce que la vue principale n'a pas montre reste accessible dessous : une vue
  // choisit une presentation, elle ne fait jamais disparaitre une case.
  const montres = new Set();
  if (['bars', 'donut'].includes(nom) && multi) montres.add(multi[0]);
  if (['tiles', 'table', 'treemap', 'timeline'].includes(nom) && items) montres.add(items[0]);
  if (nom === 'chart' && select) montres.add(select[0]);
  if (nom === 'compare') Object.entries(m.options).filter(([, o]) => o.def.kind === 'bool' && o.find).forEach(([k]) => montres.add(k));
  // Le feu et la jauge resument les mesures : en Simple, ce resume suffit ; en
  // Expert, le detail suit toujours.
  const metriquesDejaVues = mode === 'simple' && ['light', 'gauge'].includes(nom) && principal;
  const reste = Object.keys(m.options).filter((k) => !montres.has(k)).map((k) => brique(m, k, mode)).join('');
  const globales = m.notes.filter((n) => !n.cible && montre(n, mode) && nom !== 'light').map((n) => noteHtml(m, n)).join('');
  if (!principal || nom === 'checklist' || nom === 'tree') return corps(m, mode);
  return `<div class="an-vue">${principal}</div>${metriquesDejaVues ? '' : metriquesHtml(m, mode)}${barreListes(m, montres)}${reste}${globales}${mode === 'expert' ? anomaliesHtml(m) : ''}`;
}

/* -------------------------------------------------------------------------
   Le resume du mode Simple : des postes, un graphique, des chevrons
   ------------------------------------------------------------------------- */

/**
 * Les morceaux d'une action, rangés par poste. Un poste est un groupe que le
 * script declare ([group:Cle] sur une option ou un choix, son libelle dans
 * REPORT) ; sans groupe, une option fait un poste.
 */
function postesAction(m, mode = 'simple') {
  const s = selection(m);
  const parGroupe = new Map();
  const ajouter = (groupe, opt, part) => {
    if (!parGroupe.has(groupe)) parGroupe.set(groupe, { groupe, opt, parts: [] });
    parGroupe.get(groupe).parts.push(part);
  };
  for (const [k, o] of Object.entries(m.options)) {
    const d = o.def;
    if (d.scan) continue;
    const cacheOpt = !montre(d, mode);
    if (d.kind === 'multi') {
      for (const [c, f] of Object.entries(o.finds)) {
        if (!(Number(f.size) > 0) && !(Number(f.count) > 0)) continue;
        const ch = choixDe(o, c);
        ajouter(ch?.group || d.group || k, d, { k, choix: c, f, taille: Number(f.size) || 0, reco: true,
          cache: cacheOpt || !montre(f, mode) || !montre(ch, mode), get: () => !!s.choix[k][c], set: (v) => { s.choix[k][c] = v; } });
      }
    } else if (d.kind === 'bool' && o.find && o.find.state !== 'ok' && (Number(o.find.size) > 0 || Number(o.find.count) > 0 || o.find.state === 'todo')) {
      ajouter(d.group || k, d, { k, f: o.find, taille: Number(o.find.size) || 0, reco: true, reglage: o.find.state === 'todo',
        cache: cacheOpt || !montre(o.find, mode), get: () => !!s.bool[k], set: (v) => { s.bool[k] = v; } });
    } else if (d.kind === 'select' && Object.keys(o.finds).length) {
      const actuel = Object.entries(o.finds).find(([, f]) => f.current === 'true')?.[0];
      ajouter(d.group || k, d, { k, select: true, taille: 0, reco: true, reglage: true, cache: cacheOpt,
        get: () => !!s.select[k] && s.select[k] !== actuel,
        set: (v) => { if (!v) s.select[k] = actuel; else if (s.select[k] === actuel) s.select[k] = (Object.entries(o.finds).find(([, f]) => f.recommended === 'true') || [actuel])[0]; } });
    } else if (d.kind === 'items' && d.keep_one) {
      const t0 = aSupprimer(m, k).reduce((a, it) => a + (Number(it.size) || 0), 0) || o.items.reduce((a, it) => a + (Number(it.size) || 0), 0) / 2;
      if (o.items.length) ajouter(d.group || k, d, { k, garder: true, taille: t0, reco: true, cache: cacheOpt, get: () => !!s.supprimer[k], set: (v) => { s.supprimer[k] = v; } });
    } else if (d.kind === 'items') {
      for (const it of feuilles(o)) {
        const parent = o.items.find((x) => x.id === it.parent);
        ajouter(d.group || k, d, { k, it, taille: Number(it.size) || 0, reco: cocheItem(it),
          cache: cacheOpt || !montre(it, mode) || (!!parent && !montre(parent, mode)),
          get: () => !!s.items[k][it.id], set: (v) => { s.items[k][it.id] = v; } });
      }
    }
  }
  return [...parGroupe.values()].map(({ groupe, opt, parts }) => {
    const r = rapport(m.entree, groupe);
    const groupeDeclare = r?.kind === 'group';
    const reco = parts.filter((x) => x.reco);
    const regs = reco.length ? reco : parts;
    const autres = parts.filter((x) => !regs.includes(x));
    const n = regs.filter((x) => x.get()).length;
    const plus = autres.filter((x) => x.get());
    return {
      m, cle: `${m.entree.id}|${groupe}`, groupe, parts, regs,
      label: groupeDeclare ? r.label : libOption(m.entree, opt),
      sous: groupeDeclare ? r.desc : descOption(m.entree, opt),
      action: titreAction(m.entree),
      // Un element « A verifier » coche a la main entre dans le poste ; decoche, il n'y pese pas.
      taille: regs.reduce((a, x) => a + x.taille, 0) + plus.reduce((a, x) => a + x.taille, 0),
      choisi: parts.filter((x) => x.get()).reduce((a, x) => a + x.taille, 0),
      aVerifier: autres.filter((x) => !x.get() && !x.cache).reduce((a, x) => a + x.taille, 0),
      reglages: parts.filter((x) => x.reglage && x.get()).length,
      elements: parts.filter((x) => x.it && !x.taille && x.get()).length,
      coche: regs.length > 0 && n === regs.length,
      partiel: (n > 0 && n < regs.length) || (n === 0 && plus.length > 0),
      toutCache: parts.every((x) => x.cache),
    };
  }).filter((p) => !p.toutCache);
}

/** Tous les postes du resume, du plus gros au plus petit ; ceux sans taille ensuite. */
export function postesResume(modeles) {
  return modeles.flatMap((m) => postesAction(m, 'simple')).sort((a, b) => b.taille - a.taille);
}

/** Cocher : ce que le script recommande. Decocher : tout, y compris ce qu'on avait ajoute. */
function basculerPoste(p, v) {
  for (const x of v ? p.regs : p.parts) x.set(v);
}

/** Ce qu'un poste deplie montre : ses choix, ou son arborescence. */
function detailPoste(p, couleur) {
  const m = p.m;
  const html = [];
  const vus = new Set();
  for (const x of p.parts) {
    if (x.it || x.garder || x.select) {
      if (vus.has(x.k)) continue;
      vus.add(x.k);
      const o = m.options[x.k];
      if (x.garder) html.push(briqueGarder(m, x.k, 'simple'));
      else if (x.select) html.push(brique(m, x.k, 'simple'));
      else html.push(racines(o).filter((it) => montre(it, 'simple')).map((it) => ligneItem(m, x.k, it, 'simple')).join(''), notesPour(m, x.k, 'simple'));
      continue;
    }
    if (x.cache) continue;
    const d = m.options[x.k].def;
    const attr = x.choix ? attrs(m, x.k, `data-a-k="choix" data-a-choix="${esc(x.choix)}"`) : attrs(m, x.k, 'data-a-k="bool"');
    const n = Number(x.f.count) || 0;
    const etat = x.reglage ? `<span class="badge acc">${esc(t('an.a_faire'))}</span>` : n ? `<span class="muted">${esc(pl('an.n_elements', n))}</span>` : '';
    html.push(`<label class="an-ligne${x.get() ? '' : ' decoche'}"><input type="checkbox" ${attr}${x.get() ? ' checked' : ''}>
      <span class="an-t"><b>${esc(x.choix ? libChoix(m.entree, d, x.choix) : libOption(m.entree, d))}</b><span class="an-badges">${etat}</span></span>
      <span class="an-v">${x.taille ? taille(x.taille) : ''}</span></label>`);
  }
  // Ce que le detail ne montre pas, il l'annonce : rien ne se fait en cachette.
  const caches = p.parts.filter((x) => x.cache);
  const poids = caches.reduce((a, x) => a + x.taille, 0);
  const nc = caches.filter((x) => x.get()).length;
  const etat = nc === caches.length ? t(caches.length > 1 ? 'an.caches_suivent' : 'an.cache_suit')
    : nc ? t('an.n_coches_court', { n: nc }) : t(caches.length > 1 ? 'an.non_coches' : 'an.non_coche');
  const plus = caches.length
    ? `<div class="an-masques">${ico('info', 'i14')}<span>${esc(t(caches.length > 1 ? 'an.n_en_expert' : 'an.un_en_expert', { n: caches.length, taille: poids ? ` (${taille(poids)})` : '' }))} — ${esc(etat)}.</span></div>`
    : '';
  return `<div class="an-poste-detail" style="--c:${couleur}">${html.join('')}${plus}</div>`;
}

function lignePoste(p, couleur, { remplir = false, max = 1, ouverts }) {
  const ouvert = ouverts.has(p.cle);
  const indice = p.aVerifier ? `<span class="an-a-verifier">${esc(t('an.laisses_decoches', { taille: taille(p.aVerifier) }))}</span>` : '';
  const valeur = p.taille ? taille(p.taille)
    : p.reglages ? pl('an.n_reglages', p.reglages) : p.elements ? pl('an.n_elements', p.elements) : '';
  const notes = p.parts.flatMap((x) => (x.choix ? [`${x.k}.${x.choix}`] : x.it ? [] : [x.k]))
    .filter((v, i, a) => a.indexOf(v) === i)
    .map((cible) => notesPour(p.m, cible, 'simple')).join('');
  return `<div class="an-poste-bloc"><label class="an-poste${p.coche || p.partiel ? '' : ' decoche'}">
    ${remplir ? `<i class="an-remplissage" style="width:${(p.taille / max) * 100}%;background:${couleur}"></i>` : ''}
    <input type="checkbox" data-a-poste="${esc(p.cle)}"${p.coche ? ' checked' : ''}${p.partiel ? ' data-partiel="1"' : ''}>
    ${remplir ? '' : `<span class="an-pt" style="background:${couleur}"></span>`}
    <span class="an-t"><b>${esc(p.label)}</b><span>${esc(p.sous || p.action)}</span>${indice}${notes}</span>
    <span class="an-v">${esc(valeur)}${p.partiel && p.choisi ? `<small>${esc(t('an.n_coches', { taille: taille(p.choisi) }))}</small>` : ''}</span>
    <button class="an-plier-poste${ouvert ? ' ouvert' : ''}" type="button" data-a-deplier="${esc(p.cle)}" aria-expanded="${ouvert}" aria-label="${esc(t(ouvert ? 'an.replier' : 'an.deplier'))}">${ico('chevron', 'i20')}</button>
  </label>${ouvert ? detailPoste(p, couleur) : ''}</div>`;
}

/**
 * Le resume du mode Simple. `graphe` : un des GRAPHIQUES. Les postes sans
 * taille (reglages, elements sans poids) suivent ceux du graphique, et les
 * diagnostics (mesures seules) ferment la marche.
 */
export function resume(modeles, { graphe = 'donut', ouverts = new Set(), detailJauge = false } = {}) {
  const tous = postesResume(modeles);
  const pesants = tous.filter((p) => p.taille > 0);
  const legers = tous.filter((p) => !(p.taille > 0));
  const couleurs = new Map(tous.map((p, i) => [p.cle, serie(i)]));
  const total = pesants.reduce((a, p) => a + p.taille, 0) || 1;
  const choisi = pesants.reduce((a, p) => a + p.choisi, 0);
  const liste = (postes, opts = {}) => (postes.length
    ? `<div class="an-postes">${postes.map((p) => lignePoste(p, couleurs.get(p.cle), { ouverts, ...opts })).join('')}</div>` : '');
  const entete = `<div class="an-graphe-h"><b>${taille(choisi)}</b><span>${esc(t('an.a_liberer_sur', { trouve: taille(total) }))}</span></div>`;

  let graphique = '';
  if (pesants.length) {
    if (graphe === 'bar') {
      const pile = pesants.map((p) => `<i class="${p.coche || p.partiel ? '' : 'off'}" style="flex-grow:${p.taille};background:${couleurs.get(p.cle)}" title="${esc(p.label)} — ${taille(p.taille)}"></i>`).join('');
      graphique = `${entete}<div class="an-pile">${pile}</div>${liste(pesants)}`;
    } else if (graphe === 'rows') {
      graphique = `${entete}${liste(pesants, { remplir: true, max: Math.max(...pesants.map((p) => p.taille)) })}`;
    } else if (graphe === 'treemap') {
      const rangs = [[], []];
      let cumul = 0;
      pesants.forEach((p) => { (cumul < total * 0.6 ? rangs[0] : rangs[1]).push(p); cumul += p.taille; });
      const html = rangs.filter((r) => r.length).map((rang) => `<div class="an-tm-rang" style="flex-grow:${rang.reduce((a, p) => a + p.taille, 0)}">${rang.map((p) => `
        <label class="an-tm-case${p.coche || p.partiel ? '' : ' off'}${p.taille / total < 0.05 ? ' petite' : ''}" style="flex-grow:${p.taille};background:${couleurs.get(p.cle)}" title="${esc(p.label)} — ${taille(p.taille)}">
          <input type="checkbox" hidden data-a-poste="${esc(p.cle)}"${p.coche ? ' checked' : ''}><b>${esc(p.label)}</b><span>${taille(p.taille)}</span></label>`).join('')}</div>`).join('');
      graphique = `${entete}<div class="an-treemap an-treemap-resume">${html}</div><p class="an-legende">${esc(t('an.tm_clic_coche'))}</p>`;
    } else if (graphe === 'waffle') {
      const N = 100;
      const parts = pesants.map((p) => (p.taille / total) * N);
      const entiers = parts.map(Math.floor);
      let reste = N - entiers.reduce((a, b) => a + b, 0);
      parts.map((v, i) => [v - Math.floor(v), i]).sort((a, b) => b[0] - a[0]).forEach(([, i]) => { if (reste > 0) { entiers[i] += 1; reste -= 1; } });
      const cases = pesants.flatMap((p, i) => Array.from({ length: entiers[i] }, () => `<i class="${p.coche || p.partiel ? '' : 'off'}" style="background:${couleurs.get(p.cle)}"></i>`)).join('');
      graphique = `${entete}<div class="an-gaufre-zone"><div class="an-gaufre">${cases}</div>${liste(pesants)}</div>`;
    } else if (graphe === 'gauge') {
      const tout = pesants.every((p) => p.coche);
      graphique = `<div class="an-jauge-seule">${jaugeDents(choisi / total, `<b>${taille(choisi)}</b><span>${esc(t('an.a_liberer'))}</span><span>${esc(t('an.sur', { taille: taille(total) }))}</span>`)}
        <span class="an-interrupteur">${esc(t('an.tout_liberer', { taille: taille(total) }))}<button class="switch" type="button" role="switch" aria-checked="${tout}" data-a-tout-postes></button></span>
        <button class="an-lien" type="button" data-a-jauge-detail>${esc(t(detailJauge ? 'an.masquer_detail' : 'an.voir_detail'))}</button></div>
        ${detailJauge ? liste(pesants) : ''}`;
    } else if (graphe === 'crown') {
      graphique = `<div class="an-anneau-zone">${couronne(pesants, pesants.map((p) => couleurs.get(p.cle)), `<b>${taille(choisi)}</b><span>${esc(t('an.a_liberer'))}</span>`)}
        <div>${liste(pesants)}</div></div>`;
    } else {
      graphique = `<div class="an-anneau-zone">${anneau(pesants.map((p) => ({ taille: p.taille, coche: p.coche, partiel: p.partiel })), pesants.map((p) => couleurs.get(p.cle)), `<b>${taille(choisi)}</b><span>${esc(t('an.a_liberer'))}</span><span>${esc(t('an.sur', { taille: taille(total) }))}</span>`)}
        <div>${liste(pesants)}</div></div>`;
    }
  }
  const aussi = legers.length ? `${pesants.length ? `<div class="an-sous-titre">${esc(t('an.aussi'))}</div>` : ''}${liste(legers)}` : '';
  const diagnostics = modeles.map((m) => {
    const ms = m.metriques.filter((x) => montre(x, 'simple'));
    if (!ms.length) return '';
    const notes = m.notes.filter((n) => !n.cible && montre(n, 'simple')).map((n) => noteHtml(m, n)).join('');
    return `<div class="an-diagnostic"><div class="an-diagnostic-h">${esc(titreAction(m.entree))}</div>${vueFeu(m, 'simple')}${metriquesHtml(m, 'simple')}${notes}</div>`;
  }).join('');
  const globales = modeles.flatMap((m) => m.notes.filter((n) => !n.cible && montre(n, 'simple') && !m.metriques.length).map((n) => noteHtml(m, n))).join('');
  return `${graphique}${aussi}${globales}${diagnostics ? `<div class="an-sous-titre">${esc(t('an.etat_du_pc'))}</div>${diagnostics}` : ''}`
    || `<div class="an-vide">${esc(t('an.rien_trouve_tout'))}</div>`;
}

/** Ce que l'ensemble coche representera : pour la barre du bas. */
export function totalResume(modeles) {
  const tous = postesResume(modeles);
  return {
    choisi: tous.reduce((a, p) => a + p.choisi, 0),
    reglages: tous.reduce((a, p) => a + p.reglages, 0),
    elements: tous.reduce((a, p) => a + p.elements, 0),
  };
}

/* -------------------------------------------------------------------------
   Les gestes : un clic ou une case change la selection, l'appelant redessine
   ------------------------------------------------------------------------- */

const ETAT_UI = { ouverts: new Set(), detailJauge: false };
export const etatUi = () => ETAT_UI;

/**
 * Applique un geste sur l'analyse. `modeles` : Map id de script -> modele.
 * Renvoie vrai si quelque chose a change (l'appelant redessine).
 */
export function agir(ev, modeles) {
  const el = ev.target;
  const surBouton = ev.type === 'click';
  const posteDe = (cle) => {
    const [id] = cle.split('|');
    const m = modeles.get(id);
    return m && postesAction(m, 'simple').find((p) => p.cle === cle);
  };

  // --- Les postes du resume
  if (!surBouton && el.matches?.('[data-a-poste]')) {
    const p = posteDe(el.dataset.aPoste);
    if (p) basculerPoste(p, el.checked);
    return true;
  }
  const bouton = el.closest?.('button, th[data-a-tri]');
  if (surBouton && bouton) {
    const d = bouton.dataset;
    if (d.aDeplier) {
      ev.preventDefault();
      ETAT_UI.ouverts.has(d.aDeplier) ? ETAT_UI.ouverts.delete(d.aDeplier) : ETAT_UI.ouverts.add(d.aDeplier);
      return true;
    }
    if (d.aSwitch !== undefined && d.aPoste) {
      const p = posteDe(d.aPoste);
      if (p) basculerPoste(p, !p.coche);
      return true;
    }
    if (d.aJaugeDetail !== undefined) { ETAT_UI.detailJauge = !ETAT_UI.detailJauge; return true; }
    if (d.aToutPostes !== undefined) {
      const tous = postesResume([...modeles.values()]).filter((p) => p.taille > 0);
      const v = !tous.every((p) => p.coche);
      for (const p of tous) basculerPoste(p, v);
      return true;
    }
    if (d.aTri) { ETAT.tri.sens = ETAT.tri.col === d.aTri ? -ETAT.tri.sens : 1; ETAT.tri.col = d.aTri; return true; }
    const m = modeles.get(d.aScript);
    if (!m) return false;
    if (d.aAffichage) { etatListes(m).affichage = d.aAffichage; return true; }
    if (d.aTriListe) { etatListes(m).tri = d.aTriListe; return true; }
    const k = d.aOpt;
    const s = selection(m);
    const o = m.options[k];
    ev.preventDefault();
    if (d.aK === 'supprimer') { s.supprimer[k] = !s.supprimer[k]; return true; }
    if (d.aTout === 'multi') { for (const [c, f] of Object.entries(o.finds)) if (Number(f.size) > 0 || Number(f.count) > 0) s.choix[k][c] = true; return true; }
    if (d.aTout === 'items') { for (const it of o.items) if (!it.locked && it.state !== 'ok') s.items[k][it.id] = true; return true; }
    if (d.aRien === 'items') { for (const it of o.items) s.items[k][it.id] = false; return true; }
    if (d.aFiltre) { s.filtre[k] = d.aFiltre; return true; }
    if (d.aPlier) { s.plie[`${k}/${d.aPlier}`] = !estPlie(m, k, d.aPlier); return true; }
    if (d.aGarder) { s.garder[k][d.aGarder] = d.aItem; return true; }
    if (d.aTm) { ETAT.tm[`${m.entree.id}/${k}`] = d.aTm; return true; }
    if (d.aTmRetour !== undefined) { delete ETAT.tm[`${m.entree.id}/${k}`]; return true; }
    if (d.aTmItem) { s.items[k][d.aTmItem] = !s.items[k][d.aTmItem]; return true; }
    return false;
  }
  if (surBouton) return false;

  // --- Les cases
  if (el.matches?.('[data-a-cherche]')) { ETAT.recherche = el.value; return true; }
  if (el.matches?.('[data-a-cherche-liste]')) {
    const m = modeles.get(el.dataset.aScript);
    if (m) etatListes(m).q = el.value;
    return !!m;
  }
  const d = el.dataset || {};
  const m = modeles.get(d.aScript);
  if (!m || !d.aK) return false;
  const k = d.aOpt;
  const s = selection(m);
  const o = m.options[k];
  if (d.aK === 'bool') s.bool[k] = el.checked;
  else if (d.aK === 'choix') s.choix[k][d.aChoix] = el.checked;
  else if (d.aK === 'select') s.select[k] = d.aChoix;
  else if (d.aK === 'item') {
    const enfants = enfantsDe(o, d.aItem);
    // Un enfant que ce mode ne montre pas suit son parent, mais revient a l'avis
    // du script : cocher Chrome en Simple ne coche pas ses cookies, invisibles.
    const mode = d.aMode || 'expert';
    if (enfants.length) {
      for (const e of enfants) if (!e.locked && e.state !== 'ok') s.items[k][e.id] = el.checked && (montre(e, mode) || cocheItem(e));
    } else s.items[k][d.aItem] = el.checked;
  } else return false;
  return true;
}

/** Les cases a demi cochees n'ont pas d'attribut HTML : on les pose apres rendu. */
export function poserPartiels(racine) {
  racine.querySelectorAll('[data-partiel="1"]').forEach((c) => { c.indeterminate = true; });
}

/* -------------------------------------------------------------------------
   Pour les panneaux de l'Expert
   ------------------------------------------------------------------------- */

/**
 * « Ce qui sera fait » (2.0.4, M33): the exact list, after the boxes — all of
 * it, no longer the first eight of each list, grouped by option. Simple
 * settings first, without a heading; then one group per list or choice.
 */
function groupesPlan(m, mode) {
  const s = selection(m);
  const seuls = [];
  const groupes = [];
  const avant = (a, b) => String(a).localeCompare(String(b), undefined, { sensitivity: 'base', numeric: true });
  for (const [k, o] of Object.entries(m.options)) {
    const d = o.def;
    if (d.scan) continue;
    if (d.kind === 'bool' && o.find && s.bool[k]) seuls.push(`${esc(libOption(m.entree, d))}${o.find.size ? ` — ${taille(o.find.size)}` : ''}`);
    if (d.kind === 'select' && s.select[k]) {
      const actuel = Object.entries(o.finds).find(([, f]) => f.current === 'true')?.[0];
      if (actuel !== s.select[k]) seuls.push(`${esc(libOption(m.entree, d))} : ${esc(actuel ? libChoix(m.entree, d, actuel) : '?')} → ${esc(libChoix(m.entree, d, s.select[k]))}`);
    }
    if (d.kind === 'multi') {
      const l = Object.entries(o.finds).filter(([c]) => s.choix[k][c])
        .map(([c, f]) => `${esc(libChoix(m.entree, d, c))}${f.size ? ` — ${taille(f.size)}` : ''}${f.count ? ` (${esc(pl('an.n_fichiers', Number(f.count)))})` : ''}`);
      if (l.length) groupes.push({ titre: libOption(m.entree, d), lignes: l });
    }
    if (d.kind === 'items') {
      const liste = (d.keep_one ? aSupprimer(m, k) : feuilles(o).filter((it) => s.items[k][it.id]))
        .sort((a, b) => avant(nomElement(m, a), nomElement(m, b)));
      if (liste.length) {
        groupes.push({
          titre: libOption(m.entree, d),
          lignes: liste.map((it) => `<span class="an-plan-ico">${iconeElement(it)}</span><span class="an-plan-t">${esc(nomElement(m, it))}`
            + `${it.to ? ` <span class="an-pub">${esc(it.version || '')} → ${esc(it.to)}</span>` : ''}`
            + `${mode === 'expert' && it.path ? `<span class="an-chemin">${esc(it.path)}</span>` : ''}</span>`),
        });
      }
    }
  }
  if (seuls.length) groupes.unshift({ titre: '', lignes: seuls });
  return groupes;
}
const groupePlanHtml = (g) => `<div class="an-plan-g">${g.titre ? `<div class="an-plan-h">${esc(g.titre)}<span>${g.lignes.length}</span></div>` : ''}`
  + `<ul class="an-plan">${g.lignes.map((l) => `<li>${l}</li>`).join('')}</ul></div>`;

export function plan(m) {
  const g = groupesPlan(m, 'expert');
  return g.length ? g.map(groupePlanHtml).join('') : `<span class="muted">${esc(t('an.plan_vide'))}</span>`;
}

/** The same for the Simple screen (2.0.4): every action of the lot, under
 *  its title when there are several. `n`: how many lines in all. */
export function planResume(modeles) {
  const parts = modeles.map((m) => ({ m, g: groupesPlan(m, 'simple') })).filter((x) => x.g.length);
  const n = parts.reduce((a, x) => a + x.g.reduce((b, g) => b + g.lignes.length, 0), 0);
  const html = parts.map(({ m, g }) => `${parts.length > 1 ? `<div class="an-plan-action">${esc(titreAction(m.entree))}</div>` : ''}${g.map(groupePlanHtml).join('')}`).join('');
  return { n, html };
}
