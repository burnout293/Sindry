//! Ce qu'une analyse rapporte (specification §17, docs/FORMAT_SCRIPT.md).
//!
//! Un script analysable est lance avec `SINDRY_MODE=scan` : il mesure, et decrit
//! ce qu'il a trouve sur sa sortie, une ligne par constat — `[FIND]`, `[ITEM]`,
//! `[METRIC]`, `[NOTE]`. Ce module lit ces lignes **contre l'entete du script** :
//!
//! - une ligne qui vise une option, un choix, une note ou un libelle que le script
//!   ne declare pas est **ignoree et relevee** — un script ne parle que de ses
//!   propres cases, jamais au nom d'un autre ;
//! - une mesure illisible (texte a la place d'un nombre, jeton inconnu) est
//!   ignoree et relevee, jamais interpretee au mieux ;
//! - les identifiants des elements trouves sont retenus ([`Memoire`]) : quand
//!   l'interface renverra une selection au script, `run_script` refusera tout id
//!   que cette analyse n'a pas annonce.
//!
//! Le script n'ecrit jamais de phrase : des nombres, des noms, des jetons. Les
//! mots viennent de son entete, deja traduite ; c'est l'interface qui compose.

use crate::contract::Script;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Mutex;

/// Au-dela, la sortie n'est plus lue : une analyse qui ecrit davantage se trompe
/// d'outil, et l'interface ne saurait pas l'afficher.
pub const MAX_LIGNES: usize = 20_000;
/// Elements retenus par liste `[items]`.
pub const MAX_ELEMENTS: usize = 5_000;
/// Longueur maximale d'un texte (nom, chemin, editeur).
const MAX_TEXTE: usize = 400;

/// Marqueurs qu'une analyse peut ecrire et que ce module lit.
pub const MARQUEURS_ANALYSE: [&str; 5] = ["FIND", "ITEM", "METRIC", "NOTE", "STEP"];

#[derive(Debug, Clone, Serialize, Default)]
pub struct Constat {
    pub option: String,
    /// Vide pour une option `[bool]`.
    pub choice: String,
    pub fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Element {
    pub option: String,
    /// Toujours present : `fields["id"]`.
    pub fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Mesure {
    pub key: String,
    pub fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct NoteLue {
    pub key: String,
    /// `Option` ou `Option.choix` sous lequel l'afficher ; vide : en tete.
    pub target: String,
    pub show: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Analyse {
    pub finds: Vec<Constat>,
    pub items: Vec<Element>,
    pub metrics: Vec<Mesure>,
    pub notes: Vec<NoteLue>,
    /// Le texte de chaque `[STEP]` (en anglais : l'Expert seul l'affiche).
    pub steps: Vec<String>,
    /// Ce qui a ete ignore, et pourquoi — affiche en mode Expert.
    pub anomalies: Vec<String>,
    /// La sortie depassait [`MAX_LIGNES`] : la fin n'a pas ete lue.
    pub truncated: bool,
}

impl Analyse {
    /// Les ids annonces, par option `[items]`.
    pub fn ids(&self) -> BTreeMap<String, BTreeSet<String>> {
        let mut ids: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for e in &self.items {
            if let Some(id) = e.fields.get("id") {
                ids.entry(e.option.clone()).or_default().insert(id.clone());
            }
        }
        ids
    }
}

// ---------------------------------------------------------------------------
// Lecture d'une ligne
// ---------------------------------------------------------------------------

/// Une ligne `[ITEM] Opt id=a name="B C" size=3` apres son marqueur : les mots
/// nus d'abord (la cle, la cible d'une note), puis les champs `nom=valeur`,
/// valeur entre guillemets si elle contient des espaces.
fn decouper(reste: &str) -> (Vec<String>, Vec<(String, String)>) {
    let mut nus = Vec::new();
    let mut champs = Vec::new();
    let c: Vec<char> = reste.chars().collect();
    let mut i = 0;
    while i < c.len() {
        while i < c.len() && c[i].is_whitespace() {
            i += 1;
        }
        if i >= c.len() {
            break;
        }
        let debut = i;
        while i < c.len() && !c[i].is_whitespace() && c[i] != '=' {
            i += 1;
        }
        let mot: String = c[debut..i].iter().collect();
        if i < c.len()
            && c[i] == '='
            && !mot.is_empty()
            && mot.chars().all(|x| x.is_ascii_alphabetic())
        {
            i += 1;
            let valeur: String = if i < c.len() && c[i] == '"' {
                i += 1;
                let d = i;
                while i < c.len() && c[i] != '"' {
                    i += 1;
                }
                let v = c[d..i].iter().collect();
                i += 1;
                v
            } else {
                let d = i;
                while i < c.len() && !c[i].is_whitespace() {
                    i += 1;
                }
                c[d..i].iter().collect()
            };
            champs.push((mot, valeur));
        } else {
            // Un mot nu, ou un `=` qui ne suit pas un nom de champ : tout le
            // morceau jusqu'a l'espace suivant.
            while i < c.len() && !c[i].is_whitespace() {
                i += 1;
            }
            nus.push(c[debut..i].iter().collect());
        }
    }
    (nus, champs)
}

fn est_nombre(v: &str) -> bool {
    !v.is_empty() && v.parse::<f64>().is_ok_and(|n| n.is_finite() && n >= 0.0)
}

fn est_jeton(v: &str, max: usize) -> bool {
    !v.is_empty()
        && v.len() <= max
        && v.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':' | '@' | '+'))
}

fn est_date(v: &str) -> bool {
    let b = v.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

/// Les jetons admis pour un champ, ou `None` si le champ n'en limite pas la
/// valeur. Doit rester d'accord avec `$VALEURS_CHAMP` dans le validateur.
fn jetons(champ: &str) -> Option<&'static [&'static str]> {
    Some(match champ {
        "state" => &["todo", "ok"],
        "checked" | "current" | "recommended" | "keep" => &["true", "false"],
        "show" => &["simple", "expert"],
        "kind" => &[
            "folder", "file", "registry", "app", "startup", "service", "task", "driver", "browser",
        ],
        "confidence" | "impact" | "risk" => &["high", "medium", "low"],
        "locked" => &["open", "system", "protected", "inuse"],
        "unit" => &["celsius", "pct", "hours", "days", "s", "count", "cycles"],
        "health" => &["ok", "warn", "crit"],
        _ => return None,
    })
}

const CHAMPS_FIND: [&str; 8] = [
    "size",
    "count",
    "state",
    "checked",
    "ms",
    "current",
    "recommended",
    "show",
];
const CHAMPS_ITEM: [&str; 21] = [
    "id",
    "parent",
    "group",
    "name",
    "path",
    "publisher",
    "label",
    "kind",
    "confidence",
    "impact",
    "risk",
    "locked",
    "version",
    "to",
    "date",
    "keep",
    "size",
    "count",
    "state",
    "checked",
    "show",
];
const CHAMPS_METRIC: [&str; 6] = ["value", "unit", "health", "max", "group", "show"];
const NOMBRES: [&str; 4] = ["size", "count", "ms", "max"];

/// Garde les champs connus et lisibles ; releve les autres.
fn filtrer(
    marque: &str,
    cle: &str,
    champs: Vec<(String, String)>,
    admis: &[&str],
    anomalies: &mut Vec<String>,
) -> BTreeMap<String, String> {
    let mut garde = BTreeMap::new();
    for (nom, valeur) in champs {
        let refus = if !admis.contains(&nom.as_str()) {
            Some(format!("champ « {nom} » inconnu"))
        } else if NOMBRES.contains(&nom.as_str()) && !est_nombre(&valeur) {
            Some(format!("{nom}={valeur} n’est pas un nombre"))
        } else if let Some(j) = jetons(&nom) {
            (!j.contains(&valeur.as_str()))
                .then(|| format!("{nom}={valeur} n’est pas une valeur admise"))
        } else if nom == "date" && !est_date(&valeur) {
            Some(format!("date={valeur} n’est pas une date AAAA-MM-JJ"))
        } else if (nom == "id" || nom == "parent" || nom == "group") && !est_jeton(&valeur, 128) {
            Some(format!("{nom}={valeur} n’est pas un identifiant"))
        } else if nom == "value" && !est_nombre(&valeur) && !est_jeton(&valeur, 32) {
            Some(format!("value={valeur} n’est ni un nombre ni un jeton"))
        } else if valeur.chars().count() > MAX_TEXTE || valeur.chars().any(char::is_control) {
            Some(format!("{nom} trop long ou illisible"))
        } else {
            None
        };
        match refus {
            Some(r) => anomalies.push(format!("[{marque}] {cle} : {r} — ignoré")),
            None => {
                garde.insert(nom, valeur);
            }
        }
    }
    garde
}

/// Lit la sortie d'une analyse contre l'entete du script.
pub fn lire(meta: &Script, lignes: &[String]) -> Analyse {
    let mut a = Analyse::default();
    let option = |cle: &str| meta.options.iter().find(|o| o.key == cle);
    let rapport = |cle: &str, genre: &str| meta.report.get(cle).is_some_and(|r| r.kind == genre);
    let mut vus: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut cibles_vues: BTreeSet<String> = BTreeSet::new();

    for (n, brute) in lignes.iter().enumerate() {
        if n >= MAX_LIGNES {
            a.truncated = true;
            a.anomalies.push(format!(
                "Plus de {MAX_LIGNES} lignes : la fin de l’analyse n’a pas été lue."
            ));
            break;
        }
        let t = brute.trim();
        let Some(apres) = t.strip_prefix('[') else {
            continue;
        };
        let Some(fin) = apres.find(']') else {
            continue;
        };
        let marque = &apres[..fin];
        if !MARQUEURS_ANALYSE.contains(&marque) {
            continue;
        }
        let reste = apres[fin + 1..].trim();
        if marque == "STEP" {
            // « 2/4 Measuring… » : le rang n'interesse que la progression, que
            // le moteur lit deja ; on garde le texte.
            let texte = match reste.split_once(char::is_whitespace) {
                Some((rang, texte)) if rang.contains('/') => texte.trim(),
                _ => reste,
            };
            a.steps.push(texte.chars().take(MAX_TEXTE).collect());
            continue;
        }

        let (nus, champs) = decouper(reste);
        let cle = nus.first().cloned().unwrap_or_default();
        if cle.is_empty() {
            a.anomalies.push(format!("[{marque}] sans clé — ignoré"));
            continue;
        }

        match marque {
            "FIND" => {
                let (nom, choix) = cle.split_once('.').unwrap_or((cle.as_str(), ""));
                let Some(o) = option(nom) else {
                    a.anomalies.push(format!(
                        "[FIND] vise « {nom} », que le script ne déclare pas — ignoré"
                    ));
                    continue;
                };
                let correct = match o.kind.as_str() {
                    "bool" => choix.is_empty(),
                    "multi" | "select" => o.choices.iter().any(|c| c.value == choix),
                    _ => false,
                };
                if !correct {
                    a.anomalies.push(format!(
                        "[FIND] {cle} : ne désigne pas une case de « {nom} » ([{}]) — ignoré",
                        o.kind
                    ));
                    continue;
                }
                let fields = filtrer("FIND", &cle, champs, &CHAMPS_FIND, &mut a.anomalies);
                // Une seconde ligne sur la meme case la complete : un script ne
                // sait qu'a la fin lequel de ses choix recommander, une fois
                // tous mesures. Sur un meme champ, la derniere valeur l'emporte.
                if !cibles_vues.insert(cle.clone()) {
                    if let Some(c) = a
                        .finds
                        .iter_mut()
                        .find(|c| c.option == nom && c.choice == choix)
                    {
                        c.fields.extend(fields);
                    }
                    continue;
                }
                a.finds.push(Constat {
                    option: nom.to_string(),
                    choice: choix.to_string(),
                    fields,
                });
            }
            "ITEM" => {
                match option(&cle) {
                    Some(o) if o.kind == "items" => {}
                    _ => {
                        a.anomalies
                            .push(format!("[ITEM] vise « {cle} », qui n’est pas une liste [items] déclarée — ignoré"));
                        continue;
                    }
                }
                let fields = filtrer("ITEM", &cle, champs, &CHAMPS_ITEM, &mut a.anomalies);
                let Some(id) = fields.get("id").cloned() else {
                    a.anomalies
                        .push(format!("[ITEM] {cle} sans id lisible — ignoré"));
                    continue;
                };
                let deja = vus.entry(cle.clone()).or_default();
                if deja.len() >= MAX_ELEMENTS {
                    if deja.len() == MAX_ELEMENTS {
                        a.anomalies
                            .push(format!("[ITEM] {cle} : plus de {MAX_ELEMENTS} éléments, les suivants sont ignorés"));
                        deja.insert(String::new());
                    }
                    continue;
                }
                if !deja.insert(id.clone()) {
                    a.anomalies
                        .push(format!("[ITEM] {cle} : id={id} déjà annoncé — ignoré"));
                    continue;
                }
                if let Some(l) = fields.get("label") {
                    if !rapport(l, "label") {
                        a.anomalies.push(format!(
                            "[ITEM] {cle} : label={l} absent du bloc REPORT — affiché tel quel"
                        ));
                    }
                }
                a.items.push(Element {
                    option: cle,
                    fields,
                });
            }
            "METRIC" => {
                if !rapport(&cle, "label") {
                    a.anomalies.push(format!(
                        "[METRIC] « {cle} » n’est pas un libellé du bloc REPORT — ignoré"
                    ));
                    continue;
                }
                let fields = filtrer("METRIC", &cle, champs, &CHAMPS_METRIC, &mut a.anomalies);
                if !fields.contains_key("value") {
                    a.anomalies
                        .push(format!("[METRIC] {cle} sans value lisible — ignoré"));
                    continue;
                }
                a.metrics.push(Mesure { key: cle, fields });
            }
            "NOTE" => {
                if !rapport(&cle, "note") {
                    a.anomalies.push(format!(
                        "[NOTE] « {cle} » n’est pas une note du bloc REPORT — ignorée"
                    ));
                    continue;
                }
                let target = nus.get(1).cloned().unwrap_or_default();
                if !target.is_empty() {
                    let (nom, choix) = target.split_once('.').unwrap_or((target.as_str(), ""));
                    let existe = option(nom).is_some_and(|o| {
                        choix.is_empty() || o.choices.iter().any(|c| c.value == choix)
                    });
                    if !existe {
                        a.anomalies
                            .push(format!("[NOTE] {cle} vise « {target} », que le script ne déclare pas — ignorée"));
                        continue;
                    }
                }
                let fields = filtrer("NOTE", &cle, champs, &["show"], &mut a.anomalies);
                a.notes.push(NoteLue {
                    key: cle,
                    target,
                    show: fields.get("show").cloned().unwrap_or_default(),
                });
            }
            _ => {}
        }
    }

    // Un parent introuvable n'efface pas l'element : il s'affiche a la racine.
    let ids = a.ids();
    for e in &a.items {
        if let Some(p) = e.fields.get("parent") {
            if !ids.get(&e.option).is_some_and(|s| s.contains(p)) {
                a.anomalies.push(format!(
                    "[ITEM] {} : id={} vise le parent « {p} », jamais annoncé — affiché à la racine",
                    e.option,
                    e.fields.get("id").map(String::as_str).unwrap_or("")
                ));
            }
        }
    }
    a
}

// ---------------------------------------------------------------------------
// Ce qu'on retient d'une analyse
// ---------------------------------------------------------------------------

struct Memo {
    hash: String,
    ids: BTreeMap<String, BTreeSet<String>>,
}

/// Les ids annonces par la derniere analyse de chaque script, avec l'empreinte du
/// fichier analyse. En memoire seulement : une analyse ne vaut que pour la
/// session qui l'a vue.
#[derive(Default)]
pub struct Memoire {
    par_script: Mutex<HashMap<String, Memo>>,
}

impl Memoire {
    pub fn retenir(&self, script_id: &str, hash: &str, analyse: &Analyse) {
        if let Ok(mut m) = self.par_script.lock() {
            m.insert(
                script_id.to_string(),
                Memo {
                    hash: hash.to_string(),
                    ids: analyse.ids(),
                },
            );
        }
    }

    /// Verifie qu'une selection `[items]` ne renvoie au script que des ids que sa
    /// derniere analyse, sur ce contenu exact du fichier, a annonces.
    pub fn verifier(
        &self,
        script_id: &str,
        hash: &str,
        option: &str,
        ids: &[String],
    ) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        let m = self
            .par_script
            .lock()
            .map_err(|_| "etat interne corrompu")?;
        let Some(memo) = m.get(script_id).filter(|x| x.hash == hash) else {
            return Err(format!(
                "ANALYSE_PERIMEE:{option} — aucune analyse de ce script, sur ce contenu, n’a annoncé ces éléments. Relancez l’analyse."
            ));
        };
        let connus = memo.ids.get(option);
        if let Some(inconnu) = ids
            .iter()
            .find(|id| !connus.is_some_and(|c| c.contains(*id)))
        {
            return Err(format!(
                "ANALYSE_PERIMEE:{option} — « {inconnu} » n’a pas été annoncé par la dernière analyse."
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Ce que Sindry coche d'office (2.0, ligne de commande)
// ---------------------------------------------------------------------------

/// La selection d'office d'une analyse : ce que l'interface coche avant que
/// l'utilisateur ne touche a rien, et ce qui repartira vers le script.
///
/// **Le meme calcul que `selection`, `configAnalyse` et `aFaire` dans
/// `src/analyse.js`** — la ligne de commande lance ce que le mode Simple
/// lancerait sans qu'on decoche rien. Les deux sont tenus d'accord par
/// `tools/selection-cas.json` : `tools/verifier-selection.mjs` y passe le
/// JavaScript, le test `la_selection_d_office_est_celle_de_l_interface` le Rust.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct SelectionOffice {
    /// Les valeurs a poser dans `SINDRY_CONFIG`, en plus des reglages figes.
    pub config: BTreeMap<String, serde_json::Value>,
    /// Quelque chose est-il coche ? Sinon l'action n'est pas lancee : elle ne
    /// ferait que s'inscrire « faite » dans l'historique.
    pub a_faire: bool,
}

/// Un champ « vrai » au sens de l'interface : present et non vide (`locked=open`).
fn present(champs: &BTreeMap<String, String>, cle: &str) -> bool {
    champs.get(cle).is_some_and(|v| !v.is_empty())
}

fn vaut(champs: &BTreeMap<String, String>, cle: &str, valeur: &str) -> bool {
    champs.get(cle).is_some_and(|v| v == valeur)
}

fn positif(champs: &BTreeMap<String, String>, cle: &str) -> bool {
    champs
        .get(cle)
        .and_then(|v| v.trim().parse::<f64>().ok())
        .is_some_and(|n| n > 0.0)
}

/// L'avis du script, sinon la regle de Sindry : ce qui pese ou est a faire.
fn coche(c: &BTreeMap<String, String>) -> bool {
    match c.get("checked") {
        Some(v) => v == "true",
        None => {
            vaut(c, "state", "todo")
                || (!vaut(c, "state", "ok") && (positif(c, "size") || positif(c, "count")))
        }
    }
}

/// Un element « A verifier » (`confidence=low`) n'est jamais coche d'office.
fn coche_element(e: &BTreeMap<String, String>) -> bool {
    let verrou = present(e, "locked");
    match e.get("checked") {
        Some(v) => v == "true" && !verrou,
        None => !verrou && !vaut(e, "state", "ok") && !vaut(e, "confidence", "low"),
    }
}

pub fn selection_office(meta: &Script, analyse: &Analyse) -> SelectionOffice {
    let mut sel = SelectionOffice::default();
    for d in meta.options.iter().filter(|o| !o.scan) {
        let k = d.key.as_str();
        // Comme le modele de l'interface : le dernier `[FIND]` sans choix
        // l'emporte ; un choix garde sa premiere place, ses champs les derniers.
        let mut find: Option<&BTreeMap<String, String>> = None;
        let mut finds: Vec<(&str, &BTreeMap<String, String>)> = Vec::new();
        for f in analyse.finds.iter().filter(|f| f.option == k) {
            if f.choice.is_empty() {
                find = Some(&f.fields);
            } else if let Some(place) = finds.iter_mut().find(|(c, _)| *c == f.choice) {
                place.1 = &f.fields;
            } else {
                finds.push((f.choice.as_str(), &f.fields));
            }
        }
        // L'ordre d'un objet JavaScript : les cles entieres (« 16 ») d'abord,
        // croissantes, puis les autres dans l'ordre d'arrivee. C'est celui
        // de `Object.entries(o.finds)` dans l'interface.
        let entier = |c: &str| {
            (c == "0" || (!c.starts_with('0') && !c.is_empty()))
                && c.bytes().all(|b| b.is_ascii_digit())
                && c.parse::<u32>().is_ok_and(|n| n < u32::MAX)
        };
        finds.sort_by_key(|(c, _)| {
            if entier(c) {
                (0, c.parse::<u32>().unwrap_or(0))
            } else {
                (1, 0)
            }
        });
        let elements: Vec<&BTreeMap<String, String>> = analyse
            .items
            .iter()
            .filter(|e| e.option == k)
            .map(|e| &e.fields)
            .collect();
        let id = |e: &BTreeMap<String, String>| e.get("id").cloned().unwrap_or_default();

        match d.kind.as_str() {
            "bool" => {
                if let Some(f) = find {
                    let oui = coche(f) && !vaut(f, "state", "ok");
                    sel.config
                        .insert(k.to_string(), serde_json::Value::Bool(oui));
                    sel.a_faire |= oui;
                }
            }
            "multi" if !finds.is_empty() => {
                let choisis: Vec<serde_json::Value> = finds
                    .iter()
                    .filter(|(_, f)| coche(f))
                    .map(|(c, _)| serde_json::Value::String(c.to_string()))
                    .collect();
                sel.a_faire |= !choisis.is_empty();
                sel.config
                    .insert(k.to_string(), serde_json::Value::Array(choisis));
            }
            "select" if !finds.is_empty() => {
                let choisi = finds
                    .iter()
                    .find(|(_, f)| vaut(f, "recommended", "true"))
                    .or_else(|| finds.iter().find(|(_, f)| vaut(f, "current", "true")))
                    .or(finds.first())
                    .map(|(c, _)| c.to_string())
                    .unwrap_or_default();
                let actuel = finds
                    .iter()
                    .find(|(_, f)| vaut(f, "current", "true"))
                    .map(|(c, _)| *c);
                sel.a_faire |= actuel != Some(choisi.as_str());
                sel.config
                    .insert(k.to_string(), serde_json::Value::String(choisi));
            }
            "items" => {
                // Un premier avis par id : un doublon ne change pas la case.
                let mut cases: BTreeMap<String, bool> = BTreeMap::new();
                for e in &elements {
                    cases.entry(id(e)).or_insert_with(|| coche_element(e));
                }
                let ids: Vec<String> = if d.keep_one {
                    // Par groupe, l'exemplaire `keep=true`, sinon le premier ;
                    // tout le reste part, sauf ce qui est verrouille.
                    let groupe = |e: &BTreeMap<String, String>| {
                        e.get("group")
                            .filter(|g| !g.is_empty())
                            .cloned()
                            .unwrap_or_else(|| id(e))
                    };
                    let mut garder: BTreeMap<String, String> = BTreeMap::new();
                    for e in &elements {
                        let g = groupe(e);
                        if garder.contains_key(&g) {
                            continue;
                        }
                        let membres: Vec<&&BTreeMap<String, String>> =
                            elements.iter().filter(|x| groupe(x) == g).collect();
                        let garde = membres
                            .iter()
                            .find(|x| vaut(x, "keep", "true"))
                            .or(membres.first())
                            .map(|x| id(x))
                            .unwrap_or_default();
                        garder.insert(g, garde);
                    }
                    elements
                        .iter()
                        .filter(|e| garder.get(&groupe(e)) != Some(&id(e)) && !present(e, "locked"))
                        .map(|e| id(e))
                        .collect()
                } else {
                    // Les feuilles : sans enfant, ni verrouillees ni deja en place.
                    elements
                        .iter()
                        .filter(|e| {
                            let i = id(e);
                            !elements.iter().any(|x| x.get("parent") == Some(&i))
                                && !present(e, "locked")
                                && !vaut(e, "state", "ok")
                        })
                        .filter(|e| cases.get(&id(e)).copied().unwrap_or(false))
                        .map(|e| id(e))
                        .collect()
                };
                sel.a_faire |= !ids.is_empty();
                sel.config.insert(
                    k.to_string(),
                    serde_json::Value::Array(
                        ids.into_iter().map(serde_json::Value::String).collect(),
                    ),
                );
            }
            _ => {}
        }
    }
    sel
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::parse;

    fn script() -> Script {
        let chemin = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("docs")
            .join("mockups")
            .join("exemple-analyse.ps1");
        let brut = std::fs::read_to_string(chemin).expect("exemple illisible");
        parse(brut.trim_start_matches('\u{feff}'))
    }

    fn lignes(texte: &str) -> Vec<String> {
        texte.lines().map(str::to_string).collect()
    }

    /// La sortie reelle de l'exemple, telle que son analyse l'a ecrite sur une
    /// vraie machine (chemins raccourcis).
    const SORTIE: &str = r#"[STEP] 1/3 Measuring temporary folders
[FIND] Targets.user size=647362687 count=2002
[FIND] Targets.windows size=0 count=0
[FIND] RecycleBin size=4450770970 count=58
[STEP] 2/3 Measuring downloaded Windows updates
[FIND] Targets.update size=6013156 count=3 checked=false
[NOTE] UpdateNote Targets.update
[STEP] 3/3 Looking for browser caches
[ITEM] Browsers id=edge name="Microsoft Edge" kind=browser
[ITEM] Browsers id=edge-Default parent=edge kind=folder label=Cache name="Default" path="C:\Users\u\AppData\Local\Microsoft\Edge\User Data\Default\Cache" size=34632571
[LOG] Browsers Microsoft Edge / Default: 189 files
[ITEM] Browsers id=firefox name="Mozilla Firefox" kind=browser locked=open
[ITEM] Browsers id=firefox-x.default-release parent=firefox kind=folder label=Cache name="default-release" path="C:\Users\u\cache2" size=1067443809 locked=open
[NOTE] Untouched Browsers show=simple
[METRIC] FreeSpace value=50 unit=pct health=ok max=100"#;

    #[test]
    fn lit_une_vraie_sortie_sans_anomalie() {
        let a = lire(&script(), &lignes(SORTIE));
        assert!(a.anomalies.is_empty(), "{:?}", a.anomalies);
        assert_eq!(a.finds.len(), 4);
        assert_eq!(a.finds[3].choice, "update");
        assert_eq!(a.finds[3].fields["checked"], "false");
        assert_eq!(a.items.len(), 4);
        assert_eq!(a.items[1].fields["name"], "Default");
        assert!(a.items[1].fields["path"].ends_with(r"\Default\Cache"));
        assert_eq!(a.metrics[0].fields["value"], "50");
        assert_eq!(a.notes[1].show, "simple");
        assert_eq!(
            a.steps,
            vec![
                "Measuring temporary folders",
                "Measuring downloaded Windows updates",
                "Looking for browser caches"
            ]
        );
    }

    #[test]
    fn un_script_ne_parle_que_de_ses_propres_cases() {
        let a = lire(
            &script(),
            &lignes(
                "[FIND] Autre.chose size=1\n\
                 [FIND] Targets.inconnu size=1\n\
                 [FIND] RecycleBin.x size=1\n\
                 [ITEM] Targets id=a\n\
                 [METRIC] Inconnue value=3\n\
                 [NOTE] Cache\n\
                 [NOTE] UpdateNote Nulle.part",
            ),
        );
        assert!(
            a.finds.is_empty() && a.items.is_empty() && a.metrics.is_empty() && a.notes.is_empty()
        );
        assert_eq!(a.anomalies.len(), 7, "{:?}", a.anomalies);
    }

    #[test]
    fn une_mesure_illisible_est_ignoree_jamais_interpretee() {
        let a = lire(
            &script(),
            &lignes(
                "[FIND] RecycleBin size=-3 count=beaucoup state=peut-etre\n\
                 [ITEM] Browsers id=a size=12Mo kind=voiture date=hier\n\
                 [ITEM] Browsers name=sans-id\n\
                 [ITEM] Browsers id=a\n\
                 [ITEM] Browsers id=\"x y\"",
            ),
        );
        // La case reste, sans ses mesures illisibles.
        assert_eq!(a.finds.len(), 1);
        assert!(a.finds[0].fields.is_empty(), "{:?}", a.finds[0].fields);
        assert_eq!(a.items.len(), 1);
        assert_eq!(a.items[0].fields.len(), 1, "seul id=a est lisible");
        assert!(a.anomalies.len() >= 8, "{:?}", a.anomalies);
    }

    #[test]
    fn un_parent_jamais_annonce_est_releve() {
        let a = lire(&script(), &lignes("[ITEM] Browsers id=b parent=fantome"));
        assert_eq!(a.items.len(), 1);
        assert!(a.anomalies.iter().any(|x| x.contains("fantome")));
    }

    #[test]
    fn la_memoire_n_accepte_que_les_ids_annonces_sur_ce_contenu() {
        let m = Memoire::default();
        let a = lire(&script(), &lignes(SORTIE));
        m.retenir("s1", "h1", &a);
        let ok = vec!["edge-Default".to_string()];
        assert!(m.verifier("s1", "h1", "Browsers", &ok).is_ok());
        assert!(m.verifier("s1", "h1", "Browsers", &[]).is_ok());
        // Un id jamais annonce, un fichier modifie depuis, ou pas d'analyse du tout.
        assert!(m
            .verifier("s1", "h1", "Browsers", &["C:\\Windows".to_string()])
            .is_err());
        assert!(m.verifier("s1", "h2", "Browsers", &ok).is_err());
        assert!(m.verifier("s2", "h1", "Browsers", &ok).is_err());
    }

    /// Le DNS du catalogue mesure tous ses choix, puis dit lequel recommander :
    /// la seconde ligne complete la premiere au lieu d'etre ignoree.
    #[test]
    fn une_seconde_ligne_complete_le_meme_constat() {
        let meta = parse(
            r#"## SINDRY:START
## id            : 00000000-0000-4000-8000-000000000001
## lang          : en
## title         : T
## desc          : D
## category      : tools
## icon          : zap
## version       : 1.0
## admin         : false
## risk          : low
## duration      : fast
## reversible    : true
## interruptible : true
## reboot        : false
## engine        : auto
## scan          : true
## SINDRY:END
## SINDRY:OPTIONS
## Dns : [select] DNS
##   a : A
##   b : B
## SINDRY:END
$CONFIG = @{
    Dns = "a"
}
"#,
        );
        let a = lire(
            &meta,
            &lignes(
                "[FIND] Dns.a ms=12 current=true
[FIND] Dns.b ms=9 current=false
[FIND] Dns.b recommended=true",
            ),
        );
        assert!(a.anomalies.is_empty(), "{:?}", a.anomalies);
        assert_eq!(a.finds.len(), 2);
        let b = a.finds.iter().find(|c| c.choice == "b").unwrap();
        assert_eq!(b.fields.get("ms").map(String::as_str), Some("9"));
        assert_eq!(
            b.fields.get("recommended").map(String::as_str),
            Some("true")
        );
    }

    #[test]
    fn decoupe_les_champs_entre_guillemets() {
        let (nus, champs) = decouper(r#"Browsers id=a name="Mozilla Firefox" path="C:\a b" x=1"#);
        assert_eq!(nus, vec!["Browsers"]);
        assert_eq!(
            champs[1],
            ("name".to_string(), "Mozilla Firefox".to_string())
        );
        assert_eq!(champs[2].1, r"C:\a b");
    }

    /// Le contrat commun avec l'interface : `tools/selection-cas.json`, que
    /// `tools/verifier-selection.mjs` passe au JavaScript.
    #[test]
    fn la_selection_d_office_est_celle_de_l_interface() {
        use serde_json::Value;
        let chemin = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("tools")
            .join("selection-cas.json");
        let doc: Value = serde_json::from_str(&std::fs::read_to_string(chemin).expect("cas"))
            .expect("JSON des cas");
        let texte = |v: &Value| v.as_str().unwrap_or_default().to_string();
        let champs = |v: &Value| -> BTreeMap<String, String> {
            v.as_object()
                .map(|o| o.iter().map(|(k, x)| (k.clone(), texte(x))).collect())
                .unwrap_or_default()
        };
        let cas = doc["cas"].as_array().expect("cas");
        assert!(cas.len() >= 10);
        for c in cas {
            let meta = Script {
                options: c["options"]
                    .as_array()
                    .expect("options")
                    .iter()
                    .map(|o| crate::contract::Opt {
                        key: texte(&o["key"]),
                        kind: texte(&o["kind"]),
                        keep_one: o["keep_one"].as_bool().unwrap_or(false),
                        scan: o["scan"].as_bool().unwrap_or(false),
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            };
            let analyse = Analyse {
                finds: c["analyse"]["finds"]
                    .as_array()
                    .expect("finds")
                    .iter()
                    .map(|f| Constat {
                        option: texte(&f["option"]),
                        choice: texte(&f["choice"]),
                        fields: champs(&f["fields"]),
                    })
                    .collect(),
                items: c["analyse"]["items"]
                    .as_array()
                    .expect("items")
                    .iter()
                    .map(|e| Element {
                        option: texte(&e["option"]),
                        fields: champs(&e["fields"]),
                    })
                    .collect(),
                ..Default::default()
            };
            let obtenu = selection_office(&meta, &analyse);
            let attendu = &c["attendu"];
            let nom = texte(&c["nom"]);
            assert_eq!(
                Value::Object(obtenu.config.clone().into_iter().collect()),
                attendu["config"],
                "cas « {nom} »"
            );
            assert_eq!(
                Value::Bool(obtenu.a_faire),
                attendu["a_faire"],
                "cas « {nom} »"
            );
        }
    }
}
