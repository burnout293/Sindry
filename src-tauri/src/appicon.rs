//! Application icons from the Sindry-Icons repository (2.0.4, §17.4).
//!
//! A script names an icon with `[ITEM] … appicon=<name>`; Sindry downloads
//! `<name>` from **one fixed place**, the `icons/` folder of the
//! `burnout293/Sindry-Icons` repository, and shows it as a `data:` URL. The
//! script never gives an address, so it can neither make Sindry contact a
//! server of its choosing (a tracker, a machine of the local network) nor
//! send an image of its own.
//!
//! - HTTPS only, no redirection followed, no identifier sent (the catalogue's
//!   client, `catalogue::client`).
//! - Every icon an analysis names is fetched, whatever the state of its item:
//!   the server must not learn which applications are installed.
//! - A PNG or an ICO, recognised by its first bytes; anything else — SVG
//!   included — is refused. 128 KB at most.
//! - Kept on disk for 30 days (`%LOCALAPPDATA%\Sindry\cache\icons`), and
//!   checked again each time it is read: that folder belongs to the user.
//! - The user can turn it off (§8, 3.3): then nothing leaves the PC.

use base64::Engine;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

/// Where the icons live, and nowhere else: `burnout293/Sindry-Icons`.
const BASE: &str = "https://raw.githubusercontent.com/burnout293/Sindry-Icons/main/icons/";
const MAX_BYTES: u64 = 128 * 1024;
/// Names read per analysis; a real list has a few dozen.
const MAX_NAMES: usize = 400;
/// Downloads at once.
const AT_ONCE: usize = 6;
const KEEP: Duration = Duration::from_secs(30 * 24 * 3600);

/// An icon name: lowercase letters, digits and inner dashes, 64 at most —
/// it becomes a file name and the end of a URL.
pub fn is_name(s: &str) -> bool {
    (1..=64).contains(&s.len())
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
}

/// The media type of an icon, from its first bytes; `None` for anything that
/// is neither a PNG nor an ICO.
fn media_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.len() > 22 && bytes[..4] == [0, 0, 1, 0] {
        Some("image/x-icon")
    } else {
        None
    }
}

/// The `data:` URL of an icon, if its bytes are one Sindry accepts.
pub fn data_url(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_BYTES {
        return None;
    }
    let kind = media_type(bytes)?;
    Some(format!(
        "data:{kind};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

/// The icon kept on disk, if it is recent, a plain file, and still an icon.
fn cached(dir: &Path, name: &str) -> Option<Vec<u8>> {
    let path = dir.join(format!("{name}.icon"));
    let meta = std::fs::symlink_metadata(&path).ok()?;
    if !meta.is_file() || meta.len() > MAX_BYTES {
        return None;
    }
    let age = SystemTime::now()
        .duration_since(meta.modified().ok()?)
        .unwrap_or_default();
    if age > KEEP {
        return None;
    }
    let bytes = std::fs::read(&path).ok()?;
    media_type(&bytes)?;
    Some(bytes)
}

/// What this session already knows: an icon, or `None` for a name the
/// repository does not have (asked once, not at every analysis).
fn known() -> &'static Mutex<HashMap<String, Option<String>>> {
    static KNOWN: std::sync::OnceLock<Mutex<HashMap<String, Option<String>>>> =
        std::sync::OnceLock::new();
    KNOWN.get_or_init(|| Mutex::new(HashMap::new()))
}

/// The catalogue's HTTPS client (`catalogue::client`), but following no
/// redirection: the icon comes from the repository's address, or not at all.
fn client() -> Option<reqwest::Client> {
    crate::catalogue::client().ok()?;
    reqwest::Client::builder()
        .user_agent("Sindry")
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
        .build()
        .ok()
}

async fn download(client: &reqwest::Client, name: &str) -> Option<Vec<u8>> {
    let mut response = client.get(format!("{BASE}{name}.png")).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    if response.content_length().is_some_and(|n| n > MAX_BYTES) {
        return None;
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.ok()? {
        if (bytes.len() + chunk.len()) as u64 > MAX_BYTES {
            return None;
        }
        bytes.extend_from_slice(&chunk);
    }
    media_type(&bytes)?;
    Some(bytes)
}

/// Fetches the icons `names`, from the session, the disk or the repository,
/// and hands each one to `found` as soon as it is known — the screen is
/// already shown, the icons arrive one by one. `dir` is the disk cache.
pub async fn fetch(
    dir: PathBuf,
    names: Vec<String>,
    found: impl Fn(&str, Option<String>) + Send + Sync + 'static,
) {
    let mut names: Vec<String> = names.into_iter().filter(|n| is_name(n)).collect();
    names.sort();
    names.dedup();
    names.truncate(MAX_NAMES);

    let mut to_download = Vec::new();
    for name in names {
        if let Some(url) = known().lock().ok().and_then(|k| k.get(&name).cloned()) {
            found(&name, url);
            continue;
        }
        match cached(&dir, &name).and_then(|b| data_url(&b)) {
            Some(url) => {
                remember(&name, Some(url.clone()));
                found(&name, Some(url));
            }
            None => to_download.push(name),
        }
    }
    if to_download.is_empty() {
        return;
    }
    let Some(client) = client() else {
        return;
    };
    let _ = std::fs::create_dir_all(&dir);
    let found = std::sync::Arc::new(found);
    for batch in to_download.chunks(AT_ONCE) {
        let tasks: Vec<_> = batch
            .iter()
            .map(|name| {
                let (client, name, dir, found) =
                    (client.clone(), name.clone(), dir.clone(), found.clone());
                tauri::async_runtime::spawn(async move {
                    let bytes = download(&client, &name).await;
                    let url = bytes.as_deref().and_then(data_url);
                    if let Some(b) = &bytes {
                        let _ = std::fs::write(dir.join(format!("{name}.icon")), b);
                    }
                    remember(&name, url.clone());
                    found(&name, url);
                })
            })
            .collect();
        for t in tasks {
            let _ = t.await;
        }
    }
}

fn remember(name: &str, url: Option<String>) {
    if let Ok(mut k) = known().lock() {
        k.insert(name.to_string(), url);
    }
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_a_plain_token() {
        for ok in ["firefox", "vlc", "7-zip", "notepad-plus-plus", "a"] {
            assert!(is_name(ok), "{ok}");
        }
        let long = "a".repeat(65);
        for ko in [
            "",
            "Firefox",
            "fire fox",
            "../x",
            "a/b",
            "-x",
            "x-",
            "x.png",
            long.as_str(),
        ] {
            assert!(!is_name(ko), "{ko}");
        }
    }

    #[test]
    fn only_a_png_or_an_ico_becomes_an_image() {
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        png.extend_from_slice(&[0; 40]);
        assert!(data_url(&png).is_some_and(|u| u.starts_with("data:image/png;base64,")));
        let mut ico = vec![0, 0, 1, 0, 1, 0];
        ico.extend_from_slice(&[0; 40]);
        assert!(data_url(&ico).is_some_and(|u| u.starts_with("data:image/x-icon;base64,")));

        let svg = b"<svg xmlns='http://www.w3.org/2000/svg'><script>alert(1)</script></svg>";
        assert!(data_url(svg).is_none(), "never an SVG");
        assert!(data_url(b"GIF89a").is_none());
        assert!(data_url(&[]).is_none());
        let mut big = png.clone();
        big.resize(MAX_BYTES as usize + 1, 0);
        assert!(data_url(&big).is_none(), "too big");
    }

    #[test]
    fn the_disk_cache_is_checked_each_time_it_is_read() {
        let dir = std::env::temp_dir().join(format!("sindry-appicon-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        png.extend_from_slice(&[7; 40]);
        std::fs::write(dir.join("good.icon"), &png).unwrap();
        std::fs::write(dir.join("bad.icon"), b"<svg/>").unwrap();
        assert_eq!(cached(&dir, "good"), Some(png));
        assert!(
            cached(&dir, "bad").is_none(),
            "a planted file is not an icon"
        );
        assert!(cached(&dir, "absent").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
