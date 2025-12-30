# Rust OneNote® File Parser

<p align="center">A parser for Microsoft OneNote® files implemented in Rust.</p>

The project supports reading OneNote files in the FSSHTTP packaging format
([\[MS-ONESTORE\] 2.3] and [\[MS-ONESTORE\] 2.8]) as used by OneDrive and the
modern OneNote apps. Feature contributions are welcome, but otherwise the
project focuses on bugfixes and compatibility.

In addition to the publicly documented contents, this project also allows
reading ink/handwriting content as well as math/equation content.

## Goals

- Read OneNote notebooks and sections obtained via OneDrive download
- Provide a Rust API for inspecting notebook, section, and page data
- Support HTML conversion via the [one2html] project

## Non-Goals

- The ability to write OneNote files
- Support for legacy OneNote 2016 desktop files

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
onenote_parser = "1.1"
```

```rust
use onenote_parser::Parser;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::new();
    // .onetoc2 file from a OneDrive download (FSSHTTP packaging format)
    let notebook = parser.parse_notebook(Path::new("My Notebook.onetoc2"))?;
    println!("sections: {}", notebook.entries().len());
    Ok(())
}
```

## Backtraces

Enable the `backtrace` feature to capture a `std::backtrace::Backtrace` on
parser errors. This can help pinpoint where a parsing failure originated and
is exposed through `std::error::Error::backtrace()`.

```toml
[dependencies]
onenote_parser = { version = "1.1", features = ["backtrace"] }
```

## Stability

The API is considered stable and will not change without a major version bump.
Releases follow semantic versioning.

## Architecture

The code organization and architecture follows the OneNote file format which is
built from several layers of encodings:

- `fsshttpb/`: This implements the FSSHTTP binary packaging format as specified
  in [\[MS-FSSHTTPB\]: Binary Requests for File Synchronization via SOAP Protocol].
  This is the lowest level of the file format and specifies how objects and their
  relationships are encoded (and decoded) from a binary stream (in our case a file).
- `onestore/`: This implements the OneStore format as specified in
  [\[MS-ONESTORE\]: OneNote Revision Store File Format] which describes how a
  OneNote revision store file (also called OneStore) containing all OneNote objects
  is stored in a FSSHTTP binary packaging file. This also includes the file header
  ([\[MS-ONESTORE\] 2.8]) and then how the OneNote revision store is built from the
  FSSHTTP objects and revisions ([\[MS-ONESTORE\] 2.7]).
- `one/`: This implements the OneNote file format as specified in [\[MS-ONE\]:
  OneNote File Format]. This specifies how objects in a OneNote file are parsed
  from a OneStore revision file.
- `onenote/`: This finally implements an API that provides access to the data
  stored in a OneNote file. It parses the FSSHTTPB data, the revision store
  data and then constructs the objects contained by the OneNote file. This includes
  resolving all references, e.g. looking up pages' paragraphs.

## Related Resources

- [\[MS-ONESTORE\]: OneNote Revision Store File Format]
- [\[MS-ONE\]: OneNote File Format]
- [\[MS-FSSHTTPB\]: Binary Requests for File Synchronization via SOAP Protocol]
- [LibMsON]: A work in progress OneNote® revision store file parser in C++
- [FSSHTTP - parser tools for protocol FSSHTTP/B/D]: A FSSHTTPB data parser

## Disclaimer

This project is neither related to nor endorsed by Microsoft in any way. The
author does not have any affiliation with Microsoft.

[\[MS-ONESTORE\] 2.3]: https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/a1d17d79-f0aa-45fc-a90f-e70f9df16f34

[\[MS-ONESTORE\] 2.7]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/189f186c-84ea-4892-afca-633c22bf9389

[\[MS-ONESTORE\] 2.8]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/c65f7aa8-4f0e-45dc-aabd-96db97cedbd4

[\[MS-ONESTORE\]: OneNote Revision Store File Format]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/ae670cd2-4b38-4b24-82d1-87cfb2cc3725

[\[MS-ONE\]: OneNote File Format]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/73d22548-a613-4350-8c23-07d15576be50

[\[MS-FSSHTTPB\]: Binary Requests for File Synchronization via SOAP Protocol]: https://docs.microsoft.com/en-us/openspecs/sharepoint_protocols/ms-fsshttpb/f59fc37d-2232-4b14-baac-25f98e9e7b5a

[LibMsON]: https://github.com/blu-base/libmson/

[FSSHTTP - parser tools for protocol FSSHTTP/B/D]: https://github.com/marx-yu/FSSHTTP

[one2html]: https://github.com/msiemens/one2html
