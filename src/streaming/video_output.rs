//! Output placement for local recordings: the `videos/` directory beside
//! the application executable (never the process working directory) and
//! collision-safe timestamped filenames that never overwrite a previous
//! recording (TODO Task 1 §1; plan §1.6).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::streaming::error::StreamingError;
use crate::streaming::utc_timestamp::format_utc_timestamp;

const OUTPUT_DIRECTORY_NAME: &str = "videos";
const FILENAME_EXTENSION: &str = "mp4";
const FILE_EXTENSION_SEPARATOR: &str = ".";
const COLLISION_SUFFIX_SEPARATOR: &str = "_";
const FIRST_COLLISION_SUFFIX: u32 = 2;
const MAX_SUFFIX_ATTEMPTS: u32 = 1000;

/// Which pipeline names the recording: the demo command uses `demo-`;
/// `session-` is reserved for the future production/local pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputKind {
    Demo,
    // `session-` is reserved for the future production pipeline (Task-1
    // contract); the demo is the only runtime OutputKind in this phase.
    #[allow(dead_code)]
    Session,
}

impl OutputKind {
    pub(crate) fn filename_prefix(self) -> &'static str {
        match self {
            OutputKind::Demo => "demo",
            OutputKind::Session => "session",
        }
    }
}

/// All inputs for generating one recording filename. Parameter object keeps
/// the generator within the max-2-arguments rule.
pub(crate) struct RecordingNameSpec {
    output_directory: PathBuf,
    kind: OutputKind,
    epoch_seconds: u64,
}

impl RecordingNameSpec {
    pub(crate) fn new(output_directory: PathBuf, kind: OutputKind, epoch_seconds: u64) -> Self {
        Self {
            output_directory,
            kind,
            epoch_seconds,
        }
    }

    /// Returns the first non-existing candidate filename (name only, not a
    /// full path) in the deterministic order: `{prefix}-YYYYMMDD-HHMMSS.mp4`,
    /// then `_2`, `_3`, ... appended before the extension
    /// (`...-090000_2.mp4`), with the collision suffix numbered up to
    /// `MAX_SUFFIX_ATTEMPTS`. `exists` is an injected predicate so this
    /// stays pure and testable. Exceeding the attempts budget fails with
    /// [`StreamingError::OutputNameExhausted`]; names are never truncated
    /// and no candidate is ever returned twice.
    pub(crate) fn next_available(
        &self,
        exists: &mut dyn FnMut(&Path) -> bool,
    ) -> Result<String, StreamingError> {
        let stem = self.base_stem();
        let base_name = Self::with_extension(&stem);
        if !exists(&self.output_directory.join(&base_name)) {
            return Ok(base_name);
        }
        self.first_free_suffixed_name(&stem, exists)
    }

    fn base_stem(&self) -> String {
        format!(
            "{}-{}",
            self.kind.filename_prefix(),
            format_utc_timestamp(self.epoch_seconds)
        )
    }

    fn with_extension(stem: &str) -> String {
        format!("{stem}{FILE_EXTENSION_SEPARATOR}{FILENAME_EXTENSION}")
    }

    fn first_free_suffixed_name(
        &self,
        stem: &str,
        exists: &mut dyn FnMut(&Path) -> bool,
    ) -> Result<String, StreamingError> {
        for suffix in FIRST_COLLISION_SUFFIX..=MAX_SUFFIX_ATTEMPTS {
            let candidate =
                Self::with_extension(&format!("{stem}{COLLISION_SUFFIX_SEPARATOR}{suffix}"));
            if !exists(&self.output_directory.join(&candidate)) {
                return Ok(candidate);
            }
        }
        Err(StreamingError::OutputNameExhausted {
            directory: self.output_directory.clone(),
            attempts: MAX_SUFFIX_ATTEMPTS,
        })
    }
}

/// Resolves the `videos/` directory beside the executable's own directory
/// (never the process working directory). Pure path mapping, fully
/// testable; a missing parent directory is a typed
/// [`StreamingError::ExecutableLocation`].
pub(crate) fn resolve_videos_directory(executable_path: &Path) -> Result<PathBuf, StreamingError> {
    let executable_directory =
        executable_path
            .parent()
            .ok_or_else(|| StreamingError::ExecutableLocation {
                reason: "the executable path has no parent directory".to_string(),
            })?;
    Ok(executable_directory.join(OUTPUT_DIRECTORY_NAME))
}

/// Creates the output directory (including missing parents) on demand; an
/// existing plain file at the path surfaces the I/O failure as a typed
/// [`StreamingError::OutputDirectoryCreate`].
pub(crate) fn create_directory(directory: &Path) -> Result<(), StreamingError> {
    fs::create_dir_all(directory).map_err(|source| StreamingError::OutputDirectoryCreate {
        path: directory.to_path_buf(),
        source,
    })
}

/// Resolves the full recording output path: locate the executable, place
/// `videos/` beside it, create that directory on demand, then pick the
/// first free collision-safe filename for `kind` at the current UTC
/// timestamp. Composition entry point for the Task 2/3 pipeline wiring.
pub(crate) fn resolve_output_path(kind: OutputKind) -> Result<PathBuf, StreamingError> {
    let executable_path = current_executable_path()?;
    let output_directory = resolve_videos_directory(&executable_path)?;
    create_directory(&output_directory)?;
    let spec = RecordingNameSpec::new(output_directory, kind, epoch_seconds_now());
    let filename = spec.next_available(&mut path_exists)?;
    Ok(spec.output_directory.join(filename))
}

fn current_executable_path() -> Result<PathBuf, StreamingError> {
    std::env::current_exe().map_err(|source| StreamingError::ExecutableLocation {
        reason: source.to_string(),
    })
}

/// A clock read before the Unix epoch (or any other `SystemTime` failure)
/// falls back to timestamp 0; collision suffixes keep the name unique
/// regardless, so the fallback can never overwrite a recording.
fn epoch_seconds_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

/// Real existence probe for the collision walk. Conservative direction:
/// a probe ERROR (permissions etc.) is treated as "exists" via
/// `unwrap_or(true)`, so a failed probe can never let an overwrite
/// through; only a confirmed absence (`Ok(false)`) frees a name.
fn path_exists(path: &Path) -> bool {
    path.try_exists().unwrap_or(true)
}

#[cfg(test)]
#[path = "video_output_tests.rs"]
mod video_output_tests;
