# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
