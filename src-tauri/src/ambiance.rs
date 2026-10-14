//! Ambiances partageables (2.0, specification §15.7).
//!
//! Une ambiance tient dans un fichier `.sindry-ambiance` : ses couleurs, en
//! clair, en sombre ou les deux, et un decor libre — du balisage SVG ou HTML,
//! et sa feuille de style. Le format est decrit dans `docs/FORMAT_AMBIANCE.md` ;
//! ce module en est la reference executable : l'interface, l'import, la ligne
//! de commande et le test de l'exemple passent tous par [`valider`].
//!
//! Le decor vient de n'importe qui. Il s'affiche dans un cadre isole (iframe
//! `sandbox` sans aucune permission, CSP qui n'admet que les images `data:`) :
//! ni script, ni reseau, ni acces a l'interface. Les regles ci-dessous ne
//! remplacent pas cette isolation, elles la doublent — et donnent a l'auteur
//! un refus qui dit quoi corriger, plutot qu'un decor qui n'afficherait rien.

use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Runtime};

/// L'extension des fichiers d'ambiance.
pub const EXTENSION: &str = "sindry-ambiance";
/// Le format que cette version lit.
pub const FORMAT: u64 = 1;
/// Un fichier, decor compris : au-dela, ce n'est plus un decor.
const TAILLE_MAX: usize = 1024 * 1024;
/// Le decor seul, balisage et feuille de style ensemble.
const DECOR_MAX: usize = 512 * 1024;
/// Au-dela, les fichiers suivants du dossier sont ignores.
const NOMBRE_MAX: usize = 200;

/// Les couleurs d'un mode, toutes obligatoires : ce sont les jetons de
/// l'interface (`src/styles.css`), sans le `--`.
pub const COULEURS: [&str; 35] = [
    "bg",
    "surface",
    "surface-2",
    "surface-3",
    "border",
    "border-2",
    "text",
    "text-2",
    "text-3",
    "ink",
    "ok",
    "warn",
    "err",
    "ok-s",
    "warn-s",
    "err-s",
    "acc",
    "acc-2",
    "acc-on",
    "surface-soft",
    "acc-soft",
    "knob-off",
    "serie-1",
    "serie-2",
    "serie-3",
    "serie-4",
    "serie-5",
    "serie-6",
    "rouage-a",
    "rouage-b",
    "eau-a",
    "eau-b",
    "term-bg",
    "term-text",
    "term-dim",
];
/// L'opacite des deux nappes de la houle, de 0 (invisible) a 1.
pub const NOMBRES: [&str; 2] = ["eau-fond", "eau-surface"];

/// Ce qui doit se lire, et sur quoi : les memes paires que celles qui ont
/// servi a mesurer les ambiances livrees. 4,5:1 au moins, partout.
const PAIRES: [(&str, &[&str]); 10] = [
    (
        "text",
        &["bg", "surface", "surface-2", "surface-soft", "acc-soft"],
    ),
    ("text-2", &["bg", "surface", "surface-2"]),
    ("text-3", &["bg", "surface"]),
    ("acc-on", &["bg", "surface", "acc-soft"]),
    ("ok", &["bg", "surface", "ok-s"]),
    ("warn", &["bg", "surface", "warn-s"]),
    ("err", &["bg", "surface", "err-s"]),
    ("ink", &["acc", "acc-2"]),
    ("term-text", &["term-bg"]),
    ("term-dim", &["term-bg"]),
];
/// Les marqueurs du journal ont leur propre palette, commune a toutes les
/// ambiances (`--term-info` … `--term-err`) : ils doivent se lire sur `term-bg`.
const MARQUEURS_JOURNAL: [(&str, &str); 5] = [
    ("info", "#7DB8E8"),
    ("etape", "#2DD4BF"),
    ("reussite", "#4ADE80"),
    ("avertissement", "#FBBF24"),
    ("erreur", "#F87171"),
];
const CONTRASTE_MIN: f64 = 4.5;

/// Balises refusees dans le decor : ce qui execute, charge, navigue ou
/// recoit une saisie. La feuille de style va dans `css`, pas dans une balise.
const BALISES_INTERDITES: [&str; 33] = [
    "script",
    "iframe",
    "frame",
    "frameset",
    "object",
    "embed",
    "applet",
    "link",
    "meta",
    "base",
    "form",
    "input",
    "button",
    "textarea",
    "select",
    "option",
    "foreignobject",
    "audio",
    "video",
    "source",
    "track",
    "portal",
    "style",
    "template",
    "noscript",
    "html",
    "head",
    "body",
    "title",
    "dialog",
    "slot",
    "a",
    "math",
];
/// Attributs qui designent une ressource : seuls `#ancre` et une image
/// `data:` y sont admis.
const ATTRIBUTS_LIEN: [&str; 10] = [
    "href",
    "xlink:href",
    "src",
    "srcset",
    "action",
    "formaction",
    "poster",
    "background",
    "ping",
    "lowsrc",
];
/// Les seules images qu'un decor peut embarquer. Pas de SVG en `data:` : il
/// pourrait lui-meme en charger d'autres.
const IMAGES_ADMISES: [&str; 4] = [
    "data:image/png;base64,",
    "data:image/jpeg;base64,",
    "data:image/gif;base64,",
    "data:image/webp;base64,",
];

/// Un probleme du fichier : un code stable (pour la documentation et la ligne
/// de commande) et une phrase qui dit quoi corriger.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Constat {
    pub code: &'static str,
    pub message: String,
}

fn constat(code: &'static str, message: impl Into<String>) -> Constat {
    Constat {
        code,
        message: message.into(),
    }
}

/// Un texte dans les deux langues de l'interface. Un fichier peut n'en donner
/// qu'une : elle sert pour l'autre.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct Texte {
    pub fr: String,
    pub en: String,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct Decor {
    pub balisage: String,
    pub css: String,
}

/// Une ambiance valide, prete a s'appliquer : couleurs normalisees
/// (`#RRGGBB`), opacites en texte (`0.4`).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Ambiance {
    pub id: String,
    pub nom: Texte,
    pub description: Option<Texte>,
    pub auteur: String,
    pub version: String,
    /// `normale` ou `chasse-fixe` (toute l'interface dans la police du journal).
    pub police: String,
    pub clair: Option<BTreeMap<String, String>>,
    pub sombre: Option<BTreeMap<String, String>>,
    pub decor: Decor,
    /// Le fichier d'ou elle vient, une fois installee.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub fichier: String,
}

/// Lit et verifie le contenu d'un fichier d'ambiance. Tous les problemes sont
/// rendus d'un coup : l'auteur corrige tout en une fois.
pub fn valider(contenu: &str) -> Result<Ambiance, Vec<Constat>> {
    let contenu = contenu.strip_prefix('\u{feff}').unwrap_or(contenu);
    if contenu.len() > TAILLE_MAX {
        return Err(vec![constat(
            "TAILLE",
            format!(
                "{} Ko : une ambiance tient en {} Ko au plus, decor compris.",
                contenu.len() / 1024,
                TAILLE_MAX / 1024
            ),
        )]);
    }
    let racine: Value = serde_json::from_str(contenu).map_err(|e| {
        vec![constat(
            "JSON_INVALIDE",
            format!("ce n'est pas du JSON valide : {e}."),
        )]
    })?;
    let Some(objet) = racine.as_object() else {
        return Err(vec![constat(
            "JSON_INVALIDE",
            "le fichier doit etre un objet JSON : { \"format\": 1, … }.",
        )]);
    };

    let mut c = Vec::new();
    cles_connues(
        objet,
        &[
            "format",
            "id",
            "nom",
            "description",
            "auteur",
            "version",
            "police",
            "clair",
            "sombre",
            "decor",
        ],
        "",
        &mut c,
    );

    match objet.get("format").and_then(Value::as_u64) {
        Some(FORMAT) => {}
        Some(n) => c.push(constat(
            "FORMAT_INCONNU",
            format!("format {n} : cette version de Sindry lit le format {FORMAT}."),
        )),
        None => c.push(constat(
            "FORMAT_INCONNU",
            format!("« format » manquant : ecrivez \"format\": {FORMAT}."),
        )),
    }

    let id = objet.get("id").and_then(Value::as_str).unwrap_or_default();
    if !id_valide(id) {
        c.push(constat(
            "ID_INVALIDE",
            format!(
                "« id » vaut {} : minuscules, chiffres et tirets, 40 caracteres au plus (« foret-de-nuit »).",
                objet.get("id").cloned().unwrap_or(Value::Null)
            ),
        ));
    } else if crate::settings::ACCENTS.contains(&id) || id == "perso" {
        c.push(constat(
            "ID_RESERVE",
            format!(
                "« {id} » est le nom d'une ambiance livree avec Sindry : choisissez-en un autre."
            ),
        ));
    }

    let nom = texte(objet.get("nom"), "nom", 40, true, &mut c).unwrap_or_default();
    let description = texte(objet.get("description"), "description", 200, false, &mut c);
    let auteur = chaine(objet.get("auteur"), "auteur", 60, &mut c);
    let version = chaine(objet.get("version"), "version", 20, &mut c);
    let police = match objet.get("police") {
        None => "normale".to_string(),
        Some(Value::String(p)) if p == "normale" || p == "chasse-fixe" => p.clone(),
        Some(autre) => {
            c.push(constat(
                "POLICE_INCONNUE",
                format!("« police » vaut {autre} : \"normale\" ou \"chasse-fixe\"."),
            ));
            "normale".to_string()
        }
    };

    let clair = mode(objet.get("clair"), "clair", &mut c);
    let sombre = mode(objet.get("sombre"), "sombre", &mut c);
    if objet.get("clair").is_none() && objet.get("sombre").is_none() {
        c.push(constat(
            "MODE_ABSENT",
            "ni « clair » ni « sombre » : une ambiance donne ses couleurs pour l'un des deux au moins.",
        ));
    }

    let decor = decor(objet.get("decor"), &mut c);

    if !c.is_empty() {
        return Err(c);
    }
    Ok(Ambiance {
        id: id.to_string(),
        nom,
        description,
        auteur,
        version,
        police,
        clair,
        sombre,
        decor,
        fichier: String::new(),
    })
}

/// `id` : la forme des noms d'icones et des ids de visite.
pub fn id_valide(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 40
        && id.split('-').all(|p| {
            !p.is_empty()
                && p.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

/// Les cles inconnues sont refusees (une faute de frappe se voit), sauf celles
/// qui commencent par `_` : le JSON n'a pas de commentaires, elles en tiennent
/// lieu.
fn cles_connues(objet: &Map<String, Value>, connues: &[&str], ou: &str, c: &mut Vec<Constat>) {
    for cle in objet.keys() {
        if cle.starts_with('_') || connues.contains(&cle.as_str()) {
            continue;
        }
        let suggestion = plus_proche(cle, connues)
            .map(|s| format!(" — vouliez-vous dire « {s} » ?"))
            .unwrap_or_default();
        c.push(constat(
            "CLE_INCONNUE",
            format!("{ou}« {cle} » n'existe pas dans le format{suggestion}"),
        ));
    }
}

fn plus_proche<'a>(mot: &str, connus: &[&'a str]) -> Option<&'a str> {
    connus
        .iter()
        .map(|k| (distance(mot, k), *k))
        .filter(|(d, _)| *d <= 2)
        .min()
        .map(|(_, k)| k)
}

/// Distance d'edition (Levenshtein), pour suggerer la bonne cle.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut ligne: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut precedent = ligne[0];
        ligne[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let haut = ligne[j + 1];
            ligne[j + 1] = if ca == *cb {
                precedent
            } else {
                1 + precedent.min(haut).min(ligne[j])
            };
            precedent = haut;
        }
    }
    ligne[b.len()]
}

fn texte_propre(s: &str, max: usize) -> bool {
    s.chars().count() <= max && !s.chars().any(char::is_control)
}

/// Un texte : une chaine (les deux langues), ou `{ "fr": …, "en": … }`.
fn texte(
    valeur: Option<&Value>,
    nom: &str,
    max: usize,
    obligatoire: bool,
    c: &mut Vec<Constat>,
) -> Option<Texte> {
    let invalide = |c: &mut Vec<Constat>| {
        c.push(constat(
            "TEXTE_INVALIDE",
            format!(
                "« {nom} » : un texte de {max} caracteres au plus, ou {{ \"fr\": …, \"en\": … }}."
            ),
        ))
    };
    let t = match valeur {
        None if obligatoire => {
            c.push(constat("TEXTE_INVALIDE", format!("« {nom} » manquant.")));
            return None;
        }
        None => return None,
        Some(Value::String(s)) => Texte {
            fr: s.trim().to_string(),
            en: s.trim().to_string(),
        },
        Some(Value::Object(o)) => {
            cles_connues(o, &["fr", "en"], &format!("{nom} : "), c);
            let lire = |k: &str| {
                o.get(k)
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .trim()
                    .to_string()
            };
            let (fr, en) = (lire("fr"), lire("en"));
            Texte {
                fr: if fr.is_empty() {
                    en.clone()
                } else {
                    fr.clone()
                },
                en: if en.is_empty() { fr } else { en },
            }
        }
        Some(_) => {
            invalide(c);
            return None;
        }
    };
    if t.fr.is_empty() || !texte_propre(&t.fr, max) || !texte_propre(&t.en, max) {
        invalide(c);
        return None;
    }
    Some(t)
}

fn chaine(valeur: Option<&Value>, nom: &str, max: usize, c: &mut Vec<Constat>) -> String {
    match valeur {
        None => String::new(),
        Some(Value::String(s)) if texte_propre(s.trim(), max) => s.trim().to_string(),
        Some(_) => {
            c.push(constat(
                "TEXTE_INVALIDE",
                format!("« {nom} » : un texte de {max} caracteres au plus."),
            ));
            String::new()
        }
    }
}

/// `#RRGGBB` ou `#RGB`, rendu en `#RRGGBB` majuscules. Pas de transparence :
/// le contraste ne se mesure que sur des couleurs pleines.
fn couleur(s: &str) -> Option<String> {
    let hex = s.strip_prefix('#')?;
    if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let pleine = match hex.len() {
        6 => hex.to_string(),
        3 => hex.chars().flat_map(|ch| [ch, ch]).collect(),
        _ => return None,
    };
    Some(format!("#{}", pleine.to_ascii_uppercase()))
}

fn luminance(hex: &str) -> f64 {
    let canal = |i: usize| {
        let v = f64::from(u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0)) / 255.0;
        if v <= 0.039_28 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * canal(1) + 0.7152 * canal(3) + 0.0722 * canal(5)
}

/// Le rapport de contraste du WCAG, de 1 a 21.
pub fn contraste(a: &str, b: &str) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Les couleurs d'un mode : toutes presentes, bien formees, lisibles.
fn mode(
    valeur: Option<&Value>,
    nom: &str,
    c: &mut Vec<Constat>,
) -> Option<BTreeMap<String, String>> {
    let valeur = valeur?;
    let Some(o) = valeur.as_object() else {
        c.push(constat(
            "MODE_INVALIDE",
            format!("« {nom} » doit etre un objet : {{ \"bg\": \"#0B0F16\", … }}."),
        ));
        return None;
    };
    let connues: Vec<&str> = COULEURS.iter().chain(NOMBRES.iter()).copied().collect();
    cles_connues(o, &connues, &format!("{nom} : "), c);

    let avant = c.len();
    let mut jetons = BTreeMap::new();
    let manquants: Vec<&str> = connues
        .iter()
        .copied()
        .filter(|k| !o.contains_key(*k))
        .collect();
    if !manquants.is_empty() {
        c.push(constat(
            "JETON_MANQUANT",
            format!("{nom} : il manque {}.", manquants.join(", ")),
        ));
    }
    for k in COULEURS {
        match o.get(k) {
            None => {}
            Some(Value::String(s)) if couleur(s).is_some() => {
                jetons.insert(k.to_string(), couleur(s).unwrap_or_default());
            }
            Some(v) => c.push(constat(
                "COULEUR_INVALIDE",
                format!("{nom} : « {k} » vaut {v}, attendu une couleur \"#RRGGBB\"."),
            )),
        }
    }
    for k in NOMBRES {
        match o.get(k).and_then(Value::as_f64) {
            Some(n) if (0.0..=1.0).contains(&n) => {
                jetons.insert(k.to_string(), format!("{n}"));
            }
            Some(_) | None if o.contains_key(k) => c.push(constat(
                "NOMBRE_INVALIDE",
                format!("{nom} : « {k} » est une opacite, un nombre de 0 a 1."),
            )),
            _ => {}
        }
    }
    if c.len() > avant {
        return None;
    }

    for (texte, fonds) in PAIRES {
        for fond in fonds {
            let r = contraste(&jetons[texte], &jetons[*fond]);
            if r < CONTRASTE_MIN {
                c.push(constat(
                    "CONTRASTE",
                    format!(
                        "{nom} : « {texte} » {} sur « {fond} » {} = {:.2}:1, il faut {CONTRASTE_MIN}:1 au moins.",
                        jetons[texte], jetons[*fond], r
                    ),
                ));
            }
        }
    }
    for (marqueur, teinte) in MARQUEURS_JOURNAL {
        let r = contraste(teinte, &jetons["term-bg"]);
        if r < CONTRASTE_MIN {
            c.push(constat(
                "CONTRASTE",
                format!(
                    "{nom} : le marqueur « {marqueur} » du journal ({teinte}) sur « term-bg » {} = {:.2}:1, il faut {CONTRASTE_MIN}:1 au moins — assombrissez term-bg.",
                    jetons["term-bg"], r
                ),
            ));
        }
    }
    if c.len() > avant {
        return None;
    }
    Some(jetons)
}

fn decor(valeur: Option<&Value>, c: &mut Vec<Constat>) -> Decor {
    let Some(valeur) = valeur else {
        return Decor::default();
    };
    let Some(o) = valeur.as_object() else {
        c.push(constat(
            "DECOR_INVALIDE",
            "« decor » doit etre un objet : { \"balisage\": \"<svg …>\", \"css\": \"…\" }.",
        ));
        return Decor::default();
    };
    cles_connues(o, &["balisage", "css"], "decor : ", c);
    let lire = |k: &str, c: &mut Vec<Constat>| match o.get(k) {
        None => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(_) => {
            c.push(constat(
                "DECOR_INVALIDE",
                format!("decor : « {k} » doit etre un texte."),
            ));
            String::new()
        }
    };
    let d = Decor {
        balisage: lire("balisage", c),
        css: lire("css", c),
    };
    if d.balisage.len() + d.css.len() > DECOR_MAX {
        c.push(constat(
            "DECOR_TROP_GRAND",
            format!("le decor depasse {} Ko.", DECOR_MAX / 1024),
        ));
        return d;
    }
    verifier_balisage(&d.balisage, c);
    verifier_css(&d.css, "css", c);
    d
}

/// Le nom de la balise qui commence a `debut` (juste apres `<` ou `</`).
fn nom_balise(bas: &str, debut: usize) -> &str {
    let reste = &bas[debut..];
    let fin = reste
        .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == ':' || ch == '-'))
        .unwrap_or(reste.len());
    &reste[..fin]
}

fn verifier_balisage(balisage: &str, c: &mut Vec<Constat>) {
    let bas = balisage.to_ascii_lowercase();
    let mut vues = Vec::new();
    for (i, _) in bas.match_indices('<') {
        let mut j = i + 1;
        if bas[j..].starts_with('/') {
            j += 1;
        }
        let nom = nom_balise(&bas, j);
        // `svg:script`, `xhtml:iframe` : le prefixe d'espace de noms ne change rien.
        let local = nom.rsplit(':').next().unwrap_or(nom);
        if BALISES_INTERDITES.contains(&local) && !vues.contains(&local) {
            vues.push(local);
            c.push(constat(
                "DECOR_BALISE",
                format!("balisage : <{local}> n'est pas admis dans un decor (ni script, ni chargement, ni lien, ni formulaire ; le style va dans « css »)."),
            ));
        }
    }

    // Un gestionnaire d'evenement (`onload=`, `onclick=`…), meme s'il ne
    // s'executera jamais dans le cadre isole.
    let octets = bas.as_bytes();
    for (i, _) in bas.match_indices("on") {
        let avant_ok = i > 0
            && matches!(
                octets[i - 1],
                b' ' | b'\t' | b'\n' | b'\r' | b'"' | b'\'' | b'/'
            );
        if !avant_ok {
            continue;
        }
        let reste = &bas[i + 2..];
        let lettres = reste.bytes().take_while(u8::is_ascii_lowercase).count();
        if lettres > 0 && reste[lettres..].trim_start().starts_with('=') {
            c.push(constat(
                "DECOR_EVENEMENT",
                format!(
                    "balisage : « on{}= » — un decor n'a pas de gestionnaire d'evenement.",
                    &reste[..lettres]
                ),
            ));
            break;
        }
    }

    for attribut in ATTRIBUTS_LIEN {
        for (i, _) in bas.match_indices(attribut) {
            let avant_ok =
                i > 0 && matches!(octets[i - 1], b' ' | b'\t' | b'\n' | b'\r' | b'"' | b'\'');
            let reste = bas[i + attribut.len()..].trim_start();
            if !avant_ok || !reste.starts_with('=') {
                continue;
            }
            let valeur = reste[1..].trim_start().trim_start_matches(['"', '\'']);
            if !(valeur.starts_with('#') || IMAGES_ADMISES.iter().any(|p| valeur.starts_with(p))) {
                c.push(constat(
                    "DECOR_LIEN",
                    format!("balisage : « {attribut}= » ne peut viser qu'une ancre (#nom) ou une image embarquee (data:image/png;base64,…) — rien ne se charge depuis le disque ni le reseau."),
                ));
                break;
            }
        }
    }

    // Animer un attribut de lien revient a en ecrire un.
    for cible in [
        "\"href",
        "'href",
        "=href",
        "\"xlink:href",
        "'xlink:href",
        "\"src",
        "'src",
    ] {
        if bas.contains(&format!("attributename={cible}"))
            || bas.contains(&format!("attributename ={cible}"))
        {
            c.push(constat(
                "DECOR_LIEN",
                "balisage : une animation ne peut pas viser un attribut de lien (href, src).",
            ));
            break;
        }
    }

    // `style=""` et le texte : memes regles que la feuille de style.
    verifier_css(balisage, "balisage", c);
}

fn verifier_css(css: &str, ou: &str, c: &mut Vec<Constat>) {
    let bas = css.to_ascii_lowercase();
    let mut refus = |code: &'static str, message: String| {
        if !c.iter().any(|x| x.code == code && x.message == message) {
            c.push(constat(code, message));
        }
    };
    for interdit in [
        "javascript:",
        "vbscript:",
        "@import",
        "expression(",
        "-moz-binding",
        "behavior:",
        "image-set(",
        "cross-fade(",
        "element(",
    ] {
        if bas.contains(interdit) {
            refus(
                "DECOR_CSS",
                format!("{ou} : « {interdit} » n'est pas admis dans un decor."),
            );
        }
    }
    if ou == "css" && bas.contains("</") {
        refus(
            "DECOR_CSS",
            "css : « </ » n'a pas sa place dans une feuille de style.".to_string(),
        );
    }
    // Un echappement (`\75 rl(`) peut deguiser une fonction : refuse, l'UTF-8
    // ecrit directement suffit.
    let o = bas.as_bytes();
    if o.windows(2)
        .any(|w| w[0] == b'\\' && w[1].is_ascii_hexdigit())
    {
        refus(
            "DECOR_CSS",
            format!("{ou} : pas d'echappement \\XX — ecrivez le caractere lui-meme."),
        );
    }
    for (i, _) in bas.match_indices("url(") {
        let valeur = bas[i + 4..].trim_start().trim_start_matches(['"', '\'']);
        if !(valeur.starts_with('#') || IMAGES_ADMISES.iter().any(|p| valeur.starts_with(p))) {
            refus(
                "DECOR_LIEN",
                format!("{ou} : url() ne peut viser qu'une ancre (#nom) ou une image embarquee (data:image/png;base64,…)."),
            );
        }
    }
}

/* ------------------------------------------------------------------------
Le dossier des ambiances : %LOCALAPPDATA%\Sindry\ambiances
------------------------------------------------------------------------ */

/// Le dossier des ambiances de l'utilisateur : y deposer un fichier suffit.
pub fn dossier<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    Ok(crate::discovery::base_dir(app)?.join("ambiances"))
}

/// Un fichier du dossier que Sindry n'a pas retenu, et pourquoi.
#[derive(Debug, Clone, Serialize)]
pub struct Ignoree {
    pub fichier: String,
    pub constats: Vec<Constat>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Liste {
    pub dossier: String,
    pub ambiances: Vec<Ambiance>,
    pub ignorees: Vec<Ignoree>,
}

fn est_ambiance(chemin: &Path) -> bool {
    chemin
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case(EXTENSION))
}

/// Lit le dossier : chaque fichier valide, dans l'ordre de leur nom ; un `id`
/// deja pris par un fichier precedent est ignore, et dit pourquoi.
pub fn lister_dans(dossier: &Path) -> Liste {
    let mut liste = Liste {
        dossier: dossier.to_string_lossy().to_string(),
        ambiances: Vec::new(),
        ignorees: Vec::new(),
    };
    let Ok(entrees) = fs::read_dir(dossier) else {
        return liste;
    };
    let mut fichiers: Vec<PathBuf> = entrees
        .flatten()
        .map(|e| e.path())
        .filter(|p| est_ambiance(p) && p.is_file())
        .collect();
    fichiers.sort();
    for chemin in fichiers {
        let nom = chemin
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if liste.ambiances.len() >= NOMBRE_MAX {
            liste.ignorees.push(Ignoree {
                fichier: nom,
                constats: vec![constat(
                    "TROP_NOMBREUSES",
                    format!("{NOMBRE_MAX} ambiances au plus : les suivantes sont ignorees."),
                )],
            });
            continue;
        }
        let lu = match fs::metadata(&chemin) {
            Ok(m) if m.len() as usize > TAILLE_MAX => Err(vec![constat(
                "TAILLE",
                format!("une ambiance tient en {} Ko au plus.", TAILLE_MAX / 1024),
            )]),
            _ => fs::read_to_string(&chemin)
                .map_err(|e| vec![constat("ILLISIBLE", format!("lecture impossible : {e}"))])
                .and_then(|t| valider(&t)),
        };
        match lu {
            Ok(mut a) => {
                if let Some(deja) = liste.ambiances.iter().find(|x| x.id == a.id) {
                    liste.ignorees.push(Ignoree {
                        fichier: nom,
                        constats: vec![constat(
                            "ID_DOUBLON",
                            format!("l'id « {} » est deja celui de {}.", a.id, deja.fichier),
                        )],
                    });
                    continue;
                }
                a.fichier = nom;
                liste.ambiances.push(a);
            }
            Err(constats) => liste.ignorees.push(Ignoree {
                fichier: nom,
                constats,
            }),
        }
    }
    liste
}

/// Le dossier est inscriptible sans elevation, et Sindry peut tourner en
/// administrateur : une jonction posee a la place du dossier lui ferait ecrire
/// ou effacer ailleurs. Sindry refuse de toucher un dossier qui en est une.
fn dossier_sur(dossier: &Path) -> Result<(), String> {
    for d in [Some(dossier), dossier.parent()].into_iter().flatten() {
        if let Ok(m) = fs::symlink_metadata(d) {
            if m.file_type().is_symlink() {
                return Err(format!(
                    "{} est un lien vers un autre dossier : Sindry n'y ecrit pas.",
                    d.display()
                ));
            }
        }
    }
    Ok(())
}

/// Ce qu'un import peut rendre. `Existe` : une ambiance porte deja cet `id` —
/// l'interface demande avant de la remplacer. Les ambiances sont en boite :
/// un decor pese jusqu'a 512 Ko, l'enumeration reste petite.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "etat", rename_all = "lowercase")]
pub enum Import {
    Ajoutee {
        ambiance: Box<Ambiance>,
    },
    Existe {
        actuelle: Box<Ambiance>,
        nouvelle: Box<Ambiance>,
    },
    Refusee {
        constats: Vec<Constat>,
    },
}

/// Installe une ambiance : verifiee, puis ecrite telle quelle (la mise en forme
/// de l'auteur est gardee) sous `<id>.sindry-ambiance`.
pub fn importer_dans(dossier: &Path, contenu: &str, remplacer: bool) -> Result<Import, String> {
    let nouvelle = match valider(contenu) {
        Ok(a) => a,
        Err(constats) => return Ok(Import::Refusee { constats }),
    };
    let actuelle = lister_dans(dossier)
        .ambiances
        .into_iter()
        .find(|a| a.id == nouvelle.id);
    if let Some(actuelle) = &actuelle {
        if !remplacer {
            return Ok(Import::Existe {
                actuelle: Box::new(actuelle.clone()),
                nouvelle: Box::new(nouvelle),
            });
        }
    }
    fs::create_dir_all(dossier).map_err(|e| format!("{} : {e}", dossier.display()))?;
    dossier_sur(dossier)?;
    if let Some(actuelle) = actuelle {
        let _ = fs::remove_file(dossier.join(&actuelle.fichier));
    }
    let fichier = format!("{}.{EXTENSION}", nouvelle.id);
    let contenu = contenu.strip_prefix('\u{feff}').unwrap_or(contenu);
    fs::write(dossier.join(&fichier), contenu).map_err(|e| format!("{fichier} : {e}"))?;
    Ok(Import::Ajoutee {
        ambiance: Box::new(Ambiance {
            fichier,
            ..nouvelle
        }),
    })
}

/// Retire une ambiance installee : son fichier est efface. Rend le chemin de
/// ce qui a ete efface.
pub fn retirer_dans(dossier: &Path, id: &str) -> Result<String, String> {
    if !id_valide(id) {
        return Err(format!("id invalide : {id}"));
    }
    dossier_sur(dossier)?;
    let a = lister_dans(dossier)
        .ambiances
        .into_iter()
        .find(|a| a.id == id)
        .ok_or_else(|| format!("aucune ambiance « {id} »"))?;
    let chemin = dossier.join(&a.fichier);
    fs::remove_file(&chemin).map_err(|e| format!("{} : {e}", chemin.display()))?;
    Ok(chemin.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Les couleurs de Plan, sombre : une ambiance livree, donc lisible.
    fn plan_sombre() -> Value {
        json!({
            "bg": "#0B0F16", "surface": "#0F1520", "surface-2": "#141C2A", "surface-3": "#1B2536",
            "border": "#2A3445", "border-2": "#42506A", "text": "#E9EEF5", "text-2": "#8C9AAE",
            "text-3": "#8190A5", "ink": "#1A0B06", "ok": "#3FD69B", "warn": "#FFC24D", "err": "#FF6B7A",
            "ok-s": "#0C2620", "warn-s": "#2A2210", "err-s": "#2E141C", "acc": "#FD7D26",
            "acc-2": "#FF9447", "acc-on": "#FF9A55", "surface-soft": "#0D121B", "acc-soft": "#2A1A10",
            "knob-off": "#56647A", "serie-1": "#FD7D26", "serie-2": "#4AB2E7", "serie-3": "#3FD69B",
            "serie-4": "#B48CFF", "serie-5": "#FFC24D", "serie-6": "#FF6B9A", "rouage-a": "#FD7D26",
            "rouage-b": "#FF9447", "eau-a": "#FD7D26", "eau-b": "#42506A", "eau-fond": 0,
            "eau-surface": 0, "term-bg": "#070A10", "term-text": "#D6E0EC", "term-dim": "#7D8BA0"
        })
    }

    fn minimale() -> Value {
        json!({ "format": 1, "id": "essai", "nom": "Essai", "sombre": plan_sombre() })
    }

    fn codes(v: &Value) -> Vec<&'static str> {
        match valider(&v.to_string()) {
            Ok(_) => Vec::new(),
            Err(c) => c.into_iter().map(|x| x.code).collect(),
        }
    }

    #[test]
    fn une_ambiance_minimale_se_lit() {
        let a = valider(&minimale().to_string()).expect("valide");
        assert_eq!(a.id, "essai");
        assert_eq!(a.nom.fr, "Essai");
        assert_eq!(a.nom.en, "Essai");
        assert!(a.clair.is_none());
        let s = a.sombre.expect("sombre");
        assert_eq!(s["bg"], "#0B0F16");
        assert_eq!(s["eau-fond"], "0");
        assert_eq!(a.police, "normale");
    }

    #[test]
    fn l_exemple_de_la_documentation_est_valide() {
        let racine = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let exemple =
            fs::read_to_string(racine.join("docs/ambiances/foret-de-nuit.sindry-ambiance"))
                .expect("exemple de docs/ambiances");
        let a = valider(&exemple).unwrap_or_else(|c| panic!("{c:#?}"));
        assert!(
            a.clair.is_some() && a.sombre.is_some(),
            "l'exemple montre les deux modes"
        );
        assert!(
            !a.decor.balisage.is_empty() && !a.decor.css.is_empty(),
            "et un decor"
        );
        // Le guide cite chaque jeton : il ne doit pas en manquer un.
        let guide = fs::read_to_string(racine.join("docs/FORMAT_AMBIANCE.md")).expect("guide");
        for k in COULEURS.iter().chain(NOMBRES.iter()) {
            assert!(
                guide.contains(&format!("`{k}`")),
                "FORMAT_AMBIANCE.md ne decrit pas « {k} »"
            );
        }
        // Et chaque code de constat a sa ligne dans la table des controles.
        // Le code est le premier argument de chaque `constat(` et `refus(` du source.
        let source = include_str!("ambiance.rs");
        let mut codes = std::collections::BTreeSet::new();
        let appels = source
            .match_indices("constat(")
            .chain(source.match_indices("refus("))
            .map(|(i, appel)| i + appel.len());
        for debut in appels {
            let Some(reste) = source[debut..].trim_start().strip_prefix('"') else {
                continue;
            };
            let code = &reste[..reste.find('"').unwrap_or(0)];
            if !code.is_empty() && code.bytes().all(|b| b.is_ascii_uppercase() || b == b'_') {
                codes.insert(code);
            }
        }
        assert!(
            codes.len() >= 23,
            "{codes:?} : le releve ne voit plus les constats"
        );
        for code in codes {
            assert!(
                guide.contains(&format!("| `{code}` |")),
                "FORMAT_AMBIANCE.md ne decrit pas le code {code}"
            );
        }
    }

    /// Les jetons de `:root` dans `src/styles.css`, resolus comme le ferait le
    /// navigateur pour `data-accent` et `data-theme` donnes (Windows en sombre :
    /// les blocs de la `@media` s'appliquent, ils doublent ceux de `data-theme`).
    fn jetons_livres(css: &str, accent: &str, theme: &str) -> Map<String, Value> {
        /// (dans la `@media` sombre, selecteur, declarations)
        type Regle = (bool, String, Vec<(String, String)>);
        fn regles(css: &str, media: bool, sortie: &mut Vec<Regle>) {
            let mut k = 0;
            while let Some(a) = css[k..].find('{').map(|a| a + k) {
                let tete = css[k..a].trim();
                let mut prof = 0;
                let mut b = a;
                for (j, ch) in css[a..].char_indices() {
                    match ch {
                        '{' => prof += 1,
                        '}' => {
                            prof -= 1;
                            if prof == 0 {
                                b = a + j;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let corps = &css[a + 1..b];
                if tete.starts_with("@media") {
                    if tete.contains("prefers-color-scheme: dark") {
                        regles(corps, true, sortie);
                    }
                } else if !tete.starts_with('@') {
                    let decls: Vec<(String, String)> = corps
                        .split(';')
                        .filter_map(|d| {
                            let (k, v) = d.trim().strip_prefix("--")?.split_once(':')?;
                            Some((k.trim().to_string(), v.trim().to_string()))
                        })
                        .collect();
                    for sel in tete.split(',').map(str::trim) {
                        if sel.starts_with(":root") && !sel.contains(' ') && !decls.is_empty() {
                            sortie.push((media, sel.to_string(), decls.clone()));
                        }
                    }
                }
                k = b + 1;
            }
        }
        let mut sans_commentaires = String::new();
        let mut reste = css;
        while let Some(i) = reste.find("/*") {
            sans_commentaires.push_str(&reste[..i]);
            reste = reste[i..].find("*/").map_or("", |j| &reste[i + j + 2..]);
        }
        sans_commentaires.push_str(reste);
        let mut toutes = Vec::new();
        regles(&sans_commentaires, false, &mut toutes);

        let accent_pose = (accent != "ardente").then_some(accent);
        let mut gagnant: BTreeMap<String, ((usize, usize), String)> = BTreeMap::new();
        for (ordre, (_media, sel, decls)) in toutes.iter().enumerate() {
            // Les conditions : [a="v"], [a^="v"], :not([a="v"]).
            let mut spec = 1;
            let mut ok = true;
            for morceau in sel[":root".len()..].split('[').skip(1) {
                let neg = sel.contains(&format!(":not([{morceau}"));
                let cond = morceau.split(']').next().unwrap_or("");
                let (nom, op, val) = if let Some((n, v)) = cond.split_once("^=") {
                    (n, "^=", v)
                } else if let Some((n, v)) = cond.split_once('=') {
                    (n, "=", v)
                } else {
                    (cond, "", "")
                };
                let val = val.trim_matches('"');
                let pose = match nom {
                    "data-accent" => accent_pose,
                    "data-theme" => Some(theme),
                    _ => None,
                };
                let vrai = pose.is_some_and(|p| {
                    if op == "^=" {
                        p.starts_with(val)
                    } else {
                        p == val
                    }
                });
                ok &= vrai != neg;
                spec += 1;
            }
            if !ok {
                continue;
            }
            for (k, v) in decls {
                let rang = (spec, ordre);
                if gagnant.get(k).is_none_or(|(r, _)| rang >= *r) {
                    gagnant.insert(k.clone(), (rang, v.clone()));
                }
            }
        }
        let mut jetons = Map::new();
        for k in COULEURS {
            jetons.insert(k.into(), Value::String(gagnant[k].1.clone()));
        }
        for k in NOMBRES {
            let n: f64 = gagnant[k].1.parse().expect("opacite numerique");
            jetons.insert(k.into(), serde_json::json!(n));
        }
        jetons
    }

    /// Une ambiance livree, exportee (§15.7), doit passer le validateur des
    /// fichiers de la communaute : memes regles pour tous. C'est ce test qui a
    /// trouve le texte atenue du journal de Braise sombre a 3,5:1 (2.0).
    #[test]
    fn les_ambiances_livrees_passent_le_validateur() {
        let racine = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let css = fs::read_to_string(racine.join("src/styles.css")).expect("styles.css");
        let sombre_seul = [
            "terminal-vert",
            "terminal-ambre",
            "aurore-boreale",
            "aurore-australe",
        ];
        for accent in crate::settings::ACCENTS {
            let mut v = json!({ "format": 1, "id": "copie", "nom": accent });
            v["sombre"] = Value::Object(jetons_livres(&css, accent, "dark"));
            if !sombre_seul.contains(&accent) {
                v["clair"] = Value::Object(jetons_livres(&css, accent, "light"));
            }
            if let Err(c) = valider(&v.to_string()) {
                panic!("{accent} : {c:#?}");
            }
        }
        // Et le resolveur voit bien les variantes : Synthwave n'est pas Plan.
        assert_ne!(
            jetons_livres(&css, "synthwave", "dark")["acc"],
            jetons_livres(&css, "plan", "dark")["acc"]
        );
        assert_eq!(
            jetons_livres(&css, "ardente", "light")["bg"],
            json!("#FBF6F2")
        );
    }

    #[test]
    fn les_fautes_sont_toutes_rendues_d_un_coup() {
        let mut v = minimale();
        v["format"] = json!(2);
        v["id"] = json!("Mon Theme");
        v["sombre"]["surfce"] = json!("#000000");
        v["sombre"]["acc"] = json!("orange");
        let c = codes(&v);
        for attendu in [
            "FORMAT_INCONNU",
            "ID_INVALIDE",
            "CLE_INCONNUE",
            "COULEUR_INVALIDE",
        ] {
            assert!(c.contains(&attendu), "{attendu} absent de {c:?}");
        }
        let messages = valider(&v.to_string()).unwrap_err();
        assert!(messages
            .iter()
            .any(|m| m.message.contains("vouliez-vous dire « surface »")));
    }

    #[test]
    fn un_texte_illisible_est_refuse() {
        let mut v = minimale();
        v["sombre"]["text-3"] = json!("#20242C");
        let c = valider(&v.to_string()).unwrap_err();
        assert!(c.iter().all(|x| x.code == "CONTRASTE"), "{c:?}");
        assert!(c.iter().any(|x| x.message.contains("« text-3 »")));
        // Un term-bg trop clair rend les marqueurs du journal illisibles.
        let mut v = minimale();
        v["sombre"]["term-bg"] = json!("#9AA4B0");
        v["sombre"]["term-text"] = json!("#000000");
        v["sombre"]["term-dim"] = json!("#000000");
        let c = valider(&v.to_string()).unwrap_err();
        assert!(c.iter().any(|x| x.message.contains("marqueur")), "{c:?}");
    }

    #[test]
    fn un_mode_au_moins_et_les_ids_livres_sont_reserves() {
        let mut v = minimale();
        v.as_object_mut().unwrap().remove("sombre");
        assert_eq!(codes(&v), vec!["MODE_ABSENT"]);
        let mut v = minimale();
        v["id"] = json!("synthwave");
        assert_eq!(codes(&v), vec!["ID_RESERVE"]);
        let mut v = minimale();
        v["sombre"].as_object_mut().unwrap().remove("ink");
        assert_eq!(codes(&v), vec!["JETON_MANQUANT"]);
    }

    #[test]
    fn les_commentaires_et_les_noms_bilingues_passent() {
        let mut v = minimale();
        v["_commentaire"] = json!("libre");
        v["sombre"]["_note"] = json!("libre aussi");
        v["nom"] = json!({ "fr": "Foret", "en": "Forest" });
        let a = valider(&v.to_string()).expect("valide");
        assert_eq!((a.nom.fr.as_str(), a.nom.en.as_str()), ("Foret", "Forest"));
        v["nom"] = json!({ "en": "Forest" });
        assert_eq!(valider(&v.to_string()).unwrap().nom.fr, "Forest");
    }

    #[test]
    fn le_decor_ne_peut_rien_executer_ni_charger() {
        let refuse = |balisage: &str, css: &str| -> Vec<&'static str> {
            let mut v = minimale();
            v["decor"] = json!({ "balisage": balisage, "css": css });
            codes(&v)
        };
        let ok = refuse(
            r##"<svg viewBox="0 0 10 10"><defs><linearGradient id="g"/></defs><rect fill="url(#g)" width="10" height="10"/><image href="data:image/png;base64,AAAA"/></svg>"##,
            ".lune { animation: luit 4s infinite; } @keyframes luit { to { opacity: .4 } }",
        );
        assert!(ok.is_empty(), "{ok:?}");

        assert!(refuse("<script>alert(1)</script>", "").contains(&"DECOR_BALISE"));
        assert!(refuse("<SVG:Script/>", "").contains(&"DECOR_BALISE"));
        assert!(refuse("<foreignObject></foreignObject>", "").contains(&"DECOR_BALISE"));
        assert!(refuse("<style>body{}</style>", "").contains(&"DECOR_BALISE"));
        assert!(refuse("<a href=\"#x\">x</a>", "").contains(&"DECOR_BALISE"));
        assert!(refuse("<svg onload=\"x()\"></svg>", "").contains(&"DECOR_EVENEMENT"));
        assert!(refuse("<svg/onload=x></svg>", "").contains(&"DECOR_EVENEMENT"));
        assert!(refuse("<image href=\"https://exemple.org/x.png\"/>", "").contains(&"DECOR_LIEN"));
        assert!(refuse("<image href='C:/x.png'/>", "").contains(&"DECOR_LIEN"));
        assert!(refuse("<img src=\"data:image/svg+xml,<svg/>\"/>", "").contains(&"DECOR_LIEN"));
        assert!(refuse("<set attributeName=\"href\" to=\"#x\"/>", "").contains(&"DECOR_LIEN"));
        assert!(
            refuse("<div style=\"background:url(http://x)\"></div>", "").contains(&"DECOR_LIEN")
        );
        assert!(refuse("", "body { background: url(https://x/y.png) }").contains(&"DECOR_LIEN"));
        assert!(refuse("", "@import 'x.css';").contains(&"DECOR_CSS"));
        assert!(refuse("", "a { b: \\75rl(x) }").contains(&"DECOR_CSS"));
        assert!(refuse("", "</style><script>").contains(&"DECOR_CSS"));
        // Un texte qui contient « on » n'est pas un gestionnaire.
        assert!(refuse("<svg><text>lune = sur l'horizon</text></svg>", "").is_empty());
    }

    #[test]
    fn le_dossier_retient_un_id_une_fois_et_dit_pourquoi_il_ignore() {
        let dossier = std::env::temp_dir().join(format!("sindry-ambiances-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dossier);
        fs::create_dir_all(&dossier).unwrap();
        fs::write(dossier.join("a.sindry-ambiance"), minimale().to_string()).unwrap();
        fs::write(dossier.join("b.sindry-ambiance"), minimale().to_string()).unwrap();
        fs::write(dossier.join("c.sindry-ambiance"), "{ pas du json").unwrap();
        fs::write(dossier.join("d.txt"), "ignore sans un mot").unwrap();
        let l = lister_dans(&dossier);
        assert_eq!(l.ambiances.len(), 1);
        assert_eq!(l.ambiances[0].fichier, "a.sindry-ambiance");
        let ignores: Vec<(&str, &str)> = l
            .ignorees
            .iter()
            .map(|i| (i.fichier.as_str(), i.constats[0].code))
            .collect();
        assert_eq!(
            ignores,
            vec![
                ("b.sindry-ambiance", "ID_DOUBLON"),
                ("c.sindry-ambiance", "JSON_INVALIDE")
            ]
        );
        let _ = fs::remove_dir_all(&dossier);
    }

    #[test]
    fn importer_demande_avant_de_remplacer() {
        let dossier = std::env::temp_dir().join(format!("sindry-import-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dossier);
        let v1 = minimale().to_string();
        let r = importer_dans(&dossier, &v1, false).unwrap();
        assert!(
            matches!(&r, Import::Ajoutee { ambiance } if ambiance.fichier == "essai.sindry-ambiance")
        );

        let mut v2 = minimale();
        v2["version"] = json!("2");
        let r = importer_dans(&dossier, &v2.to_string(), false).unwrap();
        assert!(matches!(r, Import::Existe { .. }));
        assert_eq!(lister_dans(&dossier).ambiances[0].version, "");

        importer_dans(&dossier, &v2.to_string(), true).unwrap();
        assert_eq!(lister_dans(&dossier).ambiances[0].version, "2");

        let r = importer_dans(&dossier, "{}", false).unwrap();
        assert!(matches!(r, Import::Refusee { .. }));

        retirer_dans(&dossier, "essai").unwrap();
        assert!(lister_dans(&dossier).ambiances.is_empty());
        assert!(retirer_dans(&dossier, "../x").is_err());
        let _ = fs::remove_dir_all(&dossier);
    }
}
