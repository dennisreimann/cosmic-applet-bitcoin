# AGENTS.md

Guidelines and context for AI agents working on this repository.

## Project

**cosmic-applet-bitcoin** — A native COSMIC panel applet (fork of the
author's `cosmic-applet-homeassistant`) that displays configurable
[mempool.space](https://mempool.space/) values in the panel: BTC prices
per currency, fee estimates and the current block height.

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
