// Le site de Sindry (2.2.0) : le thème choisi, et la recherche du catalogue.
// Rien ne sort de la page : pas de requête, pas de cookie.

(() => {
  const THEMES = ['auto', 'light', 'dark'];
  const LIBELLES = { auto: 'Thème : système', light: 'Thème : clair', dark: 'Thème : sombre' };
  const lire = () => { try { return localStorage.getItem('theme') || 'auto'; } catch { return 'auto'; } };
  const appliquer = (t) => {
    if (t === 'auto') delete document.documentElement.dataset.theme;
    else document.documentElement.dataset.theme = t;
    const b = document.getElementById('themeBtn');
    if (b) b.textContent = LIBELLES[t];
  };
  appliquer(lire());
  document.addEventListener('DOMContentLoaded', () => {
    appliquer(lire());
    document.getElementById('themeBtn')?.addEventListener('click', () => {
      const t = THEMES[(THEMES.indexOf(lire()) + 1) % THEMES.length];
      try { localStorage.setItem('theme', t); } catch { /* navigation privée : le choix vaut pour la page */ }
      appliquer(t);
    });

    // Le catalogue : filtrer sur le titre, la description et le nom du fichier.
    const champ = document.getElementById('catCherche');
    if (!champ) return;
    const compte = document.getElementById('catCompte');
    const filtrer = () => {
      const q = champ.value.trim().toLowerCase().normalize('NFD').replace(/[̀-ͯ]/g, '');
      let n = 0;
      for (const s of document.querySelectorAll('.script')) {
        const ok = !q || s.dataset.cherche.includes(q);
        s.hidden = !ok;
        if (ok) n++;
      }
      for (const c of document.querySelectorAll('.cat')) c.hidden = !c.querySelector('.script:not([hidden])');
      compte.textContent = `${n} action${n > 1 ? 's' : ''}`;
    };
    champ.addEventListener('input', filtrer);
    filtrer();
  });
})();
