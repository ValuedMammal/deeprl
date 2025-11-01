# Changelog

All notable changes will be documented in this file. This project adheres to semantic versioning.

## [Unreleased]

### In progress

## [0.5.0] - 2025-11-1

### Fixed

- Improved error parsing and handling when the HTTP response is not successful

### Added

- Support for more languages Arabic (AR), Spanish Latin America (ES-419), Chinese traditional (ZH-HANT)

### Changed

- Several crate `Error` variants are changed or renamed
- deps: Removed `thiserror`
- deps: Bump `reqwest` to 0.12.24

## [0.4.0] - 2024-09-03
### Changed
- Derive `serde::Serialize` for types `Language`, `SplitSentences`, `Formality`, `TagHandling`
- Derive `Debug`, `Clone` for types `TextOptions`, `DocumentOptions`
- Simplify text form building by setting a json body
- Renamed `Language` variants to respect case convention
- Changed fields `non_splitting_tags`, `splitting_tags`, `ignore_tags` of `TextOptions` to have type `Vec<String>`.
- Added method `text` to `TextOptions` for setting the text to translate. `DeepL::translate` function no longer takes a text parameter.

## [0.3.0] - 2024-04-21
### Changed
- Renamed DeepL method `glossary_del` to `glossary_delete`.

### Fixed
- Fixed a bug while parsing a server error.

## [0.2.1] - 2023-09-13
### Added
- Improved crate documentation

## [0.2.0] - 2023-07-20

### Changed
- The signature of `DeepL::new` has changed to expect a string slice `&str` as its only parameter instead of an owned String.
- `glossary_entries` now returns `Result<HashMap<String, String>>` instead of `Result<String>`, providing a more intuitive container for glossary entries.
- `Error::Request` has been renamed to `Error::InvalidRequest`.
- `DocumentStatus` now implements the function `is_done`. Previously `is_done` was called on an instance of `DocState`. This is no longer the case.

### Added
- Adds two new methods, `client` and `set_app_info` on the `DeepL` type which allow setting a user-defined `blocking::Client` and app-info string respectively.
- The following glossary types can be serialized: `GlossariesResult`, `GlossaryLanguagePairsResult`, and `GlossaryLanguagePair`

## [0.1.1] - 2023-07-13

### Added

- Adds ability to construct a Formality from a string. (impl FromStr)
- Allow displaying `Document`, `LanguageInfo`, and `Glossary`. (derive serde::Serialize)

### Fixed

-  [Fixes](https://github.com/ValuedMammal/deeprl/commit/ee790eb967ad25073fdbe33f1a88f6197a42e707) an issue where sending many text parameters for translation caused an existing param to be overwritten.

## [0.1.0] - 2023-07-12

- Publish deeprl on [crates.io](https://crates.io/crates/deeprl)