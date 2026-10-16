use crate::{Project, document::MAX_BYTES};
use anyhow::{Context, Result, ensure};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub fn read(path: &Path) -> Result<Vec<u8>> {
    let mut b = vec![];
    std::fs::File::open(path)?
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut b)?;
    ensure!(b.len() <= MAX_BYTES, "Project exceeds 32 MB");
    Ok(b)
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(bytes)?;
    file.as_file().sync_all()?;
    file.persist(path)
        .map_err(|e| e.error)
        .context("Could not replace destination file")?;
    Ok(())
}
#[derive(Default)]
pub struct DocumentFile {
    pub path: Option<PathBuf>,
    baseline: Option<Vec<u8>>,
}
impl DocumentFile {
    pub fn open(path: PathBuf) -> Result<(Self, Project)> {
        let b = read(&path)?;
        let p = Project::parse(&b)?;
        Ok((
            Self {
                path: Some(path),
                baseline: Some(b),
            },
            p,
        ))
    }
    pub fn save(&mut self, p: &Project, new_path: Option<PathBuf>) -> Result<()> {
        let path = new_path
            .as_ref()
            .or(self.path.as_ref())
            .context("Choose a file path")?;
        if new_path.is_none()
            && let Some(expected) = &self.baseline
        {
            ensure!(
                &read(path)? == expected,
                "File changed outside TinkerCraft. Use Save As."
            );
        }
        let b = p.bytes()?;
        atomic_write(path, &b)?;
        self.path = Some(path.clone());
        self.baseline = Some(b);
        Ok(())
    }
}
