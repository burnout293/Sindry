//! La ligne de commande (2.0, specification §19, docs/LIGNE_DE_COMMANDE.md).
//!
//! `sindry.com`, installe a cote de `sindry.exe`. Un programme **console** a
//! part entiere, et non l'application graphique qui s'attacherait a la console
//! de son parent : une console n'attend pas un programme graphique, ne recoit
//! pas son code de sortie, et melange sa sortie a l'invite suivante. `.com`
//! plutot que `.exe` : taper `sindry` trouve `sindry.com` avant `sindry.exe`
//! (ordre de PATHEXT) — le choix de Visual Studio pour `devenv.com`.
//!
//! Aucune fenetre, aucune vue : l'application est construite sans sa fenetre
//! (`create: false` dans tauri.conf.json), pour ses chemins et son etat. Chaque
//! commande passe par les memes fonctions que l'interface — approbation, garde
//! des reglages, simulation, analyse, historique — : la ligne de commande ne
//! contourne rien, elle n'ouvre aucune porte que l'interface tient fermee.

use super::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc;
use std::time::Duration;

/// Les codes de sortie. Stables : un script de deploiement s'y fie.
pub mod code {
    pub const OK: i32 = 0;
    /// Une action a echoue, ou une operation n'a pas abouti.
    pub const ECHEC: i32 = 1;
    /// Commande, option ou valeur inconnue.
    pub const USAGE: i32 = 2;
    /// Refuse par une regle de Sindry : script non approuve, reglage refuse par
    /// la garde, simulation impossible, selection perimee.
    pub const REFUS: i32 = 3;
    /// Il faut les droits administrateur.
    pub const DROITS: i32 = 4;
    /// Script, lot, ambiance ou catalogue introuvable.
    pub const INTROUVABLE: i32 = 5;
    /// Termine, un redemarrage est necessaire : `ERROR_SUCCESS_REBOOT_REQUIRED`,
    /// la convention de Windows Installer, que les outils de deploiement lisent.
    pub const REDEMARRAGE: i32 = 3010;
}

/* ------------------------------------------------------------------------
Langue : celle des reglages, ou SINDRY_LANG
------------------------------------------------------------------------ */

static ANGLAIS: AtomicBool = AtomicBool::new(false);

fn anglais() -> bool {
    ANGLAIS.load(Ordering::Relaxed)
}

/// Le texte dans la langue de la ligne de commande. Seule la branche choisie
/// est evaluee.
macro_rules! l {
    ($fr:expr, $en:expr $(,)?) => {
        if anglais() {
            ($en).to_string()
        } else {
            ($fr).to_string()
        }
    };
}

/// `SINDRY_LANG` d'abord, sinon la langue des reglages, lue sans construire
/// l'application (l'aide et `ambiance verifier` s'en passent).
fn langue_initiale() -> String {
    if let Ok(l) = std::env::var("SINDRY_LANG") {
        return l.to_ascii_lowercase();
    }
    discovery::dossier_essai()
        .or_else(|| std::env::var_os("LOCALAPPDATA").map(|d| PathBuf::from(d).join("Sindry")))
        .map(|d| d.join("settings.json"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v["lang"].as_str().map(str::to_string))
        .unwrap_or_else(|| "fr".to_string())
}

/* ------------------------------------------------------------------------
Erreurs et sortie
------------------------------------------------------------------------ */

struct Echec {
    code: i32,
    message: String,
}

type Res<T> = Result<T, Echec>;

fn usage(message: String) -> Echec {
    Echec {
        code: code::USAGE,
        message,
    }
}

fn introuvable(message: String) -> Echec {
    Echec {
        code: code::INTROUVABLE,
        message,
    }
}

/// Le code de sortie d'une erreur de Sindry, d'apres sa sentinelle.
fn classer(m: &str) -> i32 {
    if m.starts_with(NON_APPROUVE)
        || m.starts_with(SANS_SIMULATION)
        || m.starts_with(SANS_ANALYSE)
        || m.starts_with("REGLAGE_REFUSE")
        || m.starts_with("ANALYSE_PERIMEE")
    {
        code::REFUS
    } else if m.contains("SANS_DROITS") {
        code::DROITS
    } else if m.starts_with("Script introuvable") || m.starts_with(SOURCE_INCONNUE) {
        code::INTROUVABLE
    } else {
        code::ECHEC
    }
}

/// Une sentinelle de Sindry, dite en clair.
fn expliquer(m: &str) -> String {
    if m.starts_with(NON_APPROUVE) {
        return l!(
            "ce script n'est pas approuve. Lisez-le (sindry montrer <id> --source), puis approuvez-le dans Sindry, ou en administrateur : sindry approuver <id> --empreinte <empreinte>.",
            "this script is not approved. Read it (sindry show <id> --source), then approve it in Sindry, or as administrator: sindry approve <id> --hash <hash>."
        );
    }
    if m.starts_with(SANS_SIMULATION) {
        return l!(
            "la simulation est demandee, et ce script ne sait pas se simuler : il n'a pas ete lance.",
            "simulation is on, and this script cannot simulate itself: it was not run."
        );
    }
    if m.starts_with(SANS_ANALYSE) {
        return l!(
            "ce script ne sait pas analyser (pas de « scan : true »).",
            "this script cannot check before acting (no \"scan : true\")."
        );
    }
    if let Some(reste) = m.strip_prefix("REGLAGE_REFUSE:") {
        let mut p = reste.splitn(3, ':');
        let (genre, cle, valeur) = (
            p.next().unwrap_or(""),
            p.next().unwrap_or(""),
            p.next().unwrap_or(""),
        );
        return match genre {
            "type" => l!(
                format!("reglage refuse : « {cle} » n'a pas le type que le script declare."),
                format!("setting refused: \"{cle}\" does not have the type the script declares.")
            ),
            "choix" => l!(
                format!("reglage refuse : « {valeur} » ne fait pas partie des choix de « {cle} »."),
                format!("setting refused: \"{valeur}\" is not one of the choices of \"{cle}\".")
            ),
            _ => l!(
                format!("reglage refuse : « {cle} » vise un emplacement protege ({valeur})."),
                format!("setting refused: \"{cle}\" targets a protected location ({valeur}).")
            ),
        };
    }
    if let Some(reste) = m.strip_prefix("ANALYSE_PERIMEE:") {
        return reste
            .split_once(" — ")
            .map_or(reste, |(_, t)| t)
            .to_string();
    }
    if m.contains("SANS_DROITS") {
        return l!(
            "cette commande demande les droits administrateur : ouvrez la console en administrateur.",
            "this command requires administrator rights: open the console as administrator."
        );
    }
    if m.starts_with(CATALOGUE_PENDANT_EXECUTION) {
        return l!("une execution est en cours.", "a run is in progress.");
    }
    if let Some(id) = m.strip_prefix(&format!("{SOURCE_INCONNUE}: ")) {
        return l!(
            format!("catalogue inconnu : {id}"),
            format!("unknown catalogue: {id}")
        );
    }
    m.to_string()
}

impl From<String> for Echec {
    fn from(m: String) -> Self {
        Echec {
            code: classer(&m),
            message: expliquer(&m),
        }
    }
}

/// Du JSON en ASCII pur : les caracteres accentues deviennent `\u00e9`. Une
/// console Windows lit la sortie redirigee dans sa page de codes, pas en
/// UTF-8 ; en ASCII, `ConvertFrom-Json` de PowerShell 5.1 la lit intacte.
pub fn json_ascii(v: &impl Serialize) -> String {
    let brut = serde_json::to_string_pretty(v).unwrap_or_else(|_| "null".to_string());
    let mut s = String::with_capacity(brut.len());
    let mut tampon = [0u16; 2];
    for c in brut.chars() {
        if c.is_ascii() {
            s.push(c);
        } else {
            for u in c.encode_utf16(&mut tampon) {
                s.push_str(&format!("\\u{u:04x}"));
            }
        }
    }
    s
}

fn json(v: &impl Serialize) {
    println!("{}", json_ascii(v));
}

fn oui_non(b: bool) -> String {
    if b {
        l!("oui", "yes")
    } else {
        l!("non", "no")
    }
}

/// Une taille lisible, en unites de 1024 comme l'Explorateur.
fn taille(octets: u64) -> String {
    let unites = if anglais() {
        ["B", "KB", "MB", "GB", "TB"]
    } else {
        ["o", "Ko", "Mo", "Go", "To"]
    };
    let mut v = octets as f64;
    let mut i = 0;
    while v >= 1024.0 && i < unites.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{octets} {}", unites[0])
    } else {
        format!("{v:.1} {}", unites[i])
    }
}

fn duree(ms: u64) -> String {
    if ms < 1000 {
        format!("{ms} ms")
    } else if ms < 60_000 {
        format!("{:.1} s", ms as f64 / 1000.0)
    } else {
        format!("{} min {:02} s", ms / 60_000, (ms % 60_000) / 1000)
    }
}

/// Coupe un texte a `n` caracteres et le complete a `n`, pour les colonnes.
fn colonne(s: &str, n: usize) -> String {
    let mut t: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        t.pop();
        t.push('…');
    }
    let manque = n.saturating_sub(t.chars().count());
    t.push_str(&" ".repeat(manque));
    t
}

/* ------------------------------------------------------------------------
Options
------------------------------------------------------------------------ */

#[derive(Default)]
struct Options {
    json: bool,
    simuler: bool,
    sans_analyse: bool,
    sans_restauration: bool,
    remplacer: bool,
    source: bool,
    oui: bool,
    nombre: Option<usize>,
    empreinte: Option<String>,
}

fn separer(args: &[String]) -> Res<(Vec<String>, Options)> {
    let mut o = Options::default();
    let mut positions = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        let mut valeur = || -> Res<String> {
            i += 1;
            args.get(i).cloned().ok_or_else(|| {
                usage(l!(
                    format!("{a} attend une valeur."),
                    format!("{a} expects a value.")
                ))
            })
        };
        match a {
            "--json" => o.json = true,
            "--simuler" | "--simulate" => o.simuler = true,
            "--sans-analyse" | "--no-scan" => o.sans_analyse = true,
            "--sans-restauration" | "--no-restore-point" => o.sans_restauration = true,
            "--remplacer" | "--replace" => o.remplacer = true,
            "--source" => o.source = true,
            "--oui" | "--yes" | "-y" => o.oui = true,
            "--nombre" | "--count" | "-n" => {
                let v = valeur()?;
                o.nombre = Some(v.parse().map_err(|_| {
                    usage(l!(
                        format!("--nombre {v} : un entier."),
                        format!("--count {v}: an integer.")
                    ))
                })?);
            }
            "--empreinte" | "--hash" => o.empreinte = Some(valeur()?),
            "--help" | "-h" | "/?" | "-?" => positions.insert(0, "aide".to_string()),
            "--version" | "-v" => positions.insert(0, "version".to_string()),
            x if x.starts_with("--") => {
                return Err(usage(l!(
                    format!("option inconnue : {x} (sindry aide)"),
                    format!("unknown option: {x} (sindry help)")
                )))
            }
            _ => positions.push(a.to_string()),
        }
        i += 1;
    }
    Ok((positions, o))
}

/* ------------------------------------------------------------------------
Entree
------------------------------------------------------------------------ */

/// Point d'entree de `sindry.com` : rend le code de sortie.
pub fn principal() -> i32 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    ANGLAIS.store(langue_initiale() == "en", Ordering::Relaxed);
    if args.first().map(String::as_str) == Some("--installeur") {
        return installeur(&args[1..]);
    }
    let (positions, o) = match separer(&args) {
        Ok(x) => x,
        Err(e) => return terminer(Err(e), false),
    };
    let r = executer(&positions, &o);
    terminer(r, o.json)
}

fn terminer(r: Res<i32>, en_json: bool) -> i32 {
    match r {
        Ok(c) => c,
        Err(e) => {
            if en_json {
                json(&serde_json::json!({ "erreur": e.message, "code": e.code }));
            } else {
                eprintln!("sindry : {}", e.message);
            }
            e.code
        }
    }
}

fn executer(pos: &[String], o: &Options) -> Res<i32> {
    let commande = pos.first().map(|s| s.to_lowercase()).unwrap_or_default();
    let reste: Vec<String> = pos.iter().skip(1).cloned().collect();
    let sous = reste.first().map(|s| s.to_lowercase()).unwrap_or_default();
    match commande.as_str() {
        "" | "aide" | "help" => {
            aide();
            Ok(code::OK)
        }
        "version" => {
            if o.json {
                json(&serde_json::json!({ "version": env!("CARGO_PKG_VERSION") }));
            } else {
                println!("Sindry {}", env!("CARGO_PKG_VERSION"));
            }
            Ok(code::OK)
        }
        // Verifier un fichier d'ambiance ne demande rien d'autre que lui : un
        // auteur s'en sert sans que Sindry ait jamais ete lance.
        "ambiance" | "look" if matches!(sous.as_str(), "verifier" | "check") => {
            ambiance_verifier(reste.get(1), o)
        }
        _ => {
            let app = application()?;
            let h = app.handle().clone();
            if let Ok(s) = settings::load(&h) {
                if std::env::var("SINDRY_LANG").is_err() {
                    ANGLAIS.store(s.lang == "en", Ordering::Relaxed);
                }
            }
            match commande.as_str() {
                "scripts" | "list" => scripts(&h, o),
                "lots" | "batches" => lots(&h, o),
                "montrer" | "show" => montrer(&h, argument(&reste, 0, "id")?, o),
                "verifier" | "check" => verifier(&h, argument(&reste, 0, "id")?, o),
                "analyser" | "scan" => analyser_commande(&h, argument(&reste, 0, "id")?, o),
                "lancer" | "run" => {
                    if reste.is_empty() {
                        return Err(usage(l!(
                            "lancer : un id de script ou lot:<id> (sindry scripts, sindry lots).",
                            "run: a script id or lot:<id> (sindry list, sindry batches)."
                        )));
                    }
                    lancer(&h, &reste, o)
                }
                "approuver" | "approve" => approuver(&h, argument(&reste, 0, "id")?, o),
                "historique" | "history" => historique(&h, o),
                "reglages" | "settings" => reglages(&h, o),
                "regler" | "set" => regler(
                    &h,
                    argument(&reste, 0, l!("reglage", "setting"))?,
                    argument(&reste, 1, l!("valeur", "value"))?,
                    o,
                ),
                "config" => config(&h, &reste, o),
                "catalogues" | "catalogs" => catalogues(&h, o),
                "catalogue" | "catalog" => catalogue(&h, &reste, o),
                "ambiances" | "looks" => ambiances(&h, o),
                "ambiance" | "look" => ambiance(&h, &reste, o),
                autre => Err(usage(l!(
                    format!("commande inconnue : {autre} (sindry aide)"),
                    format!("unknown command: {autre} (sindry help)")
                ))),
            }
        }
    }
}

fn argument(reste: &[String], i: usize, nom: impl Into<String>) -> Res<&str> {
    reste.get(i).map(String::as_str).ok_or_else(|| {
        let nom = nom.into();
        usage(l!(
            format!("il manque : <{nom}>."),
            format!("missing: <{nom}>.")
        ))
    })
}

/// L'application, sans fenetre : ses chemins, ses ressources, son etat. Le
/// processus est prepare comme celui de l'interface — ancien dossier repris,
/// environnement des scripts assaini.
fn application() -> Res<tauri::App> {
    for message in assainir_processus() {
        eprintln!("sindry : {message}");
    }
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .build(contexte())
        .map_err(|e| Echec {
            code: code::ECHEC,
            message: format!("demarrage impossible : {e}"),
        })?;
    app.manage(Arc::new(runner::Runner::default()));
    app.manage(Arc::new(analyse::Memoire::default()));
    if let Some(message) = preparer_enfants(app.handle()) {
        eprintln!("sindry : {message}");
    }
    if let Err(e) = preparer_ambiances(app.handle()) {
        eprintln!("sindry : ambiances de base non installees : {e}");
    }
    Ok(app)
}

/* ------------------------------------------------------------------------
Aide
------------------------------------------------------------------------ */

fn aide() {
    let texte = if anglais() { AIDE_EN } else { AIDE_FR };
    println!("Sindry {}\n{texte}", env!("CARGO_PKG_VERSION"));
}

const AIDE_FR: &str = r#"
Usage : sindry <commande> [arguments] [options]

Actions
  scripts                         les actions installees
  lots                            les lots et leurs actions
  montrer <id> [--source]         une action : fichier, empreinte, approbation
  verifier <id>                   contrat et syntaxe, sans rien executer
  analyser <id>                   ce que l'action ferait, sans rien modifier
  lancer <id|lot:<id>>...         lance des actions ou des lots
        --simuler                 simuler ce lancement (refuse ce qui ne sait pas)
        --sans-analyse            agir sans analyser d'abord
        --sans-restauration       sans point de restauration
  approuver <id> --empreinte <h>  approuver ce fichier exact (administrateur)
  historique [--nombre N]         les dernieres executions

Reglages
  reglages                        les reglages
  regler <reglage> <valeur>       theme, langue, ambiance, mises-a-jour, echec,
                                  journaux, politique, simulation, echelle
  config exporter <fichier>       toute la configuration dans un fichier
  config importer <fichier>       la reprendre
  config reinitialiser --oui      revenir aux reglages d'usine

Catalogues
  catalogues                      les catalogues et leur etat
  catalogue verifier [<id>]       ce qu'une mise a jour changerait
  catalogue installer [<id>]      installer ou mettre a jour (officiel par defaut)
  catalogue activer|desactiver <id>

Ambiances
  ambiances                       les ambiances installees
  ambiance verifier <fichier>     verifier un fichier .sindry-ambiance
  ambiance importer <fichier> [--remplacer]
  ambiance exporter <id> <fichier>
  ambiance retirer <id> --oui
  ambiance choisir <id>

Options : --json (sortie JSON, ASCII pur), --aide, --version
Codes de sortie : 0 reussi, 1 echec, 2 usage, 3 refuse, 4 droits administrateur,
5 introuvable, 3010 termine, redemarrage necessaire.
Langue : celle des reglages, ou SINDRY_LANG=fr|en. Commandes en anglais aussi
(list, show, check, scan, run…). Guide : docs/LIGNE_DE_COMMANDE.md
"#;

const AIDE_EN: &str = r#"
Usage: sindry <command> [arguments] [options]

Actions
  list                            installed actions
  batches                         batches and their actions
  show <id> [--source]            an action: file, hash, approval
  check <id>                      contract and syntax, without running anything
  scan <id>                       what the action would do, changing nothing
  run <id|lot:<id>>...            run actions or batches
        --simulate                simulate this run (refuses what cannot)
        --no-scan                 act without checking first
        --no-restore-point        no restore point
  approve <id> --hash <h>         approve this exact file (administrator)
  history [--count N]             the latest runs

Settings
  settings                        the settings
  set <setting> <value>           theme, lang, look, updates, failure, logs,
                                  policy, simulation, scale
  config export <file>            the whole configuration in a file
  config import <file>            take it back
  config reset --yes              back to factory settings

Catalogues
  catalogs                        catalogues and their state
  catalog check [<id>]            what an update would change
  catalog install [<id>]          install or update (official by default)
  catalog enable|disable <id>

Looks
  looks                           installed looks
  look check <file>               check a .sindry-ambiance file
  look import <file> [--replace]
  look export <id> <file>
  look remove <id> --yes
  look use <id>

Options: --json (JSON output, pure ASCII), --help, --version
Exit codes: 0 success, 1 failure, 2 usage, 3 refused, 4 administrator rights,
5 not found, 3010 done, restart required.
Language: the settings', or SINDRY_LANG=fr|en. French commands work too
(scripts, montrer, lancer…). Guide: docs/LIGNE_DE_COMMANDE.md
"#;

/* ------------------------------------------------------------------------
Actions
------------------------------------------------------------------------ */

fn langue() -> &'static str {
    if anglais() {
        "en"
    } else {
        "fr"
    }
}

/// Le titre d'une action dans la langue de la ligne de commande.
fn titre(e: &discovery::ScriptEntry) -> String {
    e.meta
        .translations
        .get(langue())
        .map(|t| t.title.clone())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| e.meta.title.clone())
}

fn nom_lot(lot: &settings::Lot) -> String {
    lot.name
        .get(langue())
        .or_else(|| lot.name.get("fr"))
        .or_else(|| lot.name.get("en"))
        .or_else(|| lot.name.values().next())
        .cloned()
        .unwrap_or_else(|| lot.id.clone())
}

fn origine(e: &discovery::ScriptEntry) -> String {
    match e.origin {
        "official" => l!("officiel", "official"),
        "tierce" => e
            .source
            .clone()
            .unwrap_or_else(|| l!("tiers", "third-party")),
        _ => l!("personnel", "personal"),
    }
}

fn trouver(h: &tauri::AppHandle, id: &str) -> Res<discovery::ScriptEntry> {
    discovery::discover(h)?
        .scripts
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| {
            introuvable(l!(
                format!("aucune action « {id} » (sindry scripts)."),
                format!("no action \"{id}\" (sindry list).")
            ))
        })
}

#[derive(Serialize)]
struct LigneScript {
    id: String,
    titre: String,
    categorie: String,
    origine: String,
    approuve: bool,
    analysable: bool,
    simulable: bool,
    administrateur: bool,
    risque: String,
    fichier: String,
}

fn scripts(h: &tauri::AppHandle, o: &Options) -> Res<i32> {
    let d = discovery::discover(h)?;
    let lignes: Vec<LigneScript> = d
        .scripts
        .iter()
        .map(|e| LigneScript {
            id: e.id.clone(),
            titre: titre(e),
            categorie: e.meta.category.clone(),
            origine: origine(e),
            approuve: script_approuve(e).unwrap_or(false),
            analysable: e.meta.scan,
            simulable: simulation::simulable(&e.meta),
            administrateur: e.meta.admin,
            risque: e.meta.risk.clone(),
            fichier: e.abs_path.clone(),
        })
        .collect();
    if o.json {
        json(&lignes);
        return Ok(code::OK);
    }
    if lignes.is_empty() {
        println!(
            "{}",
            l!(
                "Aucune action. Installez le catalogue officiel : sindry catalogue installer",
                "No action. Install the official catalogue: sindry catalog install"
            )
        );
        return Ok(code::OK);
    }
    let largeur = lignes
        .iter()
        .map(|l| l.id.chars().count())
        .max()
        .unwrap_or(2)
        .clamp(2, 40);
    println!(
        "{}  {}  {}  {}  {}",
        colonne("ID", largeur),
        colonne(&l!("TITRE", "TITLE"), 42),
        colonne(&l!("ORIGINE", "ORIGIN"), 10),
        colonne(&l!("APPROUVE", "APPROVED"), 8),
        l!("ANALYSE  SIMULE", "CHECK    SIMULATE")
    );
    for x in &lignes {
        println!(
            "{}  {}  {}  {}  {}  {}",
            colonne(&x.id, largeur),
            colonne(&x.titre, 42),
            colonne(&x.origine, 10),
            colonne(&oui_non(x.approuve), 8),
            colonne(&oui_non(x.analysable), 7),
            oui_non(x.simulable)
        );
    }
    for p in &d.problems {
        eprintln!("sindry : {p}");
    }
    Ok(code::OK)
}

#[derive(Serialize)]
struct LigneLot {
    id: String,
    nom: String,
    epingle: bool,
    actions: Vec<(String, String)>,
    manquants: Vec<String>,
}

fn lots(h: &tauri::AppHandle, o: &Options) -> Res<i32> {
    let reglages = settings::load(h)?;
    let groupes = settings::group_scripts(&reglages, discovery::discover(h)?.scripts);
    let lignes: Vec<LigneLot> = groupes
        .lots
        .iter()
        .map(|g| LigneLot {
            id: g.lot.id.clone(),
            nom: nom_lot(&g.lot),
            epingle: g.lot.pinned,
            actions: g.scripts.iter().map(|e| (e.id.clone(), titre(e))).collect(),
            manquants: g.missing.clone(),
        })
        .collect();
    if o.json {
        json(&lignes);
        return Ok(code::OK);
    }
    for x in &lignes {
        let n = x.actions.len();
        println!(
            "lot:{}  {}  ({})",
            x.id,
            x.nom,
            l!(format!("{n} action(s)"), format!("{n} action(s)"))
        );
        for (id, t) in &x.actions {
            println!("    {id}  {t}");
        }
        for m in &x.manquants {
            println!("    {m}  {}", l!("(manquante)", "(missing)"));
        }
    }
    Ok(code::OK)
}

fn montrer(h: &tauri::AppHandle, id: &str, o: &Options) -> Res<i32> {
    let e = trouver(h, id)?;
    let approuve = script_approuve(&e)?;
    let source = if o.source {
        let octets = std::fs::read(&e.abs_path).map_err(|x| format!("{} : {x}", e.abs_path))?;
        let texte = String::from_utf8_lossy(&octets).to_string();
        Some(texte.trim_start_matches('\u{feff}').to_string())
    } else {
        None
    };
    let reglages = settings::load(h)?;
    let figee = reglages
        .overrides
        .get(&e.id)
        .map(|x| x.config.clone())
        .unwrap_or_default();
    if o.json {
        json(&serde_json::json!({
            "id": e.id, "titre": titre(&e), "fichier": e.abs_path, "origine": origine(&e),
            "empreinte": e.hash, "approuve": approuve, "analysable": e.meta.scan,
            "simulable": simulation::simulable(&e.meta), "administrateur": e.meta.admin,
            "risque": e.meta.risk, "redemarrage": e.meta.reboot, "reversible": e.meta.reversible,
            "points_attention": e.attention, "constats": e.meta.findings,
            "options": e.meta.options, "reglages_figes": figee, "source": source,
        }));
        return Ok(code::OK);
    }
    println!("{}", titre(&e));
    println!("  {:<14} {}", "id", e.id);
    println!("  {:<14} {}", l!("fichier", "file"), e.abs_path);
    println!("  {:<14} {}", l!("origine", "origin"), origine(&e));
    println!("  {:<14} {}", l!("empreinte", "hash"), e.hash);
    println!("  {:<14} {}", l!("approuve", "approved"), oui_non(approuve));
    println!(
        "  {:<14} {} · {} {} · {} {} · {} {}",
        l!("risque", "risk"),
        e.meta.risk,
        l!("analyse", "check"),
        oui_non(e.meta.scan),
        l!("simulable", "simulates"),
        oui_non(simulation::simulable(&e.meta)),
        l!("administrateur", "administrator"),
        oui_non(e.meta.admin)
    );
    if !e.meta.options.is_empty() {
        println!("  {}", l!("Options :", "Options:"));
        for opt in &e.meta.options {
            let valeur = figee
                .get(&opt.key)
                .map(|v| format!("{v} ({})", l!("fige", "set")))
                .unwrap_or_default();
            let ligne = format!("    {:<20} [{}] {}  {valeur}", opt.key, opt.kind, opt.label);
            println!("{}", ligne.trim_end());
        }
    }
    if !e.attention.is_empty() {
        println!(
            "  {}",
            l!(
                "Points d'attention (indicatifs) :",
                "Points of attention (indicative):"
            )
        );
        for p in &e.attention {
            println!(
                "    {} {} : {} — {}",
                l!("ligne", "line"),
                p.ligne,
                p.code,
                p.extrait
            );
        }
    }
    if !e.meta.findings.is_empty() {
        println!("  {}", l!("Constats :", "Findings:"));
        for f in &e.meta.findings {
            println!(
                "    {} {} : {} — {}",
                l!("ligne", "line"),
                f.line,
                f.code,
                f.message
            );
        }
    }
    if !approuve {
        println!(
            "\n{}",
            l!(
                format!("Non approuve. Apres l'avoir lu (--source), en administrateur :\n  sindry approuver {} --empreinte {}", e.id, e.hash),
                format!("Not approved. After reading it (--source), as administrator:\n  sindry approve {} --hash {}", e.id, e.hash)
            )
        );
    }
    if let Some(s) = source {
        println!("\n{s}");
    }
    Ok(code::OK)
}

fn verifier(h: &tauri::AppHandle, id: &str, o: &Options) -> Res<i32> {
    trouver(h, id)?;
    let v = check_script(h.clone(), id.to_string())?;
    let erreurs = v
        .findings
        .iter()
        .filter(|f| matches!(f.severity, contract::Severity::Error))
        .count();
    let code_sortie = if v.syntax.parses && erreurs == 0 {
        code::OK
    } else {
        code::ECHEC
    };
    if o.json {
        json(&v);
        return Ok(code_sortie);
    }
    println!(
        "{} : {}",
        l!("Syntaxe", "Syntax"),
        if v.syntax.parses {
            l!(
                format!("correcte ({})", v.syntax.engine),
                format!("correct ({})", v.syntax.engine)
            )
        } else {
            l!("erreurs", "errors")
        }
    );
    for e in &v.syntax.errors {
        println!(
            "  {} {}:{} — {}",
            l!("ligne", "line"),
            e.line,
            e.column,
            e.message
        );
    }
    if v.findings.is_empty() {
        println!(
            "{}",
            l!("Contrat : aucun constat.", "Contract: no finding.")
        );
    }
    for f in &v.findings {
        let g = match f.severity {
            contract::Severity::Error => l!("erreur", "error"),
            contract::Severity::Warning => l!("avertissement", "warning"),
        };
        println!(
            "  {g} {} {} : {} — {}",
            l!("ligne", "line"),
            f.line,
            f.code,
            f.message
        );
    }
    Ok(code_sortie)
}

fn approuver(h: &tauri::AppHandle, id: &str, o: &Options) -> Res<i32> {
    let e = trouver(h, id)?;
    let Some(empreinte) = o.empreinte.as_deref() else {
        return Err(usage(l!(
            format!("approuver : --empreinte <h> est obligatoire — celle que montre « sindry montrer {id} », apres lecture du script."),
            format!("approve: --hash <h> is required — the one \"sindry show {id}\" prints, after reading the script.")
        )));
    };
    if script_approuve(&e)? {
        if !o.json {
            println!("{}", l!("Deja approuve.", "Already approved."));
        } else {
            json(&serde_json::json!({ "id": e.id, "empreinte": e.hash, "approuve": true }));
        }
        return Ok(code::OK);
    }
    // Ce qui est approuve, c'est ce qui a ete lu : l'empreinte donnee doit
    // etre celle du fichier tel qu'il est maintenant.
    if !empreinte.trim().eq_ignore_ascii_case(&e.hash) {
        return Err(Echec {
            code: code::REFUS,
            message: l!(
                "l'empreinte ne correspond pas au fichier actuel : il a change depuis votre lecture. Relisez-le (sindry montrer <id> --source).",
                "the hash does not match the current file: it changed since you read it. Read it again (sindry show <id> --source)."
            ),
        });
    }
    approve_script(e.hash.clone())?;
    if o.json {
        json(&serde_json::json!({ "id": e.id, "empreinte": e.hash, "approuve": true }));
    } else {
        println!(
            "{}",
            l!(
                format!("« {} » approuve.", titre(&e)),
                format!("\"{}\" approved.", titre(&e))
            )
        );
    }
    Ok(code::OK)
}

fn historique(h: &tauri::AppHandle, o: &Options) -> Res<i32> {
    let mut hist = history::load(h)?;
    let n = o.nombre.unwrap_or(20);
    hist.scripts.sort_by(|a, b| b.at.cmp(&a.at));
    hist.scripts.truncate(n);
    hist.lots.sort_by(|a, b| b.at.cmp(&a.at));
    hist.lots.truncate(n);
    if o.json {
        json(&hist);
        return Ok(code::OK);
    }
    if hist.scripts.is_empty() {
        println!("{}", l!("Aucune execution.", "No run yet."));
    }
    for r in &hist.scripts {
        let etat = if r.killed {
            l!("interrompu", "stopped")
        } else if r.success {
            l!("reussi", "success")
        } else {
            l!("echec", "failed")
        };
        let simule = if r.simulated {
            l!(" (simulation)", " (simulation)")
        } else {
            String::new()
        };
        println!(
            "{}  {}  {}{}  {}",
            r.at,
            colonne(&r.title, 40),
            colonne(&etat, 10),
            simule,
            duree(r.duration_ms)
        );
    }
    Ok(code::OK)
}

/* ------------------------------------------------------------------------
Executer : les lignes dans le terminal, Ctrl+C selon le §6.3
------------------------------------------------------------------------ */

enum Evenement {
    Ligne(runner::Ligne),
    Analyse(runner::AnalysisEnd),
    Fin(runner::RunEnd),
}

/// La destination du moteur pour la ligne de commande.
struct VersTerminal {
    envoi: std::sync::Mutex<mpsc::Sender<Evenement>>,
}

impl runner::Sortie for VersTerminal {
    fn ligne(&self, ligne: runner::Ligne) {
        if let Ok(e) = self.envoi.lock() {
            let _ = e.send(Evenement::Ligne(ligne));
        }
    }
    fn fin(&self, fin: runner::RunEnd) {
        if let Ok(e) = self.envoi.lock() {
            let _ = e.send(Evenement::Fin(fin));
        }
    }
    fn analyse(&self, fin: runner::AnalysisEnd) {
        if let Ok(e) = self.envoi.lock() {
            let _ = e.send(Evenement::Analyse(fin));
        }
    }
}

/// Les Ctrl+C recus depuis le debut. La console les compte, la boucle d'attente
/// les lit : le premier n'enchaine plus rien, le deuxieme interrompt ce qui
/// peut l'etre, le troisieme force (§6.3, le bouton « Arreter »).
static INTERRUPTIONS: AtomicU32 = AtomicU32::new(0);

#[cfg(windows)]
unsafe extern "system" fn sur_interruption(genre: u32) -> windows_sys::core::BOOL {
    use windows_sys::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT};
    if genre == CTRL_C_EVENT || genre == CTRL_BREAK_EVENT {
        INTERRUPTIONS.fetch_add(1, Ordering::SeqCst);
        1
    } else {
        0
    }
}

fn ecouter_interruptions() {
    #[cfg(windows)]
    // SAFETY: le gestionnaire est une fonction `extern "system"` qui ne fait
    // qu'incrementer un atomique ; il reste valide pendant toute la vie du
    // processus.
    unsafe {
        windows_sys::Win32::System::Console::SetConsoleCtrlHandler(Some(sur_interruption), 1);
    }
}

fn interrompu() -> bool {
    INTERRUPTIONS.load(Ordering::SeqCst) > 0
}

struct Execution {
    demarre: runner::RunStarted,
    fin: runner::RunEnd,
    analyse: Option<runner::AnalysisEnd>,
}

/// Lance, puis attend la fin en affichant les lignes (sauf en JSON).
fn executer_et_attendre(
    h: &tauri::AppHandle,
    afficher: bool,
    depart: impl FnOnce(Arc<dyn runner::Sortie>) -> Result<runner::RunStarted, String>,
) -> Res<Execution> {
    let (envoi, reception) = mpsc::channel();
    let sortie: Arc<dyn runner::Sortie> = Arc::new(VersTerminal {
        envoi: std::sync::Mutex::new(envoi),
    });
    let demarre = depart(sortie)?;
    let moteur = h.state::<Arc<runner::Runner>>().inner().clone();
    let mut vus = INTERRUPTIONS.load(Ordering::SeqCst);
    let mut analyse = None;
    loop {
        match reception.recv_timeout(Duration::from_millis(200)) {
            Ok(Evenement::Ligne(l)) => {
                if afficher {
                    if l.stream == "stderr" {
                        eprintln!("{}", l.text);
                    } else {
                        println!("{}", l.text);
                    }
                }
            }
            Ok(Evenement::Analyse(a)) => analyse = Some(a),
            Ok(Evenement::Fin(fin)) => {
                return Ok(Execution {
                    demarre,
                    fin,
                    analyse,
                })
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let n = INTERRUPTIONS.load(Ordering::SeqCst);
                if n == vus {
                    continue;
                }
                vus = n;
                if n == 1 {
                    eprintln!(
                        "{}",
                        l!(
                            "sindry : arret demande. L'action en cours va au bout, rien d'autre ne sera lance. Ctrl+C encore pour l'interrompre.",
                            "sindry: stop requested. The current action finishes, nothing else will start. Ctrl+C again to interrupt it."
                        )
                    );
                } else {
                    let r = runner::cancel_script(&moteur, &demarre.run_id, n >= 3)?;
                    eprintln!("sindry : {}", r.message);
                    if r.needs_confirmation {
                        eprintln!(
                            "{}",
                            l!(
                                "sindry : Ctrl+C une derniere fois pour l'arreter quand meme.",
                                "sindry: Ctrl+C one last time to stop it anyway."
                            )
                        );
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(Echec {
                    code: code::ECHEC,
                    message: l!(
                        "le moteur s'est arrete sans rendre de resultat.",
                        "the engine stopped without a result."
                    ),
                })
            }
        }
    }
}

fn analyser(
    h: &tauri::AppHandle,
    e: &discovery::ScriptEntry,
    figee: BTreeMap<String, serde_json::Value>,
    politique: &str,
    afficher: bool,
) -> Res<Execution> {
    let req = runner::RunRequest {
        script_id: e.id.clone(),
        expected_hash: e.hash.clone(),
        config: figee,
        policy: Some(politique.to_string()),
        resume_phase: None,
    };
    let (req, entree) = preparer_analyse(h, req)?;
    let base = discovery::base_dir(h)?;
    let moteur = h.state::<Arc<runner::Runner>>().inner().clone();
    let memoire = h.state::<Arc<analyse::Memoire>>().inner().clone();
    executer_et_attendre(h, afficher, move |sortie| {
        runner::run_analysis_vers(&base, moteur, req, entree, memoire, sortie)
    })
}

fn figee(reglages: &Settings, id: &str) -> BTreeMap<String, serde_json::Value> {
    reglages
        .overrides
        .get(id)
        .map(|o| o.config.clone())
        .unwrap_or_default()
}

fn analyser_commande(h: &tauri::AppHandle, id: &str, o: &Options) -> Res<i32> {
    ecouter_interruptions();
    let e = trouver(h, id)?;
    let reglages = settings::load(h)?;
    let x = analyser(
        h,
        &e,
        figee(&reglages, &e.id),
        &reglages.exec_policy,
        !o.json,
    )?;
    let Some(fin) = x.analyse else {
        return Err(Echec {
            code: code::ECHEC,
            message: l!("l'analyse n'a rien rendu.", "the check returned nothing."),
        });
    };
    let sel = analyse::selection_office(&e.meta, &fin.analysis);
    let code_sortie = if fin.success { code::OK } else { code::ECHEC };
    if o.json {
        json(&serde_json::json!({
            "id": e.id, "reussie": fin.success, "duree_ms": x.fin.duration_ms,
            "journal": x.fin.log_path, "analyse": fin.analysis,
            "selection_office": sel.config, "a_faire": sel.a_faire,
        }));
        return Ok(code_sortie);
    }
    println!();
    println!(
        "{}",
        l!(
            format!(
                "Analyse de « {} » : {} ({})",
                titre(&e),
                if fin.success { "reussie" } else { "en echec" },
                duree(x.fin.duration_ms)
            ),
            format!(
                "Check of \"{}\": {} ({})",
                titre(&e),
                if fin.success { "done" } else { "failed" },
                duree(x.fin.duration_ms)
            )
        )
    );
    for f in &fin.analysis.finds {
        let cible = if f.choice.is_empty() {
            f.option.clone()
        } else {
            format!("{}.{}", f.option, f.choice)
        };
        let mut champs: Vec<String> = Vec::new();
        for (k, v) in &f.fields {
            match (k.as_str(), v.parse::<u64>()) {
                ("size", Ok(n)) => champs.push(taille(n)),
                _ => champs.push(format!("{k}={v}")),
            }
        }
        println!("  {cible:<28} {}", champs.join("  "));
    }
    let mut par_option: BTreeMap<&str, usize> = BTreeMap::new();
    for it in &fin.analysis.items {
        *par_option.entry(it.option.as_str()).or_default() += 1;
    }
    for (opt, n) in par_option {
        println!(
            "  {opt:<28} {}",
            l!(format!("{n} element(s)"), format!("{n} item(s)"))
        );
    }
    for a in &fin.analysis.anomalies {
        println!("  ! {a}");
    }
    println!(
        "{}",
        l!(
            format!(
                "Coche d'office : {}",
                serde_json::to_string(&sel.config).unwrap_or_default()
            ),
            format!(
                "Ticked by default: {}",
                serde_json::to_string(&sel.config).unwrap_or_default()
            )
        )
    );
    println!(
        "{}",
        if sel.a_faire {
            l!(
                "Quelque chose a faire : « sindry lancer » le traitera.",
                "Something to do: \"sindry run\" will handle it."
            )
        } else {
            l!("Rien a faire.", "Nothing to do.")
        }
    );
    Ok(code_sortie)
}

/* ------------------------------------------------------------------------
Lancer : des actions et des lots, comme l'entretien du mode Simple
------------------------------------------------------------------------ */

#[derive(Serialize)]
struct ResultatAction {
    id: String,
    titre: String,
    /// `reussi`, `echec`, `refuse`, `interrompu`, `rien_a_faire`, `non_lance`.
    statut: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    raison: Option<String>,
    simule: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    code_sortie: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duree_ms: Option<u64>,
    redemarrage: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    libere: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    journal: Option<String>,
    changements: Vec<temoin::Changement>,
}

impl ResultatAction {
    fn sans_lancement(
        e: &discovery::ScriptEntry,
        statut: &'static str,
        raison: Option<String>,
    ) -> Self {
        ResultatAction {
            id: e.id.clone(),
            titre: titre(e),
            statut,
            raison,
            simule: false,
            code_sortie: None,
            duree_ms: None,
            redemarrage: false,
            libere: None,
            journal: None,
            changements: Vec::new(),
        }
    }
}

#[derive(Serialize)]
struct Bilan {
    lots: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    point_de_restauration: Option<restore::RestoreOutcome>,
    resultats: Vec<ResultatAction>,
    redemarrage: bool,
    code: i32,
}

fn lancer(h: &tauri::AppHandle, cibles: &[String], o: &Options) -> Res<i32> {
    ecouter_interruptions();
    let decouverte = discovery::discover(h)?;
    let reglages = settings::load(h)?;
    let groupes = settings::group_scripts(&reglages, decouverte.scripts.clone());

    // Les cibles, dans l'ordre donne ; une action partagee par deux lots ne
    // tourne qu'une fois (§4.1).
    let mut a_lancer: Vec<discovery::ScriptEntry> = Vec::new();
    let mut lots_lances: Vec<(String, String, Vec<String>)> = Vec::new();
    for cible in cibles {
        if let Some(id_lot) = cible.strip_prefix("lot:") {
            let g = groupes
                .lots
                .iter()
                .find(|g| {
                    g.lot.id.eq_ignore_ascii_case(id_lot)
                        || nom_lot(&g.lot).eq_ignore_ascii_case(id_lot)
                })
                .ok_or_else(|| {
                    introuvable(l!(
                        format!("aucun lot « {id_lot} » (sindry lots)."),
                        format!("no batch \"{id_lot}\" (sindry batches).")
                    ))
                })?;
            let ids: Vec<String> = g.scripts.iter().map(|e| e.id.clone()).collect();
            lots_lances.push((g.lot.id.clone(), nom_lot(&g.lot), ids));
            for e in &g.scripts {
                if !a_lancer.iter().any(|x| x.id == e.id) {
                    a_lancer.push(e.clone());
                }
            }
        } else {
            let e = decouverte
                .scripts
                .iter()
                .find(|s| &s.id == cible)
                .ok_or_else(|| {
                    introuvable(l!(
                        format!("aucune action « {cible} » (sindry scripts)."),
                        format!("no action \"{cible}\" (sindry list).")
                    ))
                })?;
            if !a_lancer.iter().any(|x| x.id == e.id) {
                a_lancer.push(e.clone());
            }
        }
    }

    for (id, nom, ids) in &lots_lances {
        history::record_lot_run(
            h,
            history::LotRunRecord {
                lot_id: id.clone(),
                lot_name: nom.clone(),
                script_ids: ids.clone(),
                at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            },
        )?;
    }

    // Un seul point de restauration avant toute la serie, si une action le
    // demande et ne sera pas simulee (§6.4).
    let general = simulation::bilan(
        &reglages,
        decouverte.scripts.iter().map(|s| (s.id.as_str(), &s.meta)),
    );
    let sera_simulee = |e: &discovery::ScriptEntry| {
        if o.simuler {
            simulation::simulable(&e.meta)
        } else {
            matches!(
                simulation::decision(&reglages, &e.id, &e.meta, &general),
                simulation::Decision::Simuler
            )
        }
    };
    let besoin_point = a_lancer.iter().any(|e| {
        let demande = reglages
            .overrides
            .get(&e.id)
            .and_then(|x| x.reversible_ack)
            .unwrap_or(!e.meta.reversible);
        demande && !sera_simulee(e)
    });
    let mut point = None;
    if besoin_point && !o.sans_restauration {
        let nom = lots_lances
            .first()
            .map(|(_, n, _)| n.clone())
            .unwrap_or_else(|| a_lancer.first().map(titre).unwrap_or_default());
        if !o.json {
            println!("{}", l!("Point de restauration…", "Restore point…"));
        }
        let r = restore::creer_point_de_restauration(&format!("Sindry - {nom}"))
            .unwrap_or_else(restore::RestoreOutcome::Failed);
        if !o.json {
            println!("  {}", message_restauration(&r));
        }
        point = Some(r);
    }

    let mut resultats: Vec<ResultatAction> = Vec::new();
    let politique = reglages.exec_policy.clone();
    for e in &a_lancer {
        if interrompu() {
            resultats.push(ResultatAction::sans_lancement(
                e,
                "non_lance",
                Some(l!("arret demande", "stop requested")),
            ));
            continue;
        }
        if !o.json {
            println!("\n== {}", titre(e));
        }
        let r = lancer_une(h, e, &reglages, &politique, o);
        let echec = matches!(r.statut, "echec" | "refuse" | "interrompu");
        if !o.json {
            println!("{}", resume_resultat(&r));
        }
        let a_suivre = r.statut == "a_suivre";
        resultats.push(r);
        if a_suivre {
            for reste in a_lancer.iter().skip(resultats.len()) {
                resultats.push(ResultatAction::sans_lancement(
                    reste,
                    "non_lance",
                    Some(l!(
                        "attend le redemarrage demande par l'action precedente",
                        "waits for the restart the previous action asked for"
                    )),
                ));
            }
            break;
        }
        if echec && reglages.failure_policy == "stop" {
            for reste in a_lancer.iter().skip(resultats.len()) {
                resultats.push(ResultatAction::sans_lancement(
                    reste,
                    "non_lance",
                    Some(l!(
                        "arret apres un echec (reglage 4.1)",
                        "stopped after a failure (setting 4.1)"
                    )),
                ));
            }
            break;
        }
    }

    let redemarrage = resultats.iter().any(|r| r.redemarrage);
    let code_sortie = if resultats
        .iter()
        .any(|r| matches!(r.statut, "echec" | "interrompu"))
    {
        code::ECHEC
    } else if resultats.iter().any(|r| r.statut == "refuse") {
        code::REFUS
    } else if redemarrage {
        code::REDEMARRAGE
    } else {
        code::OK
    };
    if o.json {
        json(&Bilan {
            lots: lots_lances.iter().map(|(id, _, _)| id.clone()).collect(),
            point_de_restauration: point,
            resultats,
            redemarrage,
            code: code_sortie,
        });
    } else {
        let reussis = resultats
            .iter()
            .filter(|r| matches!(r.statut, "reussi" | "rien_a_faire"))
            .count();
        println!(
            "\n{}",
            l!(
                format!(
                    "Termine : {reussis} action(s) sur {} reussie(s).",
                    resultats.len()
                ),
                format!(
                    "Done: {reussis} of {} action(s) succeeded.",
                    resultats.len()
                )
            )
        );
        if redemarrage {
            println!(
                "{}",
                l!(
                    "Un redemarrage est necessaire pour terminer (code de sortie 3010).",
                    "A restart is required to finish (exit code 3010)."
                )
            );
        }
    }
    Ok(code_sortie)
}

/// Une action : son analyse d'abord si elle sait analyser (et qu'on n'a pas
/// dit `--sans-analyse`), avec ce que l'interface cocherait d'office ; rien a
/// faire, elle n'est pas lancee. Puis l'action, et son historique.
fn lancer_une(
    h: &tauri::AppHandle,
    e: &discovery::ScriptEntry,
    reglages: &Settings,
    politique: &str,
    o: &Options,
) -> ResultatAction {
    let refus = |m: String| {
        let code_m = classer(&m);
        ResultatAction::sans_lancement(
            e,
            if code_m == code::REFUS || code_m == code::DROITS {
                "refuse"
            } else {
                "echec"
            },
            Some(expliquer(&m)),
        )
    };
    let mut config = figee(reglages, &e.id);

    if e.meta.scan && !o.sans_analyse {
        let x = match analyser(h, e, config.clone(), politique, !o.json) {
            Ok(x) => x,
            Err(err) => {
                return ResultatAction::sans_lancement(
                    e,
                    if err.code == code::REFUS {
                        "refuse"
                    } else {
                        "echec"
                    },
                    Some(err.message),
                )
            }
        };
        match x.analyse {
            Some(a) if a.success && x.fin.success => {
                let sel = analyse::selection_office(&e.meta, &a.analysis);
                if !sel.a_faire {
                    return ResultatAction::sans_lancement(e, "rien_a_faire", None);
                }
                config.extend(sel.config);
            }
            _ => {
                return ResultatAction::sans_lancement(
                    e,
                    "echec",
                    Some(l!(
                        "l'analyse a echoue : l'action n'a pas ete lancee.",
                        "the check failed: the action was not run."
                    )),
                )
            }
        }
        if interrompu() {
            return ResultatAction::sans_lancement(
                e,
                "non_lance",
                Some(l!("arret demande", "stop requested")),
            );
        }
    }

    let req = runner::RunRequest {
        script_id: e.id.clone(),
        expected_hash: e.hash.clone(),
        config,
        policy: Some(politique.to_string()),
        resume_phase: None,
    };
    let memoire = h.state::<Arc<analyse::Memoire>>().inner().clone();
    let (req, entree, simule) = match preparer_action(h, &memoire, req, o.simuler) {
        Ok(x) => x,
        Err(m) => return refus(m),
    };
    let base = match discovery::base_dir(h) {
        Ok(b) => b,
        Err(m) => return refus(m),
    };
    let moteur = h.state::<Arc<runner::Runner>>().inner().clone();
    let x = match executer_et_attendre(h, !o.json, move |sortie| {
        runner::run_script_vers(&base, moteur, req, entree, sortie)
    }) {
        Ok(x) => x,
        Err(err) => return ResultatAction::sans_lancement(e, "echec", Some(err.message)),
    };
    let fin = x.fin;
    let succes = fin.success && !fin.killed;
    let _ = history::record_script_run(
        h,
        history::ScriptRunRecord {
            script_id: e.id.clone(),
            title: titre(e),
            success: succes,
            killed: fin.killed,
            duration_ms: fin.duration_ms,
            simulated: simule,
            freed: fin.freed,
            changes: fin.changes.clone(),
            at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        },
    );
    let _ = enforce_log_cap(h.clone());
    ResultatAction {
        id: e.id.clone(),
        titre: titre(e),
        statut: if fin.killed {
            "interrompu"
        } else if fin.restart_phase.is_some() {
            // 2.1.0 : le script attend un redemarrage pour continuer. La ligne
            // de commande ne redemarre pas et ne reprend pas seule (§6.10).
            "a_suivre"
        } else if succes {
            "reussi"
        } else {
            "echec"
        },
        raison: None,
        simule: x.demarre.simulated || simule,
        code_sortie: fin.exit_code,
        duree_ms: Some(fin.duration_ms),
        redemarrage: fin.reboot_requested || fin.restart_phase.is_some(),
        libere: fin.freed,
        journal: Some(fin.log_path),
        changements: fin.changes,
    }
}

fn resume_resultat(r: &ResultatAction) -> String {
    let etat = match r.statut {
        "reussi" => l!("reussi", "succeeded"),
        "rien_a_faire" => l!("rien a faire", "nothing to do"),
        "refuse" => l!("refuse", "refused"),
        "interrompu" => l!("interrompu", "stopped"),
        "non_lance" => l!("non lance", "not run"),
        "a_suivre" => l!(
            "a suivre apres un redemarrage",
            "to be continued after a restart"
        ),
        _ => l!("echec", "failed"),
    };
    let mut s = format!("-- {} : {etat}", r.titre);
    if r.simule {
        s.push_str(&l!(" (simulation)", " (simulation)"));
    }
    if let Some(ms) = r.duree_ms {
        s.push_str(&format!(" · {}", duree(ms)));
    }
    if let Some(octets) = r.libere {
        s.push_str(&l!(
            format!(" · {} liberes", taille(octets)),
            format!(" · {} freed", taille(octets))
        ));
    }
    if let Some(raison) = &r.raison {
        s.push_str(&format!("\n   {raison}"));
    }
    if !r.changements.is_empty() {
        s.push_str(&l!(
            format!(
                "\n   Changements sensibles : {} (sindry historique --json)",
                r.changements.len()
            ),
            format!(
                "\n   Sensitive changes: {} (sindry history --json)",
                r.changements.len()
            )
        ));
    }
    if r.redemarrage {
        s.push_str(&l!(
            "\n   Redemarrage necessaire.",
            "\n   Restart required."
        ));
    }
    s
}

fn message_restauration(r: &restore::RestoreOutcome) -> String {
    match r {
        restore::RestoreOutcome::Created => l!("cree.", "created."),
        restore::RestoreOutcome::ThrottledRecent => l!(
            "un point de moins de 24 h existe deja : Windows n'en cree pas d'autre.",
            "a point less than 24 h old exists: Windows does not create another."
        ),
        restore::RestoreOutcome::ProtectionDisabled => l!(
            "la protection du systeme est desactivee : aucun point cree.",
            "system protection is off: no point created."
        ),
        restore::RestoreOutcome::ServiceDisabled => l!(
            "le service de restauration est desactive : aucun point cree.",
            "the restore service is disabled: no point created."
        ),
        restore::RestoreOutcome::Failed(m) => l!(format!("echec : {m}"), format!("failed: {m}")),
    }
}

/* ------------------------------------------------------------------------
Reglages et configuration
------------------------------------------------------------------------ */

fn reglages(h: &tauri::AppHandle, o: &Options) -> Res<i32> {
    let s = settings::load(h)?;
    let sim = simulation_state(h.clone())?;
    if o.json {
        json(&serde_json::json!({ "reglages": s, "simulation": sim }));
        return Ok(code::OK);
    }
    let ligne = |k: String, v: String| println!("  {:<16} {v}", k);
    ligne(l!("theme", "theme"), s.theme.clone());
    ligne(l!("langue", "lang"), s.lang.clone());
    ligne(
        l!("ambiance", "look"),
        if s.accent.is_empty() {
            "plan".into()
        } else {
            s.accent.clone()
        },
    );
    ligne(l!("mises-a-jour", "updates"), s.update_policy.clone());
    ligne(l!("echec", "failure"), s.failure_policy.clone());
    ligne(
        l!("journaux", "logs"),
        if s.log_cap_mb == 0 {
            l!("pas de limite", "no limit")
        } else {
            format!("{} Mo", s.log_cap_mb)
        },
    );
    ligne(l!("politique", "policy"), s.exec_policy.clone());
    ligne(
        l!("simulation", "simulation"),
        format!(
            "{} ({}/{})",
            serde_json::to_value(sim.etat)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_default(),
            sim.simules,
            sim.simulables
        ),
    );
    ligne(l!("echelle", "scale"), format!("{}", s.ui_scale));
    ligne(
        l!("catalogue", "catalogue"),
        format!(
            "{} · {}",
            if s.catalogue_source.is_empty() {
                "-"
            } else {
                &s.catalogue_source
            },
            s.catalogue_check
        ),
    );
    Ok(code::OK)
}

fn valeur_parmi(v: &str, choix: &[(&[&str], &str)], nom: &str) -> Res<String> {
    let bas = v.to_lowercase();
    choix
        .iter()
        .find(|(alias, _)| alias.contains(&bas.as_str()))
        .map(|(_, valeur)| valeur.to_string())
        .ok_or_else(|| {
            let admis: Vec<&str> = choix.iter().map(|(a, _)| a[0]).collect();
            usage(l!(
                format!("{nom} : « {v} » inconnu ({}).", admis.join(", ")),
                format!("{nom}: \"{v}\" unknown ({}).", admis.join(", "))
            ))
        })
}

fn regler(h: &tauri::AppHandle, cle: &str, v: &str, o: &Options) -> Res<i32> {
    let s = match cle.to_lowercase().as_str() {
        "theme" | "thème" => set_theme(
            h.clone(),
            valeur_parmi(v, &[(&["clair", "light"], "light"), (&["sombre", "dark"], "dark"), (&["systeme", "système", "system"], "system")], cle)?,
        )?,
        "langue" | "lang" | "language" => set_lang(h.clone(), valeur_parmi(v, &[(&["fr"], "fr"), (&["en"], "en")], cle)?)?,
        "ambiance" | "look" => {
            let bas = v.to_lowercase();
            let accent = if ambiance::lister_dans(&ambiance::dossier(h)?)
                .ambiances
                .iter()
                .any(|a| a.id == bas)
            {
                bas
            } else {
                return Err(introuvable(l!(
                    format!("aucune ambiance « {v} » (sindry ambiances)."),
                    format!("no look \"{v}\" (sindry looks).")
                )));
            };
            set_accent(h.clone(), accent)?
        }
        "mises-a-jour" | "maj" | "updates" => set_update_policy(
            h.clone(),
            valeur_parmi(v, &[(&["proposer", "propose", "stable"], "propose"), (&["beta"], "beta"), (&["jamais", "never"], "never")], cle)?,
        )?,
        "echec" | "échec" | "failure" => set_failure_policy(
            h.clone(),
            valeur_parmi(v, &[(&["continuer", "continue"], "continue"), (&["arreter", "arrêter", "stop"], "stop")], cle)?,
        )?,
        "journaux" | "logs" => {
            let mb: u32 = v.parse().map_err(|_| usage(l!(format!("{cle} : un nombre de Mo (0 : pas de limite)."), format!("{cle}: a number of MB (0: no limit)."))))?;
            set_log_cap(h.clone(), mb)?
        }
        "politique" | "policy" => set_exec_policy(
            h.clone(),
            valeur_parmi(v, &[(&["bypass"], "bypass"), (&["remotesigned"], "remotesigned"), (&["unrestricted"], "unrestricted")], cle)?,
        )?,
        "simulation" => {
            let oui = valeur_parmi(v, &[(&["oui", "yes", "on"], "1"), (&["non", "no", "off"], "0")], cle)? == "1";
            set_simulation_all(h.clone(), oui)?
        }
        "echelle" | "échelle" | "scale" => {
            let f: f32 = v.replace(',', ".").parse().map_err(|_| usage(l!(format!("{cle} : un nombre (1, 1.15, 1.3…)."), format!("{cle}: a number (1, 1.15, 1.3…)."))))?;
            set_ui_scale(h.clone(), f)?
        }
        autre => {
            return Err(usage(l!(
                format!("reglage inconnu : {autre} (theme, langue, ambiance, mises-a-jour, echec, journaux, politique, simulation, echelle)."),
                format!("unknown setting: {autre} (theme, lang, look, updates, failure, logs, policy, simulation, scale).")
            )))
        }
    };
    if o.json {
        json(&s);
    } else {
        println!("{}", l!("Enregistre.", "Saved."));
    }
    Ok(code::OK)
}

fn config(h: &tauri::AppHandle, reste: &[String], o: &Options) -> Res<i32> {
    let sous = reste.first().map(|s| s.to_lowercase()).unwrap_or_default();
    match sous.as_str() {
        "exporter" | "export" => {
            let fichier = argument(reste, 1, l!("fichier", "file"))?;
            let s = settings::load(h)?;
            let texte = serde_json::to_string_pretty(&s).map_err(|e| e.to_string())?;
            std::fs::write(fichier, texte).map_err(|e| format!("{fichier} : {e}"))?;
            if !o.json {
                println!(
                    "{}",
                    l!(
                        format!("Configuration exportee : {fichier}"),
                        format!("Configuration exported: {fichier}")
                    )
                );
            } else {
                json(&serde_json::json!({ "fichier": fichier }));
            }
            Ok(code::OK)
        }
        "importer" | "import" => {
            let fichier = argument(reste, 1, l!("fichier", "file"))?;
            let texte = std::fs::read_to_string(fichier).map_err(|e| format!("{fichier} : {e}"))?;
            let s = settings::import_from_str(h, texte.trim_start_matches('\u{feff}'))?;
            if o.json {
                json(&s);
            } else {
                println!(
                    "{}",
                    l!("Configuration importee.", "Configuration imported.")
                );
            }
            Ok(code::OK)
        }
        "reinitialiser" | "réinitialiser" | "reset" => {
            if !o.oui {
                return Err(usage(l!(
                    "reinitialiser efface vos lots et vos reglages : ajoutez --oui pour confirmer.",
                    "reset erases your batches and settings: add --yes to confirm."
                )));
            }
            let s = settings::reset(h)?;
            if o.json {
                json(&s);
            } else {
                println!(
                    "{}",
                    l!("Reglages d'usine retablis.", "Factory settings restored.")
                );
            }
            Ok(code::OK)
        }
        _ => Err(usage(l!(
            "config exporter|importer <fichier>, ou config reinitialiser --oui.",
            "config export|import <file>, or config reset --yes."
        ))),
    }
}

/* ------------------------------------------------------------------------
Catalogues
------------------------------------------------------------------------ */

fn catalogues(h: &tauri::AppHandle, o: &Options) -> Res<i32> {
    let etat = sources_state(h.clone())?;
    if o.json {
        json(&etat);
        return Ok(code::OK);
    }
    for s in &etat.sources {
        let installe = s
            .etat
            .installe
            .as_ref()
            .map(|r| {
                l!(
                    format!("{} ({} scripts)", r.version, r.scripts),
                    format!("{} ({} scripts)", r.version, r.scripts)
                )
            })
            .unwrap_or_else(|| l!("non installe", "not installed"));
        println!(
            "{}  {}  {}  {}{}",
            colonne(&s.id, 22),
            colonne(&s.nom, 24),
            installe,
            if s.active {
                String::new()
            } else {
                l!(" · desactive", " · disabled")
            },
            if s.officielle {
                l!(" · officiel", " · official")
            } else {
                String::new()
            }
        );
        if let Some(p) = &s.etat.probleme {
            println!("    ! {p}");
        }
    }
    for p in &etat.problemes {
        eprintln!("sindry : {p}");
    }
    Ok(code::OK)
}

fn catalogue(h: &tauri::AppHandle, reste: &[String], o: &Options) -> Res<i32> {
    let sous = reste.first().map(|s| s.to_lowercase()).unwrap_or_default();
    let id = reste
        .get(1)
        .cloned()
        .unwrap_or_else(|| catalogue::SOURCE.to_string());
    match sous.as_str() {
        "verifier" | "vérifier" | "check" => {
            let b = tauri::async_runtime::block_on(check_source(h.clone(), id))?;
            if o.json {
                json(&b);
                return Ok(code::OK);
            }
            if b.a_jour {
                println!(
                    "{}",
                    l!(
                        format!("A jour ({}).", b.version),
                        format!("Up to date ({}).", b.version)
                    )
                );
            } else {
                println!(
                    "{}",
                    l!(
                        format!(
                            "Version {} disponible (installee : {}).",
                            b.version,
                            b.installee.as_deref().unwrap_or("-")
                        ),
                        format!(
                            "Version {} available (installed: {}).",
                            b.version,
                            b.installee.as_deref().unwrap_or("-")
                        )
                    )
                );
                let n = |v: &Vec<catalogue::Element>| v.len();
                println!(
                    "  {}",
                    l!(
                        format!(
                            "nouveaux {} · mis a jour {} · remplaces {} · retires {}",
                            n(&b.nouveaux),
                            n(&b.mis_a_jour),
                            n(&b.remplaces),
                            n(&b.retires)
                        ),
                        format!(
                            "new {} · updated {} · replaced {} · removed {}",
                            n(&b.nouveaux),
                            n(&b.mis_a_jour),
                            n(&b.remplaces),
                            n(&b.retires)
                        )
                    )
                );
            }
            Ok(code::OK)
        }
        "installer" | "install" => {
            let fait = tauri::async_runtime::block_on(install_source(h.clone(), h.state(), id))?;
            if o.json {
                json(&fait);
            } else {
                println!(
                    "{}",
                    l!(
                        format!(
                            "Installe : version {}, {} script(s) ecrit(s).",
                            fait.version, fait.ecrits
                        ),
                        format!(
                            "Installed: version {}, {} script(s) written.",
                            fait.version, fait.ecrits
                        )
                    )
                );
                for c in &fait.copies {
                    println!(
                        "  {}",
                        l!(
                            format!("version modifiee gardee : {c}"),
                            format!("modified version kept: {c}")
                        )
                    );
                }
                for r in &fait.retires {
                    println!(
                        "  {}",
                        l!(
                            format!("sorti du catalogue : {} (a approuver desormais)", r.id),
                            format!("left the catalogue: {} (needs approval now)", r.id)
                        )
                    );
                }
            }
            Ok(code::OK)
        }
        "activer" | "enable" | "desactiver" | "désactiver" | "disable" => {
            let id = argument(reste, 1, "id")?.to_string();
            let actif = matches!(sous.as_str(), "activer" | "enable");
            let s = set_source_active(h.clone(), id, actif)?;
            if o.json {
                json(&s.sources_inactives);
            } else {
                println!("{}", l!("Enregistre.", "Saved."));
            }
            Ok(code::OK)
        }
        _ => Err(usage(l!(
            "catalogue verifier|installer [<id>], ou catalogue activer|desactiver <id>.",
            "catalog check|install [<id>], or catalog enable|disable <id>."
        ))),
    }
}

/* ------------------------------------------------------------------------
Ambiances (§15.7)
------------------------------------------------------------------------ */

fn ambiances(h: &tauri::AppHandle, o: &Options) -> Res<i32> {
    let liste = ambiance::lister_dans(&ambiance::dossier(h)?);
    if o.json {
        json(&liste);
        return Ok(code::OK);
    }
    println!(
        "{}",
        l!(
            format!("Dossier : {}", liste.dossier),
            format!("Folder: {}", liste.dossier)
        )
    );
    if liste.ambiances.is_empty() {
        println!("{}", l!("Aucune ambiance installee.", "No look installed."));
    }
    for a in &liste.ambiances {
        let nom = if anglais() { &a.nom.en } else { &a.nom.fr };
        let modes = match (&a.clair, &a.sombre) {
            (Some(_), Some(_)) => l!("clair et sombre", "light and dark"),
            (Some(_), None) => l!("clair", "light"),
            _ => l!("sombre", "dark"),
        };
        println!(
            "  {}  {}  {modes}{}{}",
            colonne(&a.id, 24),
            colonne(nom, 28),
            if a.decor.balisage.is_empty() {
                String::new()
            } else {
                l!(" · decor", " · scenery")
            },
            if a.progress.is_some() {
                l!(" · progression", " · progress")
            } else {
                String::new()
            }
        );
    }
    for i in &liste.ignorees {
        println!(
            "  ! {} — {}",
            i.fichier,
            i.constats.first().map(|c| c.message.as_str()).unwrap_or("")
        );
    }
    Ok(code::OK)
}

fn afficher_constats(constats: &[ambiance::Constat]) {
    for c in constats {
        println!("  {:<18} {}", c.code, c.message);
    }
}

fn ambiance_verifier(fichier: Option<&String>, o: &Options) -> Res<i32> {
    let fichier =
        fichier.ok_or_else(|| usage(l!("ambiance verifier <fichier>", "look check <file>")))?;
    let texte = std::fs::read_to_string(fichier).map_err(|e| Echec {
        code: code::INTROUVABLE,
        message: format!("{fichier} : {e}"),
    })?;
    match ambiance::valider(&texte) {
        Ok(a) => {
            if o.json {
                json(
                    &serde_json::json!({ "valide": true, "id": a.id, "nom": a.nom, "constats": [] }),
                );
            } else {
                println!(
                    "{}",
                    l!(
                        format!("« {} » ({}) : valide.", a.nom.fr, a.id),
                        format!("\"{}\" ({}): valid.", a.nom.en, a.id)
                    )
                );
            }
            Ok(code::OK)
        }
        Err(constats) => {
            if o.json {
                json(&serde_json::json!({ "valide": false, "constats": constats }));
            } else {
                println!(
                    "{}",
                    l!(
                        format!("{fichier} : {} probleme(s).", constats.len()),
                        format!("{fichier}: {} problem(s).", constats.len())
                    )
                );
                afficher_constats(&constats);
            }
            Ok(code::REFUS)
        }
    }
}

fn ambiance(h: &tauri::AppHandle, reste: &[String], o: &Options) -> Res<i32> {
    let sous = reste.first().map(|s| s.to_lowercase()).unwrap_or_default();
    let dossier = ambiance::dossier(h)?;
    match sous.as_str() {
        "importer" | "import" => {
            let fichier = argument(reste, 1, l!("fichier", "file"))?;
            let texte = std::fs::read_to_string(fichier).map_err(|e| Echec {
                code: code::INTROUVABLE,
                message: format!("{fichier} : {e}"),
            })?;
            let r = ambiance::importer_dans(&dossier, &texte, o.remplacer)?;
            let code_sortie = match &r {
                ambiance::Import::Ajoutee { .. } => code::OK,
                _ => code::REFUS,
            };
            if o.json {
                json(&r);
                return Ok(code_sortie);
            }
            match r {
                ambiance::Import::Ajoutee { ambiance } => println!(
                    "{}",
                    l!(format!("Ambiance « {} » installee ({}).", ambiance.nom.fr, ambiance.id), format!("Look \"{}\" installed ({}).", ambiance.nom.en, ambiance.id))
                ),
                ambiance::Import::Existe { actuelle, nouvelle } => println!(
                    "{}",
                    l!(
                        format!("Une ambiance « {} » est deja installee (version {}). Pour la remplacer par la version {} : --remplacer.", actuelle.id, if actuelle.version.is_empty() { "-" } else { &actuelle.version }, if nouvelle.version.is_empty() { "-" } else { &nouvelle.version }),
                        format!("A look \"{}\" is already installed (version {}). To replace it with version {}: --replace.", actuelle.id, if actuelle.version.is_empty() { "-" } else { &actuelle.version }, if nouvelle.version.is_empty() { "-" } else { &nouvelle.version })
                    )
                ),
                ambiance::Import::Refusee { constats } => {
                    println!("{}", l!("Ce fichier n'est pas une ambiance valide :", "This file is not a valid look:"));
                    afficher_constats(&constats);
                }
            }
            Ok(code_sortie)
        }
        "exporter" | "export" => {
            let id = argument(reste, 1, "id")?;
            let cible = argument(reste, 2, l!("fichier", "file"))?;
            let a = ambiance::lister_dans(&dossier)
                .ambiances
                .into_iter()
                .find(|a| a.id == id)
                .ok_or_else(|| introuvable(l!(format!("aucune ambiance installee « {id} »."), format!("no installed look \"{id}\"."))))?;
            std::fs::copy(dossier.join(&a.fichier), cible).map_err(|e| format!("{cible} : {e}"))?;
            if o.json {
                json(&serde_json::json!({ "id": a.id, "fichier": cible }));
            } else {
                println!("{}", l!(format!("Ambiance copiee : {cible}"), format!("Look copied: {cible}")));
            }
            Ok(code::OK)
        }
        "retirer" | "remove" => {
            let id = argument(reste, 1, "id")?;
            if !o.oui {
                return Err(usage(l!("retirer efface le fichier de l'ambiance : ajoutez --oui pour confirmer.", "remove deletes the look's file: add --yes to confirm.")));
            }
            let chemin = ambiance::retirer_dans(&dossier, id).map_err(introuvable)?;
            let s = settings::load(h)?;
            if s.accent == id {
                set_accent(h.clone(), "plan".to_string())?;
            }
            if o.json {
                json(&serde_json::json!({ "id": id, "efface": chemin }));
            } else {
                println!("{}", l!(format!("Ambiance retiree : {chemin}"), format!("Look removed: {chemin}")));
            }
            Ok(code::OK)
        }
        "choisir" | "use" => regler(h, "ambiance", argument(reste, 1, "id")?, o),
        _ => Err(usage(l!(
            "ambiance verifier|importer <fichier>, exporter <id> <fichier>, retirer <id> --oui, choisir <id>.",
            "look check|import <file>, export <id> <file>, remove <id> --yes, use <id>."
        ))),
    }
}

/* ------------------------------------------------------------------------
Pour l'installeur : preconfiguration et PATH (§19.3)
------------------------------------------------------------------------ */

/// `sindry.com --installeur preconfigurer <ligne de commande de l'installeur>`
/// et `sindry.com --installeur chemin retirer`. Appeles par les crochets NSIS,
/// en administrateur, sans console visible ; ce qu'ils ecrivent part dans le
/// journal de l'installeur.
fn installeur(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("preconfigurer") => match preconfiguration::depuis_options(&args[1..]) {
            Ok((p, chemin)) => {
                let mut code_sortie = code::OK;
                if !p.est_vide() {
                    match preconfiguration::dossier_installation()
                        .ok_or_else(|| "dossier d'installation introuvable".to_string())
                        .and_then(|d| preconfiguration::ecrire_dans(&d, &p))
                    {
                        Ok(f) => println!("preconfiguration : {}", f.display()),
                        Err(e) => {
                            eprintln!("preconfiguration : {e}");
                            code_sortie = code::ECHEC;
                        }
                    }
                }
                if chemin {
                    match preconfiguration::chemin_systeme(true) {
                        Ok(_) => println!("PATH : dossier d'installation ajoute"),
                        Err(e) => {
                            eprintln!("PATH : {e}");
                            code_sortie = code::ECHEC;
                        }
                    }
                }
                code_sortie
            }
            Err(e) => {
                eprintln!("{e}");
                code::USAGE
            }
        },
        Some("chemin") => {
            let ajouter = args.get(1).map(String::as_str) == Some("ajouter");
            match preconfiguration::chemin_systeme(ajouter) {
                Ok(_) => code::OK,
                Err(e) => {
                    eprintln!("PATH : {e}");
                    code::ECHEC
                }
            }
        }
        _ => code::USAGE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn le_json_sort_en_ascii_pur_et_se_relit() {
        let v = serde_json::json!({ "nom": "Forêt de nuit 🌲", "n": 1 });
        let s = json_ascii(&v);
        assert!(s.is_ascii(), "{s}");
        assert!(s.contains("\\u00ea"));
        assert_eq!(serde_json::from_str::<serde_json::Value>(&s).unwrap(), v);
    }

    #[test]
    fn les_options_se_separent_des_arguments() {
        let (p, o) = separer(&args(&[
            "lancer",
            "lot:cleaning",
            "--simuler",
            "--json",
            "x",
            "--nombre",
            "5",
        ]))
        .ok()
        .unwrap();
        assert_eq!(p, args(&["lancer", "lot:cleaning", "x"]));
        assert!(o.simuler && o.json && !o.sans_analyse);
        assert_eq!(o.nombre, Some(5));
        let (p, _) = separer(&args(&["--help"])).ok().unwrap();
        assert_eq!(p, args(&["aide"]));
        assert!(separer(&args(&["--inconnue"])).is_err());
        assert!(separer(&args(&["historique", "--nombre"])).is_err());
        assert!(separer(&args(&["historique", "--nombre", "cinq"])).is_err());
        let (_, o) = separer(&args(&[
            "approve",
            "x",
            "--hash",
            "abc",
            "--simulate",
            "--no-scan",
        ]))
        .ok()
        .unwrap();
        assert_eq!(o.empreinte.as_deref(), Some("abc"));
        assert!(o.simuler && o.sans_analyse);
    }

    #[test]
    fn les_erreurs_de_sindry_ont_leur_code_de_sortie() {
        assert_eq!(classer(NON_APPROUVE), code::REFUS);
        assert_eq!(classer(SANS_SIMULATION), code::REFUS);
        assert_eq!(
            classer("REGLAGE_REFUSE:emplacement:Dossiers:C:\\Windows"),
            code::REFUS
        );
        assert_eq!(classer("ANALYSE_PERIMEE:Fichiers — x"), code::REFUS);
        assert_eq!(classer(APPROBATION_SANS_DROITS), code::DROITS);
        assert_eq!(classer(SOURCES_SANS_DROITS), code::DROITS);
        assert_eq!(classer("Script introuvable : x"), code::INTROUVABLE);
        assert_eq!(classer(&format!("{SOURCE_INCONNUE}: x")), code::INTROUVABLE);
        assert_eq!(classer("autre chose"), code::ECHEC);
        // Les sentinelles ne s'affichent jamais telles quelles.
        for s in [
            NON_APPROUVE,
            SANS_SIMULATION,
            SANS_ANALYSE,
            APPROBATION_SANS_DROITS,
        ] {
            assert!(!expliquer(s).contains(s), "{s}");
        }
        assert!(expliquer("REGLAGE_REFUSE:choix:Dns:opendns").contains("opendns"));
    }

    #[test]
    fn colonnes_et_tailles() {
        assert_eq!(colonne("abc", 5), "abc  ");
        assert_eq!(colonne("abcdefgh", 5), "abcd…");
        assert_eq!(taille(512), "512 o");
        assert!(taille(1536).starts_with("1.5"));
        assert_eq!(duree(1500), "1.5 s");
        assert_eq!(duree(125_000), "2 min 05 s");
    }
}
