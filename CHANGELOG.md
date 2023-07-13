# Changelog

All notable changes will be documented in this file. 
This project adheres to semantic versioning.

## [Unreleased]

- In progress

## [0.1.1] - 2023-07-13

### Added

- adds ability to construct a Formality from a string. (impl FromStr)
- allow displaying `Document`, `LanguageInfo`, and `Glossary`. (derive serde::Serialize)

### Fixed

-  [fixes](https://github.com/ValuedMammal/deeprl/commit/ee790eb967ad25073fdbe33f1a88f6197a42e707) an issue where sending many text parameters for translation caused an existing param to be overwritten.

## [0.1.0] - 2023-07-12

- Publish deeprl on [crates.io](https://crates.io/crates/deeprl)