//! Native [`FileSystem`] backed by `std::fs`.
//!
//! Gated behind the `native-fs` feature so the crate can still build for
//! `wasm32-unknown-unknown` and other targets that provide their own
//! [`FileSystem`] implementation. The module supplies a single zero-sized
//! [`NativeFs`] value plus a Windows-only path-rewriting helper
//! ([`nt_path`]) that routes paths through the `\\?\` verbatim namespace
//! to avoid the DOS-device-name trap (a `.onetoc2` entry named `COM1`
//! would otherwise open the serial port).
//!
//! [`NativeFs::open_file`] returns a [`CachedFileSource`] wrapping a
//! [`FileBackedSource`], so large notebooks parse via positional reads
//! against the file handle rather than loading the whole file into
//! memory.

use crate::FileSystem;
use crate::fs::FileSource;
use crate::fs::file_source::{CachedFileSource, FileBackedSource};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Error, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use typed_path::{TypedPath, TypedPathBuf};

/// Native file system implementation using standard library I/O operations.
///
/// This is the default implementation of [`FileSystem`] that performs actual
/// file system operations using Rust's standard library.
#[derive(Clone, Copy)]
pub struct NativeFs {}

/// Convert a [`TypedPath`] into a host-native [`PathBuf`].
///
/// On Unix the path bytes are reinterpreted as an `OsStr` directly (Unix
/// paths are arbitrary bytes). On Windows the bytes must be valid UTF-8,
/// and the resulting path is additionally rewritten into the verbatim
/// namespace (`\\?\…`) to neutralise the DOS-device-name trap — see
/// [`to_verbatim`].
fn resolve_path(path: TypedPath) -> Result<PathBuf, Error> {
    #[cfg(unix)]
    {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let unix = path
            .with_unix_encoding_checked()
            .map_err(|err| Error::new(std::io::ErrorKind::InvalidInput, err))?;
        Ok(PathBuf::from(OsStr::from_bytes(unix.as_bytes())))
    }
    #[cfg(windows)]
    {
        // The checked conversion rebuilds the path component by component and
        // rejects drive prefixes, so only use it for paths that actually need
        // converting. Windows paths are used as they are.
        let win = if path.is_windows() {
            path.to_path_buf()
        } else {
            path.with_windows_encoding_checked()
                .map_err(|err| Error::new(std::io::ErrorKind::InvalidInput, err))?
        };
        let s = win.to_str().ok_or_else(|| {
            Error::new(
                std::io::ErrorKind::InvalidData,
                "path bytes are not valid UTF-8",
            )
        })?;
        to_verbatim(PathBuf::from(s))
    }
    #[cfg(not(any(unix, windows)))]
    {
        compile_error!("native-fs is only supported on Unix and Windows targets");
    }
}

/// Convert a host-native [`Path`] returned by `std::fs` back into a
/// [`TypedPathBuf`] in the host's native encoding. Used by [`read_dir`]
/// to surface enumerated entries through the [`FileSystem`] API.
fn as_typed_path(path: &Path) -> Result<TypedPathBuf, Error> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        Ok(TypedPath::unix(path.as_os_str().as_bytes()).to_path_buf())
    }
    #[cfg(windows)]
    {
        let s = path.to_str().ok_or_else(|| {
            Error::new(
                std::io::ErrorKind::InvalidData,
                "directory entry path is not valid UTF-8",
            )
        })?;
        Ok(TypedPath::windows(s.as_bytes()).to_path_buf())
    }
}

/// On Windows, rewrite `path` into the verbatim namespace (`\\?\…`).
///
/// Standard Win32 file APIs (which back `std::fs`) interpret a path
/// whose basename matches a reserved DOS device name — `CON`, `PRN`,
/// `AUX`, `NUL`, `COM0`–`COM9`, `LPT0`–`LPT9`, with or without an
/// extension — as a handle to the corresponding device, not as a
/// filename. A `.onetoc2` entry of `COM1` would therefore cause
/// `File::open` to grab the serial port. Routing through `\\?\`
/// disables that parsing and treats the path literally.
///
/// Produces an absolute, backslash-form path with the appropriate
/// `\\?\` (drive) or `\\?\UNC\` (UNC share) prefix.
#[cfg(windows)]
fn to_verbatim(path: PathBuf) -> Result<PathBuf, Error> {
    use std::ffi::OsString;
    let abs = std::path::absolute(&path)?;
    let s = abs.to_string_lossy().replace('/', "\\");
    if s.starts_with(r"\\?\") {
        return Ok(PathBuf::from(s));
    }
    let mut out = OsString::new();
    if s.starts_with(r"\\") {
        // UNC share path → \\?\UNC\server\share\...
        out.push(r"\\?\UNC\");
        out.push(&s[2..]);
    } else {
        out.push(r"\\?\");
        out.push(&s);
    }
    Ok(PathBuf::from(out))
}

impl FileSystem for NativeFs {
    fn is_directory(&self, path: TypedPath) -> Result<bool, Error> {
        let path = resolve_path(path)?;
        match fs::metadata(&path) {
            Ok(meta) => Ok(meta.is_dir()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(err) => Err(err),
        }
    }

    fn read_dir(&self, path: TypedPath) -> Result<Vec<TypedPathBuf>, Error> {
        let path = resolve_path(path)?;
        let mut result = Vec::new();

        for item in fs::read_dir(&path)? {
            let item = item?.path();
            result.push(as_typed_path(&item)?);
        }

        Ok(result)
    }

    fn read_file(&self, path: TypedPath) -> Result<Vec<u8>, Error> {
        let path = resolve_path(path)?;
        let file = File::open(&path)?;
        let size = file.metadata()?.len();
        let mut data = Vec::with_capacity(size as usize);

        let mut buf = BufReader::new(file);
        buf.read_to_end(&mut data)?;

        Ok(data)
    }

    fn write_file(&self, path: TypedPath, data: &[u8]) -> Result<(), Error> {
        let path = resolve_path(path)?;
        fs::write(path, data)
    }

    /// Streams `reader` directly into the destination file via
    /// `std::io::copy`, so large payloads never need to be materialised
    /// in process memory.
    fn stream_to_file(&self, path: TypedPath, reader: &mut dyn Read) -> Result<(), Error> {
        let path = resolve_path(path)?;
        let mut file = File::create(path)?;
        std::io::copy(reader, &mut file)?;
        Ok(())
    }

    fn make_dir(&self, path: TypedPath) -> Result<(), Error> {
        let resolved_path = resolve_path(path)?;
        let result = fs::create_dir_all(resolved_path);

        // Don't fail if it already existed as a directory; surface other errors
        // (e.g. path exists as a file).
        if self.is_directory(path)? {
            Ok(())
        } else {
            result
        }
    }

    fn canonicalize(&self, path: TypedPath) -> Result<TypedPathBuf, Error> {
        let resolved = resolve_path(path)?;
        let canon = fs::canonicalize(&resolved)?;
        as_typed_path(&canon)
    }

    fn exists(&self, path: TypedPath) -> Result<bool, Error> {
        let path = resolve_path(path)?;
        fs::exists(&path)
    }

    /// Opens the file as an on-demand [`FileSource`].
    ///
    /// Each [`FileSource::read_at`] call issues a positional read against
    /// the open file handle (`pread` on Unix, overlapped `ReadFile` on
    /// Windows), so the file's bytes never need to live in process memory
    /// in their entirety — multi-GB notebooks parse with a working set
    /// proportional to active reads, not file size. The kernel's page
    /// cache fronts repeated reads cheaply.
    ///
    /// # File mutation during parsing
    ///
    /// Reads are positional and recoverable: a truncated or replaced file
    /// surfaces as an [`std::io::Error`] (or short read) from `read_at`,
    /// not a signal. The parse may still produce garbage or
    /// `MalformedData` if a concurrent writer mutates bytes the parser
    /// has yet to read — there's no way to make that consistent — but the
    /// process won't be aborted.
    fn open_file(&self, path: TypedPath) -> Result<Arc<dyn FileSource>, Error> {
        let path = resolve_path(path)?;
        let file = File::open(&path)?;
        let byte_length = file.metadata()?.len();
        let raw = FileBackedSource { file, byte_length };
        Ok(Arc::new(CachedFileSource::new(raw)))
    }
}
