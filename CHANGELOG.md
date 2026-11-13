# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The **Unreleased** section accumulates changes as they land; its entries are
used verbatim as release notes when cutting a new release (the section is
then renamed to the new version and emptied).

## [Unreleased]

## [1.3.1] - 2026-11-13

### Fixed

- Dependency refresh closes the Dependabot findings (rustls ≥ 0.23.46,
  quinn-proto ≥ 0.11.15) via `cargo update` + targeted bumps; direct
  dependencies updated to dirs 7 and toml 1.
- libcosmic is now pinned to the verified revision so `cargo update`
  cannot silently move the applet's UI base (see AGENTS.md).

## [1.3.0] - 2026-11-13

### Fixed

- Fractional BTC prices rendered doubled decimal separators
  (`79.941,,23 $`); now exactly one localized separator with cent
  rounding.
- Failed fetch groups no longer vanish silently: last known values stay
  and the popup shows the error cause. Refresh errors no longer clear
  good values.
- A settings save mid-refresh no longer spawns parallel refresh chains
  (generation-based sequencing; stale results and timers are discarded).
- Settings no longer drop saved currencies that the live catalog does
  not offer; invalid intervals and save failures are shown instead of
  silently applied.
- Config writes are atomic (tempfile, 0600 before persist); invalid TOML
  is reported in the settings window instead of silently replaced by
  defaults.
- Release archives now ship the settings binary and icon; `install.sh`
  works from a source checkout and an extracted archive, from any cwd.

## [1.2.0] - 2026-11-09

### Added

- Fee estimates now fall back to `/api/v1/fees/recommended` on mempool
  instances that don't provide `/api/v1/fees/precise` (older mempool
  versions).

## [1.1.0] - 2026-10-30

### Fixed

- Failed fetch cycles (network errors, HTTP errors, invalid responses) now
  keep the last known values and clock instead of wiping them.
- Values missing from an otherwise valid response are shown as a dash
  placeholder instead of being dropped.

### Changed

- CI and development docs updates.

## [1.0.0] - 2026-10-29

### Added

- Initial release: native COSMIC panel applet showing configurable
  [mempool.space](https://mempool.space/) values in the panel — BTC prices
  in any supported currency, fee estimates and the current block height.
- Settings window (`cosmic-applet-bitcoin-settings`) for configuring the
  mempool instance, refresh interval and the displayed values.
- TOML-based config (`~/.config/cosmic-applet-bitcoin/config.toml`) with
  hot-reload on save.

[Unreleased]: https://github.com/dennisreimann/cosmic-applet-bitcoin/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/dennisreimann/cosmic-applet-bitcoin/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/dennisreimann/cosmic-applet-bitcoin/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/dennisreimann/cosmic-applet-bitcoin/releases/tag/v1.0.0
