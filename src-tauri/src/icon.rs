//! The real icon of an application found by an analysis (2.0.3, §17).
//!
//! A script names a file of the system with `[ITEM] … exe=<path>`; Sindry
//! reads its icon here and attaches it to the item as a `data:` URL. The
//! script never sends an image, and nothing is downloaded: asking a service
//! for the icon would hand it the list of the user's programs.
//!
//! Sindry runs elevated and the file may be planted, so this module trusts
//! nothing in it:
//!
//! - **No Windows loader, no shell.** `ExtractIconEx`, `LoadLibraryEx` as a
//!   data file, `SHGetFileInfo`… all parse the file inside this process, and
//!   the shell ones may load third-party icon handlers. The resource tree is
//!   read here instead, with bounds checked at every step; no code from the
//!   file ever runs.
//! - **Local paths only.** A UNC path (`\\server\share\…`) would make the
//!   elevated process open an SMB connection to a server of the script's
//!   choosing, and leak the account's NTLM hash. Device paths (`\\?\`,
//!   `\\.\`), alternate streams, relative paths and shortcuts are refused too.
//! - **Small output.** One image, the size nearest 32 px, 64 KB at most,
//!   rebuilt into a one-entry `.ico` that WebView2 displays as is.

use base64::Engine;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

/// Largest image kept, in bytes.
const MAX_IMAGE: usize = 64 * 1024;
/// Largest resource section read.
const MAX_SECTION: usize = 32 * 1024 * 1024;
/// Largest `.ico` file read.
const MAX_ICO: u64 = 4 * 1024 * 1024;
/// Entries read per resource directory: a real one has a few dozen.
const MAX_ENTRIES: usize = 4096;
/// The size the item rows want: 20 px, up to 1.5× scaling.
const TARGET: u32 = 32;

const RT_ICON: u32 = 3;
const RT_GROUP_ICON: u32 = 14;

/// The path, if it is an absolute path on a local drive to an `.exe`, a
/// `.dll` or an `.ico`. Forward slashes count as backslashes.
pub fn local_path(raw: &str) -> Option<PathBuf> {
    let p = raw.trim().replace('/', "\\");
    let b = p.as_bytes();
    let drive = b.len() > 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && b[2] == b'\\';
    // `C:\a\b.exe:stream`, or any second colon: an alternate data stream.
    if !drive || p[2..].contains(':') || p.contains("\\\\") {
        return None;
    }
    if p.split('\\').any(|part| part == "..") || p.chars().any(char::is_control) {
        return None;
    }
    // `CON.exe`, `COM1.dll`…: a device, not a file. Opening one may block.
    let stem = p
        .rsplit('\\')
        .next()?
        .split('.')
        .next()?
        .trim_end()
        .to_ascii_uppercase();
    let device = ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"].contains(&stem.as_str())
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem.as_bytes()[3].is_ascii_digit());
    if device {
        return None;
    }
    let ext = p.rsplit('.').next()?.to_ascii_lowercase();
    ["exe", "dll", "ico"]
        .contains(&ext.as_str())
        .then(|| PathBuf::from(p))
}

/// The icon of the file at `raw`, as a `data:image/x-icon` URL; `None` when
/// the path is refused, the file absent, or no icon could be read.
pub fn data_url(raw: &str) -> Option<String> {
    let path = local_path(raw)?;
    let mut f = File::open(&path).ok()?;
    if !f.metadata().ok()?.is_file() {
        return None;
    }
    let is_ico = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("ico"));
    let ico = if is_ico {
        from_ico_file(&mut f)?
    } else {
        from_pe(&mut f).or_else(|| from_system_resources(&path))?
    };
    Some(format!(
        "data:image/x-icon;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(ico)
    ))
}

/// Since Windows 10 1903, many system files (notepad.exe, shell32.dll…) keep
/// their icons apart, in `<Windows>\SystemResources\<name>.mun`, where the
/// loader finds them. Only for a file under the Windows folder, read from
/// HKLM (`systeme.rs`): a program elsewhere that happens to be called
/// notepad.exe must not borrow the system's icon.
fn from_system_resources(path: &std::path::Path) -> Option<Vec<u8>> {
    let windows = crate::systeme::emplacements()
        .windows
        .to_string_lossy()
        .to_ascii_lowercase();
    let lower = path.to_string_lossy().to_ascii_lowercase();
    if !lower.starts_with(&format!("{}\\", windows.trim_end_matches('\\'))) {
        return None;
    }
    let name = path.file_name()?.to_string_lossy().into_owned();
    let mun = crate::systeme::emplacements()
        .windows
        .join("SystemResources")
        .join(format!("{name}.mun"));
    let mut f = File::open(mun).ok()?;
    from_pe(&mut f)
}

// ---------------------------------------------------------------------------
// Bounded reads
// ---------------------------------------------------------------------------

fn u16_at(b: &[u8], off: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(off..off + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], off: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(off..off + 4)?.try_into().ok()?))
}

fn read_at(f: &mut File, off: u64, len: usize) -> Option<Vec<u8>> {
    f.seek(SeekFrom::Start(off)).ok()?;
    let mut buf = vec![0; len];
    f.read_exact(&mut buf).ok()?;
    Some(buf)
}

// ---------------------------------------------------------------------------
// Choosing an image and rebuilding the .ico
// ---------------------------------------------------------------------------

/// One image of an icon: its size, colour depth and bytes.
struct Image {
    width: u32,
    height_byte: u8,
    colors: u8,
    planes: u16,
    bits: u16,
    data: Vec<u8>,
}

/// The candidate nearest [`TARGET`] (larger rather than smaller, then the
/// deepest colours) among those small enough to keep.
fn best<T>(candidates: Vec<T>, describe: impl Fn(&T) -> (u32, u16, usize)) -> Option<T> {
    candidates
        .into_iter()
        .filter(|c| (1..=MAX_IMAGE).contains(&describe(c).2))
        .min_by_key(|c| {
            let (w, bits, _) = describe(c);
            let distance = if w >= TARGET {
                w - TARGET
            } else {
                (TARGET - w) * 4
            };
            (distance, u16::MAX - bits)
        })
}

/// A PNG, or a DIB as icons store them (a 40-byte BITMAPINFOHEADER).
fn looks_like_image(data: &[u8]) -> bool {
    data.starts_with(b"\x89PNG\r\n\x1a\n") || u32_at(data, 0) == Some(40)
}

fn rebuild(img: &Image) -> Option<Vec<u8>> {
    if !looks_like_image(&img.data) {
        return None;
    }
    let mut out = Vec::with_capacity(22 + img.data.len());
    out.extend_from_slice(&[0, 0, 1, 0, 1, 0]);
    out.push(if img.width >= 256 { 0 } else { img.width as u8 });
    out.push(img.height_byte);
    out.push(img.colors);
    out.push(0);
    out.extend_from_slice(&img.planes.to_le_bytes());
    out.extend_from_slice(&img.bits.to_le_bytes());
    out.extend_from_slice(&(img.data.len() as u32).to_le_bytes());
    out.extend_from_slice(&22u32.to_le_bytes());
    out.extend_from_slice(&img.data);
    Some(out)
}

fn width_of(byte: u8) -> u32 {
    if byte == 0 {
        256
    } else {
        byte as u32
    }
}

// ---------------------------------------------------------------------------
// .ico files
// ---------------------------------------------------------------------------

fn from_ico_file(f: &mut File) -> Option<Vec<u8>> {
    let len = f.metadata().ok()?.len();
    if len > MAX_ICO {
        return None;
    }
    let bytes = read_at(f, 0, len as usize)?;
    from_ico_bytes(&bytes)
}

fn from_ico_bytes(b: &[u8]) -> Option<Vec<u8>> {
    if u16_at(b, 0)? != 0 || u16_at(b, 2)? != 1 {
        return None;
    }
    let count = (u16_at(b, 4)? as usize).min(MAX_ENTRIES);
    let mut entries = Vec::new();
    for i in 0..count {
        let e = 6 + i * 16;
        let head = b.get(e..e + 16)?;
        entries.push((
            head[0],
            head[1],
            head[2],
            u16_at(head, 4)?,
            u16_at(head, 6)?,
            u32_at(head, 8)? as usize,
            u32_at(head, 12)? as usize,
        ));
    }
    let (w, h, colors, planes, bits, size, offset) =
        best(entries, |&(w, _, _, _, bits, size, _)| {
            (width_of(w), bits, size)
        })?;
    rebuild(&Image {
        width: width_of(w),
        height_byte: h,
        colors,
        planes,
        bits,
        data: b.get(offset..offset.checked_add(size)?)?.to_vec(),
    })
}

// ---------------------------------------------------------------------------
// Executables and libraries: RT_GROUP_ICON, then the RT_ICON it names
// ---------------------------------------------------------------------------

/// The resource section of a PE file, and where in it the resource tree
/// starts. Data entries give RVAs: `va` turns them into buffer offsets.
struct Resources {
    buf: Vec<u8>,
    root: usize,
    va: u32,
}

impl Resources {
    /// The entries of the directory at `off` (relative to the tree root):
    /// (name or id, is a name, offset, is a subdirectory).
    fn entries(&self, off: usize) -> Option<Vec<(u32, bool, usize, bool)>> {
        let d = self.root.checked_add(off)?;
        let n = u16_at(&self.buf, d + 12)? as usize + u16_at(&self.buf, d + 14)? as usize;
        let mut out = Vec::new();
        for i in 0..n.min(MAX_ENTRIES) {
            let e = d + 16 + i * 8;
            let name = u32_at(&self.buf, e)?;
            let target = u32_at(&self.buf, e + 4)?;
            out.push((
                name & 0x7FFF_FFFF,
                name & 0x8000_0000 != 0,
                (target & 0x7FFF_FFFF) as usize,
                target & 0x8000_0000 != 0,
            ));
        }
        Some(out)
    }

    /// The bytes of a leaf: down through the first language of `off`.
    fn first_data(&self, off: usize, is_dir: bool) -> Option<&[u8]> {
        let (_, _, leaf, leaf_is_dir) = if is_dir {
            *self.entries(off)?.first()?
        } else {
            (0, false, off, false)
        };
        if leaf_is_dir {
            return None;
        }
        let e = self.root.checked_add(leaf)?;
        let rva = u32_at(&self.buf, e)?;
        let size = u32_at(&self.buf, e + 4)? as usize;
        let start = rva.checked_sub(self.va)? as usize;
        self.buf.get(start..start.checked_add(size)?)
    }
}

fn from_pe(f: &mut File) -> Option<Vec<u8>> {
    let res = resources(f)?;
    let types = res.entries(0)?;
    let dir_of = |t: u32| {
        types
            .iter()
            .find(|&&(id, named, _, dir)| !named && id == t && dir)
            .map(|&(_, _, off, _)| off)
    };
    let groups = res.entries(dir_of(RT_GROUP_ICON)?)?;
    let icons = res.entries(dir_of(RT_ICON)?)?;

    // The first group is the one Explorer shows for the file.
    let &(_, _, group_off, group_dir) = groups.first()?;
    let g = res.first_data(group_off, group_dir)?;
    if u16_at(g, 0)? != 0 || u16_at(g, 2)? != 1 {
        return None;
    }
    let count = (u16_at(g, 4)? as usize).min(MAX_ENTRIES);
    let mut entries = Vec::new();
    for i in 0..count {
        let e = 6 + i * 14;
        entries.push((
            *g.get(e)?,
            *g.get(e + 1)?,
            *g.get(e + 2)?,
            u16_at(g, e + 4)?,
            u16_at(g, e + 6)?,
            u32_at(g, e + 8)? as usize,
            u16_at(g, e + 12)? as u32,
        ));
    }
    let (w, h, colors, planes, bits, _, id) = best(entries, |&(w, _, _, _, bits, size, _)| {
        (width_of(w), bits, size)
    })?;
    let &(_, _, icon_off, icon_dir) = icons.iter().find(|&&(i, named, _, _)| !named && i == id)?;
    let data = res.first_data(icon_off, icon_dir)?;
    if data.len() > MAX_IMAGE {
        return None;
    }
    rebuild(&Image {
        width: width_of(w),
        height_byte: h,
        colors,
        planes,
        bits,
        data: data.to_vec(),
    })
}

/// Reads the headers, then only the section holding the resource directory.
fn resources(f: &mut File) -> Option<Resources> {
    let len = f.metadata().ok()?.len();
    let head = read_at(f, 0, len.min(4096) as usize)?;
    if !head.starts_with(b"MZ") {
        return None;
    }
    let pe = u32_at(&head, 0x3C)? as usize;
    if pe > 4096 - 24 || head.get(pe..pe + 4)? != b"PE\0\0" {
        return None;
    }
    let coff = pe + 4;
    let sections = u16_at(&head, coff + 2)? as usize;
    let opt_size = u16_at(&head, coff + 16)? as usize;
    let opt = coff + 20;
    let (dirs, count_at) = match u16_at(&head, opt)? {
        0x10b => (opt + 96, opt + 92),
        0x20b => (opt + 112, opt + 108),
        _ => return None,
    };
    if u32_at(&head, count_at)? < 3 {
        return None;
    }
    let rva = u32_at(&head, dirs + 16)?;
    if rva == 0 {
        return None;
    }

    // The section table may run past the first 4 KB on odd files.
    let table = opt.checked_add(opt_size)?;
    let need = table.checked_add(sections.min(96) * 40)?;
    let head = if need > head.len() {
        if need as u64 > len || need > 64 * 1024 {
            return None;
        }
        read_at(f, 0, need)?
    } else {
        head
    };
    for i in 0..sections.min(96) {
        let s = table + i * 40;
        let vsize = u32_at(&head, s + 8)?;
        let va = u32_at(&head, s + 12)?;
        let raw_size = u32_at(&head, s + 16)? as usize;
        let raw_at = u32_at(&head, s + 20)? as u64;
        let span = vsize.max(raw_size as u32);
        if rva < va || rva - va >= span {
            continue;
        }
        let size = raw_size.min(MAX_SECTION);
        if raw_at.checked_add(size as u64)? > len {
            return None;
        }
        return Some(Resources {
            buf: read_at(f, raw_at, size)?,
            root: (rva - va) as usize,
            va,
        });
    }
    None
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_local_paths_to_exe_dll_or_ico_are_read() {
        for ok in [
            r"C:\Program Files\App\app.exe",
            r"D:\Tools\lib.DLL",
            r"c:\icons\a.ico",
            "C:/Program Files/App/app.exe",
        ] {
            assert!(local_path(ok).is_some(), "{ok}");
        }
        for ko in [
            r"\\server\share\app.exe",
            "//server/share/app.exe",
            r"\\?\C:\app.exe",
            r"\\.\pipe\app.exe",
            r"app.exe",
            r"Program Files\app.exe",
            r"C:app.exe",
            r"C:\app.exe:stream",
            r"C:\x\..\Windows\app.exe",
            r"C:\x\app.lnk",
            r"C:\x\app.exe.txt",
            r"C:\x\app",
            "C:\\x\\a\u{7}.exe",
            r"C:\x\CON.exe",
            r"C:\x\com1.dll",
            r"C:\x\nul.tar.ico",
            "",
        ] {
            assert!(local_path(ko).is_none(), "{ko}");
        }
    }

    /// A one-image icon file, built by hand: a 16 px DIB of 32 bits.
    fn ico_with(images: &[(u8, u16, Vec<u8>)]) -> Vec<u8> {
        let mut out = vec![0, 0, 1, 0, images.len() as u8, 0];
        let mut offset = 6 + 16 * images.len() as u32;
        for (w, bits, data) in images {
            out.extend_from_slice(&[*w, *w, 0, 0, 1, 0]);
            out.extend_from_slice(&bits.to_le_bytes());
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&offset.to_le_bytes());
            offset += data.len() as u32;
        }
        for (_, _, data) in images {
            out.extend_from_slice(data);
        }
        out
    }

    fn dib(fill: u8) -> Vec<u8> {
        let mut d = 40u32.to_le_bytes().to_vec();
        d.resize(200, fill);
        d
    }

    #[test]
    fn an_icon_file_keeps_the_image_nearest_32_pixels() {
        let file = ico_with(&[
            (16, 32, dib(1)),
            (32, 8, dib(2)),
            (32, 32, dib(3)),
            (48, 32, dib(4)),
        ]);
        let out = from_ico_bytes(&file).unwrap();
        assert_eq!(&out[..6], &[0, 0, 1, 0, 1, 0], "one entry");
        assert_eq!(out[6], 32, "32 px");
        assert_eq!(u16_at(&out, 12), Some(32), "the deepest colours");
        assert_eq!(out[22 + 40], 3);

        // Neither a PNG nor a DIB: refused, not handed to the web view.
        let mut junk = dib(0);
        junk[0] = 7;
        assert!(from_ico_bytes(&ico_with(&[(32, 32, junk)])).is_none());
        // An offset past the end, a truncated header: nothing, no panic.
        let mut broken = ico_with(&[(32, 32, dib(1))]);
        broken[18] = 0xFF;
        assert!(from_ico_bytes(&broken).is_none());
        assert!(from_ico_bytes(&file[..10]).is_none());
    }

    #[cfg(windows)]
    #[test]
    fn the_icon_of_explorer_is_read_and_a_damaged_copy_never_panics() {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
        let explorer = format!(r"{windir}\explorer.exe");
        let url = data_url(&explorer).expect("explorer.exe has an icon");
        assert!(url.starts_with("data:image/x-icon;base64,"));
        assert!(url.len() < MAX_IMAGE * 2, "one small image");

        let bytes = std::fs::read(&explorer).unwrap();
        let dir = std::env::temp_dir().join(format!("sindry-icon-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let copy = dir.join("damaged.exe");
        for cut in [
            64,
            600,
            4096,
            bytes.len() / 3,
            bytes.len() / 2,
            bytes.len() - 1,
        ] {
            std::fs::write(&copy, &bytes[..cut]).unwrap();
            let _ = data_url(copy.to_str().unwrap());
        }
        // Flipped bytes all over the headers and the resource tree.
        let mut flipped = bytes.clone();
        for i in (0..flipped.len()).step_by(997) {
            flipped[i] ^= 0xA5;
        }
        std::fs::write(&copy, &flipped).unwrap();
        let _ = data_url(copy.to_str().unwrap());
        std::fs::write(&copy, b"MZ not really a program").unwrap();
        assert!(data_url(copy.to_str().unwrap()).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
