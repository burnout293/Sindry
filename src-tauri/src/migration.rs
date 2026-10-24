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
//! WinTool. Les deux disparaitront en v3.

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

/// True for anything that must not be followed: a symbolic link, a junction or
/// any other reparse point — or a path that cannot be read at all.
fn is_link(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Err(_) => true,
        #[cfg(windows)]
        Ok(m) => {
            use std::os::windows::fs::MetadataExt;
            const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
            m.file_type().is_symlink() || m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
        }
        #[cfg(not(windows))]
        Ok(m) => m.file_type().is_symlink(),
    }
}

/// Moves the content of `old` into `new`: in one go when `new` does not exist,
/// otherwise entry by entry, going down into the folders both sides have, and
/// never overwriting what is already there. `old` goes once it is empty.
/// Returns the number of entries moved.
///
/// Up to 2.0.1 the merge stopped at the first level: a `scripts` folder that
/// already existed on the Sindry side kept the user's personal scripts back in
/// the old one. Links are never followed: both folders are writable without
/// elevation, and the installer runs this as administrator.
pub fn move_dir_merging(old: &Path, new: &Path) -> Result<usize, String> {
    if is_link(old) || !old.is_dir() {
        return Ok(0);
    }
    if fs::symlink_metadata(new).is_err() {
        fs::rename(old, new).map_err(|e| format!("{} : {e}", old.display()))?;
        return Ok(1);
    }
    if is_link(new) || !new.is_dir() {
        return Ok(0);
    }
    let mut moved = 0;
    for entry in fs::read_dir(old).map_err(|e| e.to_string())?.flatten() {
        let source = entry.path();
        let target = new.join(entry.file_name());
        if fs::symlink_metadata(&target).is_err() {
            if fs::rename(&source, &target).is_ok() {
                moved += 1;
            }
        } else if !is_link(&source) && source.is_dir() {
            moved += move_dir_merging(&source, &target)?;
        }
    }
    let _ = fs::remove_dir(old);
    Ok(moved)
}

/// What the old installer leaves in the user's hive: NSIS remembers the
/// installer language under `HKCU\Software\WinTool\WinTool`. The key goes only
/// if that is all it holds, and the vendor key with it if nothing else lives
/// there.
#[cfg(windows)]
fn remove_old_installer_key() -> bool {
    let Ok(software) = windows_registry::CURRENT_USER
        .options()
        .read()
        .write()
        .open("Software")
    else {
        return false;
    };
    let product = format!(r"{ANCIEN}\{ANCIEN}");
    let only_language = software.open(&product).is_ok_and(|k| {
        k.keys().is_ok_and(|mut sub| sub.next().is_none())
            && k.values()
                .is_ok_and(|mut v| v.all(|(name, _)| name == "Installer Language"))
    });
    if !only_language || software.remove_tree(&product).is_err() {
        return false;
    }
    let vendor_empty = software.open(ANCIEN).is_ok_and(|k| {
        k.keys().is_ok_and(|mut sub| sub.next().is_none())
            && k.values().is_ok_and(|mut v| v.next().is_none())
    });
    if vendor_empty {
        let _ = software.remove_tree(ANCIEN);
    }
    true
}

#[cfg(not(windows))]
fn remove_old_installer_key() -> bool {
    false
}

/// Ce qui ne demande pas de droits : le dossier local de l'utilisateur, et le
/// profil WebView2 de l'ancienne interface (son cache, rien d'autre).
pub fn donnees_utilisateur() -> Vec<String> {
    let mut faits = Vec::new();
    if remove_old_installer_key() {
        faits.push(format!(
            r"cle HKCU\Software\{ANCIEN} de l'ancien installeur effacee"
        ));
    }
    let Some(base) = local_appdata() else {
        return faits;
    };
    match move_dir_merging(&base.join(ANCIEN), &base.join(NOUVEAU)) {
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
    // Ce que son desinstalleur laisse : les scripts que l'application livrait
    // jusqu'a la 1.1.1, sous Program Files. Plus rien ne les lit (§4.3).
    let dossier = crate::systeme::emplacements().program_files.join(ANCIEN);
    match effacer_scripts_livres(&dossier) {
        Ok(0) => {}
        Ok(n) => faits.push(format!(
            "{} : {n} ancien(s) script(s) livre(s) efface(s)",
            dossier.display()
        )),
        Err(e) => faits.push(format!("{} non efface : {e}", dossier.display())),
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

/// Efface `<dossier>\scripts\Default\*.ps1`, puis les dossiers devenus vides,
/// jusqu'a `dossier` lui-meme. Rien d'autre : un fichier qui n'est pas un
/// `.ps1` livre reste, et ses dossiers avec. Jamais a travers un lien — le
/// dossier est sous Program Files, mais un effacement en administrateur ne
/// suit pas une jonction, ou qu'elle soit (§12.4). Rend le nombre de scripts
/// effaces.
pub fn effacer_scripts_livres(dossier: &Path) -> Result<usize, String> {
    let scripts = dossier.join("scripts");
    let livres = scripts.join("Default");
    for d in [dossier, &scripts, &livres] {
        match fs::symlink_metadata(d) {
            Err(_) => return Ok(0),
            Ok(m) if m.file_type().is_symlink() || !m.is_dir() => return Ok(0),
            Ok(_) => {}
        }
    }
    let mut n = 0;
    for entree in fs::read_dir(&livres).map_err(|e| e.to_string())?.flatten() {
        let chemin = entree.path();
        let script = entree.file_type().is_ok_and(|t| t.is_file())
            && chemin
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("ps1"));
        if script && fs::remove_file(&chemin).is_ok() {
            n += 1;
        }
    }
    // `remove_dir` n'efface qu'un dossier vide : ce qui reste les garde.
    for d in [&livres, &scripts, &dossier.to_path_buf()] {
        let _ = fs::remove_dir(d);
    }
    Ok(n)
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
        assert_eq!(move_dir_merging(&a, &n).unwrap(), 1);
        assert!(!a.exists() && n.join("logs").is_dir());

        // Les deux existent : ce qui manque arrive, ce qui est la reste.
        fs::create_dir_all(a.join("sources")).unwrap();
        fs::write(a.join("settings.json"), b"plus ancien").unwrap();
        assert_eq!(move_dir_merging(&a, &n).unwrap(), 1);
        assert_eq!(fs::read(n.join("settings.json")).unwrap(), b"ancien");
        assert!(n.join("sources").is_dir());
        assert!(a.exists(), "un fichier en conflit reste a sa place");

        // Deja fait : rien ne bouge.
        let _ = fs::remove_dir_all(&a);
        assert_eq!(move_dir_merging(&a, &n).unwrap(), 0);
        let _ = fs::remove_dir_all(&racine);
    }

    #[test]
    fn the_merge_goes_down_into_folders_both_sides_have() {
        let root = std::env::temp_dir().join(format!("sindry-merge-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let (old, new) = (root.join("Old"), root.join("New"));
        // Up to 2.0.1, an existing `scripts` on the new side kept the user's
        // own scripts back in the old folder.
        fs::create_dir_all(old.join("scripts").join("Mine")).unwrap();
        fs::write(old.join("scripts").join("Mine").join("a.ps1"), b"mine").unwrap();
        fs::write(old.join("scripts").join("b.ps1"), b"old b").unwrap();
        fs::create_dir_all(new.join("scripts")).unwrap();
        fs::write(new.join("scripts").join("b.ps1"), b"new b").unwrap();

        assert_eq!(move_dir_merging(&old, &new).unwrap(), 1);
        assert_eq!(
            fs::read(new.join("scripts").join("Mine").join("a.ps1")).unwrap(),
            b"mine"
        );
        assert_eq!(
            fs::read(new.join("scripts").join("b.ps1")).unwrap(),
            b"new b",
            "nothing is overwritten"
        );
        assert!(
            old.join("scripts").join("b.ps1").exists(),
            "a conflicting file stays where it was"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn seuls_les_scripts_livres_sont_effaces() {
        let racine = std::env::temp_dir().join(format!("sindry-livres-{}", std::process::id()));
        let _ = fs::remove_dir_all(&racine);
        let livres = racine.join("scripts").join("Default");
        fs::create_dir_all(&livres).unwrap();
        fs::write(livres.join("100_CLEAN.ps1"), b"x").unwrap();
        fs::write(livres.join("300_TELEMETRY.PS1"), b"x").unwrap();
        assert_eq!(effacer_scripts_livres(&racine).unwrap(), 2);
        assert!(!racine.exists(), "plus rien dedans : le dossier part");

        // Un fichier qui n'est pas un script livre reste, et ses dossiers avec.
        fs::create_dir_all(&livres).unwrap();
        fs::write(livres.join("a.ps1"), b"x").unwrap();
        fs::write(livres.join("notes.txt"), b"x").unwrap();
        assert_eq!(effacer_scripts_livres(&racine).unwrap(), 1);
        assert!(livres.join("notes.txt").exists());
        let _ = fs::remove_dir_all(&racine);

        // Rien a cet endroit : rien a faire.
        assert_eq!(effacer_scripts_livres(&racine).unwrap(), 0);
    }
}
