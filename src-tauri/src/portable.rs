//! La version portable (2.4.0) : Sindry sans installation, sur une cle USB.
//!
//! Un fichier `sindry-portable` a cote de l'exe fait de ce dossier une version
//! portable. Les donnees vivent alors dans `Donnees\`, a cote de lui, au lieu
//! de `%LOCALAPPDATA%\Sindry` : reglages, catalogues, scripts, journaux,
//! historique, et le dossier de WebView2. Rien de ce que fait une installation
//! ne se fait :
//! - ni migration depuis WinTool, qui deplacerait les donnees du PC hote et
//!   desinstallerait son ancienne version ;
//! - ni preconfiguration d'installation silencieuse (§19.3), propre au PC ;
//! - ni mise a jour par l'installeur, qui installerait dans Program Files :
//!   la version portable se remplace par la nouvelle archive ;
//! - ni tache de relance apres un redemarrage (§6.10) : elle ne lance qu'un
//!   exe de Program Files. Le journal de reprise reste : rouvert a la main,
//!   Sindry propose de reprendre.
//!
//! **La confiance ne change pas de place.** Approbations, garde, catalogues
//! ajoutes et plafond de redemarrages restent dans le HKLM du PC ou Sindry
//! tourne (§12.4). Une cle USB est inscriptible par quiconque la tient : rien
//! de ce qui fonde une decision de securite n'y est lu. Les catalogues qu'elle
//! porte sont reverifies par leur signature a chaque decouverte, comme ailleurs.
//!
//! Le marqueur est lu a cote de l'exe, jamais depuis une variable
//! d'environnement : l'emplacement des donnees se decide par ce qui est
//! installe, pas par qui lance (meme regle que `discovery::dossier_essai`).

use std::path::PathBuf;
use std::sync::OnceLock;

/// Le fichier qui fait du dossier de l'exe une version portable.
pub const MARQUEUR: &str = "sindry-portable";
/// Le dossier des donnees, a cote de l'exe.
pub const DOSSIER: &str = "Donnees";

/// Le dossier des donnees, si cet exe est une version portable.
pub fn dossier() -> Option<PathBuf> {
    static CACHE: OnceLock<Option<PathBuf>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let exe = std::env::current_exe().ok()?;
            dossier_pour(exe.parent()?)
        })
        .clone()
}

/// Coeur de [`dossier`], pour un dossier donne (verifiable sans exe).
pub fn dossier_pour(dossier_exe: &std::path::Path) -> Option<PathBuf> {
    dossier_exe
        .join(MARQUEUR)
        .is_file()
        .then(|| crate::discovery::sans_prefixe_long(dossier_exe.join(DOSSIER)))
}

/// Sindry tourne-t-il en version portable ?
pub fn actif() -> bool {
    dossier().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_marqueur_seul_fait_une_version_portable() {
        let d = std::env::temp_dir().join(format!("sindry-portable-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        assert_eq!(dossier_pour(&d), None, "sans marqueur, une installation");
        // Un dossier du meme nom ne compte pas : le marqueur est un fichier.
        std::fs::create_dir_all(d.join(MARQUEUR)).unwrap();
        assert_eq!(dossier_pour(&d), None);
        std::fs::remove_dir_all(d.join(MARQUEUR)).unwrap();
        std::fs::write(d.join(MARQUEUR), b"").unwrap();
        assert_eq!(dossier_pour(&d), Some(d.join(DOSSIER)));
        let _ = std::fs::remove_dir_all(&d);
    }
}
