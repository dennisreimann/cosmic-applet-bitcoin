# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The **Unreleased** section accumulates changes as they land; its entries are
used verbatim as release notes when cutting a new release (the section is
then renamed to the new version and emptied).

## [Unreleased]

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
