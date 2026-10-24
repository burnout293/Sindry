//! Reprendre un entretien apres un redemarrage (2.1.0, specification §6.10).
//!
//! Un script demande un redemarrage au milieu d'un entretien par
//! `[RESTART] <phase>` puis `exit 3010` (`runner.rs`). Sindry garde ici ce qu'il
//! faut pour continuer, cree une tache d'ouverture de session qui le relancera
//! eleve, puis redemarre. A la relance, il verifie tout avant de reprendre.
//!
//! **Ou vit la confiance.** Le journal decide de ce qu'un processus eleve va
//! executer : il vit dans `HKLM\SOFTWARE\Sindry\Reprise`, qu'un programme sans
//! droits ne peut pas ecrire — jamais dans `%LOCALAPPDATA%`, ni dans
//! `settings.json`. Le plafond de redemarrages aussi : le relever affaiblit une
//! protection (meme raisonnement que la garde, `garde.rs`).
//!
//! **La tache** pointe vers l'exe de Sindry, et seulement s'il est installe sous
//! Program Files : un exe modifiable sans droits, lance eleve a chaque ouverture
//! de session, serait une porte ouverte. Elle ne porte ni script, ni reglage :
//! juste `--reprendre`. Le reste est lu dans le journal.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Sous `HKEY_LOCAL_MACHINE` : rien ne s'y ecrit sans elevation.
pub const CLE: &str = r"SOFTWARE\Sindry\Reprise";
const VALEUR_JOURNAL: &str = "Journal";
const VALEUR_PLAFOND: &str = "MaxRedemarrages";

/// Le plafond de redemarrages par entretien : un reglage Expert, borne ici.
pub const PLAFOND_DEFAUT: u32 = 3;
pub const PLAFOND_MIN: u32 = 1;
pub const PLAFOND_MAX: u32 = 10;

/// Un journal plus vieux n'est plus repris : ce qui a ete choisi il y a une
/// semaine ne vaut plus pour le PC d'aujourd'hui.
pub const VALIDITE_JOURS: i64 = 7;

/// L'argument que porte la tache de relance.
pub const ARGUMENT: &str = "--reprendre";

/// La tache : `\Sindry\Reprise`. Le temoin ignore ce dossier (`temoin.rs`).
pub const DOSSIER_TACHE: &str = "Sindry";
pub const NOM_TACHE: &str = "Reprise";

/// Au-dela de cet ecart, l'heure de demarrage du PC a change : il a redemarre.
const ECART_DEMARRAGE_S: i64 = 120;

pub const FORMAT: u32 = 1;

/// Une action qui reste a faire, telle qu'elle a ete validee avant le
/// redemarrage. A la reprise, elle repasse quand meme par le chemin habituel
/// (`lib.rs::preparer_action`) : empreinte, approbation, garde.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Action {
    pub script_id: String,
    /// L'empreinte du fichier au moment de l'enregistrement : un script modifie
    /// entre-temps n'est pas repris.
    pub hash: String,
    #[serde(default)]
    pub config: BTreeMap<String, serde_json::Value>,
    /// Les selections `[items]`, par option : des ids deja verifies contre
    /// l'analyse au moment de l'enregistrement (`analyse::Memoire::reprendre`).
    #[serde(default)]
    pub ids: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Journal {
    pub format: u32,
    /// L'entretien, pour le bilan et l'historique.
    pub id: String,
    /// Le compte qui l'a lance : un autre compte ne le reprend pas.
    pub sid: String,
    /// Secondes Unix.
    pub cree: i64,
    /// L'heure de demarrage du PC au dernier enregistrement (secondes Unix) :
    /// c'est elle qui dit si le PC a redemarre depuis.
    pub demarrage: i64,
    /// Ce qui reste a faire, dans l'ordre. La premiere action tourne, ou va
    /// tourner.
    pub actions: Vec<Action>,
    /// La premiere action a demarre (ecrit avant de la lancer) : un journal
    /// retrouve dans cet etat, sans phase, dit un entretien coupe net.
    #[serde(default)]
    pub en_cours: bool,
    /// La phase a laquelle la premiere action reprend, apres le redemarrage
    /// qu'elle a demande.
    #[serde(default)]
    pub phase: Option<String>,
    #[serde(default)]
    pub redemarrages: u32,
    /// Ce dont l'interface a besoin pour continuer et dessiner le bilan : mode,
    /// lot, resultats deja obtenus… Rien ici n'est execute.
    #[serde(default)]
    pub interface: serde_json::Value,
}

/// Ce que vaut un journal retrouve.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Le PC a redemarre comme demande : on reprend.
    Reprendre,
    /// Un redemarrage a ete demande mais n'a pas encore eu lieu (« Plus tard »).
    EnAttente,
    /// L'entretien s'est arrete sans le demander : panne, redemarrage imprevu,
    /// Sindry ferme de force.
    Interrompu,
    /// Lance par un autre compte.
    AutreCompte,
    /// Plus vieux que `VALIDITE_JOURS`.
    Expire,
}

/// Juge un journal. Fonction pure : testable sans registre ni redemarrage.
pub fn juger(j: &Journal, sid: &str, maintenant: i64, demarrage: i64) -> Verdict {
    if !j.sid.eq_ignore_ascii_case(sid) {
        return Verdict::AutreCompte;
    }
    if maintenant - j.cree > VALIDITE_JOURS * 86_400 || j.cree - maintenant > 86_400 {
        return Verdict::Expire;
    }
    let a_redemarre = (demarrage - j.demarrage).abs() > ECART_DEMARRAGE_S;
    match (&j.phase, a_redemarre) {
        (Some(_), true) => Verdict::Reprendre,
        (Some(_), false) => Verdict::EnAttente,
        (None, _) => Verdict::Interrompu,
    }
}

/// Un journal lisible et de ce format, ou rien : un journal abime n'est jamais
/// a moitie repris.
pub fn depuis_json(texte: &str) -> Option<Journal> {
    let j: Journal = serde_json::from_str(texte).ok()?;
    let phase_ok = j.phase.as_deref().is_none_or(crate::runner::phase_valide);
    (j.format == FORMAT && phase_ok && !j.actions.is_empty()).then_some(j)
}

/// Borne un plafond lu ou demande.
pub fn borner(n: u32) -> u32 {
    n.clamp(PLAFOND_MIN, PLAFOND_MAX)
}

// ---------------------------------------------------------------------------
// Registre
// ---------------------------------------------------------------------------

#[cfg(windows)]
pub fn lire_depuis(racine: &windows_registry::Key, chemin: &str) -> Option<Journal> {
    let texte = racine.open(chemin).ok()?.get_string(VALEUR_JOURNAL).ok()?;
    depuis_json(&texte)
}

#[cfg(windows)]
pub fn ecrire_dans(
    racine: &windows_registry::Key,
    chemin: &str,
    j: &Journal,
) -> Result<(), String> {
    let texte = serde_json::to_string(j).map_err(|e| e.to_string())?;
    racine
        .create(chemin)
        .and_then(|c| c.set_string(VALEUR_JOURNAL, &texte))
        .map_err(|e| format!("ecriture du journal de reprise : {e}"))
}

#[cfg(windows)]
pub fn effacer_dans(racine: &windows_registry::Key, chemin: &str) {
    // `open` n'ouvre qu'en lecture ; `create` ouvre en ecriture, et seulement si
    // la cle existe deja : rien n'est cree pour etre efface.
    if racine.open(chemin).is_ok() {
        if let Ok(c) = racine.create(chemin) {
            let _ = c.remove_value(VALEUR_JOURNAL);
        }
    }
}

#[cfg(windows)]
pub fn plafond_depuis(racine: &windows_registry::Key, chemin: &str) -> u32 {
    racine
        .open(chemin)
        .ok()
        .and_then(|c| c.get_u32(VALEUR_PLAFOND).ok())
        .map(borner)
        .unwrap_or(PLAFOND_DEFAUT)
}

#[cfg(windows)]
pub fn fixer_plafond_dans(
    racine: &windows_registry::Key,
    chemin: &str,
    n: u32,
) -> Result<(), String> {
    if borner(n) != n {
        return Err(format!(
            "De {PLAFOND_MIN} a {PLAFOND_MAX} redemarrages par entretien."
        ));
    }
    racine
        .create(chemin)
        .and_then(|c| c.set_u32(VALEUR_PLAFOND, n))
        .map_err(|e| format!("ecriture du plafond de redemarrages : {e}"))
}

#[cfg(windows)]
pub fn lire() -> Option<Journal> {
    lire_depuis(windows_registry::LOCAL_MACHINE, CLE)
}

#[cfg(windows)]
pub fn ecrire(j: &Journal) -> Result<(), String> {
    ecrire_dans(windows_registry::LOCAL_MACHINE, CLE, j)
}

#[cfg(windows)]
pub fn effacer() {
    effacer_dans(windows_registry::LOCAL_MACHINE, CLE)
}

#[cfg(windows)]
pub fn plafond() -> u32 {
    plafond_depuis(windows_registry::LOCAL_MACHINE, CLE)
}

#[cfg(windows)]
pub fn fixer_plafond(n: u32) -> Result<(), String> {
    fixer_plafond_dans(windows_registry::LOCAL_MACHINE, CLE, n)
}

#[cfg(not(windows))]
pub fn lire() -> Option<Journal> {
    None
}
#[cfg(not(windows))]
pub fn ecrire(_j: &Journal) -> Result<(), String> {
    Err("Windows seulement".into())
}
#[cfg(not(windows))]
pub fn effacer() {}
#[cfg(not(windows))]
pub fn plafond() -> u32 {
    PLAFOND_DEFAUT
}
#[cfg(not(windows))]
pub fn fixer_plafond(_n: u32) -> Result<(), String> {
    Err("Windows seulement".into())
}

// ---------------------------------------------------------------------------
// Le compte, l'heure de demarrage du PC
// ---------------------------------------------------------------------------

pub fn maintenant() -> i64 {
    chrono::Local::now().timestamp()
}

/// L'heure de demarrage du PC, en secondes Unix : maintenant moins le temps
/// ecoule depuis le demarrage.
#[cfg(windows)]
pub fn demarrage_pc() -> i64 {
    // SAFETY: aucun argument, aucune memoire partagee.
    let depuis_ms = unsafe { windows_sys::Win32::System::SystemInformation::GetTickCount64() };
    maintenant() - (depuis_ms / 1000) as i64
}

#[cfg(not(windows))]
pub fn demarrage_pc() -> i64 {
    0
}

/// Le SID du compte qui fait tourner ce processus (`S-1-5-21-…`), lu dans son
/// jeton : jamais dans l'environnement, que le compte peut changer.
#[cfg(windows)]
pub fn sid_courant() -> Option<String> {
    use windows_sys::Win32::Foundation::{CloseHandle, LocalFree, HANDLE};
    use windows_sys::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows_sys::Win32::Security::{GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    // SAFETY: chaque pointeur vient d'un appel qui vient de reussir, chaque
    // tampon a la taille annoncee par Windows, et chaque ressource est rendue.
    unsafe {
        let mut jeton: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut jeton) == 0 {
            return None;
        }
        let mut taille = 0u32;
        GetTokenInformation(jeton, TokenUser, std::ptr::null_mut(), 0, &mut taille);
        // Des u64 : un tampon aligne pour TOKEN_USER.
        let mut tampon = vec![0u64; (taille as usize).div_ceil(8).max(1)];
        let ok = GetTokenInformation(
            jeton,
            TokenUser,
            tampon.as_mut_ptr().cast(),
            taille,
            &mut taille,
        );
        CloseHandle(jeton);
        if ok == 0 {
            return None;
        }
        let utilisateur = &*(tampon.as_ptr() as *const TOKEN_USER);
        let mut texte: *mut u16 = std::ptr::null_mut();
        if ConvertSidToStringSidW(utilisateur.User.Sid, &mut texte) == 0 || texte.is_null() {
            return None;
        }
        let mut n = 0;
        while *texte.add(n) != 0 {
            n += 1;
        }
        let sid = String::from_utf16_lossy(std::slice::from_raw_parts(texte, n));
        LocalFree(texte.cast());
        sid.starts_with("S-1-").then_some(sid)
    }
}

#[cfg(not(windows))]
pub fn sid_courant() -> Option<String> {
    None
}

// ---------------------------------------------------------------------------
// La reprise sans mot de passe : Automatic Restart Sign-On (ARSO)
// ---------------------------------------------------------------------------

/// Apres un redemarrage `shutdown /g`, Windows peut rouvrir la session du dernier
/// utilisateur et la verrouiller aussitot, avec des identifiants qu'il garde
/// lui-meme. Sindry ne fait que lire si c'est permis : il ne voit, ne demande et
/// ne stocke jamais un mot de passe.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Arso {
    Activee,
    Desactivee,
    /// Interdite par une strategie (`DisableAutomaticRestartSignOn`).
    Strategie,
    /// Windows ne dit rien : l'utilisateur n'a jamais touche au reglage. On ne
    /// pretend pas savoir.
    Inconnue,
}

const STRATEGIE_SYSTEME: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System";
const ARSO_UTILISATEURS: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\UserARSO";

#[cfg(windows)]
pub fn arso(sid: &str) -> Arso {
    let hklm = windows_registry::LOCAL_MACHINE;
    let strategie = hklm
        .open(STRATEGIE_SYSTEME)
        .ok()
        .and_then(|c| c.get_u32("DisableAutomaticRestartSignOn").ok());
    if strategie == Some(1) {
        return Arso::Strategie;
    }
    match hklm
        .open(format!(r"{ARSO_UTILISATEURS}\{sid}"))
        .ok()
        .and_then(|c| c.get_u32("OptOut").ok())
    {
        Some(0) => Arso::Activee,
        Some(_) => Arso::Desactivee,
        None => Arso::Inconnue,
    }
}

#[cfg(not(windows))]
pub fn arso(_sid: &str) -> Arso {
    Arso::Inconnue
}

// ---------------------------------------------------------------------------
// La tache de relance
// ---------------------------------------------------------------------------

/// L'exe de Sindry, s'il peut porter une tache elevee : installe sous Program
/// Files (installation par machine), donc modifiable en administrateur
/// seulement. Une build de developpement, ailleurs, n'en cree pas.
pub fn exe_relancable() -> Result<std::path::PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let e = crate::systeme::emplacements();
    let sous = |racine: &Path| exe.starts_with(racine);
    if sous(&e.program_files) || e.program_files_x86.as_deref().is_some_and(sous) {
        Ok(exe)
    } else {
        Err("La relance automatique demande Sindry installe dans Program Files.".into())
    }
}

/// Echappe un texte pour un element XML.
fn xml(texte: &str) -> String {
    texte
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// La definition de la tache : a l'ouverture de session de ce compte, Sindry
/// eleve avec `--reprendre`, une seule instance, expiree au bout d'une semaine
/// puis effacee par Windows si Sindry ne l'a pas fait.
pub fn xml_tache(exe: &Path, sid: &str, fin: &str) -> String {
    let exe = xml(&exe.to_string_lossy());
    let sid = xml(sid);
    let fin = xml(fin);
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo><Description>Sindry : reprend un entretien apres un redemarrage. Effacee par Sindry des la reprise.</Description></RegistrationInfo>
  <Triggers>
    <LogonTrigger><EndBoundary>{fin}</EndBoundary><Enabled>true</Enabled><UserId>{sid}</UserId></LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author"><UserId>{sid}</UserId><LogonType>InteractiveToken</LogonType><RunLevel>HighestAvailable</RunLevel></Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <DeleteExpiredTaskAfter>P1D</DeleteExpiredTaskAfter>
    <Enabled>true</Enabled>
  </Settings>
  <Actions Context="Author">
    <Exec><Command>{exe}</Command><Arguments>{ARGUMENT}</Arguments></Exec>
  </Actions>
</Task>"#
    )
}

/// PowerShell du systeme, par l'objet COM du Planificateur : ni module a
/// charger, ni politique d'execution en jeu. Le texte est fixe ; ce qui varie
/// arrive par une variable d'environnement posee ici.
fn planificateur(script: &str, xml: Option<&str>) -> Result<(), String> {
    let mut c = std::process::Command::new(crate::systeme::emplacements().powershell());
    c.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ]);
    match xml {
        Some(x) => c.env("SINDRY_TACHE_XML", x),
        None => c.env_remove("SINDRY_TACHE_XML"),
    };
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let sortie = c
        .output()
        .map_err(|e| format!("Planificateur de taches injoignable : {e}"))?;
    let texte = String::from_utf8_lossy(&sortie.stdout);
    if texte.lines().any(|l| l.trim() == "SINDRY:OK") {
        Ok(())
    } else {
        let detail = texte
            .lines()
            .find_map(|l| l.trim().strip_prefix("SINDRY:ERREUR"))
            .unwrap_or("")
            .trim()
            .to_string();
        Err(format!("Tache de reprise : {detail}"))
    }
}

/// Cree (ou remplace) la tache de relance pour ce compte.
pub fn creer_tache(sid: &str) -> Result<(), String> {
    let exe = exe_relancable()?;
    let fin = (chrono::Local::now() + chrono::Duration::days(VALIDITE_JOURS))
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();
    planificateur(
        &format!(
            "$ErrorActionPreference = 'Stop'; \
             try {{ \
               $s = New-Object -ComObject Schedule.Service; $s.Connect(); \
               $racine = $s.GetFolder('\\'); \
               try {{ $d = $s.GetFolder('\\{DOSSIER_TACHE}') }} catch {{ $d = $racine.CreateFolder('{DOSSIER_TACHE}') }}; \
               $null = $d.RegisterTask('{NOM_TACHE}', $env:SINDRY_TACHE_XML, 6, $null, $null, 3); \
               'SINDRY:OK' \
             }} catch {{ 'SINDRY:ERREUR ' + $_.Exception.Message }}"
        ),
        Some(&xml_tache(&exe, sid, &fin)),
    )
}

/// Efface la tache de relance, et son dossier s'il est vide. Absente : rien a
/// faire, ce n'est pas une erreur.
pub fn supprimer_tache() -> Result<(), String> {
    planificateur(
        &format!(
            "try {{ \
               $s = New-Object -ComObject Schedule.Service; $s.Connect(); \
               try {{ $s.GetFolder('\\{DOSSIER_TACHE}').DeleteTask('{NOM_TACHE}', 0) }} catch {{ }}; \
               try {{ $s.GetFolder('\\').DeleteFolder('{DOSSIER_TACHE}', 0) }} catch {{ }}; \
               'SINDRY:OK' \
             }} catch {{ 'SINDRY:ERREUR ' + $_.Exception.Message }}"
        ),
        None,
    )
}

// ---------------------------------------------------------------------------
// Redemarrer
// ---------------------------------------------------------------------------

/// Redemarre le PC, tout de suite. `shutdown /g` : un redemarrage complet apres
/// lequel Windows rouvre et verrouille la session du dernier utilisateur, si
/// ARSO le permet ; sinon, un redemarrage ordinaire. `shutdown.exe` du systeme,
/// par son chemin lu dans HKLM.
pub fn redemarrer() -> Result<(), String> {
    let exe = crate::systeme::emplacements()
        .system32()
        .join("shutdown.exe");
    let mut c = std::process::Command::new(exe);
    // /d p:4:1 : planifie, application, maintenance.
    c.args(["/g", "/t", "0", "/d", "p:4:1"]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let statut = c
        .status()
        .map_err(|e| format!("Redemarrage impossible : {e}"))?;
    if statut.success() {
        Ok(())
    } else {
        Err(format!(
            "Windows a refuse le redemarrage (code {}).",
            statut.code().unwrap_or(-1)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn journal() -> Journal {
        Journal {
            format: FORMAT,
            id: "e1".into(),
            sid: "S-1-5-21-1-2-3-1001".into(),
            cree: 1_000_000,
            demarrage: 900_000,
            actions: vec![Action {
                script_id: "s1".into(),
                hash: "h".into(),
                config: BTreeMap::new(),
                ids: BTreeMap::new(),
            }],
            en_cours: true,
            phase: Some("apres-pilote".into()),
            redemarrages: 1,
            interface: serde_json::Value::Null,
        }
    }

    #[test]
    fn a_journal_is_resumed_only_by_its_account_after_a_restart_within_a_week() {
        let j = journal();
        let sid = j.sid.clone();
        assert_eq!(juger(&j, &sid, 1_000_100, 990_000), Verdict::Reprendre);
        assert_eq!(juger(&j, &sid, 1_000_100, 900_060), Verdict::EnAttente);
        assert_eq!(
            juger(&j, "S-1-5-21-9-9-9-1002", 1_000_100, 990_000),
            Verdict::AutreCompte
        );
        assert_eq!(
            juger(&j, &sid, 1_000_000 + 8 * 86_400, 990_000),
            Verdict::Expire
        );
        let coupe = Journal {
            phase: None,
            ..journal()
        };
        assert_eq!(juger(&coupe, &sid, 1_000_100, 900_000), Verdict::Interrompu);
        assert_eq!(juger(&coupe, &sid, 1_000_100, 990_000), Verdict::Interrompu);
    }

    #[test]
    fn a_damaged_journal_is_never_half_resumed() {
        let bon = serde_json::to_string(&journal()).unwrap();
        assert!(depuis_json(&bon).is_some());
        assert!(depuis_json("{").is_none());
        let mut j = journal();
        j.phase = Some("Phase avec espaces".into());
        assert!(depuis_json(&serde_json::to_string(&j).unwrap()).is_none());
        j = journal();
        j.actions.clear();
        assert!(depuis_json(&serde_json::to_string(&j).unwrap()).is_none());
        j = journal();
        j.format = 99;
        assert!(depuis_json(&serde_json::to_string(&j).unwrap()).is_none());
    }

    #[test]
    fn the_restart_cap_is_bounded() {
        assert_eq!(borner(0), PLAFOND_MIN);
        assert_eq!(borner(3), 3);
        assert_eq!(borner(50), PLAFOND_MAX);
    }

    #[test]
    fn the_task_definition_escapes_what_it_carries() {
        let x = xml_tache(
            Path::new(r"C:\Program Files\Sindry & Co\<sindry>.exe"),
            "S-1-5-21-1-2-3-1001",
            "2026-10-31T10:00:00",
        );
        assert!(
            x.contains(r"<Command>C:\Program Files\Sindry &amp; Co\&lt;sindry&gt;.exe</Command>")
        );
        assert!(x.contains("<Arguments>--reprendre</Arguments>"));
        assert!(x.contains("<RunLevel>HighestAvailable</RunLevel>"));
        assert_eq!(x.matches("<UserId>S-1-5-21-1-2-3-1001</UserId>").count(), 2);
        assert!(x.contains("<EndBoundary>2026-10-31T10:00:00</EndBoundary>"));
    }

    #[cfg(windows)]
    #[test]
    fn the_journal_and_the_cap_survive_the_registry() {
        use windows_registry::CURRENT_USER;
        let chemin = format!(r"SOFTWARE\Sindry\Tests\Reprise-{}", std::process::id());
        let _ = CURRENT_USER.remove_tree(&chemin);
        assert!(lire_depuis(CURRENT_USER, &chemin).is_none());
        assert_eq!(plafond_depuis(CURRENT_USER, &chemin), PLAFOND_DEFAUT);
        ecrire_dans(CURRENT_USER, &chemin, &journal()).unwrap();
        assert_eq!(lire_depuis(CURRENT_USER, &chemin), Some(journal()));
        fixer_plafond_dans(CURRENT_USER, &chemin, 5).unwrap();
        assert_eq!(plafond_depuis(CURRENT_USER, &chemin), 5);
        assert!(fixer_plafond_dans(CURRENT_USER, &chemin, 11).is_err());
        effacer_dans(CURRENT_USER, &chemin);
        assert!(lire_depuis(CURRENT_USER, &chemin).is_none());
        let _ = CURRENT_USER.remove_tree(&chemin);
    }

    #[cfg(windows)]
    #[test]
    fn the_account_and_the_boot_time_are_read() {
        assert!(sid_courant().is_some_and(|s| s.starts_with("S-1-5-")));
        let d = demarrage_pc();
        assert!(d > 1_500_000_000 && d <= maintenant());
    }
}
