# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`dioxus-iconify` is a CLI tool that generates Iconify icons as Rust code for Dioxus projects. Unlike traditional icon libraries that bundle all icons as dependencies, this tool takes a shadcn-ui inspired approach: it generates icon code directly in the user's project at build time, eliminating runtime dependencies.

The tool fetches icons from the Iconify API (275,000+ icons from 200+ collections) and generates type-safe Rust constants with an `Icon` component.

## Build and Development Commands

This project uses `mise` for task management. Always use `mise` tasks instead of direct `cargo` commands:

```bash
# Build the project
mise run build

# Run tests (uses cargo-nextest)
mise run test              # All tests
mise run test:unit         # Unit tests only

# Linting
mise run lint              # All linting
mise run lint:rust         # Rust-specific linting (fmt check + clippy)

# Format code
mise run format            # Format code and sort dependencies

# CI workflow (runs info, lint, test)
mise run ci

# Update dependencies
mise run deps:update

# View all available tasks
mise tasks
```

### Running a Single Test

```bash
# Using cargo-nextest (preferred)
cargo nextest run test_name

# Or with standard cargo test
cargo test test_name
```

## Architecture

### Code Structure

- **src/main.rs**: CLI entry point using `clap`. Parses commands (`add`, `init`) and orchestrates the workflow.
- **src/api.rs**: Iconify API client. Fetches icon data from `https://api.iconify.design` using `reqwest`. Handles JSON responses and ensures icons have proper dimensions and viewBox.
- **src/naming.rs**: Icon identifier parsing and naming conversions. Converts `collection:icon-name` format to valid Rust identifiers (PascalCase constants, handles keywords and leading numbers).
- **src/generator.rs**: Core code generation logic. Creates/updates icon files, manages collections, parses existing icons to enable incremental updates.

### Code Generation Flow

1. **Parse icon identifiers** (`naming.rs`): `mdi:home` → `IconIdentifier { collection: "mdi", icon_name: "home" }`
2. **Fetch from API** (`api.rs`): Download icon SVG data, dimensions, and viewBox
3. **Group by collection** (`generator.rs`): Organize icons by their collection prefix
4. **Generate/update files** (`generator.rs`):
   - Create `src/icons/` directory if needed
   - Generate or update collection files (e.g., `mdi.rs`, `heroicons.rs`)
   - Update `mod.rs` with Icon component and module declarations
5. **Incremental updates**: Parser reads existing icon files to preserve previously generated icons when adding new ones

### Generated Code Structure

When users run `dioxus-iconify add mdi:home`, it generates:

```
src/icons/
├── mod.rs         # Icon component + IconData struct + module declarations
└── mdi.rs         # Collection-specific icons as const IconData
```

The `Icon` component uses Dioxus 0.7's `#[props(extends = GlobalAttributes)]` pattern, allowing users to override any SVG attribute (size, color, etc.).

## Key Design Decisions

### Icon Constant Naming
- Icons are generated as `pub const` (not `pub static` or enums) with `#[allow(non_upper_case_globals)]`
- Uses PascalCase for flexibility to later switch to struct/enum if needed
- Handles edge cases: leading numbers (prefix with `_`), Rust keywords (append `Icon`)

### Incremental Code Generation
- The generator parses existing collection files to preserve previously added icons
- Uses simple string-based parsing (not regex) to extract IconData from generated files
- BTreeMap ensures alphabetical ordering of icons within collection files

### Collection Organization
- Icons grouped by collection in separate files (e.g., `mdi.rs`, `heroicons.rs`)
- Collection names with hyphens converted to underscores for module names (`simple-icons` → `simple_icons`)

## Clippy Configuration

The project has specific clippy rules in `.clippy.toml`:
- **Disallowed macro**: `std::dbg` - Use during development but don't commit to main branch
- Print macros (`println!`, `eprintln!`) are allowed since this is a CLI tool that reports to users on console
- Tests are allowed to use `expect` and `unwrap`

## Testing Strategy

- **Unit tests**: Test parsing, naming conversions, etc. Run with `mise run test`
- **Integration tests**: Located in `tests/` directory
  - `test_generated_icons_are_valid_rust`: Tests icon generation without network (always runs)
  - `test_generated_code_compiles_without_warnings`: Tests that generated code compiles cleanly with `#![deny(warnings)]` (requires network, marked `#[ignore]`)
- API tests are marked `#[ignore]` since they require internet connection
- To run internet-dependent tests: `cargo nextest run --run-ignored ignored-only`
- Tests use blocking reqwest client for simplicity (CLI tool, not async runtime)

### Running Specific Tests

```bash
# Run all tests (excluding ignored)
mise run test

# Run a specific test by name
cargo nextest run test_name

# Run only ignored tests (require network)
cargo nextest run --run-ignored ignored-only

# Run all tests including ignored
cargo nextest run --run-ignored all
```
