use anyhow::{bail, Context, Result};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};
use zip::ZipArchive;

const MAX_ENTRY: u64 = 128 * 1024 * 1024;

/// Read-only PK3 mount. Later alphabetically sorted packs override earlier packs.
pub struct Assets {
    packs: Vec<ZipArchive<File>>,
    paths: Vec<PathBuf>,
    entries: BTreeMap<String, (usize, usize)>,
    pub base: PathBuf,
}

impl Assets {
    /// Stable identity of the mounted, override-resolved corpus, independent of its folder.
    /// Central-directory CRC/size metadata avoids decoding every asset at startup.
    pub fn fingerprint(&mut self) -> Result<String> {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        for (name, &(pack, index)) in &self.entries {
            let entry = self.packs[pack].by_index(index)?;
            hash.update((name.len() as u64).to_le_bytes());
            hash.update(name.as_bytes());
            hash.update(entry.size().to_le_bytes());
            hash.update(entry.crc32().to_le_bytes());
        }
        Ok(format!("{:x}", hash.finalize()))
    }
    pub fn open(base: &Path) -> Result<Self> {
        let paths = fs::read_dir(base)
            .with_context(|| {
                format!(
                    "Cannot read game data at {}. Pass --data <base folder>.",
                    base.display()
                )
            })?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|s| s.eq_ignore_ascii_case("pk3")))
            .collect::<Vec<_>>();
        Self::open_paths(base, paths)
    }

    fn open_paths(base: &Path, mut paths: Vec<PathBuf>) -> Result<Self> {
        paths.sort_by_key(|p| p.file_name().unwrap_or_default().to_ascii_lowercase());
        if paths.is_empty() {
            bail!("No PK3 archives found in {}", base.display());
        }
        let mut entries = BTreeMap::new();
        let mut packs = Vec::new();
        for path in &paths {
            let mut pack = ZipArchive::new(File::open(&path)?)
                .with_context(|| format!("Invalid PK3: {}", path.display()))?;
            let p = packs.len();
            for i in 0..pack.len() {
                let entry = pack.by_index(i)?;
                if !entry.is_dir() {
                    entries.insert(normalize(entry.name()), (p, i));
                }
            }
            packs.push(pack);
        }
        Ok(Self {
            packs,
            paths,
            entries,
            base: base.to_path_buf(),
        })
    }

    /// Saves identify core data independently of all known language packs.
    /// Reconstruct older identities with and without English, using the same
    /// core packs and available translations. Runtime mounting is unchanged.
    pub(crate) fn save_fingerprints(&mut self) -> Result<(String, Vec<String>)> {
        let core = self
            .paths
            .iter()
            .filter(|p| !is_language_pack(p))
            .cloned()
            .collect::<Vec<_>>();
        let identity = Self::open_paths(&self.base, core.clone())?.fingerprint()?;
        let mut legacy = vec![self.fingerprint()?];
        let mut languages = self
            .paths
            .iter()
            .filter(|p| is_language_pack(p))
            .cloned()
            .collect::<Vec<_>>();
        if let Some(managed) = self.base.ancestors().find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.eq_ignore_ascii_case("game-data"))
        }) {
            if let Ok(entries) = fs::read_dir(managed.join("languages")) {
                languages.extend(
                    entries
                        .filter_map(|e| e.ok().map(|e| e.path()))
                        .filter(|p| is_language_pack(p)),
                );
            }
        }
        let mut bases = vec![core.clone()];
        for english in languages.iter().filter(|p| is_english_pack(p)) {
            let mut paths = core.clone();
            paths.push(english.clone());
            // Includes the previous save identity that kept English in the core.
            if let Ok(fingerprint) = Self::open_paths(&self.base, paths.clone())
                .and_then(|mut mounted| mounted.fingerprint())
            {
                legacy.push(fingerprint);
                bases.push(paths);
            }
        }
        for language in languages.iter().filter(|p| !is_english_pack(p)) {
            for base in &bases {
                let mut paths = base.clone();
                paths.push(language.clone());
                // An unreadable inactive cache must not prevent launching.
                if let Ok(fingerprint) = Self::open_paths(&self.base, paths)
                    .and_then(|mut mounted| mounted.fingerprint())
                {
                    legacy.push(fingerprint);
                }
            }
        }
        legacy.sort();
        legacy.dedup();
        Ok((identity, legacy))
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }
    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(&normalize(name))
    }
    pub fn pack_count(&self) -> usize {
        self.packs.len()
    }
    pub fn read(&mut self, name: &str) -> Result<Vec<u8>> {
        let &(p, i) = self
            .entries
            .get(&normalize(name))
            .with_context(|| format!("Asset not found: {name}"))?;
        let entry = self.packs[p].by_index(i)?;
        if entry.size() > MAX_ENTRY {
            bail!("Asset exceeds 128 MiB limit: {name}");
        }
        let mut data = Vec::with_capacity(entry.size() as usize);
        entry
            .take(MAX_ENTRY + 1)
            .read_to_end(&mut data)
            .with_context(|| format!("Reading {name}"))?;
        if data.len() as u64 > MAX_ENTRY {
            bail!("Expanded asset exceeds limit: {name}");
        }
        Ok(data)
    }
    pub fn maps(&self) -> Vec<String> {
        self.names()
            .filter(|n| n.starts_with("maps/") && n.ends_with(".bsp"))
            .map(|n| {
                n.trim_start_matches("maps/")
                    .trim_end_matches(".bsp")
                    .to_owned()
            })
            .collect()
    }
}
fn normalize(s: &str) -> String {
    s.replace('\\', "/").to_ascii_lowercase()
}

fn is_language_pack(path: &Path) -> bool {
    matches!(path.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_ascii_lowercase().as_str(),
        "pak4_english.pk3" | "pak4_german.pk3" | "pak4_french.pk3" | "pak4_spanish.pk3" | "pak4_dutch.pk3" |
        "base_g.pk3" | "base_f.pk3" | "base_s.pk3" | "base_d.pk3")
}

fn is_english_pack(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case("pak4_english.pk3"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn language_switch_preserves_save_identity_and_alphabetical_precedence() -> Result<()> {
        let root = std::env::temp_dir().join(format!("looking-glass-language-{}", std::process::id()));
        let base = root.join("game-data/base");
        let cache = root.join("game-data/languages");
        fs::create_dir_all(&base).context("create test base")?;
        fs::create_dir_all(&cache).context("create test cache")?;
        let write = |path: &Path, entry: &str, text: &[u8]| -> Result<()> {
            let mut z = zip::ZipWriter::new(File::create(path)?);
            z.start_file(entry, zip::write::FileOptions::default())?;
            z.write_all(text)?;
            z.finish()?;
            Ok(())
        };
        write(&base.join("pak0.pk3"), "ui/main.urc", b"English")?;
        let core_id = Assets::open(&base)?.fingerprint()?;
        write(&base.join("pak4_german.pk3"), "ui/main.urc", b"German")?;
        let mut german = Assets::open(&base)?;
        let old_german = german.fingerprint()?;
        assert_eq!(german.read("ui/main.urc")?, b"German");
        assert_eq!(german.save_fingerprints()?.0, core_id);
        drop(german);
        fs::copy(base.join("pak4_german.pk3"), cache.join("pak4_german.pk3")).context("cache German test pack")?;
        fs::remove_file(base.join("pak4_german.pk3"))?;
        write(&base.join("pak4_french.pk3"), "ui/main.urc", b"French")?;
        let mut french = Assets::open(&base)?;
        let (id, legacy) = french.save_fingerprints()?;
        assert_eq!(id, core_id);
        assert!(legacy.contains(&old_german));
        assert_eq!(french.read("ui/main.urc")?, b"French");
        drop(french);
        write(&base.join("pak5_mod.pk3"), "ui/main.urc", b"Mod wins")?;
        let mut modified = Assets::open(&base)?;
        assert_eq!(modified.read("ui/main.urc")?, b"Mod wins");
        let (changed, legacy) = modified.save_fingerprints()?;
        assert_ne!(changed, core_id);
        assert!(!legacy.contains(&old_german));
        drop(modified);
        for path in [base.join("pak0.pk3"), base.join("pak4_french.pk3"), base.join("pak5_mod.pk3"), cache.join("pak4_german.pk3")] { fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?; }
        for path in [base, cache, root.join("game-data"), root] { fs::remove_dir(&path).with_context(|| format!("remove directory {}", path.display()))?; }
        Ok(())
    }
    #[test]
    fn later_pack_wins_and_paths_are_case_insensitive() -> Result<()> {
        let dir = std::env::temp_dir().join(format!("looking-glass-vfs-{}", std::process::id()));
        fs::create_dir_all(&dir)?;
        for (name, value) in [("pak0.pk3", b"old"), ("pak5.pk3", b"new")] {
            let mut z = zip::ZipWriter::new(File::create(dir.join(name))?);
            z.start_file("Textures/Test.ftx", zip::write::FileOptions::default())?;
            z.write_all(value)?;
            z.finish()?;
        }
        let mut a = Assets::open(&dir)?;
        assert_eq!(a.read("TEXTURES\\test.ftx")?, b"new");
        assert!(a.read("missing").is_err());
        drop(a);
        for name in ["pak0.pk3", "pak5.pk3"] {
            fs::remove_file(dir.join(name))?;
        }
        fs::remove_dir(dir)?;
        Ok(())
    }
}
