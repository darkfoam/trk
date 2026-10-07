use std::fs;
use std::path::PathBuf;

use crate::error::TrkError;

/// The undo ring: whole-file snapshots in the backups directory (spec 6.6).
pub struct UndoRing {
    dir: PathBuf,
    depth: usize,
}

impl UndoRing {
    pub fn new(dir: impl Into<PathBuf>, depth: usize) -> Self {
        Self {
            dir: dir.into(),
            depth: depth.max(1),
        }
    }

    fn seq_files(&self) -> Result<Vec<(usize, PathBuf)>, TrkError> {
        let mut files = Vec::new();
        let entries = match fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(files),
            Err(e) => return Err(e.into()),
        };
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("trk") {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                && let Ok(seq) = stem.parse::<usize>()
            {
                files.push((seq, path));
            }
        }
        files.sort_by_key(|(seq, _)| *seq);
        Ok(files)
    }

    /// Copy the pre-change store into the ring, then trim to `depth`.
    pub fn snapshot(&self, before_text: &str, label: &str) -> Result<(), TrkError> {
        fs::create_dir_all(&self.dir)?;
        let files = self.seq_files()?;
        let next = files.last().map(|(s, _)| s + 1).unwrap_or(1);
        let path = self.dir.join(format!("{next:06}.trk"));
        let content = format!("# undo-label: {label}\n{before_text}");
        fs::write(&path, content)?;

        let files = self.seq_files()?;
        if files.len() > self.depth {
            let remove = files.len() - self.depth;
            for (_, old) in files.into_iter().take(remove) {
                let _ = fs::remove_file(old);
            }
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.seq_files().map(|f| f.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Restore the `n`th newest snapshot, discarding newer ones. Returns the
    /// label and store text, or `None` when the ring is empty.
    pub fn pop(&self, n: usize) -> Result<Option<(String, String)>, TrkError> {
        let files = self.seq_files()?;
        if files.is_empty() {
            return Ok(None);
        }
        if n == 0 || n > files.len() {
            return Err(TrkError::Message(format!(
                "undo: only {} snapshot(s) available",
                files.len()
            )));
        }
        let target = files.len() - n;
        let (_, target_path) = &files[target];
        let raw = fs::read_to_string(target_path)?;
        let (label, text) = split_label(&raw);
        for (_, path) in files.iter().skip(target) {
            let _ = fs::remove_file(path);
        }
        Ok(Some((label, text)))
    }
}

fn split_label(raw: &str) -> (String, String) {
    if let Some(first) = raw.lines().next()
        && let Some(label) = first.strip_prefix("# undo-label: ")
    {
        let rest = raw
            .split_once('\n')
            .map(|(_, r)| r.to_string())
            .unwrap_or_default();
        return (label.to_string(), rest);
    }
    (String::new(), raw.to_string())
}
