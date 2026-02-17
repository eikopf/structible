# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `*_iter_mut()` methods for unknown/extension fields on both the main struct and `Fields` companion struct

## [0.3.0] - 2026-02-17

### Changed

- `Debug` implementations now show fields like normal structs instead of exposing internal map structure
- Only present fields are shown in `Debug` output (sparse representation)
- Extended documentation generation to cover all generated methods (getters, mutable getters, setters)
- Updated README to document `with_len`, `take_*` methods, and ownership extraction

## [0.2.0] - 2026-02-16

### Changed

- **Breaking:** `len()` and `is_empty()` methods are now opt-in via `#[structible(with_len)]` attribute
- **Breaking:** `take_*` methods are no longer generated for required (non-Option) fields
- **Breaking:** The `Fields` companion struct is now backed by a HashMap instead of plain struct fields
- Improved handling of raw identifiers in enum variant name generation

## [0.1.0] - 2026-02-16

### Added

- `#[structible]` attribute macro for generating map-backed structs
- `BackingMap` trait with implementations for `HashMap` and `BTreeMap`
- `IterableMap` trait for unknown/extension field support
- Field accessors: getters, mutable getters, setters
- `remove_*` methods for optional fields
- `take_*` methods for ownership extraction
- `into_fields()` for full struct decomposition
- Unknown/extension field support via `#[structible(key = Type)]`
- Custom constructor naming via `constructor = name`
- Custom accessor naming via `get`, `get_mut`, `set`, `remove` attributes
