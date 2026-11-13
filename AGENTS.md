# AGENTS.md

Guidelines and context for AI agents working on this repository.

## Project

**cosmic-applet-bitcoin** — A native COSMIC panel applet that displays
configurable [mempool.space](https://mempool.space/) values in the panel:
BTC prices per currency, fee estimates and the current block height.

## Architecture (inherited from the source project)

- **Native COSMIC applet** (`cosmic::applet::run`), **NOT** an SNI/ksni
  tray item. Panel: `button::custom(panel_element)` with
  `theme::Button::AppletIcon`, wrapped in `autosize::autosize`.
- **Popup:** `on_press_with_rectangle` → `Message::TogglePopup(bounds)` →
  `window::Id::unique()` + `core.applet.get_popup_settings(...)` with
  `positioner.size = None`; close via `destroy_popup` +
  `on_close_requested`. Popup content in `core.applet.popup_container(...)`.
- **Settings window:** separate window is impossible from the applet
  process (privileged panel wayland socket, only layer surfaces) — the
  applet SPAWNS the helper binary `cosmic-applet-bitcoin-settings`
  (`src/bin/`, path resolved beside `current_exe`, see
  `applet::settings_binary_path()`).
- **Text colors:** do NOT use iced's `Text::color_maybe`; use
  `.class(color)` on the text widget instead.
- **Refresh:** self-rescheduling task chain (`init` → `fetch_all` →
  `Fetched` → `sleep(interval)` → `Tick` → fetch …). A 1 s `stat()`-watcher
  on the config file picks up settings saves automatically (no restart).
- Deprecation note: if the popup ever stops showing, check the applet
  example in the libcosmic checkout and `src/applet/mod.rs`.

## Value model

- `src/mempool.rs`: value IDs (`price:<CURRENCY>`, `fee:<tier>`,
  `block:height`) + REST fetch. Per cycle only the endpoints needed by the
  selection are requested (`/api/v1/prices`, `/api/v1/fees/precise`,
  `/api/blocks/tip/height`). Pure parsers + `format_value()`
  (thousands separator with space, fees as `x.xx sat/vB`).
- Default instance: `https://mempool.space`. Fee JSON keys:
  `fastestFee`, `halfHourFee`, `hourFee`, `economyFee`, `minimumFee`.
- Cross-boundary move semantics: moving a value across the panel line
  changes only THAT value's membership (`config::move_value_across`).
- Settings rows are dropdown-based (libcosmic `dropdown`) — the currency
  list comes from the configured instance's `/api/v1/prices`
  (fallback: `mempool::DEFAULT_CURRENCIES`).

## Module structure

- `src/config.rs` — TOML `AppConfig` (`~/.config/cosmic-applet-bitcoin/config.toml`,
  `0600`): instance URL, interval, value list, panel line position.
- `src/mempool.rs` — value IDs, REST client, pure parsers/formatters.
- `src/applet.rs` — the libcosmic applet (panel, popup, fetch chain,
  watcher, settings spawn).
- `src/bin/cosmic-applet-bitcoin-settings.rs` — settings window.
- `src/i18n.rs` + `i18n/<lang>/cosmic_applet_bitcoin.ftl` — Fluent strings
  (EN + DE, `fl!` validates against the EN catalog at compile time).

## Testing & building

- `cargo test` — mempool parsers/formatters, config roundtrip,
  panel/popup formatting, i18n, settings-spawn path.
- `cargo clippy --all-targets -- -D warnings`.
- `cargo build --release`, then `./install.sh`
  (binaries → `~/.local/bin`, desktop entry + icon →
  `~/.local/share/...`).
- `./restart-applet.sh` toggles the panel entry for a hot reload
  (`~/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings`).
- Before a release: update dependencies to the latest versions
  (`cargo update` in Cargo.lock; check for available major bumps in
  `Cargo.toml`), then run the full test suite and clippy again.

## Reliability patterns (v1.3.0)

- **Refresh sequencing:** `src/refresh.rs` (`RefreshFlow`) — a generation
  counter plus one timer slot. Every fetch/config change calls `start()`
  (invalidates old results and their pending timers); a completed fetch is
  only applied when its generation is current (`finished()`), and exactly
  one timer is installed afterwards. Messages carry the generation:
  `Fetched(u64, FetchResponse)` / `Tick(u64)`. No parallel refresh chains
  accumulate when the config changes mid-flight.
- **Fetch result envelope:** `mempool::FetchResponse { values, errors }`.
  A failed group (network, HTTP status, bad JSON) produces NO entries — old
  values stay — and one error string; a missing value inside a successful
  response produces `Err(())` (→ „—“ placeholder). `merge_fetched` keeps
  that contract. The popup renders group errors in a `fetch-error` line
  next to the last known values.
- **Config robustness:** `config::load_report()` distinguishes a missing
  file (defaults, no error) from an invalid/unreadable TOML (defaults plus
  an error string shown in the settings window — never silently reinterpreted).
  `save_to` writes atomically (`tempfile` in the config dir, `0600` before
  persist). Interval must be a whole number 1–1440 minutes
  (`settings::parse_refresh_minutes`).
- **Settings window:** `rebuild_catalog` uses
  `settings::merge_catalog` — saved value IDs that the live catalog does
  not offer are preserved, never dropped on reload. Invalid intervals and
  save failures set `self.error` (shown via `error-line`) and keep the
  window open.
- **Installer/packaging:** `install.sh` works from a source checkout
  (`target/release/…`) and from an extracted release archive (binaries +
  `data/*.desktop` + `assets/*.svg` side by side, from any cwd). CI must
  package the settings binary and the icon into the tar.gz.
- Tests for the pure boundaries: `tests/refresh.rs` (timer semantics),
  `tests/settings.rs` (catalog merge, interval validation),
  `tests/mempool.rs` (group-failure errors + last-known-good merge).
