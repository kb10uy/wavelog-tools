# AGENTS.md of Wavelog Tools

## Overview

Wavelog Tools is a single Rust binary (`wavelog-tools`) that bundles tools using the
[Wavelog](https://www.wavelog.org/) REST API v2. Each tool is a clap subcommand.

- `qcgen`: generates JSON data for QSL cards from ADIF (local file or Wavelog export) through a Lua script

## Project Layout

- `src/main.rs`: entry point, dispatches subcommands
- `src/cli.rs`: top-level clap definitions (`Cli`, `Command`)
- `src/wavelog.rs`, `src/wavelog/`: Wavelog API v2 client, independent from each tool
- `src/qso.rs`, `src/qso/`: common QSO data extracted from ADIF records
- `src/schope.rs`, `src/schope/`: Lua scripting engine and Lua-facing data types
- `src/<tool>.rs`, `src/<tool>/`: implementation of each subcommand (e.g. `qcgen`)
- `assets/`: example files, Lua scripts and Lua type definitions

### Adding a Tool

1. Create `src/<tool>.rs` exposing `Arguments` (`clap::Args`) and `run(args)`
2. Add a variant to `Command` in `src/cli.rs` and dispatch it in `src/main.rs`
3. Put API calls in `src/wavelog/` rather than in the tool module

## Commands

- Build: `cargo build`
- Lint: `cargo clippy --all-targets`
- Format: `cargo fmt`
- Test: `cargo test`

## Conventions

- **Use English**

### Rust

- Use `modname.rs` with `modname/` for modules that have submodules; _do not_ use `modname/mod.rs`
- Run `cargo fmt` and keep `cargo clippy` free of warnings
- Keep Wavelog API types and requests inside `src/wavelog/`

### Text Files

- Use LF line endings and UTF-8
- Follow `.editorconfig`

### Doc Comments

- Use third-person singular, present tense, active voice
- End with period except for listings
- Use capital letter at the beginning of the sentence

### Commit Message, Issue and PR

- Use imperative mood
- _Do not_ use conventional commit message

### Branch

- Use lowercase
- Hyphenate to separate words
- Use categories below:
    - feature/
    - fix/
    - refactor/
    - docs/
    - test/

### Documentations for Humans

- Use English or Japanese

### File Arrangement

- Example files should be placed under `assets/`
- Create subdirectories if necessary
