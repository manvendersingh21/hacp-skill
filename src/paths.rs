use anyhow::{Result, ensure};
use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Component, Path, PathBuf},
};

pub fn normalize(root: &Path, raw: &str) -> Result<String> {
    ensure!(
        !raw.is_empty() && !raw.ends_with('/') && !raw.contains(['*', '?', '[', ']', '\0']),
        "ownership requires a concrete file path: {raw:?}"
    );
    let p = Path::new(raw);
    ensure!(!p.is_absolute(), "path must be project-relative: {raw}");
    let mut clean = PathBuf::new();
    for c in p.components() {
        match c {
            Component::Normal(x) => clean.push(x),
            Component::CurDir => (),
            Component::ParentDir => {
                ensure!(clean.pop(), "path escapes project: {raw}");
            }
            _ => anyhow::bail!("invalid path: {raw}"),
        }
    }
    ensure!(!clean.as_os_str().is_empty(), "path must name a file");
    let resolved = resolve(&root.join(&clean))?;
    let relative = resolved
        .strip_prefix(root)
        .map_err(|_| anyhow::anyhow!("path escapes project through a filesystem alias: {raw}"))?;
    ensure!(
        !relative.starts_with(".hacp"),
        "path must not be inside .hacp"
    );
    if let Ok(meta) = fs::metadata(&resolved) {
        ensure!(meta.is_file(), "path must be a regular file: {raw}");
    }
    Ok(relative
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("path must be UTF-8"))?
        .to_string())
}
fn resolve(p: &Path) -> Result<PathBuf> {
    match fs::symlink_metadata(p) {
        Ok(_) => Ok(p.canonicalize()?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let parent = p
                .parent()
                .ok_or_else(|| anyhow::anyhow!("no existing path ancestor"))?;
            Ok(resolve(parent)?.join(
                p.file_name()
                    .ok_or_else(|| anyhow::anyhow!("missing file name"))?,
            ))
        }
        Err(e) => Err(e.into()),
    }
}
pub fn overlaps(root: &Path, a: &str, b: &str) -> Result<bool> {
    let a = normalize(root, a)?;
    let b = normalize(root, b)?;
    if a == b {
        return Ok(true);
    }
    // Existing hard links have distinct canonical paths but the same file identity.
    if let (Ok(x), Ok(y)) = (fs::metadata(root.join(&a)), fs::metadata(root.join(&b)))
        && x.dev() == y.dev()
        && x.ino() == y.ino()
    {
        return Ok(true);
    }
    // A future file cannot also be the ancestor directory of another output.
    Ok(Path::new(&a).starts_with(&b) || Path::new(&b).starts_with(&a))
}
pub fn list(root: &Path, paths: &[String]) -> Result<Vec<String>> {
    let mut out: Vec<String> = vec![];
    for p in paths {
        let p = normalize(root, p)?;
        for q in &out {
            ensure!(!overlaps(root, &p, q)?, "duplicate/aliased output: {p}");
        }
        out.push(p);
    }
    out.sort();
    Ok(out)
}
pub fn disjoint(root: &Path, left: &[String], right: &[String]) -> Result<()> {
    for a in left {
        for b in right {
            ensure!(
                !overlaps(root, a, b)?,
                "ownership conflict: {a} overlaps {b}"
            );
        }
    }
    Ok(())
}
