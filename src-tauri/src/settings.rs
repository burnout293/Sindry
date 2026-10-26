//! Réglages, lots et catégories — les données utilisateur de la specification §4 et §8.
//!
//! Deux notions que la 1.4 sépare (§4.1) :
//!
//! - un **lot** est un groupe de scripts à lancer ensemble — ce que le mode Simple
//!   propose. C'est une donnée que l'utilisateur possède : créée, renommée, réordonnée,
//!   supprimée en mode Expert. Jusqu'à la 1.2.1, le code et `settings.json` les
//!   appelaient « catégories » : [`Settings`] relit encore cet ancien nom ;
//! - une **catégorie** range un script dans la liste du mode Expert, et rien d'autre.
//!   Elle vient de `category:` dans l'entête, résolue contre `tools/categories.json`.
//!
//! `category:` sert aussi de **suggestion** pour les lots d'usine : un script
//! `category: cleaning` arrive dans « Faire le ménage » tant que l'utilisateur n'a rien
//! rangé lui-même (`resolve_lots`) — jamais un ordre.
//!
//! Ce module ne gère que `settings.json` (`%LOCALAPPDATA%`, rien de sensible, aux côtés
//! des scripts utilisateur et des journaux — `discovery::base_dir`). Le magasin
//! d'approbation de sécurité (`approved.json`, §12.1/§12.4) est volontairement séparé,
//! dans [`crate::approval`] : il vit dans `%ProgramData%`, inscriptible seulement par un
//! administrateur, parce qu'un magasin accessible en écriture à l'utilisateur serait
//! décoratif — un script malveillant pourrait y ajouter son propre hash.

use crate::discovery::{self, ScriptEntry};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, Runtime};

/// Pseudo-lot des scripts rangés dans aucun lot (spécification §5.1 : « sinon il part
/// en Non classé »). Jamais persisté : calculé.
pub const NON_CLASSE: &str = "unclassified";

/// Catégorie des scripts dont `category:` ne correspond à aucune catégorie connue.
/// Jamais persistée : calculée.
pub const AUTRES: &str = "other";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lot {
    /// Stable, jamais renommé après création — c'est ce que référencent `scripts[]`
    /// des autres lots et les overrides, pas la position dans la liste.
    pub id: String,
    /// Une clé par langue affichée (« fr », « en »…). Un lot d'usine porte les deux ;
    /// un lot créé par l'utilisateur n'a que celle qu'il a tapée — même règle de repli
    /// que les traductions de script (§5.1) : langue absente -> langue de base.
    pub name: BTreeMap<String, String>,
    #[serde(default)]
    pub description: String,
    pub icon: String,
    #[serde(default)]
    pub pinned: bool,
    /// Ordre manuel d'exécution (§6.1) — par lot, pas global.
    #[serde(default)]
    pub scripts: Vec<String>,
    /// Tokens de `category:` qui font arriver un script dans ce lot tant que
    /// l'utilisateur n'a rien rangé (ex. `["cleaning","nettoyage"]`). Vide pour un lot
    /// créé par l'utilisateur : aucun script ne peut le viser par suggestion, seul un
    /// rangement manuel l'y place (voir `resolve_lots`).
    #[serde(default)]
    pub factory_aliases: Vec<String>,
    /// Vrai uniquement pour « Entretien complet » (§4.1) : son `scripts[]` n'est
    /// jamais rempli directement, `group_scripts` le recalcule dynamiquement comme
    /// l'union de tout ce qui est déjà rangé ailleurs. Jamais posé par `create_lot` —
    /// un lot créé par l'utilisateur ne l'est pas.
    #[serde(default)]
    pub aggregate: bool,
}

/// Une catégorie de scripts (§4.1) : elle range la liste du mode Expert, rien de
/// plus. Lue dans `tools/categories.json`, jamais persistée dans `settings.json`.
#[derive(Debug, Clone, Serialize)]
pub struct ScriptCategory {
    pub id: String,
    pub name: BTreeMap<String, String>,
    pub icon: String,
    /// Tokens de `category:` qui la désignent : son id anglais et son id français.
    pub aliases: Vec<String>,
}

/// Ce que l'utilisateur a figé pour un script donné (§4.2 : « le script propose,
/// l'humain dispose »). Un champ absent continue de suivre le script.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScriptOverride {
    /// Rangement à un seul lot d'avant l'appartenance multiple, relu tel quel.
    #[serde(default, alias = "category_locked")]
    pub lot_locked: Option<String>,
    /// Lots auxquels ce script appartient (§4.1 : « un script peut figurer dans
    /// plusieurs lots »).
    ///
    /// `None` = l'utilisateur n'a jamais touche au rangement de ce script ;
    /// c'est alors la suggestion de son entete (`category:`) qui s'applique.
    /// `Some(liste)` fait foi, y compris `Some(vide)` qui signifie « retire de
    /// partout, volontairement » — a ne pas confondre avec `None`, sinon la
    /// suggestion d'usine le reclasserait aussitot.
    ///
    /// `alias` : jusqu'a la 1.2.1, ce champ s'appelait `categories`.
    #[serde(default, alias = "categories")]
    pub lots: Option<Vec<String>>,
    #[serde(default)]
    pub config: BTreeMap<String, serde_json::Value>,
    /// « Nécessite un point de restauration » : seul booléen Sindry encore
    /// réglable par script (§4.2).
    ///
    /// Jusqu'à la 1.6, deux autres l'accompagnaient, retirés en 1.7 :
    /// `enabled` (« Actif » — l'étape « Vérifier » écarte déjà une action pour
    /// un lancement) et `reboot_ack` (« Nécessite un redémarrage » — le script
    /// le déclare, et le confirme par `[REBOOT]`). Un `settings.json` qui les
    /// porte encore se lit sans eux : serde ignore les champs inconnus, et ils
    /// disparaissent à la prochaine écriture. Un script désactivé en 1.6 se
    /// relance donc de nouveau — il n'y a plus de quoi le réactiver.
    pub reversible_ack: Option<bool>,
}

impl ScriptOverride {
    /// Rien n'est figé : l'entrée n'a pas lieu d'être dans `settings.json`.
    fn est_vide(&self) -> bool {
        // `lots` compte comme le reste : sans lui, figer puis relacher une
        // option effacait aussi le rangement manuel du script dans ses lots.
        self.lot_locked.is_none()
            && self.lots.is_none()
            && self.config.is_empty()
            && self.reversible_ack.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub lang: String,
    /// `propose` (versions stables, defaut), `beta` (versions de test aussi) ou
    /// `never` — voir `update::POLITIQUES`.
    pub update_policy: String,
    pub failure_policy: String,
    pub log_cap_mb: u32,
    pub exec_policy: String,
    /// Les lots de l'utilisateur. `alias` : jusqu'a la 1.2.1, `settings.json` les
    /// appelait `categories` — un fichier de cette epoque se relit sans rien perdre,
    /// et se reecrit sous le nouveau nom au premier enregistrement.
    #[serde(alias = "categories")]
    pub lots: Vec<Lot>,
    #[serde(default)]
    pub overrides: BTreeMap<String, ScriptOverride>,
    #[serde(default)]
    pub onboarded: bool,
    /// Numerotation discrete de chaque reglage (« 2.3 ») dans le panneau des
    /// Reglages. Utile pour designer une option precisement a l'oral ou par
    /// ecrit, superflue le reste du temps : d'ou le reglage.
    /// `serde(default)` vaut `false` — un fichier de reglages ecrit avant
    /// l'arrivee de ce champ se relit donc sans erreur, numerotation masquee.
    #[serde(default)]
    pub show_setting_numbers: bool,
    /// Echelle de rendu de l'interface (accessibilite).
    ///
    /// `serde(default)` donne `0.0` pour un fichier ecrit avant ce champ :
    /// `ui_scale_ou_defaut` ramene toute valeur hors bornes a 1.0, donc un
    /// ancien fichier se relit sans que l'interface devienne minuscule.
    #[serde(default)]
    pub ui_scale: f32,
    /// Catalogue de scripts retenu (§16.6) : `""` tant que l'utilisateur n'a
    /// rien decide, `official` pour le catalogue officiel, `none` s'il a choisi
    /// de continuer sans. Une preference, pas une decision de confiance : celle-
    /// ci tient a la cle compilee dans le binaire, pas a ce fichier.
    #[serde(default)]
    pub catalogue_source: String,
    /// Quand verifier le catalogue : `startup` (par defaut, `""` compris) ou
    /// `manual`. Dans les deux cas, rien n'est installe sans un clic (§16.5).
    #[serde(default)]
    pub catalogue_check: String,
    /// « Ne plus afficher » sur le rappel des scripts sans source (§16.6) :
    /// respecte definitivement.
    #[serde(default)]
    pub catalogue_reminder_hidden: bool,
    /// Sources desactivees, par identifiant (§16.2) : ni interrogees, ni
    /// montrees — leurs scripts restent sur le disque. Une preference : la
    /// decision de confiance, elle, est l'inscription de la source (HKLM).
    #[serde(default)]
    pub sources_inactives: Vec<String>,
    /// Par source, les scripts que l'utilisateur a decoches dans la page du
    /// catalogue : ni telecharges, ni montres. Les scripts qu'une nouvelle
    /// version ajoute arrivent coches ; on ne retient que les refus.
    #[serde(default)]
    pub sources_exclus: BTreeMap<String, Vec<String>>,
    /// L'ambiance (reglage 2.3, §15.7) : l'`id` d'un fichier du dossier des
    /// ambiances. Introuvable ou vide : Plan, l'allure de secours de la feuille.
    /// Jusqu'a la 2.0.0, une ambiance livree (`synthwave`…) ou installee
    /// (`perso:<id>`) : `ambiance::passer_aux_fichiers` la ramene a un id.
    #[serde(default)]
    pub accent: String,
    /// Le passage aux ambiances en fichiers (2.0.1) est fait : les ambiances de
    /// base ont ete posees une fois. Retirees ensuite, elles ne reviennent que
    /// sur demande.
    #[serde(default)]
    pub ambiances_fichiers: bool,
    /// The user turned off the download of application icons (3.3, 2.0.4).
    /// Stored the other way round so that the default, `false`, means "on".
    #[serde(default)]
    pub icones_sans_telechargement: bool,
    /// 1.6 (2.2.0): number the settings of an action — `jamais`, `auto` (beyond
    /// ten settings, the default, `""` included) or `toujours`.
    #[serde(default)]
    pub numeros_options: String,
    /// 1.7 (2.2.0): started without rights, Sindry relaunches itself as
    /// administrator (Windows asks). A preference, not a trust decision: set by
    /// someone else, it only brings up the Windows prompt.
    #[serde(default)]
    pub admin_au_demarrage: bool,
    /// 5.7 (2.3.0): the diagnostic log. Every script receives
    /// `SINDRY_DIAG=1` and may write the full detail of its errors. A
    /// preference: it only makes scripts say more.
    #[serde(default)]
    pub diagnostic: bool,
    /// 2.4.0: the last version whose news were shown (« Nouveautés »). After
    /// an update, Sindry shows them once; `""` before 2.4.0.
    #[serde(default)]
    pub nouveautes_vues: String,
}

/// The values of `numeros_options` (1.6).
pub const NUMEROS_OPTIONS: [&str; 3] = ["jamais", "auto", "toujours"];

/// Valeurs admises pour `catalogue_source` une fois la decision prise.
pub const SOURCES_CATALOGUE: [&str; 2] = ["official", "none"];
/// Valeurs admises pour `catalogue_check`.
pub const VERIFICATIONS_CATALOGUE: [&str; 2] = ["startup", "manual"];

/// Echelle utilisable, bornee. Au-dela de 2x l'interface ne tient plus dans
/// une fenetre de taille raisonnable ; en deca de 1x elle devient illisible,
/// ce qui serait l'inverse du service rendu.
pub fn ui_scale_ou_defaut(v: f32) -> f32 {
    if v.is_finite() && (1.0..=2.0).contains(&v) {
        v
    } else {
        1.0
    }
}

/// Une ligne de `tools/categories.json` — la seule source des catégories, partagée
/// avec `tools/lint-scripts.ps1` pour que les deux listes ne puissent pas diverger
/// (c'est exactement la dérive qui a rendu `CREER_UN_SCRIPT.txt` trompeur).
#[derive(Debug, Deserialize)]
struct CategorieUsine {
    id: String,
    id_fr: String,
    name_fr: String,
    name_en: String,
    icon: String,
}

/// Une ligne de `tools/lots.json` : un lot d'usine, et les catégories dont les
/// scripts y arrivent d'eux-mêmes.
#[derive(Debug, Deserialize)]
struct LotUsine {
    id: String,
    name_fr: String,
    name_en: String,
    icon: String,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    pinned: bool,
    #[serde(default)]
    aggregate: bool,
}

/// Parse `tools/categories.json` (contenu brut). Fonction pure : testable sans
/// `AppHandle`.
pub fn categories_from_json(raw: &str) -> Result<Vec<ScriptCategory>, String> {
    let usine: Vec<CategorieUsine> =
        serde_json::from_str(raw).map_err(|e| format!("categories.json invalide : {e}"))?;
    Ok(usine
        .into_iter()
        .map(|c| ScriptCategory {
            aliases: vec![c.id.clone(), c.id_fr],
            name: BTreeMap::from([("fr".to_string(), c.name_fr), ("en".to_string(), c.name_en)]),
            id: c.id,
            icon: c.icon,
        })
        .collect())
}

/// Parse `tools/lots.json` en lots d'usine prêts à persister. Les catégories qu'un
/// lot nomme deviennent ses `factory_aliases` (id anglais et id français) ; une
/// catégorie inconnue est une erreur de construction, pas une donnée à ignorer.
pub fn factory_lots_from_json(
    raw: &str,
    categories: &[ScriptCategory],
) -> Result<Vec<Lot>, String> {
    let usine: Vec<LotUsine> =
        serde_json::from_str(raw).map_err(|e| format!("lots.json invalide : {e}"))?;
    usine
        .into_iter()
        .map(|l| {
            let mut alias = Vec::new();
            for c in &l.categories {
                let cat = categories.iter().find(|x| &x.id == c).ok_or_else(|| {
                    format!(
                        "lots.json : catégorie inconnue « {c} » dans le lot « {} »",
                        l.id
                    )
                })?;
                alias.extend(cat.aliases.iter().cloned());
            }
            Ok(Lot {
                factory_aliases: alias,
                name: BTreeMap::from([
                    ("fr".to_string(), l.name_fr),
                    ("en".to_string(), l.name_en),
                ]),
                id: l.id,
                description: String::new(),
                icon: l.icon,
                pinned: l.pinned,
                scripts: Vec::new(),
                aggregate: l.aggregate,
            })
        })
        .collect()
}

/// Réglages de premier lancement : les lots d'usine de `tools/lots.json`, dont
/// « Entretien complet » épinglé — renommable et supprimable comme n'importe quel
/// autre lot (§4.1). Sa seule particularité est `aggregate: true` : `group_scripts`
/// lui donne dynamiquement l'union de tout ce qui est déjà rangé, il ne peut pas être
/// visé par le `category:` d'un script (`factory_aliases` vide).
pub fn default_settings(factory: Vec<Lot>) -> Settings {
    Settings {
        theme: "system".to_string(),
        lang: "fr".to_string(),
        update_policy: "propose".to_string(),
        failure_policy: "continue".to_string(),
        log_cap_mb: 50,
        exec_policy: "bypass".to_string(),
        lots: factory,
        overrides: BTreeMap::new(),
        onboarded: false,
        show_setting_numbers: false,
        ui_scale: 1.0,
        catalogue_source: String::new(),
        catalogue_check: "startup".to_string(),
        catalogue_reminder_hidden: false,
        sources_inactives: Vec::new(),
        sources_exclus: BTreeMap::new(),
        accent: String::new(),
        ambiances_fichiers: false,
        icones_sans_telechargement: false,
        numeros_options: "auto".to_string(),
        admin_au_demarrage: false,
        diagnostic: false,
        nouveautes_vues: String::new(),
    }
}

/// Charge `settings.json` depuis `path`, ou l'initialise si absent/illisible. Un JSON
/// corrompu ne bloque jamais l'application (même esprit que le lint : constater,
/// jamais planter) — on repart de réglages par défaut plutôt que de refuser de lancer.
pub fn load_from(path: &Path, factory: Vec<Lot>) -> Result<Settings, String> {
    if let Ok(raw) = fs::read_to_string(path) {
        if let Ok(mut s) = serde_json::from_str::<Settings>(&raw) {
            // Une entree qui ne figeait qu'un reglage retire (1.7) ne fige
            // plus rien : « aucune entree » doit continuer de vouloir dire
            // « rien de fige ».
            s.overrides.retain(|_, o| !o.est_vide());
            return Ok(s);
        }
    }
    let defaults = default_settings(factory);
    save_to(path, &defaults)?;
    Ok(defaults)
}

/// Écriture atomique (fichier temporaire puis renommage) pour ne jamais laisser un
/// `settings.json` tronqué si l'application est tuée pendant l'écriture.
pub fn save_to(path: &Path, settings: &Settings) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("creation de {} : {e}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &json).map_err(|e| format!("ecriture de {} : {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| format!("renommage vers {} : {e}", path.display()))?;
    Ok(())
}

/// Tous les lots auxquels un script appartient (§4.1).
///
/// Ordre de decision, du plus explicite au plus suppose :
///   1. `overrides[id].lots` — ce que l'utilisateur a decide, liste vide
///      comprise ;
///   2. `lot_locked` — l'ancien rangement a un seul lot, relu tel quel pour
///      qu'un `settings.json` ecrit avant l'appartenance multiple garde son
///      rangement ;
///   3. `category:` de l'entete — une **suggestion**, jamais un ordre.
///
/// Une liste vide signifie « Non classe ». Les identifiants qui ne designent
/// plus aucun lot sont ecartes : supprimer un lot ne doit pas laisser des
/// appartenances fantomes.
pub fn resolve_lots(settings: &Settings, script_id: &str, declared: &str) -> Vec<String> {
    let connue = |id: &String| settings.lots.iter().any(|c| &c.id == id && !c.aggregate);

    if let Some(over) = settings.overrides.get(script_id) {
        if let Some(liste) = &over.lots {
            return liste.iter().filter(|id| connue(id)).cloned().collect();
        }
        if let Some(locked) = &over.lot_locked {
            return if connue(locked) {
                vec![locked.clone()]
            } else {
                Vec::new()
            };
        }
    }

    settings
        .lots
        .iter()
        .filter(|cat| !cat.aggregate)
        .find(|cat| {
            cat.factory_aliases
                .iter()
                .any(|a| a.eq_ignore_ascii_case(declared))
        })
        .map(|cat| vec![cat.id.clone()])
        .unwrap_or_default()
}

/// Ajoute ou retire un script d'un lot, sans toucher aux autres.
///
/// Premiere modification : la liste est amorcee avec le rangement effectif du
/// moment, pour qu'ajouter un lot n'efface pas celui d'origine.
pub fn set_script_lot_membership(
    settings: &mut Settings,
    script_id: &str,
    declared: &str,
    lot_id: &str,
    member: bool,
) {
    let actuelles = resolve_lots(settings, script_id, declared);
    let over = settings.overrides.entry(script_id.to_string()).or_default();
    let mut liste = over.lots.clone().unwrap_or(actuelles);

    liste.retain(|id| id != lot_id);
    if member {
        liste.push(lot_id.to_string());
    }
    over.lots = Some(liste);
    // L'ancien champ ne doit plus peser : il n'a de sens que tant qu'aucune
    // liste explicite n'existe.
    over.lot_locked = None;
}

/// Le booléen « réglage Sindry » de la §4.2, désigné par un jeton stable côté
/// interface plutôt que par un booléen positionnel — `set_flag(id, "restore",
/// true)` se relit, `set_flag(id, true)` non.
pub const FLAG_RESTORE: &str = "restore";

/// Fige une valeur de `$CONFIG` pour un script (§4.2 : « le script propose,
/// l'utilisateur dispose »).
///
/// `value = None` **efface** l'override au lieu d'écrire `null` : c'est le seul
/// chemin de retour vers le défaut déclaré, et il compte. Tant que la clé est
/// absente, la valeur suit le script et se met à jour à la Ré-analyse (§5.5) ;
/// dès qu'elle est présente, plus rien venant du fichier ne la réécrit.
pub fn set_config_value(
    settings: &mut Settings,
    script_id: &str,
    key: &str,
    value: Option<serde_json::Value>,
) {
    let entree = settings.overrides.entry(script_id.to_string()).or_default();
    match value {
        Some(v) => {
            entree.config.insert(key.to_string(), v);
        }
        None => {
            entree.config.remove(key);
        }
    }
    oublier_si_vide(settings, script_id);
}

/// Pose (ou relâche) le booléen Sindry de la §4.2. `flag` vaut
/// [`FLAG_RESTORE`] ; `value = None` rend le réglage à sa valeur initiale, tirée
/// de `reversible` dans l'entête du script. Les jetons `reboot` et `enabled`
/// d'avant la 1.7 sont refusés comme n'importe quel jeton inconnu.
pub fn set_flag(
    settings: &mut Settings,
    script_id: &str,
    flag: &str,
    value: Option<bool>,
) -> Result<(), String> {
    let entree = settings.overrides.entry(script_id.to_string()).or_default();
    match flag {
        FLAG_RESTORE => entree.reversible_ack = value,
        autre => return Err(format!("reglage par script inconnu : {autre}")),
    }
    oublier_si_vide(settings, script_id);
    Ok(())
}

/// Rend au script **toutes** ses valeurs de `$CONFIG` et sa case Sindry d'un
/// coup. Ne touche pas au classement en catégorie : ranger un script ailleurs
/// et régler ses options sont deux décisions distinctes, annuler l'une ne doit
/// pas annuler l'autre en silence.
pub fn reset_script_config(settings: &mut Settings, script_id: &str) {
    if let Some(entree) = settings.overrides.get_mut(script_id) {
        entree.config.clear();
        entree.reversible_ack = None;
    }
    oublier_si_vide(settings, script_id);
}

/// Un override qui ne fige plus rien est retiré de `settings.json` : sans ça le
/// fichier accumulerait des objets vides à chaque aller-retour sur un
/// interrupteur, et « aucune entrée » cesserait de vouloir dire « rien figé ».
fn oublier_si_vide(settings: &mut Settings, script_id: &str) {
    let vide = settings
        .overrides
        .get(script_id)
        .is_some_and(ScriptOverride::est_vide);
    if vide {
        settings.overrides.remove(script_id);
    }
}

/// Un lot (mode Expert, §2) et les scripts qui y sont rangés, dans l'ordre
/// manuel (§6.1). `missing` liste les ids que le lot référence (rangement
/// manuel ou Ré-analyse passée) mais que la découverte
/// actuelle ne retrouve plus — affichés comme « manquant » (§4.3), jamais une
/// erreur bloquante.
#[derive(Debug, Clone, Serialize)]
pub struct LotGroup {
    pub lot: Lot,
    pub scripts: Vec<ScriptEntry>,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupedScripts {
    pub lots: Vec<LotGroup>,
    pub unclassified: Vec<ScriptEntry>,
}

/// Une catégorie et les scripts qu'elle range (onglet « Scripts » du mode Expert).
/// Les scripts sont désignés par leur id : leurs entrées complètes voyagent déjà
/// avec les lots.
#[derive(Debug, Clone, Serialize)]
pub struct CategoryBucket {
    pub category: ScriptCategory,
    pub scripts: Vec<String>,
}

/// La catégorie d'un script : celle que `category:` désigne, par son id anglais ou
/// français, sans égard à la casse — sinon [`AUTRES`].
pub fn resolve_category<'a>(
    categories: &'a [ScriptCategory],
    declared: &str,
) -> Option<&'a ScriptCategory> {
    categories.iter().find(|c| {
        c.aliases
            .iter()
            .any(|a| a.eq_ignore_ascii_case(declared.trim()))
    })
}

/// Range des scripts par catégorie, dans l'ordre de `tools/categories.json` ;
/// « Autres » ferme la marche. Une catégorie sans script n'apparaît pas.
pub fn group_by_category(
    categories: &[ScriptCategory],
    scripts: &[ScriptEntry],
) -> Vec<CategoryBucket> {
    let mut seaux: Vec<CategoryBucket> = categories
        .iter()
        .map(|c| CategoryBucket {
            category: c.clone(),
            scripts: Vec::new(),
        })
        .collect();
    let mut autres = CategoryBucket {
        category: ScriptCategory {
            id: AUTRES.to_string(),
            name: BTreeMap::from([
                ("fr".to_string(), "Autres".to_string()),
                ("en".to_string(), "Other".to_string()),
            ]),
            icon: "folder".to_string(),
            aliases: Vec::new(),
        },
        scripts: Vec::new(),
    };
    for s in scripts {
        match resolve_category(categories, &s.meta.category) {
            Some(c) => {
                if let Some(seau) = seaux.iter_mut().find(|x| x.category.id == c.id) {
                    seau.scripts.push(s.id.clone());
                }
            }
            None => autres.scripts.push(s.id.clone()),
        }
    }
    seaux.push(autres);
    seaux.retain(|x| !x.scripts.is_empty());
    seaux
}

/// Groups discovered scripts by resolved lot (a pure function, testable without
/// an `AppHandle` or a disk).
pub fn group_scripts(settings: &Settings, scripts: Vec<ScriptEntry>) -> GroupedScripts {
    let mut by_lot: BTreeMap<String, Vec<ScriptEntry>> = BTreeMap::new();
    let mut unclassified = Vec::new();

    for entry in scripts {
        // A script can land in several lots (§4.1). The "run only once when
        // two lots share it" guard lives at run time, not here: grouping must
        // show the truth.
        let ids = resolve_lots(settings, &entry.id, &entry.meta.category);
        if ids.is_empty() {
            unclassified.push(entry);
        } else {
            for lot_id in ids {
                by_lot.entry(lot_id).or_default().push(entry.clone());
            }
        }
    }

    // Every regular lot in its own order: the manual order (§6.1) first; a
    // script newly placed here (not yet in `lot.scripts`) comes last, in
    // discovery order — that is, file order — kept by the stable sort.
    let mut ordered: BTreeMap<&str, Vec<ScriptEntry>> = BTreeMap::new();
    for lot in settings.lots.iter().filter(|l| !l.aggregate) {
        let mut found = by_lot.get(&lot.id).cloned().unwrap_or_default();
        found.sort_by_key(|s| {
            lot.scripts
                .iter()
                .position(|id| id == &s.id)
                .unwrap_or(usize::MAX)
        });
        ordered.insert(&lot.id, found);
    }

    // "Full maintenance" (the aggregate lot, §4.1): every script placed
    // elsewhere, once — two lots sharing a script must not run it twice —
    // following the order of the lots, then each lot's own order. It used to
    // be sorted by script id, a GUID: an arbitrary run order.
    let mut seen = std::collections::HashSet::new();
    let union: Vec<ScriptEntry> = settings
        .lots
        .iter()
        .filter_map(|l| ordered.get(l.id.as_str()))
        .flatten()
        .filter(|s| seen.insert(s.id.clone()))
        .cloned()
        .collect();

    let lots = settings
        .lots
        .iter()
        .map(|lot| {
            if lot.aggregate {
                return LotGroup {
                    lot: lot.clone(),
                    scripts: union.clone(),
                    missing: Vec::new(),
                };
            }
            let found = ordered.remove(lot.id.as_str()).unwrap_or_default();
            let missing = lot
                .scripts
                .iter()
                .filter(|id| !found.iter().any(|s| &s.id == *id))
                .cloned()
                .collect();
            LotGroup {
                lot: lot.clone(),
                scripts: found,
                missing,
            }
        })
        .collect();

    GroupedScripts { lots, unclassified }
}

// -----------------------------------------------------------------------------
// Points d'entrée côté application (résolution de chemins via AppHandle)
// -----------------------------------------------------------------------------

pub fn settings_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    Ok(discovery::base_dir(app)?.join("settings.json"))
}

/// Une ressource livrée avec l'application (`tauri.conf.json`, `bundle.resources`).
fn lire_ressource<R: Runtime>(app: &AppHandle<R>, nom: &str) -> Result<String, String> {
    let path = app
        .path()
        .resolve(nom, tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("ressources introuvables : {e}"))?;
    fs::read_to_string(&path).map_err(|e| format!("lecture de {} : {e}", path.display()))
}

/// Les catégories de `tools/categories.json`.
pub fn read_categories<R: Runtime>(app: &AppHandle<R>) -> Result<Vec<ScriptCategory>, String> {
    categories_from_json(&lire_ressource(app, "categories.json")?)
}

fn read_factory_lots<R: Runtime>(app: &AppHandle<R>) -> Result<Vec<Lot>, String> {
    factory_lots_from_json(&lire_ressource(app, "lots.json")?, &read_categories(app)?)
}

pub fn load<R: Runtime>(app: &AppHandle<R>) -> Result<Settings, String> {
    let chemin = settings_path(app)?;
    // Premier lancement de ce compte : la preconfiguration d'une installation
    // silencieuse (§19.3), s'il y en a une. Un compte qui a deja ses reglages
    // — repris de l'ancien nom compris — les garde.
    if !chemin.exists() {
        // Pas pour une version portable (2.4.0) : la preconfiguration est
        // celle d'une installation sur ce PC, pas de la cle.
        if let Some(pre) = crate::preconfiguration::lire().filter(|_| !crate::portable::actif()) {
            let s = pre.appliquer(default_settings(read_factory_lots(app)?));
            save_to(&chemin, &s)?;
            return Ok(s);
        }
    }
    load_from(&chemin, read_factory_lots(app)?)
}

pub fn save<R: Runtime>(app: &AppHandle<R>, settings: &Settings) -> Result<(), String> {
    save_to(&settings_path(app)?, settings)
}

/// Reinitialisation aux reglages d'usine (§8) : reperd les lots et overrides de
/// l'utilisateur, exactement comme un premier lancement.
pub fn reset<R: Runtime>(app: &AppHandle<R>) -> Result<Settings, String> {
    let defaults = default_settings(read_factory_lots(app)?);
    save(app, &defaults)?;
    Ok(defaults)
}

/// Export (§8) : toute la configuration tient dans un fichier, a cote des
/// scripts et journaux de l'utilisateur — pas besoin d'un selecteur de
/// fichier natif pour un unique fichier a un emplacement previsible.
pub fn export<R: Runtime>(app: &AppHandle<R>) -> Result<String, String> {
    let s = load(app)?;
    let chemin = discovery::base_dir(app)?.join("settings-export.json");
    let json = serde_json::to_string_pretty(&s).map_err(|e| e.to_string())?;
    fs::write(&chemin, json).map_err(|e| format!("ecriture de {} : {e}", chemin.display()))?;
    Ok(chemin.to_string_lossy().to_string())
}

/// Import (§8) : valide le contenu avant d'ecrire quoi que ce soit — un
/// fichier corrompu ou d'une version incompatible est rejete proprement,
/// jamais partiellement applique.
pub fn import_from_str<R: Runtime>(app: &AppHandle<R>, raw: &str) -> Result<Settings, String> {
    let parsed: Settings = serde_json::from_str(raw)
        .map_err(|e| format!("fichier de configuration invalide : {e}"))?;
    save(app, &parsed)?;
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::Script;
    use serde_json::json;

    /// Entree minimale pour les tests de regroupement : seuls `id` et
    /// `category` comptent, le reste n'est jamais lu par `group_scripts`.
    fn script_entry(id: &str, category: &str) -> ScriptEntry {
        ScriptEntry {
            id: id.to_string(),
            path: format!("{id}.ps1"),
            abs_path: format!("{id}.ps1"),
            origin: "user",
            source: None,
            verified: false,
            hash: "0".repeat(64),
            declared_id: true,
            attention: Vec::new(),
            meta: Script {
                id: id.to_string(),
                lang: "fr".to_string(),
                title: id.to_string(),
                desc: String::new(),
                category: category.to_string(),
                icon: "wrench".to_string(),
                tags: Vec::new(),
                version: "1.0".to_string(),
                admin: false,
                risk: "low".to_string(),
                duration: "fast".to_string(),
                reversible: true,
                interruptible: true,
                reboot: false,
                engine: "auto".to_string(),
                options: Vec::new(),
                translations: BTreeMap::new(),
                findings: Vec::new(),
                ..Default::default()
            },
        }
    }

    const JSON_CATEGORIES: &str = r#"[
        { "id": "cleaning", "id_fr": "nettoyage", "name_fr": "Nettoyage", "name_en": "Cleaning", "icon": "broom" },
        { "id": "tools", "id_fr": "outillage", "name_fr": "Outillage", "name_en": "Tools", "icon": "wrench" }
    ]"#;

    const JSON_LOTS: &str = r#"[
        { "id": "cleaning", "name_fr": "Faire le ménage", "name_en": "Cleaning", "icon": "broom", "categories": ["cleaning"] },
        { "id": "tools", "name_fr": "Outillage", "name_en": "Tools", "icon": "wrench", "categories": ["tools"] },
        { "id": "maintenance", "name_fr": "Entretien complet", "name_en": "Full maintenance", "icon": "sparkles", "pinned": true, "aggregate": true }
    ]"#;

    fn categories() -> Vec<ScriptCategory> {
        categories_from_json(JSON_CATEGORIES).unwrap()
    }

    fn lots_usine() -> Vec<Lot> {
        factory_lots_from_json(JSON_LOTS, &categories()).unwrap()
    }

    fn reglages_nus() -> Settings {
        default_settings(lots_usine())
    }

    /// Ou se retrouve un script, lot par lot.
    fn rangement(g: &GroupedScripts, id: &str) -> Vec<String> {
        g.lots
            .iter()
            .filter(|c| c.scripts.iter().any(|s| s.id == id))
            .map(|c| c.lot.id.clone())
            .collect()
    }

    // ----- Appartenance multiple (§4.1) ------------------------------------

    #[test]
    fn un_script_peut_figurer_dans_plusieurs_categories() {
        let mut r = reglages_nus();
        let s = script_entry("nettoie", "cleaning");

        // Suggestion d'entete : une seule categorie au depart.
        assert_eq!(
            rangement(&group_scripts(&r, vec![s.clone()]), "nettoie"),
            vec!["cleaning", "maintenance"]
        );

        // On le coche AUSSI dans Outillage : l'ancienne appartenance reste.
        set_script_lot_membership(&mut r, "nettoie", "cleaning", "tools", true);
        let mut ou = rangement(&group_scripts(&r, vec![s.clone()]), "nettoie");
        ou.retain(|c| c != "maintenance");
        ou.sort();
        assert_eq!(
            ou,
            vec!["cleaning", "tools"],
            "l'ajout a efface l'appartenance d'origine"
        );
    }

    #[test]
    fn l_agregat_ne_compte_un_script_qu_une_fois() {
        let mut r = reglages_nus();
        let s = script_entry("nettoie", "cleaning");
        set_script_lot_membership(&mut r, "nettoie", "cleaning", "tools", true);

        let g = group_scripts(&r, vec![s]);
        let agregat = g.lots.iter().find(|c| c.lot.aggregate).expect("agregat");
        // Sans deduplication, « Entretien complet » l'executerait deux fois.
        assert_eq!(
            agregat.scripts.iter().filter(|s| s.id == "nettoie").count(),
            1
        );
    }

    #[test]
    fn retirer_de_partout_ne_le_fait_pas_reclasser_par_la_suggestion() {
        let mut r = reglages_nus();
        let s = script_entry("nettoie", "cleaning");
        set_script_lot_membership(&mut r, "nettoie", "cleaning", "cleaning", false);

        let g = group_scripts(&r, vec![s]);
        assert!(
            rangement(&g, "nettoie").is_empty(),
            "il est revenu dans une categorie"
        );
        assert_eq!(g.unclassified.len(), 1, "il aurait du passer en Non classe");
        // C'est la distinction entre None et Some(vide) : sans elle, la
        // suggestion `category: cleaning` l'y remettrait aussitot.
        assert_eq!(r.overrides["nettoie"].lots, Some(Vec::new()));
    }

    #[test]
    fn un_rangement_ecrit_par_une_version_anterieure_est_conserve() {
        let mut r = reglages_nus();
        // settings.json d'avant l'appartenance multiple : un seul champ.
        r.overrides.insert(
            "nettoie".to_string(),
            ScriptOverride {
                lot_locked: Some("tools".to_string()),
                ..Default::default()
            },
        );
        let g = group_scripts(&r, vec![script_entry("nettoie", "cleaning")]);
        // L'ordre suit `settings.lots`, pas l'ordre d'ecriture : on
        // compare l'ensemble, pas la sequence.
        let mut ou = rangement(&g, "nettoie");
        ou.sort();
        assert_eq!(ou, vec!["maintenance", "tools"]);
    }

    #[test]
    fn une_appartenance_vers_une_categorie_supprimee_est_ignoree() {
        let mut r = reglages_nus();
        set_script_lot_membership(&mut r, "nettoie", "cleaning", "tools", true);
        // L'utilisateur supprime Outillage ensuite.
        r.lots.retain(|c| c.id != "tools");

        let g = group_scripts(&r, vec![script_entry("nettoie", "cleaning")]);
        let ou = rangement(&g, "nettoie");
        assert!(
            !ou.iter().any(|c| c == "tools"),
            "appartenance fantome conservee"
        );
        assert!(
            ou.iter().any(|c| c == "cleaning"),
            "l'autre appartenance a ete perdue"
        );
    }

    #[test]
    fn une_valeur_figee_se_relit_et_seul_none_la_rend_au_script() {
        let mut s = reglages_nus();
        set_config_value(&mut s, "abc", "SleepOnAC_Min", Some(json!(0)));
        assert_eq!(s.overrides["abc"].config["SleepOnAC_Min"], json!(0));

        // Le cas qui compte : effacer, ce n'est pas ecrire `null`. Une cle
        // absente veut dire « suit le script » (§4.2) ; si l'effacement
        // laissait `null`, la valeur resterait figee sur rien.
        set_config_value(&mut s, "abc", "SleepOnAC_Min", None);
        assert!(!s.overrides.contains_key("abc"));
    }

    #[test]
    fn le_booleen_sindry_se_pose_et_se_relache() {
        let mut s = reglages_nus();
        set_config_value(&mut s, "abc", "Cle", Some(json!(1)));
        set_flag(&mut s, "abc", FLAG_RESTORE, Some(true)).unwrap();
        assert_eq!(s.overrides["abc"].reversible_ack, Some(true));

        set_flag(&mut s, "abc", FLAG_RESTORE, None).unwrap();
        assert_eq!(s.overrides["abc"].reversible_ack, None);

        assert!(set_flag(&mut s, "abc", "autre_chose", Some(true)).is_err());
        // Retires en 1.7 : une interface d'avant ne doit rien pouvoir figer.
        assert!(set_flag(&mut s, "abc", "reboot", Some(true)).is_err());
        assert!(set_flag(&mut s, "abc", "enabled", Some(false)).is_err());
    }

    #[test]
    fn les_reglages_retires_en_1_7_se_lisent_sans_eux() {
        let dir = std::env::temp_dir().join(format!("sindry-retires-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        let mut s = reglages_nus();
        set_config_value(&mut s, "garde", "Cle", Some(json!(1)));
        save_to(&path, &s).unwrap();

        // Un settings.json de la 1.6 : « Actif » decoche sur un script,
        // « Redemarrage » fige sur un autre, a cote d'une vraie valeur.
        let mut brut: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        brut["overrides"]["garde"]["enabled"] = json!(false);
        brut["overrides"]["seul"] =
            json!({ "config": {}, "reversible_ack": null, "reboot_ack": true, "enabled": false });
        fs::write(&path, brut.to_string()).unwrap();

        let relu = load_from(&path, Vec::new()).unwrap();
        assert_eq!(
            relu.overrides["garde"].config["Cle"],
            json!(1),
            "le reste du fichier survit"
        );
        assert!(
            !relu.overrides.contains_key("seul"),
            "une entree qui ne figeait qu'un reglage retire disparait"
        );
        let reecrit = serde_json::to_string(&relu.overrides).unwrap();
        assert!(!reecrit.contains("enabled") && !reecrit.contains("reboot_ack"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn reinitialiser_les_options_ne_defait_pas_le_classement() {
        let mut s = reglages_nus();
        s.overrides.entry("abc".into()).or_default().lot_locked = Some("tools".into());
        set_config_value(&mut s, "abc", "Cle", Some(json!("valeur")));
        set_flag(&mut s, "abc", FLAG_RESTORE, Some(true)).unwrap();

        reset_script_config(&mut s, "abc");

        let over = &s.overrides["abc"];
        assert!(over.config.is_empty());
        assert_eq!(over.reversible_ack, None);
        assert_eq!(over.lot_locked.as_deref(), Some("tools"));
    }

    #[test]
    fn un_aller_retour_sur_un_interrupteur_ne_laisse_pas_de_trace() {
        let mut s = reglages_nus();
        set_flag(&mut s, "abc", FLAG_RESTORE, Some(true)).unwrap();
        set_flag(&mut s, "abc", FLAG_RESTORE, None).unwrap();
        assert!(
            s.overrides.is_empty(),
            "un override vide doit disparaitre de settings.json"
        );
    }

    #[test]
    fn un_lot_d_usine_recoit_les_deux_alias_de_ses_categories() {
        let lots = lots_usine();
        assert_eq!(lots.len(), 3);
        assert_eq!(lots[0].factory_aliases, vec!["cleaning", "nettoyage"]);
        assert_eq!(lots[0].name["fr"], "Faire le ménage");
        assert_eq!(lots[0].name["en"], "Cleaning");
        // La categorie, elle, garde son propre nom : « Nettoyage » range la liste
        // de l'Expert, « Faire le ménage » est ce que le Simple propose.
        assert_eq!(categories()[0].name["fr"], "Nettoyage");
        assert!(lots[2].aggregate && lots[2].pinned && lots[2].factory_aliases.is_empty());
    }

    #[test]
    fn un_lot_d_usine_qui_nomme_une_categorie_inconnue_est_refuse() {
        let faux = r#"[{ "id": "x", "name_fr": "X", "name_en": "X", "icon": "x", "categories": ["bidon"] }]"#;
        assert!(factory_lots_from_json(faux, &categories()).is_err());
    }

    #[test]
    fn un_settings_json_de_la_1_2_relit_ses_categories_comme_des_lots() {
        // Fichier tel que l'ecrivait la 1.2.1 : `categories` en tete, et dans les
        // overrides `categories` et `category_locked`.
        let ancien = r#"{
            "theme": "dark", "lang": "fr", "update_policy": "propose", "failure_policy": "continue",
            "log_cap_mb": 50, "exec_policy": "bypass",
            "categories": [
                { "id": "cleaning", "name": { "fr": "Faire le ménage" }, "icon": "broom",
                  "scripts": ["a"], "factory_aliases": ["cleaning", "nettoyage"] },
                { "id": "mien", "name": { "fr": "Mon lot" }, "icon": "star", "scripts": ["b", "a"] }
            ],
            "overrides": {
                "a": { "categories": ["cleaning", "mien"] },
                "b": { "category_locked": "mien", "config": { "Cle": 1 } }
            }
        }"#;
        let s: Settings =
            serde_json::from_str(ancien).expect("un fichier de la 1.2 doit se relire");
        assert_eq!(s.lots.len(), 2);
        assert_eq!(s.lots[1].scripts, vec!["b", "a"]);
        assert_eq!(
            s.overrides["a"].lots,
            Some(vec!["cleaning".to_string(), "mien".to_string()])
        );
        assert_eq!(s.overrides["b"].lot_locked.as_deref(), Some("mien"));

        // Reecrit sous les nouveaux noms : l'ancien ne reapparait jamais.
        let neuf = serde_json::to_string(&s).unwrap();
        assert!(
            neuf.contains("\"lots\"")
                && !neuf.contains("\"categories\"")
                && !neuf.contains("category_locked")
        );
    }

    #[test]
    fn relacher_une_option_ne_fait_pas_perdre_le_rangement_dans_les_lots() {
        let mut r = reglages_nus();
        set_script_lot_membership(&mut r, "nettoie", "cleaning", "tools", true);
        set_config_value(&mut r, "nettoie", "Cle", Some(json!(1)));
        set_config_value(&mut r, "nettoie", "Cle", None);
        let g = group_scripts(&r, vec![script_entry("nettoie", "cleaning")]);
        assert!(
            rangement(&g, "nettoie").contains(&"tools".to_string()),
            "le rangement manuel a ete efface"
        );
    }

    #[test]
    fn range_par_categorie_et_met_l_inconnu_dans_autres() {
        let scripts = vec![
            script_entry("s1", "Nettoyage"),
            script_entry("s2", "tools"),
            script_entry("s3", "bidon"),
            script_entry("s4", "cleaning"),
        ];
        let seaux = group_by_category(&categories(), &scripts);
        let vu: Vec<(&str, Vec<&str>)> = seaux
            .iter()
            .map(|b| {
                (
                    b.category.id.as_str(),
                    b.scripts.iter().map(String::as_str).collect(),
                )
            })
            .collect();
        assert_eq!(
            vu,
            vec![
                ("cleaning", vec!["s1", "s4"]),
                ("tools", vec!["s2"]),
                (AUTRES, vec!["s3"])
            ]
        );
    }

    #[test]
    fn premier_lancement_ajoute_entretien_complet_epingle() {
        let settings = default_settings(lots_usine());
        let maintenance = settings
            .lots
            .iter()
            .find(|c| c.id == "maintenance")
            .expect("categorie maintenance absente");
        assert!(maintenance.pinned);
        assert_eq!(settings.lots.len(), 3);
    }

    #[test]
    fn resout_un_alias_francais_ou_anglais() {
        let cats = lots_usine();
        let settings = default_settings(cats);
        assert_eq!(resolve_lots(&settings, "s1", "nettoyage"), vec!["cleaning"]);
        assert_eq!(resolve_lots(&settings, "s1", "CLEANING"), vec!["cleaning"]);
        // Une suggestion inconnue ne range nulle part : depuis l'appartenance
        // multiple, « Non classe » se lit a une liste VIDE, pas a un id special.
        assert!(resolve_lots(&settings, "s1", "bidon").is_empty());
    }

    #[test]
    fn un_classement_manuel_prime_toujours_sur_la_suggestion() {
        let cats = lots_usine();
        let mut settings = default_settings(cats);
        settings.overrides.insert(
            "s1".to_string(),
            ScriptOverride {
                lot_locked: Some("tools".to_string()),
                ..Default::default()
            },
        );
        // Le script suggere "cleaning" mais l'utilisateur l'a range dans "tools" :
        // ce rangement manuel l'emporte, quoi que dise `category:` par la suite.
        assert_eq!(resolve_lots(&settings, "s1", "cleaning"), vec!["tools"]);
    }

    #[test]
    fn charge_puis_recharge_a_l_identique() {
        let dir = std::env::temp_dir().join(format!("sindry-test-settings-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        let _ = fs::remove_file(&path);

        let cats = lots_usine();
        let charge = load_from(&path, cats.clone()).unwrap();
        assert!(
            path.exists(),
            "load_from doit ecrire les reglages par defaut"
        );

        let mut modifie = charge;
        modifie.lang = "en".to_string();
        // Les reglages par script (§4.2) font partie du voyage : une valeur
        // figee qui ne survit pas a un redemarrage n'est pas figee du tout.
        set_config_value(&mut modifie, "abc", "SleepOnAC_Min", Some(json!(0)));
        set_flag(&mut modifie, "abc", FLAG_RESTORE, Some(true)).unwrap();
        save_to(&path, &modifie).unwrap();

        let recharge = load_from(&path, cats).unwrap();
        assert_eq!(recharge.lang, "en");
        let over = &recharge.overrides["abc"];
        assert_eq!(over.config["SleepOnAC_Min"], json!(0));
        assert_eq!(over.reversible_ack, Some(true));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn un_settings_json_corrompu_ne_bloque_pas_le_lancement() {
        let dir = std::env::temp_dir().join(format!("sindry-test-corrompu-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        fs::write(&path, "{ pas du json valide").unwrap();

        let cats = lots_usine();
        let settings = load_from(&path, cats).expect("doit repartir de zero, pas planter");
        assert_eq!(settings.lang, "fr");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn regroupe_par_categorie_et_isole_le_non_classe() {
        let cats = lots_usine();
        let settings = default_settings(cats);
        let scripts = vec![
            script_entry("s1", "cleaning"),
            script_entry("s2", "nettoyage"),
            script_entry("s3", "bidon"),
        ];
        let groupes = group_scripts(&settings, scripts);

        let nettoyage = groupes
            .lots
            .iter()
            .find(|g| g.lot.id == "cleaning")
            .unwrap();
        assert_eq!(
            nettoyage
                .scripts
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            vec!["s1", "s2"]
        );
        assert_eq!(groupes.unclassified.len(), 1);
        assert_eq!(groupes.unclassified[0].id, "s3");
    }

    #[test]
    fn entretien_complet_rassemble_dynamiquement_tout_ce_qui_est_classe() {
        let cats = lots_usine();
        let settings = default_settings(cats);
        let scripts = vec![
            script_entry("s1", "cleaning"),
            script_entry("s2", "outillage"),
            script_entry("s3", "bidon"), // non classe : ne doit pas apparaitre
        ];
        let groupes = group_scripts(&settings, scripts);

        let entretien = groupes
            .lots
            .iter()
            .find(|g| g.lot.id == "maintenance")
            .expect("categorie maintenance absente");
        assert!(entretien.lot.pinned);
        assert_eq!(
            entretien
                .scripts
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            vec!["s1", "s2"]
        );
        assert!(entretien.missing.is_empty());
    }

    #[test]
    fn full_maintenance_follows_lot_order_then_each_lot_order() {
        let mut settings = default_settings(lots_usine());
        // Discovery (file) order: a2, z1, m3. Sorting by id would give a2, m3, z1.
        let scripts = vec![
            script_entry("a2", "tools"),
            script_entry("z1", "cleaning"),
            script_entry("m3", "cleaning"),
        ];
        let run_order = |s: &Settings| -> Vec<String> {
            group_scripts(s, scripts.clone())
                .lots
                .into_iter()
                .find(|g| g.lot.aggregate)
                .expect("aggregate lot")
                .scripts
                .into_iter()
                .map(|x| x.id)
                .collect()
        };
        // Lots in their order (cleaning, then tools), each in discovery order.
        assert_eq!(run_order(&settings), vec!["z1", "m3", "a2"]);
        // A manual order inside a lot carries over to full maintenance.
        settings.lots[0].scripts = vec!["m3".to_string()];
        assert_eq!(run_order(&settings), vec!["m3", "z1", "a2"]);
        // So does the order of the lots themselves.
        settings.lots.swap(0, 1);
        assert_eq!(run_order(&settings), vec!["a2", "m3", "z1"]);
    }

    #[test]
    fn respecte_l_ordre_manuel_et_signale_les_manquants() {
        let cats = lots_usine();
        let mut settings = default_settings(cats);
        settings
            .lots
            .iter_mut()
            .find(|c| c.id == "cleaning")
            .unwrap()
            .scripts = vec!["s2".to_string(), "s1".to_string(), "s-disparu".to_string()];

        let scripts = vec![
            script_entry("s1", "cleaning"),
            script_entry("s2", "cleaning"),
        ];
        let groupes = group_scripts(&settings, scripts);

        let nettoyage = groupes
            .lots
            .iter()
            .find(|g| g.lot.id == "cleaning")
            .unwrap();
        // L'ordre manuel (s2 puis s1) l'emporte sur l'ordre de decouverte.
        assert_eq!(
            nettoyage
                .scripts
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            vec!["s2", "s1"]
        );
        assert_eq!(nettoyage.missing, vec!["s-disparu".to_string()]);
    }
}
