pub mod format;
pub mod undo;

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::error::TrkError;
use crate::model::Doc;

pub struct Store {
    pub path: PathBuf,
}

impl Store {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn lock_path(&self) -> PathBuf {
        let mut os = self.path.clone().into_os_string();
        os.push(".lock");
        PathBuf::from(os)
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    fn ensure_parent(&self) -> Result<(), TrkError> {
        if let Some(parent) = self.path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Read without taking the lock (read-only commands, spec 4.5).
    pub fn read(&self) -> Result<Doc, TrkError> {
        match fs::read_to_string(&self.path) {
            Ok(text) => Ok(format::parse(&text)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Doc::empty()),
            Err(e) => Err(e.into()),
        }
    }

    /// Take the exclusive lock with a 2 second timeout (spec 4.5, 6.7).
    pub fn lock(&self) -> Result<Lock, TrkError> {
        self.ensure_parent()?;
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(self.lock_path())?;
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(Lock { file }),
                Err(TryLockError::WouldBlock) => {
                    if Instant::now() >= deadline {
                        return Err(TrkError::StoreLocked);
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
                Err(TryLockError::Error(e)) => return Err(e.into()),
            }
        }
    }

    /// Write the store atomically (spec 6.5).
    pub fn save(&self, doc: &Doc) -> Result<(), TrkError> {
        self.ensure_parent()?;
        let text = format::serialize(doc);
        write_atomic(&self.path, text.as_bytes())
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), TrkError> {
    let mut os = path.as_os_str().to_os_string();
    os.push(".tmp");
    let tmp = PathBuf::from(os);
    {
        let mut file = File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Advisory exclusive lock held for a whole mutation (spec 4.5, 6.7).
pub struct Lock {
    file: File,
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}
