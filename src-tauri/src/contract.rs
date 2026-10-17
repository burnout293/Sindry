//! Lecture du contrat de script v2.
//!
//! Ce module est le pendant Rust de `tools/lint-scripts.ps1`. Les deux lisent le
//! meme format et doivent rester d'accord : voir `docs/FORMAT_SCRIPT.md`.
//!
//! Principe de lecture (style B) :
//!   `## SINDRY:START`    ce qu'est le script
//!   `## SINDRY:OPTIONS`  ses reglages, types et nommes
//!   `## SINDRY:REPORT`   ce que son analyse nomme : libelles, notes, groupes
//!   `## SINDRY:LANG xx`  les memes, traduits
//!   `$CONFIG = @{ ... }`  les valeurs par defaut, du PowerShell pur
//!
//! Regle de la specification 5.4 : **constater, jamais bloquer**. Un script non
//! conforme est lu quand meme, et ses anomalies sont rapportees dans `findings`
//! avec leur numero de ligne. L'application les affiche en mode Expert ; elle
//! n'empeche jamais l'execution.

use serde::Serialize;
use std::collections::BTreeMap;

/// Separateur entre le libelle et la description : un tiret cadratin entoure
/// d'espaces. Le reste de la ligne apres lui est la description.
const SEP_DESC: &str = " — ";

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

/// Une anomalie de conformite, rapportee et jamais bloquante.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub line: usize,
    pub severity: Severity,
    pub code: String,
    pub message: String,
}

/// Les vues qu'un script peut demander pour son analyse (`## view`, `[view:…]`).
/// Doit rester d'accord avec `$VUES` dans `tools/lint-scripts.ps1`.
pub const VUES: [&str; 14] = [
    "minimal",
    "light",
    "gauge",
    "checklist",
    "bars",
    "donut",
    "tiles",
    "compare",
    "tree",
    "table",
    "treemap",
    "timeline",
    "chart",
    "history",
];

/// Les graphiques du resume de l'analyse en mode Simple (`## chart`, 1.7) :
/// l'anneau-engrenage, la double couronne, la jauge a dents, la barre d'espace,
/// le fond des lignes, les rectangles, la gaufre. Doit rester d'accord avec
/// `$GRAPHIQUES` dans `tools/lint-scripts.ps1` et `resume()` dans analyse.js.
pub const GRAPHIQUES: [&str; 7] = [
    "donut", "crown", "gauge", "bar", "rows", "treemap", "waffle",
];

/// Les panneaux du mode Expert qu'un script peut ouvrir d'office (`## panels`).
pub const PANNEAUX: [&str; 8] = [
    "config",
    "progress",
    "plan",
    "payload",
    "attention",
    "history",
    "origin",
    "disk",
];

/// Un choix d'une option `select` ou `multi`.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Choice {
    pub value: String,
    pub label: String,
    pub desc: String,
    /// `[group:Cle]` : le poste du resume Simple auquel ce choix appartient.
    pub group: String,
    /// `[show:expert]` : ce choix ne s'affiche que dans ce mode.
    pub show: String,
}

/// Une entree du bloc `REPORT` : ce que l'analyse nomme par un jeton.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ReportEntry {
    /// `label` (un nom : `label=`, `[METRIC]`), `note`, `group` or `error`.
    pub kind: String,
    /// Pour une note : `info` ou `warn`.
    pub level: String,
    /// For an error code: what went wrong.
    pub label: String,
    /// Pour un groupe : son sous-titre. For an error code: the way out, if any.
    pub desc: String,
}

/// An error code declared with `[error]` in REPORT (2.0.3): one to three
/// capital letters, then two to four digits — `E104`, `SVC12`. A script
/// writes `[ERR] E104 <technical detail>`; Sindry shows the translated
/// description and solution instead of the bare code. Must match
/// `Test-CodeErreur` in `tools/lint-scripts.ps1`.
pub fn is_error_code(word: &str) -> bool {
    let letters = word.chars().take_while(|c| c.is_ascii_uppercase()).count();
    let digits = &word[letters..];
    (1..=3).contains(&letters)
        && (2..=4).contains(&digits.len())
        && digits.chars().all(|c| c.is_ascii_digit())
}

/// Valeur par defaut telle que declaree dans `$CONFIG`.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum DefaultValue {
    Bool(bool),
    Number(f64),
    Text(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Opt {
    pub key: String,
    /// bool | number | string | select | multi | hidden | items
    pub kind: String,
    pub label: String,
    pub desc: String,
    pub choices: Vec<Choice>,
    pub default: Option<DefaultValue>,
    /// `hidden` n'est visible qu'en mode Expert (specification 5.2).
    pub hidden: bool,
    /// `[items:keep-one]` : des groupes dont on garde un exemplaire.
    pub keep_one: bool,
    /// `[scan]` : un reglage de l'analyse elle-meme, pas une case a cocher.
    pub scan: bool,
    /// `[group:Cle]`, `[show:expert]`, `[view:tree]` (§17).
    pub group: String,
    pub show: String,
    pub view: String,
}

/// Libelles traduits d'une langue : options, choix et bloc REPORT.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Translation {
    pub title: String,
    pub desc: String,
    /// cle d'option -> (libelle, description)
    pub options: BTreeMap<String, (String, String)>,
    /// "cle_option/valeur_choix" -> (libelle, description)
    pub choices: BTreeMap<String, (String, String)>,
    /// cle du bloc REPORT -> (texte, sous-titre pour un groupe)
    pub report: BTreeMap<String, (String, String)>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Script {
    /// Identifiant stable issu de l'entete. Vide si le script n'en declare pas :
    /// l'appelant retombe alors sur le chemin relatif (specification 4.3).
    pub id: String,
    pub lang: String,
    pub title: String,
    pub desc: String,
    pub category: String,
    pub icon: String,
    pub tags: Vec<String>,
    pub version: String,
    /// `author`, `created`, `updated` (2.0.3): who wrote the script and when,
    /// dates as `YYYY-MM-DD`. Optional; shown in the Expert sheet.
    pub author: String,
    pub created: String,
    pub updated: String,
    pub admin: bool,
    pub risk: String,
    pub duration: String,
    pub reversible: bool,
    pub interruptible: bool,
    pub reboot: bool,
    pub engine: String,
    pub options: Vec<Opt>,
    pub translations: BTreeMap<String, Translation>,
    pub findings: Vec<Finding>,
    /// `scan : true` — le script sait analyser avant d'agir (§17).
    pub scan: bool,
    /// `show : expert` — l'action n'apparait qu'en Expert : ni montree ni lancee
    /// en mode Simple. Vide sinon.
    pub show: String,
    /// `view : donut` — la vue de l'analyse ; `view_expert`, celle du mode
    /// Expert quand elle differe (`view : gauge expert=table`). Vides : la
    /// liste a cocher.
    pub view: String,
    pub view_expert: String,
    /// `chart : crown` — le graphique du resume quand le lot rassemble plusieurs
    /// actions analysables (1.7). Jusqu'a la 1.6, c'etait un reglage de
    /// l'utilisateur ; c'est au script de savoir ce qui montre le mieux ce qu'il
    /// trouve. Vide : l'anneau-engrenage.
    pub chart: String,
    /// `panels : plan progress` — panneaux de l'Expert ouverts d'office.
    pub panels: Vec<String>,
    /// Le bloc REPORT, par cle.
    pub report: BTreeMap<String, ReportEntry>,
}

// ---------------------------------------------------------------------------
// Outils de lecture
// ---------------------------------------------------------------------------

/// Separe `Libelle — Description`. Sans separateur, tout est le libelle.
fn split_label(reste: &str) -> (String, String) {
    match reste.split_once(SEP_DESC) {
        Some((l, d)) => (l.trim().to_string(), d.trim().to_string()),
        None => (reste.trim().to_string(), String::new()),
    }
}

/// Le contrat d'avant la 2.0, quand l'application s'appelait WinTool : ses
/// marqueurs `## WINTOOL:START`, `## WINTOOL:OPTIONS`… et ses variables
/// `$env:WINTOOL_CONFIG`, `$env:WINTOOL_MODE`. **Lu jusqu'a la 2.1**, le temps
/// que les catalogues passent a `SINDRY` : c'est le seul endroit, avec la
/// migration (`migration.rs`), ou l'ancien nom subsiste.
const ANCIEN: &str = "WINTOOL";

/// Le texte d'un script a l'ancien contrat, ramene au nouveau : les marqueurs
/// des lignes `##` et les deux variables. Les numeros de ligne ne bougent pas,
/// les constats tombent donc au bon endroit. `None` : le script est deja au
/// contrat de Sindry.
pub fn depuis_ancien_contrat(source: &str) -> Option<String> {
    let marqueur = format!("{ANCIEN}:");
    let ancien = source
        .lines()
        .any(|l| l.trim_start().starts_with("##") && l.contains(&marqueur));
    if !ancien {
        return None;
    }
    let lignes: Vec<String> = source
        .lines()
        .map(|l| {
            let l = if l.trim_start().starts_with("##") {
                l.replace(&marqueur, "SINDRY:")
            } else {
                l.to_string()
            };
            l.replace(&format!("{ANCIEN}_CONFIG"), "SINDRY_CONFIG")
                .replace(&format!("{ANCIEN}_MODE"), "SINDRY_MODE")
        })
        .collect();
    Some(lignes.join("\n"))
}

/// Repere un bloc `## SINDRY:<tete>` ... `## SINDRY:END`.
/// Renvoie l'intervalle exclusif des lignes interieures.
fn find_block(lines: &[&str], head: &str) -> Option<(usize, usize)> {
    let mut start = None;
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim();
        if start.is_none() {
            if t.starts_with("##") && t[2..].trim() == head {
                start = Some(i);
            }
            continue;
        }
        if t.starts_with("##") && t[2..].trim() == "SINDRY:END" {
            return Some((start.unwrap() + 1, i));
        }
    }
    None
}

/// Une ligne `##<indentation><nom> : <reste>`.
/// L'indentation distingue une option (un espace) d'un choix (deux ou plus).
fn parse_decl(line: &str) -> Option<(usize, String, String)> {
    let t = line.trim_start();
    if !t.starts_with("##") {
        return None;
    }
    let apres = &t[2..];
    let indent = apres.len() - apres.trim_start().len();
    if indent == 0 {
        return None;
    }
    let (nom, reste) = apres.trim_start().split_once(':')?;
    let nom = nom.trim();
    if nom.is_empty() || nom.contains(' ') {
        return None;
    }
    Some((indent, nom.to_string(), reste.trim().to_string()))
}

fn as_bool(v: &str) -> bool {
    matches!(v.trim().to_ascii_lowercase().as_str(), "true" | "$true")
}

/// Les etiquettes posees en tete d'un libelle — `[scan]`, `[group:Junk]`,
/// `[show:expert]`, `[view:tree]` — et le texte qui suit. Meme forme que
/// `Split-Etiquettes` dans le validateur : un nom en minuscules, une valeur
/// facultative sans espace.
fn lire_etiquettes(texte: &str) -> (Vec<(String, String)>, String) {
    let mut reste = texte.trim();
    let mut etiquettes = Vec::new();
    while let Some(apres) = reste.strip_prefix('[') {
        let Some(fin) = apres.find(']') else { break };
        let dedans = &apres[..fin];
        let (nom, valeur) = dedans.split_once(':').unwrap_or((dedans, ""));
        if nom.is_empty()
            || !nom.chars().all(|c| c.is_ascii_lowercase())
            || valeur.contains(char::is_whitespace)
        {
            break;
        }
        etiquettes.push((nom.to_string(), valeur.to_string()));
        reste = apres[fin + 1..].trim_start();
    }
    (etiquettes, reste.to_string())
}

/// `created` and `updated` (2.0.3): `YYYY-MM-DD`, updated not before created,
/// neither in the future. One day of slack for the future: an author ahead
/// of UTC writes a date the CI runner has not reached yet. Same rules as
/// `Test-Dates` in `tools/lint-scripts.ps1`; only the validator asks for the
/// fields themselves (in `-Strict`).
fn check_dates(
    head: &BTreeMap<String, String>,
    today: chrono::NaiveDate,
    findings: &mut Vec<Finding>,
) {
    let mut dates = BTreeMap::new();
    for key in ["created", "updated"] {
        let Some(value) = head.get(key) else { continue };
        match chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d") {
            Ok(d) if value.len() == 10 => {
                if d > today + chrono::Duration::days(1) {
                    findings.push(constat(
                        1,
                        false,
                        "DATE_FUTURE",
                        format!("'{key}' vaut {value}, une date a venir."),
                    ));
                }
                dates.insert(key, d);
            }
            _ => findings.push(constat(
                1,
                true,
                "DATE_INVALIDE",
                format!("'{key}' vaut '{value}' ; forme attendue : AAAA-MM-JJ (2026-10-17)."),
            )),
        }
    }
    if let (Some(c), Some(u)) = (dates.get("created"), dates.get("updated")) {
        if u < c {
            findings.push(constat(
                1,
                true,
                "DATES_INVERSEES",
                format!("'updated' ({u}) precede 'created' ({c})."),
            ));
        }
    }
}

fn constat(ligne: usize, grave: bool, code: &str, message: String) -> Finding {
    Finding {
        line: ligne,
        severity: if grave {
            Severity::Error
        } else {
            Severity::Warning
        },
        code: code.into(),
        message,
    }
}

/// Lit une valeur par defaut telle qu'ecrite dans `$CONFIG`.
fn parse_default(brut: &str) -> DefaultValue {
    let v = brut.trim();

    if v.starts_with("@(") {
        let interieur = v.trim_start_matches("@(").trim_end_matches(')');
        let mut elements = Vec::new();
        for morceau in interieur.split(',') {
            let m = morceau.trim().trim_matches(['"', '\'']);
            if !m.is_empty() {
                elements.push(m.to_string());
            }
        }
        return DefaultValue::List(elements);
    }
    if v.eq_ignore_ascii_case("$true") {
        return DefaultValue::Bool(true);
    }
    if v.eq_ignore_ascii_case("$false") {
        return DefaultValue::Bool(false);
    }
    if let Ok(n) = v.parse::<f64>() {
        return DefaultValue::Number(n);
    }
    DefaultValue::Text(v.trim_matches(['"', '\'']).to_string())
}

// ---------------------------------------------------------------------------
// Lecture principale
// ---------------------------------------------------------------------------

pub fn parse(source: &str) -> Script {
    let converti = depuis_ancien_contrat(source);
    let lines: Vec<&str> = converti.as_deref().unwrap_or(source).lines().collect();
    let mut findings = Vec::new();
    if converti.is_some() {
        findings.push(Finding {
            line: 1,
            severity: Severity::Warning,
            code: "CONTRAT_ANCIEN".into(),
            message: format!(
                "Contrat d'avant Sindry 2.0 ({ANCIEN}:START, $env:{ANCIEN}_CONFIG) : lu jusqu'a la 2.1. \
                 Remplacez {ANCIEN} par SINDRY dans les marqueurs et les variables."
            ),
        });
    }

    // --- Entete -----------------------------------------------------------
    let mut head: BTreeMap<String, String> = BTreeMap::new();
    match find_block(&lines, "SINDRY:START") {
        Some((a, b)) => {
            for l in &lines[a..b] {
                let t = l.trim();
                if !t.starts_with("##") {
                    continue;
                }
                if let Some((k, v)) = t[2..].trim().split_once(':') {
                    head.entry(k.trim().to_ascii_lowercase())
                        .or_insert_with(|| v.trim().to_string());
                }
            }
        }
        None => findings.push(Finding {
            line: 1,
            severity: Severity::Error,
            code: "ENTETE_ABSENT".into(),
            message: "Aucun bloc ## SINDRY:START.".into(),
        }),
    }

    let get = |k: &str| head.get(k).cloned().unwrap_or_default();

    for requis in [
        "id",
        "lang",
        "title",
        "desc",
        "category",
        "icon",
        "version",
        "admin",
        "risk",
        "duration",
        "reversible",
        "interruptible",
        "reboot",
        "engine",
    ] {
        if !head.contains_key(requis) {
            findings.push(Finding {
                line: 1,
                severity: Severity::Error,
                code: "CHAMP_MANQUANT".into(),
                message: format!("Champ obligatoire absent de l'entete : '{requis}'."),
            });
        }
    }
    check_dates(&head, chrono::Local::now().date_naive(), &mut findings);

    // --- Options ----------------------------------------------------------
    let mut options: Vec<Opt> = Vec::new();
    match find_block(&lines, "SINDRY:OPTIONS") {
        Some((a, b)) => {
            for (decalage, l) in lines[a..b].iter().enumerate() {
                let Some((indent, nom, reste)) = parse_decl(l) else {
                    continue;
                };
                let no_ligne = a + decalage + 1;

                if indent >= 2 {
                    // Sous-ligne : un choix rattache a l'option precedente.
                    let (etiquettes, texte) = lire_etiquettes(&reste);
                    let (label, desc) = split_label(&texte);
                    let mut choix = Choice {
                        value: nom.clone(),
                        label,
                        desc,
                        ..Default::default()
                    };
                    for (e, v) in etiquettes {
                        match e.as_str() {
                            "group" => choix.group = v,
                            "show" => choix.show = v,
                            _ => findings.push(constat(
                                no_ligne,
                                true,
                                "ETIQUETTE_INCONNUE",
                                format!("Etiquette '[{e}]' inconnue sur un choix ; admises : [group] [show]."),
                            )),
                        }
                    }
                    match options.last_mut() {
                        Some(o) => o.choices.push(choix),
                        None => findings.push(Finding {
                            line: no_ligne,
                            severity: Severity::Error,
                            code: "CHOIX_ORPHELIN".into(),
                            message: format!("Le choix '{nom}' ne suit aucune option."),
                        }),
                    }
                    continue;
                }

                // Ligne d'option : `[type] Libelle — Description`
                let (kind, apres) = if reste.starts_with('[') {
                    match reste.split_once(']') {
                        Some((t, r)) => (t[1..].trim().to_string(), r.trim().to_string()),
                        None => (String::new(), reste.clone()),
                    }
                } else {
                    (String::new(), reste.clone())
                };

                if kind.is_empty() {
                    findings.push(Finding {
                        line: no_ligne,
                        severity: Severity::Error,
                        code: "TYPE_ABSENT".into(),
                        message: format!("L'option '{nom}' n'a pas de type."),
                    });
                }
                // `[items:keep-one]` : un type et sa variante.
                let (kind, variante) = match kind.split_once(':') {
                    Some((k, v)) => (k.to_string(), v.to_string()),
                    None => (kind, String::new()),
                };
                if !variante.is_empty() && !(kind == "items" && variante == "keep-one") {
                    findings.push(constat(
                        no_ligne,
                        true,
                        "TYPE_INVALIDE",
                        format!("Type '[{kind}:{variante}]' inconnu ; seule variante admise : [items:keep-one]."),
                    ));
                }

                let (etiquettes, texte) = lire_etiquettes(&apres);
                let (label, desc) = split_label(&texte);
                let mut o = Opt {
                    key: nom,
                    hidden: kind == "hidden",
                    keep_one: variante == "keep-one",
                    kind,
                    label,
                    desc,
                    ..Default::default()
                };
                for (e, v) in etiquettes {
                    match e.as_str() {
                        "scan" => o.scan = true,
                        "group" => o.group = v,
                        "show" => o.show = v,
                        "view" => o.view = v,
                        _ => findings.push(constat(
                            no_ligne,
                            true,
                            "ETIQUETTE_INCONNUE",
                            format!("Etiquette '[{e}]' inconnue ; admises : [scan] [group] [show] [view]."),
                        )),
                    }
                }
                if !o.view.is_empty() && !VUES.contains(&o.view.as_str()) {
                    findings.push(constat(
                        no_ligne,
                        true,
                        "VUE_INCONNUE",
                        format!("[view:{}] : vue inconnue.", o.view),
                    ));
                }
                options.push(o);
            }
        }
        None => findings.push(Finding {
            line: 1,
            severity: Severity::Error,
            code: "OPTIONS_ABSENT".into(),
            message: "Aucun bloc ## SINDRY:OPTIONS ; l'interface afficherait les cles brutes."
                .into(),
        }),
    }

    // --- Rapport : ce que l'analyse nomme ----------------------------------
    let mut report: BTreeMap<String, ReportEntry> = BTreeMap::new();
    if let Some((a, b)) = find_block(&lines, "SINDRY:REPORT") {
        for (decalage, l) in lines[a..b].iter().enumerate() {
            let Some((_, nom, reste)) = parse_decl(l) else {
                continue;
            };
            let no_ligne = a + decalage + 1;
            let mut entree = ReportEntry {
                kind: "label".into(),
                ..Default::default()
            };
            let texte = if let Some(r) = reste.strip_prefix("[group]") {
                entree.kind = "group".into();
                r.trim().to_string()
            } else if let Some(r) = reste.strip_prefix("[error]") {
                entree.kind = "error".into();
                if !is_error_code(&nom) {
                    findings.push(constat(
                        no_ligne,
                        true,
                        "CODE_ERREUR_FORME",
                        format!(
                            "'{nom}' : un code d'erreur s'ecrit d'une a trois majuscules, \
                             puis deux a quatre chiffres (E104, SVC12)."
                        ),
                    ));
                }
                r.trim().to_string()
            } else if let Some(r) = reste.strip_prefix("[note:") {
                entree.kind = "note".into();
                let (niveau, suite) = r.split_once(']').unwrap_or((r, ""));
                if !["info", "warn"].contains(&niveau) {
                    findings.push(constat(
                        no_ligne,
                        true,
                        "NOTE_NIVEAU",
                        format!("La note '{nom}' est de niveau '{niveau}' ; niveaux admis : info | warn."),
                    ));
                }
                entree.level = niveau.to_string();
                suite.trim().to_string()
            } else {
                reste
            };
            if entree.kind == "group" || entree.kind == "error" {
                let (l, d) = split_label(&texte);
                entree.label = l;
                entree.desc = d;
            } else {
                entree.label = texte;
            }
            report.insert(nom, entree);
        }
    }

    // Les groupes : chaque [group:X] doit avoir son libelle dans REPORT.
    for o in &options {
        let groupes = std::iter::once(&o.group).chain(o.choices.iter().map(|c| &c.group));
        for g in groupes.filter(|g| !g.is_empty()) {
            if report.get(g).map(|r| r.kind.as_str()) != Some("group") {
                findings.push(constat(
                    1,
                    true,
                    "GROUPE_INCONNU",
                    format!("[group:{g}] : le groupe n'est pas declare dans REPORT."),
                ));
            }
        }
        for show in std::iter::once(&o.show).chain(o.choices.iter().map(|c| &c.show)) {
            if !show.is_empty() && show != "simple" && show != "expert" {
                findings.push(constat(
                    1,
                    true,
                    "VALEUR_INVALIDE",
                    format!("[show:{show}] ; valeurs admises : simple | expert."),
                ));
            }
        }
    }

    // --- Traductions -------------------------------------------------------
    let mut translations: BTreeMap<String, Translation> = BTreeMap::new();
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim();
        if !t.starts_with("##") {
            continue;
        }
        let apres = t[2..].trim();
        let Some(code) = apres.strip_prefix("SINDRY:LANG ") else {
            continue;
        };
        let code = code.trim().to_ascii_lowercase();

        // Fin du bloc : le prochain SINDRY:END.
        let mut fin = lines.len();
        for (j, m) in lines.iter().enumerate().skip(i + 1) {
            let mt = m.trim();
            if mt.starts_with("##") && mt[2..].trim() == "SINDRY:END" {
                fin = j;
                break;
            }
        }

        let mut tr = Translation::default();
        let mut derniere_option = String::new();
        for l2 in &lines[i + 1..fin] {
            let Some((indent, nom, reste)) = parse_decl(l2) else {
                continue;
            };
            if indent >= 2 {
                if !derniere_option.is_empty() {
                    let (lab, de) = split_label(&reste);
                    tr.choices
                        .insert(format!("{derniere_option}/{nom}"), (lab, de));
                }
                continue;
            }
            match nom.as_str() {
                "title" => tr.title = reste,
                "desc" => tr.desc = reste,
                _ if report.contains_key(&nom) => {
                    // Un texte du bloc REPORT : seul un groupe porte un
                    // sous-titre (an error code, its solution) ; une note ou
                    // un libelle se lit en entier.
                    let (lab, de) = if matches!(report[&nom].kind.as_str(), "group" | "error") {
                        split_label(&reste)
                    } else {
                        (reste, String::new())
                    };
                    tr.report.insert(nom.clone(), (lab, de));
                    derniere_option = String::new();
                }
                _ => {
                    let (lab, de) = split_label(&reste);
                    tr.options.insert(nom.clone(), (lab, de));
                    derniere_option = nom;
                }
            }
        }
        translations.insert(code, tr);
    }

    // --- Ce que l'entete dit de l'affichage --------------------------------
    let show = head.get("show").cloned().unwrap_or_default();
    if !show.is_empty() && show != "expert" {
        findings.push(constat(
            1,
            true,
            "VALEUR_INVALIDE",
            format!("'show' vaut '{show}' ; valeur admise : expert."),
        ));
    }
    let (mut view, mut view_expert) = (String::new(), String::new());
    for morceau in head
        .get("view")
        .map(String::as_str)
        .unwrap_or("")
        .split_whitespace()
    {
        let (mode, nom) = morceau.split_once('=').unwrap_or(("", morceau));
        match mode {
            "" | "simple" => view = nom.to_string(),
            "expert" => view_expert = nom.to_string(),
            _ => {}
        }
        if !VUES.contains(&nom) || !["", "simple", "expert"].contains(&mode) {
            findings.push(constat(
                1,
                true,
                "VUE_INCONNUE",
                format!("'view' : '{morceau}' ; forme admise : '<vue>' ou '<vue> expert=<vue>'."),
            ));
        }
    }
    let chart = head.get("chart").cloned().unwrap_or_default();
    if !chart.is_empty() && !GRAPHIQUES.contains(&chart.as_str()) {
        findings.push(constat(
            1,
            true,
            "GRAPHIQUE_INCONNU",
            format!(
                "'chart' : graphique '{chart}' inconnu ; graphiques admis : {}.",
                GRAPHIQUES.join(" ")
            ),
        ));
    }
    let panels: Vec<String> = head
        .get("panels")
        .map(String::as_str)
        .unwrap_or("")
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect();
    for p in &panels {
        if !PANNEAUX.contains(&p.as_str()) {
            findings.push(constat(
                1,
                true,
                "PANNEAU_INCONNU",
                format!("'panels' : panneau '{p}' inconnu."),
            ));
        }
    }

    // --- Valeurs par defaut ------------------------------------------------
    let mut config: BTreeMap<String, (DefaultValue, usize)> = BTreeMap::new();
    if let Some(debut) = lines.iter().position(|l| {
        let t = l.trim_start();
        t.starts_with("$CONFIG") && t.contains("@{")
    }) {
        for (decalage, l) in lines[debut + 1..].iter().enumerate() {
            let t = l.trim();
            if t == "}" {
                break;
            }
            if t.is_empty() || t.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = t.split_once('=') {
                let cle = k.trim();
                if cle.is_empty() {
                    continue;
                }
                // Un commentaire de fin de ligne n'appartient pas a la valeur.
                let valeur = v
                    .split('#')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_end_matches(',');
                config.insert(
                    cle.to_string(),
                    (parse_default(valeur), debut + decalage + 2),
                );
            }
        }
    } else {
        findings.push(Finding {
            line: 1,
            severity: Severity::Error,
            code: "CONFIG_ABSENT".into(),
            message: "Aucun bloc $CONFIG = @{ }.".into(),
        });
    }

    // --- Croisement OPTIONS / $CONFIG, dans les deux sens ------------------
    for o in options.iter_mut() {
        match config.get(&o.key) {
            Some((v, _)) => o.default = Some(v.clone()),
            None => findings.push(Finding {
                line: 1,
                severity: Severity::Error,
                code: "OPTION_ORPHELINE".into(),
                message: format!(
                    "'{}' est declaree dans OPTIONS mais absente de $CONFIG.",
                    o.key
                ),
            }),
        }
    }
    for (cle, (_, ligne)) in &config {
        if !options.iter().any(|o| &o.key == cle) {
            findings.push(Finding {
                line: *ligne,
                severity: Severity::Error,
                code: "OPTION_NON_DECLAREE".into(),
                message: format!("'{cle}' est dans $CONFIG mais absente du bloc OPTIONS."),
            });
        }
    }

    // --- Ligne d'override --------------------------------------------------
    if !lines.iter().any(|l| l.contains("SINDRY_CONFIG")) {
        findings.push(Finding {
            line: 1,
            severity: Severity::Error,
            code: "OVERRIDE_ABSENT".into(),
            message: "Ligne d'override absente : les reglages de l'interface seraient ignores."
                .into(),
        });
    }

    Script {
        id: get("id"),
        lang: if head.contains_key("lang") {
            get("lang")
        } else {
            "en".into()
        },
        title: get("title"),
        desc: get("desc"),
        category: get("category"),
        icon: get("icon"),
        tags: get("tags")
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        version: get("version"),
        author: get("author"),
        created: get("created"),
        updated: get("updated"),
        admin: as_bool(&get("admin")),
        risk: get("risk"),
        duration: get("duration"),
        reversible: as_bool(&get("reversible")),
        interruptible: as_bool(&get("interruptible")),
        reboot: as_bool(&get("reboot")),
        engine: if head.contains_key("engine") {
            get("engine")
        } else {
            "auto".into()
        },
        options,
        translations,
        findings,
        scan: as_bool(&get("scan")),
        show,
        view,
        view_expert,
        chart,
        panels,
        report,
    }
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const EXEMPLE: &str = r#"
## SINDRY:START
## id            : 985863a2-4150-4790-90b5-fdb31feee22b
## lang          : en
## title         : Set fast DNS
## desc          : Speeds up browsing
## category      : performance
## icon          : zap
## tags          : dns, network
## version       : 2.0
## admin         : true
## risk          : medium
## duration      : fast
## reversible    : true
## interruptible : true
## reboot        : false
## engine        : auto
## SINDRY:END

## SINDRY:OPTIONS
## DnsProvider  : [select] DNS provider — the resolver service
##   cloudflare : Cloudflare — 1.1.1.1
##   quad9      : Quad9 — 9.9.9.9
## CleanTargets : [multi]  What to clean — pick one or more
##   temp       : Temporary files
##   cache      : Browser caches
## FlushCache   : [hidden] Flush the cache afterwards
## SINDRY:END

## SINDRY:LANG fr
## title        : Internet plus rapide
## desc         : Accelere la navigation
## DnsProvider  : Fournisseur DNS — le service de resolution
##   cloudflare : Cloudflare — 1.1.1.1
## SINDRY:END

$CONFIG = @{
    DnsProvider  = "cloudflare"
    CleanTargets = @("temp", "cache")
    FlushCache   = $true
}

if ($env:SINDRY_CONFIG) { }
"#;

    #[test]
    fn lit_l_entete() {
        let s = parse(EXEMPLE);
        assert_eq!(s.id, "985863a2-4150-4790-90b5-fdb31feee22b");
        assert_eq!(s.title, "Set fast DNS");
        assert_eq!(s.risk, "medium");
        assert!(s.admin && s.reversible && s.interruptible && !s.reboot);
        assert_eq!(s.tags, vec!["dns", "network"]);
    }

    #[test]
    fn lit_les_options_et_leurs_choix() {
        let s = parse(EXEMPLE);
        assert_eq!(s.options.len(), 3);

        let dns = &s.options[0];
        assert_eq!(dns.kind, "select");
        assert_eq!(dns.label, "DNS provider");
        assert_eq!(dns.desc, "the resolver service");
        assert_eq!(dns.choices.len(), 2);
        assert_eq!(dns.choices[0].value, "cloudflare");
        assert_eq!(dns.choices[0].desc, "1.1.1.1");

        // Les choix se rattachent bien a leur propre option, pas a la precedente.
        assert_eq!(s.options[1].choices.len(), 2);
        assert!(s.options[2].hidden);
    }

    #[test]
    fn associe_les_valeurs_par_defaut() {
        let s = parse(EXEMPLE);
        assert!(matches!(&s.options[0].default, Some(DefaultValue::Text(t)) if t == "cloudflare"));
        assert!(matches!(&s.options[1].default, Some(DefaultValue::List(v)) if v.len() == 2));
        assert!(matches!(
            s.options[2].default,
            Some(DefaultValue::Bool(true))
        ));
    }

    #[test]
    fn lit_la_traduction() {
        let s = parse(EXEMPLE);
        let fr = s.translations.get("fr").expect("bloc fr attendu");
        assert_eq!(fr.title, "Internet plus rapide");
        assert_eq!(fr.options["DnsProvider"].0, "Fournisseur DNS");
        assert_eq!(fr.choices["DnsProvider/cloudflare"].0, "Cloudflare");
    }

    #[test]
    fn un_script_conforme_ne_produit_aucune_anomalie() {
        let s = parse(EXEMPLE);
        assert!(
            s.findings.is_empty(),
            "anomalies inattendues : {:?}",
            s.findings
        );
    }

    /// Fait passer les vrais scripts livres dans le parseur.
    ///
    /// C'est ce test qui detectera une divergence entre `tools/lint-scripts.ps1`
    /// et ce module : les deux lisent le meme format et doivent rester d'accord.
    /// Un echec ici signifie que l'un des deux a evolue sans l'autre.
    /// Le squelette de `docs/FORMAT_SCRIPT.md` doit se lire sans une seule
    /// anomalie.
    ///
    /// Ce test lit le document lui-meme, pas une copie : c'est ce qui empeche
    /// l'exemple de reference de diverger du parseur. Une convention que rien
    /// ne verifie finit toujours morte — c'est ce qui a tue la v3, ou 0 script
    /// sur 13 respectait une convention pourtant documentee.
    ///
    /// Il a remplace un test qui lisait `scripts/Default/`. Ce dossier peut
    /// etre vide (catalogue en cours de reecriture), et un test qui depend de
    /// son contenu echoue alors pour une raison qui n'a rien a voir avec le
    /// parseur.
    #[test]
    fn le_squelette_de_la_documentation_se_lit_sans_anomalie() {
        let doc = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("docs")
            .join("FORMAT_SCRIPT.md");
        let texte = std::fs::read_to_string(&doc)
            .unwrap_or_else(|e| panic!("{} illisible : {e}", doc.display()));

        // Premier bloc ```powershell qui contient l'entete : c'est le squelette.
        let squelette = texte
            .split("```powershell")
            .skip(1)
            .filter_map(|bloc| bloc.split("```").next())
            .find(|bloc| bloc.contains("SINDRY:START"))
            .expect("aucun squelette dans docs/FORMAT_SCRIPT.md");

        let s = parse(squelette);

        let erreurs: Vec<_> = s
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .collect();
        assert!(
            erreurs.is_empty(),
            "le squelette documente porte des anomalies : {erreurs:?}"
        );

        assert!(!s.id.is_empty(), "squelette sans id");
        assert!(!s.title.is_empty(), "squelette sans titre");
        assert!(!s.options.is_empty(), "squelette sans option");
        assert!(
            s.translations.contains_key("fr"),
            "squelette sans bloc de traduction fr"
        );

        // Chaque option doit avoir recu sa valeur par defaut depuis $CONFIG.
        for o in &s.options {
            assert!(o.default.is_some(), "'{}' sans valeur par defaut", o.key);
        }

        // Le mode test refuse un script qui ne declare pas cette option : le
        // modele que tout le monde copie doit donc la porter (§6.9).
        assert!(
            s.options.iter().any(|o| o.key == "SafeTest"),
            "le squelette ne declare pas SafeTest : un script copie dessus serait refuse en mode test"
        );
    }

    /// Tous les scripts complets de la documentation, pas seulement le premier :
    /// l'exemple du mode analyse est un modele que l'agent catalogue recopie, il
    /// doit rester lisible par le parseur au meme titre que le squelette.
    #[test]
    fn chaque_exemple_de_la_documentation_se_lit_sans_anomalie() {
        let doc = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("docs")
            .join("FORMAT_SCRIPT.md");
        let texte = std::fs::read_to_string(&doc)
            .unwrap_or_else(|e| panic!("{} illisible : {e}", doc.display()));

        let exemples: Vec<&str> = texte
            .split("```powershell")
            .skip(1)
            .filter_map(|bloc| bloc.split("```").next())
            .filter(|bloc| bloc.contains("SINDRY:START"))
            .collect();

        // Le squelette et l'exemple d'analyse. Si l'un disparait, ce test doit le
        // dire plutot que de passer en silence sur ce qui reste.
        assert!(
            exemples.len() >= 2,
            "{} exemple(s) complet(s) seulement dans FORMAT_SCRIPT.md",
            exemples.len()
        );
        assert!(
            exemples.iter().any(|e| e.contains("[FIND]")),
            "plus aucun exemple de mode analyse dans FORMAT_SCRIPT.md"
        );

        for exemple in exemples {
            let s = parse(exemple);
            let erreurs: Vec<_> = s
                .findings
                .iter()
                .filter(|f| f.severity == Severity::Error)
                .collect();
            assert!(
                erreurs.is_empty(),
                "l'exemple '{}' porte des anomalies : {erreurs:?}",
                s.title
            );
        }
    }

    /// Le script d'exemple de la norme d'analyse (docs/mockups/exemple-analyse.ps1) :
    /// tout ce que la 1.4 ajoute a l'entete, lu par le parseur.
    #[test]
    fn lit_la_norme_d_analyse() {
        let chemin = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("docs")
            .join("mockups")
            .join("exemple-analyse.ps1");
        let brut = std::fs::read_to_string(&chemin).expect("exemple illisible");
        let s = parse(brut.trim_start_matches('\u{feff}'));
        assert!(s.findings.is_empty(), "anomalies : {:?}", s.findings);
        assert!(s.scan);
        assert_eq!((s.view.as_str(), s.view_expert.as_str()), ("donut", ""));
        assert_eq!(s.panels, vec!["plan", "progress"]);

        let cibles = s.options.iter().find(|o| o.key == "Targets").unwrap();
        // Les etiquettes quittent le libelle : c'est le libelle nu qui s'affiche.
        assert_eq!(cibles.choices[1].label, "Windows temporary files");
        assert_eq!(cibles.choices[1].group, "Junk");
        assert_eq!(cibles.choices[1].show, "expert");
        assert_eq!(cibles.choices[2].group, "OldUpdates");
        let caches = s.options.iter().find(|o| o.key == "Browsers").unwrap();
        assert_eq!(
            (caches.kind.as_str(), caches.view.as_str()),
            ("items", "tree")
        );
        assert_eq!(caches.label, "Browser caches");
        assert!(matches!(&caches.default, Some(DefaultValue::List(v)) if v.is_empty()));

        assert_eq!(s.report["Junk"].kind, "group");
        assert_eq!(s.report["Junk"].desc, "Temporary files and the recycle bin");
        assert_eq!(
            (
                s.report["UpdateNote"].kind.as_str(),
                s.report["UpdateNote"].level.as_str()
            ),
            ("note", "warn")
        );
        assert_eq!(s.report["Cache"].kind, "label");

        let fr = &s.translations["fr"];
        assert_eq!(fr.report["Junk"].0, "Fichiers inutiles");
        assert_eq!(fr.report["Cache"].0, "Fichiers en cache");
        assert!(
            !fr.options.contains_key("Junk"),
            "un texte de REPORT pris pour une option"
        );
    }

    /// Le guide reproduit l'exemple de la norme d'analyse : les deux ne doivent
    /// jamais diverger. C'est le fichier que l'on donne a lire, et le guide que
    /// l'agent catalogue recopie.
    #[test]
    fn le_guide_reproduit_l_exemple_d_analyse() {
        let racine = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let lire = |p: std::path::PathBuf| {
            std::fs::read_to_string(&p)
                .unwrap_or_else(|e| panic!("{} illisible : {e}", p.display()))
                .trim_start_matches('\u{feff}')
                .replace("\r\n", "\n")
        };
        let exemple = lire(
            racine
                .join("docs")
                .join("mockups")
                .join("exemple-analyse.ps1"),
        );
        let guide = lire(racine.join("docs").join("FORMAT_SCRIPT.md"));
        assert!(
            guide.contains(exemple.trim_end()),
            "docs/FORMAT_SCRIPT.md ne reproduit plus docs/mockups/exemple-analyse.ps1 à l'identique"
        );
    }

    #[test]
    fn signale_une_etiquette_ou_un_groupe_inconnu() {
        let src = EXEMPLE.replace(
            "##   temp       : Temporary files",
            "##   temp       : [group:Nulle] [colour:red] Temporary files",
        );
        let s = parse(&src);
        let codes: Vec<&str> = s.findings.iter().map(|f| f.code.as_str()).collect();
        assert!(codes.contains(&"ETIQUETTE_INCONNUE"), "{codes:?}");
        assert!(codes.contains(&"GROUPE_INCONNU"), "{codes:?}");
    }

    #[test]
    fn l_ancien_contrat_se_lit_encore_avec_un_constat() {
        // Un script ecrit pour WinTool (avant la 2.0) : memes informations,
        // un constat qui demande la conversion, aucune autre anomalie.
        let ancien = EXEMPLE
            .replace("SINDRY:", "WINTOOL:")
            .replace("SINDRY_CONFIG", "WINTOOL_CONFIG");
        assert!(ancien.contains("## WINTOOL:START"));
        let s = parse(&ancien);
        assert_eq!(s.title, "Set fast DNS");
        assert_eq!(s.options.len(), parse(EXEMPLE).options.len());
        let codes: Vec<&str> = s.findings.iter().map(|f| f.code.as_str()).collect();
        assert_eq!(codes, vec!["CONTRAT_ANCIEN"], "{codes:?}");
        assert!(parse(EXEMPLE)
            .findings
            .iter()
            .all(|f| f.code != "CONTRAT_ANCIEN"));
    }

    #[test]
    fn lit_le_graphique_du_resume() {
        // 1.7 : le graphique du resume se declare dans l'entete.
        let src = EXEMPLE.replace(
            "## engine        : auto",
            "## engine        : auto
## chart         : crown",
        );
        let s = parse(&src);
        assert_eq!(s.chart, "crown");
        assert!(!s.findings.iter().any(|f| f.code == "GRAPHIQUE_INCONNU"));
        assert_eq!(
            parse(EXEMPLE).chart,
            "",
            "absent : l'anneau-engrenage, choisi par l'interface"
        );

        let inconnu = parse(&EXEMPLE.replace(
            "## engine        : auto",
            "## engine        : auto
## chart         : camembert",
        ));
        assert!(inconnu
            .findings
            .iter()
            .any(|f| f.code == "GRAPHIQUE_INCONNU"));
    }

    #[test]
    fn constate_sans_bloquer() {
        // Cle presente dans $CONFIG mais absente du bloc OPTIONS.
        let casse = EXEMPLE.replace(
            "## FlushCache   : [hidden] Flush the cache afterwards\n",
            "",
        );
        let s = parse(&casse);
        assert!(s.findings.iter().any(|f| f.code == "OPTION_NON_DECLAREE"));
        // Le script reste lisible malgre l'anomalie.
        assert_eq!(s.title, "Set fast DNS");
    }

    #[test]
    fn error_codes_carry_a_translated_description_and_solution() {
        let source = EXEMPLE
            .replace(
                "## SINDRY:LANG fr\n",
                "## SINDRY:REPORT\n\
                 ## E104 : [error] Access denied on a protected key — Restart the PC, then run the action again\n\
                 ## SVC12 : [error] The service did not stop\n\
                 ## e5 : [error] Badly named\n\
                 ## SINDRY:END\n\n\
                 ## SINDRY:LANG fr\n\
                 ## E104 : Accès refusé sur une clé protégée — Redémarrez le PC, puis relancez l'action\n",
            );
        let s = parse(&source);
        let e = &s.report["E104"];
        assert_eq!(e.kind, "error");
        assert_eq!(e.label, "Access denied on a protected key");
        assert_eq!(e.desc, "Restart the PC, then run the action again");
        assert_eq!(s.report["SVC12"].desc, "", "the solution is optional");
        assert_eq!(
            s.translations["fr"].report["E104"],
            (
                "Accès refusé sur une clé protégée".to_string(),
                "Redémarrez le PC, puis relancez l'action".to_string()
            )
        );
        let bad: Vec<_> = s
            .findings
            .iter()
            .filter(|f| f.code == "CODE_ERREUR_FORME")
            .collect();
        assert_eq!(bad.len(), 1, "{:?}", s.findings);
        assert!(bad[0].message.contains("'e5'"));

        for ok in ["E104", "SVC12", "W01", "ABC1234"] {
            assert!(is_error_code(ok), "{ok}");
        }
        for ko in ["E1", "E12345", "ABCD12", "e104", "104", "E10A", ""] {
            assert!(!is_error_code(ko), "{ko}");
        }
    }

    #[test]
    fn author_and_dates_are_read_and_checked() {
        let with = |lines: &str| {
            EXEMPLE.replace(
                "## engine        : auto\n",
                &format!("## engine        : auto\n{lines}"),
            )
        };
        let s = parse(&with(
            "## author        : Buly\n## created       : 2026-10-01\n## updated       : 2026-10-17\n",
        ));
        assert_eq!(
            (s.author.as_str(), s.created.as_str(), s.updated.as_str()),
            ("Buly", "2026-10-01", "2026-10-17")
        );
        assert!(
            !s.findings.iter().any(|f| f.code.starts_with("DATE")),
            "{:?}",
            s.findings
        );

        let codes = |lines: &str, today: &str| {
            let mut head = BTreeMap::new();
            for l in lines.lines() {
                let (k, v) = l.split_once('=').unwrap();
                head.insert(k.to_string(), v.to_string());
            }
            let mut f = Vec::new();
            let today = chrono::NaiveDate::parse_from_str(today, "%Y-%m-%d").unwrap();
            check_dates(&head, today, &mut f);
            f.into_iter().map(|f| f.code).collect::<Vec<_>>()
        };
        assert_eq!(codes("created=17/10/2026", "2026-10-17"), ["DATE_INVALIDE"]);
        assert_eq!(codes("created=2026-10-32", "2026-10-17"), ["DATE_INVALIDE"]);
        assert_eq!(codes("created=2026-1-7", "2026-10-17"), ["DATE_INVALIDE"]);
        assert_eq!(
            codes("created=2026-10-17\nupdated=2026-10-01", "2026-10-17"),
            ["DATES_INVERSEES"]
        );
        // One day ahead is an author east of UTC, not a mistake.
        assert!(codes("updated=2026-10-18", "2026-10-17").is_empty());
        assert_eq!(codes("updated=2026-10-19", "2026-10-17"), ["DATE_FUTURE"]);
        // Missing fields are the validator's business (-Strict), not the app's.
        assert!(codes("", "2026-10-17").is_empty());
    }
}
