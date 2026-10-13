//! Concrete `FileCredentialStore`: JSON credential persistence at one
//! configured path.
//!
//! Durability: `save` writes a fully-serialized temp file in the SAME
//! directory and atomically renames it onto the target; on Windows the
//! rename-over-existing step fails with AlreadyExists and the store
//! falls back to remove + rename (the only std-only portable
//! replacement). An interrupted save can therefore leave a stale temp
//! file (removed by the next save) but never a truncated or
//! half-written credential file. The Windows remove + rename fallback
//! arm cannot execute in the Linux verification container (POSIX rename
//! replaces in place), so only the POSIX arm is covered by automated
//! tests; the Windows arm is a reviewed-but-untested path.
//!
//! Permissions: temp files are created with Unix mode 0600, so every
//! credential file lands with owner-only access on Linux/macOS.
//! Windows file ACLs are NOT hardened by this store (std provides no
//! ACL management): the file inherits the directory's ACL restrictions.
//! This is a documented limitation, not encryption; the credential file
//! is no more protected than its containing directory's defaults.
//!
//! Redaction: parse failures report fixed descriptive messages (never
//! `serde_json`'s value-quoting text) and errors always name the
//! operator-facing target path, so no secret value can reach a log or
//! an error line; the store derives `Debug` over only its `path` (no
//! token material) and emits no tracing events, so failure reporting is
//! the returned typed error only.
//!
//! Schema: the on-disk JSON field names and load/save conversions live
//! in the sibling `persisted_credentials` module; this file owns file IO
//! (temp creation, permissions, rename) and io-error mapping only.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::credentials::{CredentialStore, StoredCredentials};
use super::error::CredentialStoreError;
use super::persisted_credentials::{parse_credentials, serialized_document};

/// Concrete credential store persisting JSON token data at one file
/// path. Atomic writes (temp file + same-directory rename), Unix
/// permission 0600; see the module docs for the Windows ACL
/// limitation.
#[derive(Debug)]
pub(crate) struct FileCredentialStore {
    path: PathBuf,
}

impl FileCredentialStore {
    /// Store targeting the given credential-file path (typically the
    /// configured `oauth.credentials_path`).
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn ensure_parent_directory(&self) -> Result<(), CredentialStoreError> {
        fs::create_dir_all(self.parent_directory()).map_err(|source| {
            CredentialStoreError::FileOpen {
                path: self.path.clone(),
                source,
            }
        })
    }

    fn parent_directory(&self) -> PathBuf {
        match self.path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => PathBuf::from("."),
        }
    }

    fn temp_path(&self) -> Result<PathBuf, CredentialStoreError> {
        let file_name = self
            .path
            .file_name()
            .ok_or_else(|| CredentialStoreError::FileOpen {
                path: self.path.clone(),
                source: io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "credential path has no file name",
                ),
            })?;
        let temp_name = format!(".{}.tmp", file_name.to_string_lossy());
        Ok(self.parent_directory().join(temp_name))
    }

    fn open_temp_file(&self, temp_path: &Path) -> Result<fs::File, CredentialStoreError> {
        open_temp_with_platform_permissions(temp_path).map_err(|source| {
            CredentialStoreError::FileOpen {
                path: self.path.clone(),
                source,
            }
        })
    }

    fn write_temp_file(
        &self,
        file: fs::File,
        credentials: &StoredCredentials,
    ) -> Result<(), CredentialStoreError> {
        let document =
            serialized_document(credentials).map_err(|source| CredentialStoreError::FileWrite {
                path: self.path.clone(),
                source,
            })?;
        write_and_sync(file, document.as_bytes()).map_err(|source| {
            CredentialStoreError::FileWrite {
                path: self.path.clone(),
                source,
            }
        })
    }

    fn replace_target(&self, temp_path: &Path) -> Result<(), CredentialStoreError> {
        let replacement = sync_replace(temp_path, &self.path);
        if replacement.is_err() {
            let _ = fs::remove_file(temp_path);
        }
        replacement.map_err(|source| CredentialStoreError::FileWrite {
            path: self.path.clone(),
            source,
        })
    }

    fn read_credential_text(&self) -> Result<String, CredentialStoreError> {
        fs::read_to_string(&self.path).map_err(|source| map_read_error(&self.path, source))
    }
}

impl CredentialStore for FileCredentialStore {
    fn load(&self) -> Result<StoredCredentials, CredentialStoreError> {
        let text = self.read_credential_text()?;
        parse_credentials(&text, &self.path)
    }

    fn save(&self, credentials: &StoredCredentials) -> Result<(), CredentialStoreError> {
        self.ensure_parent_directory()?;
        let temp_path = self.temp_path()?;
        let _ = fs::remove_file(&temp_path);
        let file = self.open_temp_file(&temp_path)?;
        self.write_temp_file(file, credentials)?;
        self.replace_target(&temp_path)
    }
}

fn map_read_error(path: &Path, source: io::Error) -> CredentialStoreError {
    if source.kind() == io::ErrorKind::NotFound {
        CredentialStoreError::ContentNotAvailable
    } else {
        CredentialStoreError::FileOpen {
            path: path.to_path_buf(),
            source,
        }
    }
}

fn write_and_sync(mut file: fs::File, bytes: &[u8]) -> io::Result<()> {
    file.write_all(bytes)?;
    file.sync_all()
}

/// Atomically replaces the target file with the prepared temp file.
/// POSIX: rename alone replaces the destination. Windows: a rename over
/// an existing file fails with AlreadyExists, so the two-step remove +
/// rename is the required fallback there; a same-directory replace
/// never leaves a truncated credential file behind.
fn sync_replace(temp_path: &Path, target_path: &Path) -> Result<(), io::Error> {
    match fs::rename(temp_path, target_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            fs::remove_file(target_path)?;
            fs::rename(temp_path, target_path)
        }
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
fn open_temp_with_platform_permissions(temp_path: &Path) -> io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(temp_path)
}

#[cfg(not(unix))]
fn open_temp_with_platform_permissions(temp_path: &Path) -> io::Result<fs::File> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temp_path)
}

#[cfg(test)]
#[path = "file_credential_store_tests.rs"]
mod file_credential_store_tests;
