//! Mise a jour de WinTool (specification §11).
//!
//! Le greffon officiel `tauri-plugin-updater` fait le travail delicat, et son
//! code a ete lu avant d'etre retenu :
//!
//! - il telecharge le manifeste `latest.json` de la derniere release publiee.
//!   La requete ne porte que `User-Agent: tauri-plugin-updater/<version>` et
//!   `Accept: application/json` ; notre adresse etant fixe, aucune variable
//!   (`{{current_version}}`, `{{arch}}`...) n'y est substituee. Rien n'identifie
//!   la machine, pas meme la version installee — ce que la licence promet ;
//! - il telecharge l'installeur **en memoire**, verifie sa signature minisign
//!   avec la cle publique de `tauri.conf.json`, et seulement ensuite l'execute.
//!   Rien n'est ecrit ni lance avant la verification ;
//! - `requireSignedVersion` est active : la version annoncee par le manifeste,
//!   qui n'est pas signe, doit egaler celle inscrite dans la partie signee de la
//!   signature. Sans cela, une reponse falsifiee pourrait associer « 9.9.9 » a
//!   un ancien installeur authentique mais vulnerable. Verifie empiriquement :
//!   la CLI 2.11.5 inscrit bien `version:` dans le commentaire signe.
//!
//! Ce module ne fait qu'exposer ce greffon a l'interface, dans le respect du
//! §11 : detecter et proposer, rien d'installe sans un clic.
//!
//! **Les versions de test (1.7).** L'adresse du manifeste, `releases/latest`,
//! ignore les pre-versions : c'est voulu, elles ne doivent pas atteindre tout
//! le monde. Qui coche « Proposer les versions de test » interroge plutot la
//! liste des releases publiees (API de GitHub, sans compte : la requete ne
//! porte que l'en-tete User-Agent « WinTool ») et prend le manifeste de la plus
//! recente, pre-version ou non. La verification de la signature ne change pas :
//! une pre-version est signee avec la meme cle.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime, Url};
use tauri_plugin_updater::{Updater, UpdaterExt};

/// Le depot des releases de WinTool.
const DEPOT: &str = "burnout293/WinTool";

/// Une version plus recente, telle que l'interface la presente.
#[derive(Debug, Clone, Serialize)]
pub struct Disponible {
    pub version: String,
    pub actuelle: String,
    /// Notes de version, telles que publiees sur la release.
    pub notes: Option<String>,
    pub date: Option<String>,
}

/// Progression du telechargement, emise sur `update:progress`.
#[derive(Debug, Clone, Serialize)]
struct Progression {
    recu: u64,
    total: Option<u64>,
}

/// Politiques admises pour le reglage `update_policy`.
pub const POLITIQUES: [&str; 2] = ["propose", "never"];

pub fn politique_valide(p: &str) -> bool {
    POLITIQUES.contains(&p)
}

/// Dans une liste de releases (API de GitHub), l'adresse du `latest.json` de la
/// plus recente qui soit publiee. Les brouillons ne comptent pas : publier,
/// c'est livrer.
fn manifeste_le_plus_recent(releases: &serde_json::Value) -> Option<String> {
    releases.as_array()?.iter().find_map(|r| {
        if r["draft"].as_bool().unwrap_or(true) {
            return None;
        }
        r["assets"].as_array()?.iter().find_map(|a| {
            (a["name"] == "latest.json")
                .then(|| a["browser_download_url"].as_str().map(str::to_string))
                .flatten()
        })
    })
}

async fn manifeste_beta() -> Result<Option<Url>, String> {
    let reponse = crate::catalogue::client()?
        .get(format!(
            "https://api.github.com/repos/{DEPOT}/releases?per_page=10"
        ))
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !reponse.status().is_success() {
        return Err(format!("liste des versions : HTTP {}", reponse.status()));
    }
    let releases: serde_json::Value = reponse.json().await.map_err(|e| e.to_string())?;
    manifeste_le_plus_recent(&releases)
        .map(|u| Url::parse(&u).map_err(|e| e.to_string()))
        .transpose()
}

/// Le greffon, sur le bon canal : l'adresse de `tauri.conf.json`, ou le
/// manifeste de la release la plus recente si les versions de test sont
/// proposees.
async fn greffon<R: Runtime>(app: &AppHandle<R>, beta: bool) -> Result<Updater, String> {
    let mut b = app.updater_builder();
    if beta {
        if let Some(url) = manifeste_beta().await? {
            b = b.endpoints(vec![url]).map_err(|e| e.to_string())?;
        }
    }
    b.build().map_err(|e| e.to_string())
}

/// Interroge la derniere release publiee. `None` : WinTool est a jour.
pub async fn verifier<R: Runtime>(
    app: &AppHandle<R>,
    beta: bool,
) -> Result<Option<Disponible>, String> {
    let maj = greffon(app, beta)
        .await?
        .check()
        .await
        .map_err(|e| e.to_string())?;
    Ok(maj.map(|u| Disponible {
        version: u.version.clone(),
        actuelle: u.current_version.clone(),
        notes: u.body.clone(),
        date: u.date.map(|d| d.to_string()),
    }))
}

/// Telecharge, verifie et installe. Sous Windows, le greffon lance
/// l'installeur puis ferme l'application : cette fonction ne revient donc
/// qu'en cas d'echec.
///
/// On re-interroge le manifeste plutot que de reutiliser le resultat de
/// `verifier` : entre les deux, l'utilisateur a pu laisser l'application
/// ouverte des heures, et c'est la release publiee *maintenant* qu'on installe.
pub async fn installer<R: Runtime>(app: &AppHandle<R>, beta: bool) -> Result<(), String> {
    let maj = greffon(app, beta)
        .await?
        .check()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "WinTool est deja a jour.".to_string())?;

    let emetteur = app.clone();
    let mut recu: u64 = 0;
    maj.download_and_install(
        move |morceau, total| {
            recu += morceau as u64;
            let _ = emetteur.emit("update:progress", Progression { recu, total });
        },
        || {},
    )
    .await
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seules_les_politiques_connues_sont_admises() {
        assert!(politique_valide("propose"));
        assert!(politique_valide("never"));
        // L'interface n'en propose pas d'autre ; une valeur inconnue enregistree
        // serait lue au demarrage sans que personne ne sache quoi en faire.
        assert!(!politique_valide("auto"));
        assert!(!politique_valide(""));
        assert!(!politique_valide("Propose"));
    }

    #[test]
    fn le_canal_de_test_prend_la_release_publiee_la_plus_recente() {
        let r = serde_json::json!([
            { "draft": true, "assets": [{ "name": "latest.json", "browser_download_url": "https://x/brouillon" }] },
            { "draft": false, "assets": [{ "name": "notes.txt", "browser_download_url": "https://x/notes" }] },
            { "draft": false, "prerelease": true, "assets": [{ "name": "latest.json", "browser_download_url": "https://x/beta" }] },
            { "draft": false, "assets": [{ "name": "latest.json", "browser_download_url": "https://x/stable" }] }
        ]);
        assert_eq!(
            manifeste_le_plus_recent(&r).as_deref(),
            Some("https://x/beta")
        );
        assert_eq!(manifeste_le_plus_recent(&serde_json::json!([])), None);
    }
}
