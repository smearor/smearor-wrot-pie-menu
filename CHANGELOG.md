# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

### Changed

### Fixed

### Distribution

### Infrastructure

---

## [0.2.0] - 2026-08-15

### Changed

- Extracted `smearor-wrot-pie-menu` as a standalone crate from `smearor-wrot`
- Self-contained `RgbaColor` and `RgbColor` types with hex parsing
- `MenuItem` with `TypedBuilder` construction
- `PieMenuMessage` generalized to `Rotate(f32)` and `Event(String)` only
- Configurable menu items via `PieMenuMenuItemHandler` trait (no hardcoded `DefaultMenuProvider`)
- Book documentation
- Infrastructure files (.github, .run, book, AGENTS.md, etc.)

