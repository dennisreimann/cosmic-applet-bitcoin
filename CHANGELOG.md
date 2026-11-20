# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The **Unreleased** section accumulates changes as they land; its entries are
used verbatim as release notes when cutting a new release (the section is
then renamed to the new version and emptied).

## [Unreleased]

### Dependencies

- `cargo update` works fully again: upstream renamed
  `cosmic-settings-daemon` and `switcheroo-control` in
  dbus-settings-bindings `main` (to `-zbus`, zbus 6), while the pinned
  libcosmic rev still requires the old names. Both crates are now
  vendored under `vendor/` (MPL-2.0 snapshots at the revision the build
  already used) and attached via `[patch]`; all other dependencies of
  that repo (and the rest of the tree) update normally.
- Dependency refresh via full `cargo update` (git dependencies of the
  pinned libcosmic tree: cosmic-panel, cosmic-protocols,
  freedesktop-icons, …).

## [1.3.2] - 2026-11-20

### Changed

- Fee estimates served by the `/api/v1/fees/recommended` fallback (integer
  values, older Mempool instances) are now shown without decimal places
  (`3 sat/vB`); values from `/api/v1/fees/precise` keep two decimals
  (`3,00 sat/vB`).

### Dependencies

- reqwest 0.12 → 0.13: the removed `rustls-tls-native-roots` feature is
  replaced by `rustls` (now backed by rustls-platform-verifier), which
  still trusts the system CA store so instances with custom CAs (e. g.
  Start9) keep working.
- rust-embed 8.13 → 8.14, httpmock (dev) 0.7 → 0.9; all other direct
  crates.io dependencies are already on the latest stable release.
  Full `cargo update` stays blocked by the libcosmic dependency tree
  (upstream dbus-settings-bindings master no longer carries
  `cosmic-settings-daemon`); git dependencies remain pinned as intended.

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
