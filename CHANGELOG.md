# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.1.0] - 2026-10-19

### Added

- `Image::web_picture()` and `EmbeddedFile::icon()` expose supplemental pictures
  through the new `Picture` type. `Picture::extension()` is attacker-controlled
  and must be sanitised before use in file names. See PR [#44], issue [#43].
- `file_identity()` on `Notebook`, `SectionGroup` and `Section` returns an
  opaque `FileIdentity` that stays the same when a file is renamed or moved.
  See PR [#45].
- `InkStroke::absolute_path()` returns absolute stroke coordinates; the stored
  delta encoding returned by `InkStroke::path()` is now documented. See PR
  [#47], issue [#4].

### Fixed

- Accept a standalone `ObjectInfoDependencyOverridesFND` in a revision manifest
  instead of rejecting the whole section. See PR [#46].
- `NativeFs` on Windows no longer rejects paths with a drive letter, so
  absolute paths can be parsed.

[#4]: https://github.com/msiemens/onenote.rs/issues/4
[#43]: https://github.com/msiemens/onenote.rs/issues/43
[#44]: https://github.com/msiemens/onenote.rs/pull/44
[#45]: https://github.com/msiemens/onenote.rs/pull/45
[#46]: https://github.com/msiemens/onenote.rs/pull/46
[#47]: https://github.com/msiemens/onenote.rs/pull/47

## [2.0.0] - 2026-08-06

Headline release: the parser now reads **OneNote desktop files** in addition to
OneDrive downloads. This release also introduces a pluggable `FileSystem`
abstraction, streams attachments lazily instead of buffering them, and collects
non-fatal issues as warnings. See PR [#28].

### Security

- No longer pre-allocate `Vec`s from attacker-controlled element counts when
  parsing property values and object streams. A crafted file of a few
  kilobytes could previously trigger an allocation attempt of up to ~192 GB,
  aborting the consuming process. Collections now grow as elements actually
  parse, keeping allocation proportional to the input size.
- Documented that `Image::image_filename()` returns attacker-controlled data
  that must be sanitised before use in filesystem paths, matching the existing
  warning on `EmbeddedFile::filename()`.

### Added

- **OneNote desktop file parsing** (2016, 2019, LTSC, …) for `.one`/`.onetoc2`
  files. The top-level entry point sniffs the header GUIDs and dispatches
  between the desktop and FSSHTTP (OneDrive) code paths automatically.
- `.onepkg` package support via `Parser::parse_package` behind the new `onepkg`
  feature. The CAB is decompressed in memory; nothing is written to disk.
- A pluggable `FileSystem` abstraction (`fs.rs`) so the parser can run in
  `no_std`/WASM and against custom backends. `Parser::new()` remains available
  with the default `native-fs` feature; other backends use
  `Parser::new_with_fs(fs)`. A WASM (`wasm32-unknown-unknown`) build target is
  now exercised in CI.
- A warning-collection system: non-fatal issues are gathered into a `Report`
  attached to each `Section`/`Notebook` (reachable via `Section::report()`)
  instead of aborting the parse.
- Page creation and last-modified timestamps are now exposed on `Page`.
- Support for the `hyperlink_protected` and `hidden` text-run properties, and
  for importing nested ink containers.
- **Handwriting recognition (OCR)** output written by OneNote for Windows is
  now parsed. `Page::ink_recognition()` exposes the recognized lines and words
  — best guess plus alternative candidates and the recognizer's locale — in
  the recognizer's reading order. `InkStroke::recognized_word()` attaches each
  handwriting stroke to the word it belongs to, and a stable
  `InkRecognizedWord::id()` deduplicates the recognized words encountered when
  iterating strokes — a word spans multiple strokes, each of which surfaces
  the same word with the same id. See [#6].
- An `inspect` binary for dumping `.one` debug output.
- **Structured rich-text hyperlinks**: `TextHyperlink` exposes each link's
  target URL together with its UTF-16 source range in the text, reconstructed
  from OneNote's hidden `HYPERLINK` markers and hyperlink-styled runs — so
  consumers can render clickable links without duplicating parser-internal
  conventions. See PR [#42].
- `Image::data_status()` and `EmbeddedFile::data_status()` expose a
  non-exhaustive `FileDataStatus`, so consumers can distinguish an attachment
  whose payload is invalid or missing from a legitimate zero-byte file. See
  PR [#36].

### Changed

- **BREAKING**: Attachments are now streamed lazily rather than returned as
  buffered byte slices. `EmbeddedFile`/`Image` expose `read() -> Box<dyn Read>`
  and `size()`, backed by a lazily read `FileSource`; a `stream_to_file` helper
  is provided. FSSHTTPB blob payloads are no longer materialised at parse time.
- Diagnostics now go through the `log` crate; the library is silent unless the
  consuming application installs a logger.
- Replaced panicking and assertion code paths throughout with fallible error
  handling.
- Performance: store `PropertySet` entries in a `Vec` instead of a `HashMap`.
- Replaced the `sanitise-file-name` dependency with `sanitize-filename`,
  switched parameterized tests to `yare`, and updated dependencies.

### Fixed

- Continue parsing past sections and attachments that fail instead of aborting
  the whole notebook. Attachments with an explicitly invalid (`<invfdo>`) or
  missing payload are preserved with a `FileDataStatus` and a page-scoped
  warning; sections without a file-data store no longer fail before the
  reference can be classified. See PR [#36], issue [#35].
- Desktop revision materialization: revisions are now kept as immutable deltas
  linked to their declared dependency, and only the latest default-context
  role-1 dependency chain is materialized. Pages no longer expose stale or
  incomplete content mixed in from unrelated coauthor or history branches.
  See PR [#41].
- Accept outline elements and outline groups without a creation or
  last-modified timestamp, as written by some OneNote Desktop backups: the
  outline tree is preserved and a page-scoped warning is reported instead of
  rejecting the whole section. See PR [#37], issue [#33].
- Accept `PageSize` and `ParagraphAlignment` values encoded with wider integer
  widths (`U16`/`U32`/`U64`), as written by OneNote Desktop backups. Values
  above `u8::MAX` are still rejected rather than truncated. See PR [#38],
  issue [#34].
- Preserve the latest notebook TOC ordering when a `.onetoc2` object retains
  multiple ordering snapshots: repeated filenames resolve to their most recent
  reference before sorting, so sections no longer appear twice and section
  groups no longer move among ordinary sections. See PR [#40], issue [#5].
- Ink: allow negative coordinates in bounding boxes, skip ink when the data
  object is missing, read `InkScalingY` from the correct property type, and warn
  (instead of erroring) when an ink data node has no strokes.
- Text runs: ignore hidden runs for the title text, handle missing run styles
  gracefully, and handle leading VT misalignment.
- Warn and fall back to a nil GUID for a missing page series GUID, and surface
  the latest revision of each object.
- Make `cached_title` optional during page metadata parsing.
- Rework the `.onetoc2` path-traversal guard introduced in 1.1.1: it no longer
  rejects legitimate OneNote backups whose section files use reserved Windows
  device names (e.g. `CON.one`, `NUL.one`, `COM1.one`). Such names are now
  sanitised, while structural traversal (absolute paths, `..`, drive prefixes)
  is still hard-rejected, and `NativeFs` reads go through the Windows verbatim
  (`\\?\`) namespace.
- Restore MSRV (Rust 1.85) compilation.
- Parse desktop `.onetoc2` files that describe their table of contents using
  object revisions (`ObjectRevisionWithRefCountFNDX`/`ObjectRevisionWithRefCount2FNDX`),
  which OneNote Desktop writes after copying pages between sections. Such
  revisions inherit their object's JCID from the declaration in a dependency
  revision, and their global ID table entries (`GlobalIdTableEntry2FNDX`) are now
  resolved against that dependency revision's table instead of being ignored. See
  [#31].

[#28]: https://github.com/msiemens/onenote.rs/pull/28
[#31]: https://github.com/msiemens/onenote.rs/issues/31
[#6]: https://github.com/msiemens/onenote.rs/issues/6
[#5]: https://github.com/msiemens/onenote.rs/issues/5
[#33]: https://github.com/msiemens/onenote.rs/issues/33
[#34]: https://github.com/msiemens/onenote.rs/issues/34
[#35]: https://github.com/msiemens/onenote.rs/issues/35
[#36]: https://github.com/msiemens/onenote.rs/pull/36
[#37]: https://github.com/msiemens/onenote.rs/pull/37
[#38]: https://github.com/msiemens/onenote.rs/pull/38
[#40]: https://github.com/msiemens/onenote.rs/pull/40
[#41]: https://github.com/msiemens/onenote.rs/pull/41
[#42]: https://github.com/msiemens/onenote.rs/pull/42

## [1.1.1] - 2026-05-15

### Security

- Reject absolute paths, parent-directory components, and other invalid path
  characters when resolving section entries listed in a `.onetoc2` file. A
  malicious notebook could previously cause the parser to open files outside
  the notebook's base directory.

### Fixed

- Guard against underflow and overflow when computing transaction log offsets.
- Avoid panicking when parsing malformed ink data.

### Changed

- Internal: use `bytes::try_*` for bounds-checked reads.
- Docs: add `SECURITY.md`.
- Infra: pin explicit permissions for GitHub Actions workflows.

## [1.1.0] - 2025-12-30

### Added

- Support for inline maths.

## [1.0.0] - 2025-12-28

### Added

- Support for non-legacy MS-ONESTORE format.
- Parse notebook colors.

### Changed

- Improve crate documentation.
- Upgrade to Rust 2024 edition.
- Improve revision manifest resolution and fallback logic.
- Replace panics with error handling.
- Improve error handling in parsing logic.
- Internal refactorings for maintainability.

## [0.4.1] - 2025-12-27

### Fixed

- Remove `provide_any` feature that has been removed from Rust

## [0.4.0] - 2025-12-27

### Added

- Feature: Add ability to parse section from an in-memory buffer (see PR [#13]).

### Fixed

- Make `last_modified` optional for images (see issue [#11]).
- Specify discriminant type for `PropertType` (see PR [#14]).

### Changed

- Internal: Update dependencies.
- Internal: Update `paste` to `pastey` and revise cargo-deny configuration.
- Internal: Update cargo-deny-action to v2.
- Internal: Fix code formatting and clippy warnings.

[#11]: https://github.com/msiemens/onenote.rs/issues/11

[#13]: https://github.com/msiemens/onenote.rs/pull/13

[#14]: https://github.com/msiemens/onenote.rs/pull/14

## [0.3.1] - 2022-11-19

### Added

- Feature: Add support for parsing embed URLs for images.

### Changed

- Internal: Update dependencies
- Internal: Add `provide_any` and `error_generic_member_access` features when
  `backtrace` feature is enabled

## [0.3.0] - 2021-02-20

### Added

- Feature: Added support for parsing ink drawings.

### Changed

- **BREAKING**: Renamed `Outline::items_level` to `Outline::child_level` for
  consistency
- Internal: Reorganized the OneNote parser code for more consistency

### Fixed

- Fixed incorrect parsing of internal object references in some
  cases (see [c3e8a11], [8ac69a1] and [bb4abef])

[c3e8a11]: https://github.com/msiemens/onenote.rs/commit/c3e8a112901f2789241ecf6b7a878463d98ed415

[bb4abef]: https://github.com/msiemens/onenote.rs/commit/bb4abef1205a0a438ab4236719ea8bd7ed1d308a

[8ac69a1]: https://github.com/msiemens/onenote.rs/commit/8ac69a1fa44be9f774d9293ec1e3f3908cb447ec

## [0.2.1] - 2020-10-27

### Changed

- Removed some debug output.
- Added a test suite.

## [0.2.0] - 2020-10-24

### Changed

- Reorganized the public API.
