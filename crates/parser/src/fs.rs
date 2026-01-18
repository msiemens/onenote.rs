use std::fs;
use std::fs::File;
use std::io::{BufReader, Error, Read};
use std::path::{Path, PathBuf};

/// Abstraction over file system operations.
///
/// This trait provides an interface for file system operations used by the OneNote parser.
/// It enables dependency injection for testing and alternative file system implementations.
///
/// All implementations must be thread-safe (`Send + Sync`) as the parser may be used
/// across threads.
pub trait FileSystem: Send + Sync {
    /// Checks if the given path points to a directory.
    ///
    /// # Arguments
    /// * `path` - The path to check
    ///
    /// # Returns
    /// * `Ok(true)` if the path exists and is a directory
    /// * `Ok(false)` if the path exists but is not a directory
    /// * `Err` if the path doesn't exist or an I/O error occurs
    ///
    /// # Usage
    /// Used by the parser to distinguish between section files (.one) and section groups
    /// (directories containing .onetoc2 files).
    fn is_directory(&self, path: &Path) -> Result<bool, Error>;

    /// Lists all entries in a directory.
    ///
    /// # Arguments
    /// * `path` - The directory path to read
    ///
    /// # Returns
    /// A vector of paths for all entries in the directory, or an error if the
    /// directory cannot be read.
    ///
    /// # Usage
    /// Used to enumerate section files and subdirectories when parsing section groups.
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, Error>;

    /// Reads the entire contents of a file into memory.
    ///
    /// # Arguments
    /// * `path` - The file path to read
    ///
    /// # Returns
    /// The complete file contents as a byte vector, or an error if the file
    /// cannot be read.
    ///
    /// # Usage
    /// Used to load OneNote files (.one, .onetoc2) for parsing. Files are read
    /// entirely into memory as the parser needs random access to the data.
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Error>;

    /// Writes data to a file, replacing any existing content.
    ///
    /// # Arguments
    /// * `path` - The file path to write to
    /// * `data` - The data to write
    ///
    /// # Returns
    /// Ok(()) on success, or an error if the file cannot be written.
    ///
    /// # Usage
    /// May be used for extracting embedded content or creating output files.
    fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), Error>;

    /// Creates a directory if it doesn't already exist.
    ///
    /// # Arguments
    /// * `path` - The directory path to create
    ///
    /// # Returns
    /// `Ok(())` if the directory was created or already exists, or an error
    /// if the directory cannot be created.
    ///
    /// # Note
    /// This method should not fail if the directory already exists (idempotent).
    fn make_dir(&self, path: &Path) -> Result<(), Error>;

    /// Checks if a path exists in the file system.
    ///
    /// # Arguments
    /// * `path` - The path to check
    ///
    /// # Returns
    /// * `Ok(true)` if the path exists (file or directory)
    /// * `Ok(false)` if the path does not exist
    /// * `Err` if the existence check fails due to permissions or other I/O errors
    ///
    /// # Usage
    /// Used to filter out non-existent section entries and verify paths before
    /// attempting to parse them.
    fn exists(&self, path: &Path) -> Result<bool, Error>;
}

/// Native file system implementation using standard library I/O operations.
///
/// This is the default implementation of [`FileSystem`] that performs actual
/// file system operations using Rust's standard library.
pub struct NativeFs {}

impl FileSystem for NativeFs {
    fn is_directory(&self, path: &Path) -> Result<bool, Error> {
        Ok(fs::metadata(path)?.is_dir())
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, Error> {
        let mut result = Vec::new();

        for item in fs::read_dir(path)? {
            let item = item?.path();
            result.push(item)
        }

        Ok(result)
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, Error> {
        let file = File::open(path)?;
        let size = file.metadata()?.len();
        let mut data = Vec::with_capacity(size as usize);

        let mut buf = BufReader::new(file);
        buf.read_to_end(&mut data)?;

        Ok(data)
    }

    fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), Error> {
        fs::write(path, data)
    }

    fn make_dir(&self, path: &Path) -> Result<(), Error> {
        let result = fs::create_dir(path);

        // Don't fail if it already existed
        if self.exists(path)? { Ok(()) } else { result }
    }

    fn exists(&self, path: &Path) -> Result<bool, Error> {
        fs::exists(path)
    }
}
