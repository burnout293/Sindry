//! Migration depuis WinTool, le nom de l'application jusqu'a la 1.8 (2.0).
//!
//! Sindry reprend ce que WinTool avait sur le PC, puis le retire. Rien ne se
//! perd : reglages, historique, journaux, catalogues installes, scripts de
//! l'utilisateur (le dossier local) ; approbations, emplacements proteges et
//! catalogues tiers (HKLM).
//!
//! Deux moments :
//!
//! - **l'installeur** (crochet NSIS_HOOK_POSTINSTALL, en administrateur) lance
//!   `sindry.exe --migrer-depuis-wintool`, sans fenetre : tout est fait, y
//!   compris la desinstallation de WinTool ;
//! - **chaque demarrage** refait ce qui ne demande pas de droits — le dossier
//!   local, s'il reste —, et le reste s'il en a. Chaque etape ne fait rien si
//!   elle est deja faite : relancer la migration est sans effet.
//!
//! C'est, avec la lecture de l'ancien contrat des scripts
//! (`contract::depuis_ancien_contrat`), le seul endroit qui nomme encore
//! WinTool. Les deux disparaitront en 2.1.

use std::fs;
use std::path::{Path, PathBuf};

/// L'ancien nom : dossiers, cle de registre, desinstallation, processus.
const ANCIEN: &str = "WinTool";
/// L'ancien identifiant Tauri : le profil WebView2 de l'ancienne interface.
const ANCIEN_IDENTIFIANT: &str = "com.wintool.app";
/// Le nouveau nom, celui de `discovery::base_dir` et des cles HKLM.
const NOUVEAU: &str = "Sindry";

fn local_appdata() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

/// Deplace le contenu de `ancien` dans `nouveau` : d'un bloc si `nouveau`
/// n'existe pas, sinon entree par entree, sans jamais ecraser ce qui y est
/// deja. `ancien` disparait s'il ne reste rien dedans.
pub fn deplacer_dossier(ancien: &Path, nouveau: &Path) -> Result<usize, String> {
    if !ancien.is_dir() {
        return Ok(0);
    }
    if !nouveau.exists() {
        fs::rename(ancien, nouveau).map_err(|e| format!("{} : {e}", ancien.display()))?;
        return Ok(1);
    }
    let mut n = 0;
    for entree in fs::read_dir(ancien).map_err(|e| e.to_string())?.flatten() {
        let cible = nouveau.join(entree.file_name());
        if !cible.exists() && fs::rename(entree.path(), &cible).is_ok() {
            n += 1;
        }
    }
    let _ = fs::remove_dir(ancien);
    Ok(n)
}

/// Ce qui ne demande pas de droits : le dossier local de l'utilisateur, et le
/// profil WebView2 de l'ancienne interface (son cache, rien d'autre).
pub fn donnees_utilisateur() -> Vec<String> {
    let mut faits = Vec::new();
    let Some(base) = local_appdata() else {
        return faits;
    };
    match deplacer_dossier(&base.join(ANCIEN), &base.join(NOUVEAU)) {
        Ok(0) => {}
        Ok(_) => faits.push(format!(
            r"%LOCALAPPDATA%\{ANCIEN} repris dans %LOCALAPPDATA%\{NOUVEAU}"
        )),
        Err(e) => faits.push(format!("dossier local non repris : {e}")),
    }
    let profil = base.join(ANCIEN_IDENTIFIANT);
    if profil.is_dir() && fs::remove_dir_all(&profil).is_ok() {
        faits.push(format!(
            "profil de l'ancienne interface ({ANCIEN_IDENTIFIANT}) efface"
        ));
    }
    faits
}

/// Ce qui demande les droits administrateur : le magasin HKLM (approbations,
/// emplacements proteges, catalogues tiers), puis l'ancienne installation.
#[cfg(windows)]
pub fn systeme() -> Vec<String> {
    let mut faits = Vec::new();
    let copie = copier_magasin();
    match &copie {
        Ok(0) => {}
        Ok(n) => faits.push(format!(r"HKLM\SOFTWARE\{ANCIEN} : {n} cle(s) reprise(s)")),
        Err(e) => faits.push(format!("registre non repris : {e}")),
    }
    match desinstaller_wintool() {
        Ok(true) => faits.push(format!("{ANCIEN} desinstalle")),
        Ok(false) => {}
        Err(e) => faits.push(format!("{ANCIEN} non desinstalle : {e}")),
    }
    // Ce que la desinstallation laisse : la cle de produit de Tauri et le
    // reste du magasin — seulement s'il a bien ete copie.
    if copie.is_err() {
        return faits;
    }
    let _ = windows_registry::LOCAL_MACHINE
        .options()
        .read()
        .write()
        .open("SOFTWARE")
        .and_then(|k| k.remove_tree(ANCIEN));
    faits
}

#[cfg(not(windows))]
pub fn systeme() -> Vec<String> {
    Vec::new()
}

/// Copie une cle et tout ce qu'elle contient, sans ecraser une valeur deja la.
#[cfg(windows)]
fn copier_cle(source: &windows_registry::Key, cible: &windows_registry::Key) -> Result<(), String> {
    for (nom, valeur) in source.values().map_err(|e| e.to_string())? {
        if cible.get_value(&nom).is_err() {
            cible.set_value(&nom, &valeur).map_err(|e| e.to_string())?;
        }
    }
    for nom in source.keys().map_err(|e| e.to_string())? {
        let s = source.open(&nom).map_err(|e| e.to_string())?;
        let c = cible.create(&nom).map_err(|e| e.to_string())?;
        copier_cle(&s, &c)?;
    }
    Ok(())
}

/// `HKLM\SOFTWARE\WinTool` vers `HKLM\SOFTWARE\Sindry`, sous-cle par sous-cle.
/// Pas un renommage : l'installeur de Sindry a deja cree `SOFTWARE\Sindry`
/// (sa cle de produit, `Software\<editeur>`, que le registre confond avec la
/// notre faute de distinguer la casse). La cle de produit de WinTool, elle,
/// ne se copie pas.
#[cfg(windows)]
fn copier_magasin() -> Result<usize, String> {
    let lm = windows_registry::LOCAL_MACHINE;
    let Ok(ancien) = lm.open(format!(r"SOFTWARE\{ANCIEN}")) else {
        return Ok(0);
    };
    let nouveau = lm
        .create(format!(r"SOFTWARE\{NOUVEAU}"))
        .map_err(|e| e.to_string())?;
    let mut n = 0;
    for nom in ancien.keys().map_err(|e| e.to_string())? {
        if nom.eq_ignore_ascii_case(ANCIEN) {
            continue;
        }
        let s = ancien.open(&nom).map_err(|e| e.to_string())?;
        let c = nouveau.create(&nom).map_err(|e| e.to_string())?;
        copier_cle(&s, &c)?;
        n += 1;
    }
    Ok(n)
}

/// Ferme WinTool s'il tourne encore, puis lance son desinstalleur en silence
/// (ses donnees ont deja ete reprises). Rend `false` s'il n'etait pas installe.
#[cfg(windows)]
fn desinstaller_wintool() -> Result<bool, String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let cle = format!(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{ANCIEN}");
    let Ok(inscription) = windows_registry::LOCAL_MACHINE.open(&cle) else {
        return Ok(false);
    };
    let commande = inscription
        .get_string("UninstallString")
        .unwrap_or_default();
    let desinstalleur = PathBuf::from(commande.trim().trim_matches('"'));
    if !desinstalleur.is_file() {
        let _ = windows_registry::LOCAL_MACHINE
            .options()
            .read()
            .write()
            .open(r"Software\Microsoft\Windows\CurrentVersion\Uninstall")
            .and_then(|k| k.remove_tree(ANCIEN));
        return Ok(false);
    }
    let dossier = desinstalleur
        .parent()
        .map(Path::to_path_buf)
        .ok_or("dossier de WinTool introuvable")?;

    // L'ancienne application ne doit plus tenir ses fichiers ouverts.
    let systeme32 = crate::systeme::emplacements().system32();
    let _ = std::process::Command::new(systeme32.join("taskkill.exe"))
        .args(["/F", "/IM", &format!("{}.exe", ANCIEN.to_ascii_lowercase())])
        .creation_flags(CREATE_NO_WINDOW)
        .status();

    // `_?=` : le desinstalleur NSIS travaille sur place et rend la main a la
    // fin, au lieu de se recopier ailleurs et de revenir aussitot.
    let statut = std::process::Command::new(&desinstalleur)
        .arg("/S")
        .arg(format!("_?={}", dossier.display()))
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| e.to_string())?;
    // Lance ainsi, il ne peut pas s'effacer lui-meme : on le fait pour lui.
    let _ = fs::remove_file(&desinstalleur);
    let _ = fs::remove_dir(&dossier);
    if !statut.success() {
        return Err(format!("code {:?}", statut.code()));
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deplacer_un_dossier_ne_perd_ni_n_ecrase_rien() {
        let racine = std::env::temp_dir().join(format!("sindry-migration-{}", std::process::id()));
        let _ = fs::remove_dir_all(&racine);
        let (a, n) = (racine.join("Ancien"), racine.join("Nouveau"));

        // Le cas courant : rien de neuf encore, le dossier part d'un bloc.
        fs::create_dir_all(a.join("logs")).unwrap();
        fs::write(a.join("settings.json"), b"ancien").unwrap();
        assert_eq!(deplacer_dossier(&a, &n).unwrap(), 1);
        assert!(!a.exists() && n.join("logs").is_dir());

        // Les deux existent : ce qui manque arrive, ce qui est la reste.
        fs::create_dir_all(a.join("sources")).unwrap();
        fs::write(a.join("settings.json"), b"plus ancien").unwrap();
        assert_eq!(deplacer_dossier(&a, &n).unwrap(), 1);
        assert_eq!(fs::read(n.join("settings.json")).unwrap(), b"ancien");
        assert!(n.join("sources").is_dir());
        assert!(a.exists(), "un fichier en conflit reste a sa place");

        // Deja fait : rien ne bouge.
        let _ = fs::remove_dir_all(&a);
        assert_eq!(deplacer_dossier(&a, &n).unwrap(), 0);
        let _ = fs::remove_dir_all(&racine);
    }
}
