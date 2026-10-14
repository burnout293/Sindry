//! Installation silencieuse avec options (2.0, specification §19.3).
//!
//! `Sindry_2.0.0_x64-setup.exe /S /LANG=en /THEME=sombre /SANSBIENVENUE` : les
//! options ne peuvent pas etre ecrites dans les reglages de l'utilisateur —
//! l'installeur tourne souvent sous un autre compte (administrateur, SYSTEM
//! d'un outil de deploiement), et les reglages vivent dans le profil de chacun.
//! Elles sont donc ecrites a cote de l'executable, dans
//! `preconfiguration.json`, que Sindry lit au **premier lancement de chaque
//! compte** : quand ce compte n'a pas encore de `settings.json`. Un compte qui a
//! deja ses reglages les garde, une mise a jour ne touche a rien.
//!
//! Le fichier est dans le dossier d'installation, que seul un administrateur
//! peut ecrire. Il ne porte que des reglages — ceux que l'utilisateur peut de
//! toute facon changer lui-meme —, et ce qui part a un script passe toujours
//! par la garde (§12.4).

use crate::settings::{self, Settings};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const FICHIER: &str = "preconfiguration.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Preconfiguration {
    pub format: u32,
    /// Toute une configuration exportee (Reglages, 5.1) : lots, reglages par
    /// action, preferences. Les autres champs s'appliquent par-dessus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<Settings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub langue: Option<String>,
    /// `light`, `dark` ou `system`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    /// Une ambiance livree (`plan`…) ou installee (`perso:<id>`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ambiance: Option<String>,
    /// Faux : pas de bienvenue au premier lancement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bienvenue: Option<bool>,
}

impl Preconfiguration {
    pub fn est_vide(&self) -> bool {
        self.config.is_none()
            && self.langue.is_none()
            && self.theme.is_none()
            && self.ambiance.is_none()
            && self.bienvenue.is_none()
    }

    /// Les reglages d'un premier lancement : ceux d'usine, ou la configuration
    /// fournie, puis chaque option.
    pub fn appliquer(&self, usine: Settings) -> Settings {
        let mut s = self.config.clone().unwrap_or(usine);
        if let Some(l) = &self.langue {
            s.lang = l.clone();
        }
        if let Some(t) = &self.theme {
            s.theme = t.clone();
        }
        if let Some(a) = &self.ambiance {
            s.accent = a.clone();
        }
        if self.bienvenue == Some(false) {
            s.onboarded = true;
        }
        s
    }
}

/// Le dossier de l'executable : celui de l'installation.
pub fn dossier_installation() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
}

/// La preconfiguration du dossier d'installation, si elle existe et se lit.
/// Illisible, elle est ignoree : un premier lancement n'est jamais bloque.
pub fn lire() -> Option<Preconfiguration> {
    lire_dans(&dossier_installation()?)
}

pub fn lire_dans(dossier: &Path) -> Option<Preconfiguration> {
    let brut = fs::read_to_string(dossier.join(FICHIER)).ok()?;
    serde_json::from_str::<Preconfiguration>(brut.trim_start_matches('\u{feff}'))
        .ok()
        .filter(|p| p.format == 1)
}

/// Les options de l'installeur, telles que NSIS les passe (sa ligne de
/// commande entiere : `/S /LANG=en /CONFIG="C:\a b\c.json" /D=…`). Ce qui
/// n'est pas une option de Sindry est ignore — les options de l'installeur
/// lui-meme, le chemin de `/D=` coupe a ses espaces. Rend aussi `/PATH`.
pub fn depuis_options(arguments: &[String]) -> Result<(Preconfiguration, bool), String> {
    let mut p = Preconfiguration {
        format: 1,
        ..Default::default()
    };
    let mut chemin = false;
    for a in arguments {
        let (nom, valeur) = match a.split_once('=') {
            Some((n, v)) => (n.to_ascii_uppercase(), Some(v.trim().trim_matches('"'))),
            None => (a.to_ascii_uppercase(), None),
        };
        match (nom.as_str(), valeur) {
            ("/LANG", Some(v)) => {
                let v = v.to_ascii_lowercase();
                if !["fr", "en"].contains(&v.as_str()) {
                    return Err(format!("/LANG={v} : fr ou en."));
                }
                p.langue = Some(v);
            }
            ("/THEME", Some(v)) => {
                p.theme = Some(
                    match v.to_ascii_lowercase().as_str() {
                        "clair" | "light" => "light",
                        "sombre" | "dark" => "dark",
                        "systeme" | "système" | "system" => "system",
                        autre => return Err(format!("/THEME={autre} : clair, sombre ou systeme.")),
                    }
                    .to_string(),
                );
            }
            ("/AMBIANCE", Some(v)) => {
                let v = v.to_ascii_lowercase();
                if settings::ACCENTS.contains(&v.as_str()) {
                    p.ambiance = Some(v);
                } else if crate::ambiance::id_valide(&v) {
                    // Une ambiance a installer ensuite : elle n'existe pas encore.
                    p.ambiance = Some(format!("perso:{v}"));
                } else {
                    return Err(format!("/AMBIANCE={v} : le nom d'une ambiance."));
                }
            }
            ("/SANSBIENVENUE" | "/NOWELCOME", None) => p.bienvenue = Some(false),
            ("/CONFIG", Some(v)) => {
                let brut = fs::read_to_string(v).map_err(|e| format!("/CONFIG={v} : {e}"))?;
                let s: Settings = serde_json::from_str(brut.trim_start_matches('\u{feff}'))
                    .map_err(|e| format!("/CONFIG={v} : configuration invalide ({e})."))?;
                p.config = Some(s);
            }
            ("/PATH", None) => chemin = true,
            _ => {}
        }
    }
    Ok((p, chemin))
}

/// Ecrit la preconfiguration dans le dossier d'installation.
pub fn ecrire_dans(dossier: &Path, p: &Preconfiguration) -> Result<PathBuf, String> {
    let cible = dossier.join(FICHIER);
    let json = serde_json::to_string_pretty(p).map_err(|e| e.to_string())?;
    fs::write(&cible, json).map_err(|e| format!("{} : {e}", cible.display()))?;
    Ok(cible)
}

/* ------------------------------------------------------------------------
Le dossier d'installation dans le PATH du systeme (/PATH)
------------------------------------------------------------------------ */

fn meme_dossier(a: &str, b: &str) -> bool {
    let n = |s: &str| s.trim().trim_end_matches('\\').to_ascii_lowercase();
    !a.trim().is_empty() && n(a) == n(b)
}

/// Le `Path` avec `dossier` a la fin, s'il n'y est pas deja.
pub fn chemin_avec(path: &str, dossier: &str) -> String {
    if path.split(';').any(|e| meme_dossier(e, dossier)) {
        return path.to_string();
    }
    let base = path.trim_end_matches(';');
    if base.is_empty() {
        dossier.to_string()
    } else {
        format!("{base};{dossier}")
    }
}

/// Le `Path` sans `dossier`. Les autres entrees ne bougent pas.
pub fn chemin_sans(path: &str, dossier: &str) -> String {
    path.split(';')
        .filter(|e| !meme_dossier(e, dossier))
        .collect::<Vec<_>>()
        .join(";")
}

#[cfg(windows)]
const CLE_ENVIRONNEMENT: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";

/// Ajoute (`true`) ou retire le dossier d'installation du `Path` du systeme,
/// puis previent les programmes ouverts. Administrateur seulement.
#[cfg(windows)]
pub fn chemin_systeme(ajouter: bool) -> Result<bool, String> {
    let dossier = dossier_installation().ok_or("dossier d'installation introuvable")?;
    let dossier = dossier.to_string_lossy().to_string();
    let cle = windows_registry::LOCAL_MACHINE
        .options()
        .read()
        .write()
        .open(CLE_ENVIRONNEMENT)
        .map_err(|e| format!("HKLM\\{CLE_ENVIRONNEMENT} : {e}"))?;
    let actuel = cle
        .get_hstring("Path")
        .map(|h| h.to_string_lossy())
        .unwrap_or_default();
    let nouveau = if ajouter {
        chemin_avec(&actuel, &dossier)
    } else {
        chemin_sans(&actuel, &dossier)
    };
    if nouveau == actuel {
        return Ok(false);
    }
    cle.set_expand_string("Path", &nouveau)
        .map_err(|e| format!("Path : {e}"))?;
    prevenir_changement_environnement();
    Ok(true)
}

#[cfg(not(windows))]
pub fn chemin_systeme(_ajouter: bool) -> Result<bool, String> {
    Ok(false)
}

/// `WM_SETTINGCHANGE` « Environment » : l'Explorateur relit le `Path`, les
/// consoles ouvertes ensuite le voient.
#[cfg(windows)]
fn prevenir_changement_environnement() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };
    let texte: Vec<u16> = "Environment".encode_utf16().chain(Some(0)).collect();
    let mut resultat = 0usize;
    // SAFETY: `texte` est une chaine UTF-16 terminee par un zero, vivante
    // pendant tout l'appel ; le resultat est ecrit dans une variable locale.
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            texte.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            &mut resultat,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn les_options_de_l_installeur_se_lisent_et_le_reste_est_ignore() {
        let (p, chemin) = depuis_options(&args(&[
            r"C:\Temp\Sindry_2.0.0_x64-setup.exe",
            "/S",
            "/lang=EN",
            "/THEME=sombre",
            "/AMBIANCE=foret-de-nuit",
            "/SANSBIENVENUE",
            "/PATH",
            r"/D=C:\Program",
            "Files\\Sindry",
        ]))
        .unwrap();
        assert!(chemin);
        assert_eq!(p.langue.as_deref(), Some("en"));
        assert_eq!(p.theme.as_deref(), Some("dark"));
        assert_eq!(p.ambiance.as_deref(), Some("perso:foret-de-nuit"));
        assert_eq!(p.bienvenue, Some(false));
        let (p, _) = depuis_options(&args(&["/AMBIANCE=Synthwave"])).unwrap();
        assert_eq!(p.ambiance.as_deref(), Some("synthwave"));
        let (p, chemin) = depuis_options(&args(&["/S"])).unwrap();
        assert!(p.est_vide() && !chemin);
        assert!(depuis_options(&args(&["/LANG=de"])).is_err());
        assert!(depuis_options(&args(&["/THEME=rose"])).is_err());
        assert!(depuis_options(&args(&["/CONFIG=Z:/introuvable.json"])).is_err());
    }

    #[test]
    fn la_preconfiguration_part_de_l_usine_ou_de_la_configuration_fournie() {
        let usine = settings::default_settings(Vec::new());
        let p = Preconfiguration {
            format: 1,
            langue: Some("en".into()),
            bienvenue: Some(false),
            ..Default::default()
        };
        let s = p.appliquer(usine.clone());
        assert_eq!((s.lang.as_str(), s.onboarded), ("en", true));
        assert_eq!(s.theme, usine.theme);

        let mut fournie = usine.clone();
        fournie.failure_policy = "stop".into();
        let p = Preconfiguration {
            format: 1,
            config: Some(fournie),
            theme: Some("dark".into()),
            ..Default::default()
        };
        let s = p.appliquer(usine);
        assert_eq!(
            (s.failure_policy.as_str(), s.theme.as_str()),
            ("stop", "dark")
        );

        let dossier = std::env::temp_dir().join(format!("sindry-preconfig-{}", std::process::id()));
        fs::create_dir_all(&dossier).unwrap();
        ecrire_dans(&dossier, &p).unwrap();
        let relue = lire_dans(&dossier).expect("relue");
        assert_eq!(
            serde_json::to_value(&relue).unwrap(),
            serde_json::to_value(&p).unwrap()
        );
        fs::write(dossier.join(FICHIER), "{ pas du json").unwrap();
        assert!(lire_dans(&dossier).is_none());
        let _ = fs::remove_dir_all(&dossier);
    }

    #[test]
    fn le_path_gagne_ou_perd_le_dossier_une_seule_fois() {
        let d = r"C:\Program Files\Sindry";
        assert_eq!(
            chemin_avec(r"C:\Windows;C:\Outils", d),
            r"C:\Windows;C:\Outils;C:\Program Files\Sindry"
        );
        assert_eq!(
            chemin_avec(r"C:\Windows;c:\program files\sindry\", d),
            r"C:\Windows;c:\program files\sindry\"
        );
        assert_eq!(chemin_avec("", d), d);
        assert_eq!(
            chemin_avec(r"C:\Windows;", d),
            r"C:\Windows;C:\Program Files\Sindry"
        );
        assert_eq!(
            chemin_sans(r"C:\Windows;C:\Program Files\Sindry;C:\Outils", d),
            r"C:\Windows;C:\Outils"
        );
        assert_eq!(
            chemin_sans(r"C:\Windows;C:\Outils", d),
            r"C:\Windows;C:\Outils"
        );
    }
}
