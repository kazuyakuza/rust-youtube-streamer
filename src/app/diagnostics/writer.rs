//! The optional structured JSONL diagnostics sink. The file is opened
//! lazily on the first row, every row is flushed immediately (crash-safe
//! tailing), and a size-cap roll keeps at most `max_file_bytes ×
//! max_generations` bytes on disk. Rotation I/O failures are swallowed on
//! purpose: diagnostics must never disturb the run. The chat log path is
//! never opened here — validation forbids the collision.

use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::collector_stages::CollectorSettings;

pub(crate) struct WriterSettings {
    pub(crate) path: PathBuf,
    pub(crate) max_file_bytes: u32,
    pub(crate) max_generations: u32,
}

/// Builds the optional shared sink from the collector settings; `None`
/// (no configured file) keeps every row on the tracing debug path.
pub(super) fn build_writer(
    settings: &CollectorSettings,
) -> Option<Arc<Mutex<DiagnosticsFileWriter>>> {
    settings.output_file.clone().map(|path| {
        let writer_settings = WriterSettings {
            path,
            max_file_bytes: settings.max_file_bytes,
            max_generations: settings.max_generations,
        };
        Arc::new(Mutex::new(DiagnosticsFileWriter::new(writer_settings)))
    })
}

pub(crate) struct DiagnosticsFileWriter {
    path: PathBuf,
    max_file_bytes: u32,
    max_generations: u32,
    writer: Option<BufWriter<std::fs::File>>,
    written_bytes: u64,
}

impl DiagnosticsFileWriter {
    pub(crate) fn new(settings: WriterSettings) -> Self {
        Self {
            path: settings.path,
            max_file_bytes: settings.max_file_bytes,
            max_generations: settings.max_generations,
            writer: None,
            written_bytes: 0,
        }
    }

    pub(crate) fn write_event(&mut self, line: &str) {
        let row_bytes = line.len() as u64 + 1;
        self.open_if_needed();
        self.rotate_if_needed(row_bytes);
        self.open_if_needed();
        if self.append_line(line) {
            self.written_bytes = self.written_bytes.saturating_add(row_bytes);
        }
    }

    fn append_line(&mut self, line: &str) -> bool {
        let Some(writer) = self.writer.as_mut() else {
            return false;
        };
        writer
            .write_all(line.as_bytes())
            .and_then(|()| writer.write_all(b"\n"))
            .and_then(|()| writer.flush())
            .is_ok()
    }

    fn rotate_if_needed(&mut self, row_bytes: u64) {
        if self.should_rotate(row_bytes) {
            self.rotate();
        }
    }

    fn should_rotate(&self, row_bytes: u64) -> bool {
        let limit = u64::from(self.max_file_bytes);
        self.written_bytes > 0 && self.written_bytes + row_bytes > limit
    }

    fn rotate(&mut self) {
        self.writer = None;
        self.written_bytes = 0;
        if self.max_generations <= 1 {
            let _ = std::fs::remove_file(&self.path);
            return;
        }
        let _ = std::fs::remove_file(self.generation_path(self.max_generations - 1));
        for generation in (1..self.max_generations - 1).rev() {
            self.shift_generation(generation);
        }
        let _ = std::fs::rename(&self.path, self.generation_path(1));
    }

    fn shift_generation(&self, generation: u32) {
        let source = self.generation_path(generation);
        if source.exists() {
            let _ = std::fs::rename(source, self.generation_path(generation + 1));
        }
    }

    fn open_if_needed(&mut self) {
        if self.writer.is_some() {
            return;
        }
        self.written_bytes = existing_size(&self.path);
        let opened = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path);
        if let Ok(file) = opened {
            self.writer = Some(BufWriter::new(file));
        }
    }

    fn generation_path(&self, generation: u32) -> PathBuf {
        PathBuf::from(format!("{}.{generation}", self.path.display()))
    }
}

fn existing_size(path: &std::path::Path) -> u64 {
    std::fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0)
}
