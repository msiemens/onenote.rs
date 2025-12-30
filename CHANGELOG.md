# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
