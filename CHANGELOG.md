# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.2](https://github.com/davidB/dioxus-iconify/compare/0.4.1...0.4.2) - 2025-12-07

### Fixed

- enclose url in documentation

## [0.4.1](https://github.com/davidB/dioxus-iconify/compare/0.4.0...0.4.1) - 2025-12-03

### Fixed

- the generated svg should have SVG's attributes, not global html

### Other

- update tagline

## [0.4.0](https://github.com/davidB/dioxus-iconify/compare/0.3.0...0.4.0) - 2025-11-27

### Added

- icons can be added from a local svg file or folder
- icons can be added from a local svg file or folder

### Fixed

- extraction of collection info/metadata

### Other

- format & lint
- *(test)* remove the `lib.rs` (exposed only for test) and use the

## [0.3.0](https://github.com/davidB/dioxus-iconify/compare/0.2.3...0.3.0) - 2025-11-27

### Added

- preserve visibility of sub modules in `icons/mod.rs`
- add information (license, author,...) about the icon collections

### Other

- how to create application inconset with aliases
- update sample code in README

## [0.2.3](https://github.com/davidB/dioxus-iconify/compare/0.2.2...0.2.3) - 2025-11-26

### Fixed

- the format of generated collection

## [0.2.2](https://github.com/davidB/dioxus-iconify/compare/0.2.1...0.2.2) - 2025-11-26

### Fixed

- Option for `size` failed to compile

## [0.2.1](https://github.com/davidB/dioxus-iconify/compare/0.2.0...0.2.1) - 2025-11-26

### Fixed

- `update` command also update the `mod.rs`

### Other

- update the installation instruction

## [0.2.0](https://github.com/davidB/dioxus-iconify/compare/0.1.1...0.2.0) - 2025-11-26

### Added

- add the `Icon.size` optional attributes to set width and height to
- add the `update` command
- add the `list` command

### Other

- fix the repository url
- replace blocking by async for reqwest
- update badges
- replace dependency to `openssl` by `rustls`
- update the tagline

## [0.1.1](https://github.com/davidB/dioxus-iconify/compare/0.1.0...0.1.1) - 2025-11-26

### Fixed

- generate code without warning

### Other

- refactor and add a test to validate the generated code compiles
- add CLAUDE.md
- update Comparison table, remove license section
