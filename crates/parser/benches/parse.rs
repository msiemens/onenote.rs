//! Performance baseline for the parser.
//!
//! Two comparison points per fixture:
//!
//! - `parse_section/in_memory/...` — `Parser::parse_section` against a
//!   custom in-memory [`FileSystem`] that wraps the file's bytes in a
//!   refcount-shared [`BytesSource`]. No file I/O, no per-call
//!   allocation: every read is a `Bytes::slice` refcount bump. Isolates
//!   parsing cost.
//! - `parse_section/file_backed/...` — `Parser::parse_section` against
//!   `NativeFs`, exercising the pread + LRU page-cache path.
//!
//! Plus `parse_notebook` against `NativeFs`.
//!
//! Fixtures live under `crates/parser/tests/samples/`. The corpus
//! covers a tiny FSSHTTPB section (`Math.one`), a tiny desktop-format
//! section, a medium FSSHTTPB section, an attachment-heavy small
//! FSSHTTPB section, a 108 MB desktop section, and a notebook of small
//! sections.

use bytes::Bytes;
use criterion::{Criterion, criterion_group, criterion_main};
use onenote_parser::Parser;
use onenote_parser::fs::file_source::BytesSource;
use onenote_parser::fs::native_fs::NativeFs;
use onenote_parser::fs::{FileSource, FileSystem};
use std::hint::black_box;
use std::io;
use std::io::Error;
use std::sync::Arc;
use typed_path::{TypedPath, TypedPathBuf};

const SECTION_FIXTURES: &[(&str, &str)] = &[
    ("math_tiny_fsshttpb", "tests/samples/joplin/Math.one"),
    (
        "onenote_desktop",
        "tests/samples/joplin/onenote_desktop.one",
    ),
    ("new_section", "tests/samples/New Section 1.one"),
    ("schnelle_notizen", "tests/samples/Schnelle Notizen.one"),
    ("large_desktop", "tests/samples/Large Desktop.one"),
    ("large_onedrive", "tests/samples/Large OneDrive.one"),
];

const NOTEBOOK_FIXTURE: &str = "tests/samples/Open Notebook.onetoc2";

/// A [`FileSystem`] that holds a single file's bytes in memory as a shared
/// [`Bytes`]. Used by the bench to isolate parser cost from any I/O —
/// `open_file` returns a [`BytesSource`] (as_bytes = Some), so reads
/// inside the parser are refcount-only slice bumps with no allocation.
#[derive(Clone, Copy)]
struct InMemFs<'a> {
    bytes: &'a Bytes,
}

impl FileSystem for InMemFs<'_> {
    fn is_directory(&self, _: TypedPath) -> io::Result<bool> {
        Ok(false)
    }
    fn read_dir(&self, _: TypedPath) -> io::Result<Vec<TypedPathBuf>> {
        Ok(Vec::new())
    }
    fn read_file(&self, _: TypedPath) -> io::Result<Vec<u8>> {
        Ok(self.bytes.to_vec())
    }
    fn write_file(&self, _: TypedPath, _: &[u8]) -> io::Result<()> {
        unimplemented!("bench fs is read-only")
    }
    fn stream_to_file(&self, _: TypedPath, _: &mut dyn io::Read) -> io::Result<()> {
        unimplemented!("bench fs is read-only")
    }
    fn make_dir(&self, _: TypedPath) -> io::Result<()> {
        unimplemented!("bench fs is read-only")
    }

    fn canonicalize(&self, path: TypedPath) -> Result<TypedPathBuf, Error> {
        Ok(path.to_path_buf())
    }

    fn exists(&self, _: TypedPath) -> io::Result<bool> {
        Ok(true)
    }
    fn open_file(&self, _: TypedPath) -> io::Result<Arc<dyn FileSource>> {
        Ok(Arc::new(BytesSource::new(self.bytes.clone())))
    }
}

fn parse_section_in_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_section/in_memory");
    for (name, path) in SECTION_FIXTURES {
        // Read once outside the timed loop. `Bytes::from(Vec<u8>)` is a
        // zero-copy ownership transfer; `clone` inside the loop is a
        // refcount bump.
        let bytes = Bytes::from(std::fs::read(path).expect("fixture not found"));
        let typed_path = TypedPath::derive(path).to_path_buf();
        group.bench_function(*name, |b| {
            b.iter(|| {
                let fs = InMemFs { bytes: &bytes };
                let parser = Parser::new_with_fs(fs);
                let _ = black_box(parser.parse_section(black_box(typed_path.to_path())));
            });
        });
    }
    group.finish();
}

fn parse_section_file_backed(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_section/file_backed");
    for (name, path) in SECTION_FIXTURES {
        let path = TypedPath::derive(path).to_path_buf();
        group.bench_function(*name, |b| {
            b.iter(|| {
                let parser = Parser::new_with_fs(NativeFs {});
                let _ = black_box(parser.parse_section(black_box(path.to_path())));
            });
        });
    }
    group.finish();
}

fn parse_notebook(c: &mut Criterion) {
    let path = TypedPath::derive(NOTEBOOK_FIXTURE).to_path_buf();
    c.bench_function("parse_notebook/open_notebook", |b| {
        b.iter(|| {
            let parser = Parser::new();
            let _ = black_box(parser.parse_notebook(black_box(path.to_path())));
        });
    });
}

criterion_group!(
    benches,
    parse_section_in_memory,
    parse_section_file_backed,
    parse_notebook
);
criterion_main!(benches);
