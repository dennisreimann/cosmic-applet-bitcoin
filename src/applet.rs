use crate::config::{AppConfig, ValueId};
use crate::fl;
use crate::mempool;
use chrono::{DateTime, Local};
use cosmic::app::{Core, Task};
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{widget::svg, Alignment, Length, Rectangle, window};
use cosmic::widget::{autosize, button, column, container, divider, row, text, Id};
use cosmic::{applet, theme, Element};
use std::collections::HashMap;
use std::sync::LazyLock;

pub const APP_ID: &str = "net.d11n.CosmicBitcoin";

const LOGO_SVG: &str = include_str!("../assets/bitcoin.svg");

/// Letzter bekannter Wert je Wert-ID; `Err(())` = fetch fehlgeschlagen.
pub type ValueMap = HashMap<ValueId, Result<f64, ()>>;

/// Panel-Zeile: Werte aller angewählten Werte in Auswahloreihenfolge.
/// Keine Auswahl → leerer String (dann nur das Logo).
pub fn format_panel_line(values: &ValueMap, selected: &[ValueId]) -> String {
    format_panel_line_lang(mempool::is_app_language_de(), values, selected)
}

/// Panel-Zeile mit expliziter Sprachauswahl (deterministisch testbar).
pub fn format_panel_line_lang(
    is_de: bool,
    values: &ValueMap,
    selected: &[ValueId],
) -> String {
    selected
        .iter()
        .map(|id| match values.get(id) {
            Some(Ok(v)) => mempool::format_value_lang(is_de, id, *v),
            _ => "—".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

/// Anzeigename einer Wert-ID (lokalisiert), z. B. "Kurs USD".
#[must_use]
pub fn value_display_name(id: &str) -> String {
    if let Some(cur) = id.strip_prefix("price:") {
        return fl!("value-price", cur = cur);
    }
    if let Some(tier) = id.strip_prefix("fee:") {
        return match tier {
            "fastest" => fl!("value-fee-fastest"),
            "halfHour" => fl!("value-fee-halfHour"),
            "hour" => fl!("value-fee-hour"),
            "economy" => fl!("value-fee-economy"),
            "minimum" => fl!("value-fee-minimum"),
            _ => id.to_string(),
        };
    }
    if id == "block:height" {
        return fl!("value-height");
    }
    match id {
        "diff:progress" => fl!("value-diff-progress"),
        "diff:change" => fl!("value-diff-change"),
        "diff:date" => fl!("value-diff-date"),
        "diff:blocks" => fl!("value-diff-blocks"),
        _ => id.to_string(),
    }
}

/// Popup-Zeilen: (Anzeigename, formatierter Wert) in Auswahloreihenfolge.
pub fn popup_rows(values: &ValueMap, selected: &[ValueId]) -> Vec<(String, String)> {
    popup_rows_lang(mempool::is_app_language_de(), values, selected)
}

/// Popup-Zeilen mit expliziter Sprachauswahl (deterministisch testbar).
pub fn popup_rows_lang(
    is_de: bool,
    values: &ValueMap,
    selected: &[ValueId],
) -> Vec<(String, String)> {
    selected
        .iter()
        .map(|id| {
            let value = match values.get(id) {
                Some(Ok(v)) => mempool::format_value_lang(is_de, id, *v),
                _ => "—".to_string(),
            };
            (value_display_name(id), value)
        })
        .collect()
}

/// Unix-Sekunden als lokale `HH:MM:SS`-Zeit.
#[must_use]
pub fn fmt_updated(unix: i64) -> String {
    DateTime::from_timestamp(unix, 0)
        .map(|d| d.with_timezone(&Local).format("%H:%M:%S").to_string())
        .unwrap_or_else(|| unix.to_string())
}

/// Mergt Fetch-Ergebnisse in die Wert-Map.
///
/// Regel: `Ok` überschreibt immer (neue Daten). `Err` überschreibt auch —
/// es entsteht nur bei einer **erfolgreichen** Antwort, in der genau dieser
/// Wert fehlt (→ „—“). Fehlgeschlagene *Gruppen* liefern von `fetch_values`
/// gar keine Einträge, dadurch bleiben die alten Werte bei Verbindungs-/API-
/// Fehlern unangetastet.
///
/// Rückgabe: `true`, wenn mindestens ein neuer Wert übernommen wurde —
/// nur dann läuft die „Aktualisiert“-Uhr weiter.
pub fn merge_fetched(values: &mut ValueMap, results: Vec<(ValueId, Result<f64, ()>)>) -> bool {
    let mut any = false;
    for (id, r) in results {
        any |= r.is_ok();
        values.insert(id, r);
    }
    any
}

/// Holt alle angewählten Werte parallel; leerer instance_url → default-Instanz.
async fn fetch_all(cfg: AppConfig) -> Vec<(ValueId, Result<f64, ()>)> {
    if cfg.values.is_empty() {
        return vec![];
    }
    let base = if cfg.instance_url.is_empty() {
        mempool::DEFAULT_INSTANCE.to_string()
    } else {
        cfg.instance_url.clone()
    };
    mempool::fetch_values(&base, &cfg.values)
        .await
        .into_iter()
        .map(|(id, r)| (id, r.map_err(|_| ())))
        .collect()
}

/// Wartet, bis sich die Config-Datei ändert (mtime), und liefert dann die
/// neue mtime. Ein Polling per stat() pro Sekunde ist negligible billig —
/// das erlaubt Auto-Refresh nach Speichern im Settings-Fenster ohne
/// Subscription-Mechanik.
async fn wait_for_config_change(
    baseline: Option<std::time::SystemTime>,
) -> Option<std::time::SystemTime> {
    let path = crate::config::config_path()?;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        if let Some(t) = mtime {
            if baseline.is_none_or(|b| t != b) {
                return Some(t);
            }
        }
    }
}

static AUTOSIZE_MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize-main"));

/// Resolves `name` as a sibling of `exe_path`.
pub fn sibling_binary(exe_path: &str, name: &str) -> String {
    match std::path::Path::new(exe_path).parent() {
        Some(dir) => dir.join(name).to_string_lossy().into_owned(),
        None => name.to_string(),
    }
}

/// Path of the settings binary next to the running applet binary.
pub fn settings_binary_path() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    Some(sibling_binary(&exe.to_string_lossy(), "cosmic-applet-bitcoin-settings"))
}

/// Startet das Applet (aufgerufen aus `main`).
pub fn run() -> cosmic::iced::Result {
    cosmic::applet::run::<BitcoinApplet>(())
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Button-Bounds (in Fenster-Koordinaten) für die Popup-Positionierung.
    TogglePopup(Rectangle),
    PopupClosed(window::Id),
    Fetched(Vec<(ValueId, Result<f64, ()>)>),
    Tick,
    // Config-Datei hat sich geändert (Settings-Fenster hat gespeichert).
    ConfigChanged(Option<std::time::SystemTime>),
    OpenSettings,
    Quit,
}

#[derive(Default)]
pub struct BitcoinApplet {
    core: Core,
    popup: Option<window::Id>,
    cfg: AppConfig,
    values: ValueMap,
    /// mtime der Config beim letzten Beobachten (Basis für den Watcher).
    config_mtime: Option<std::time::SystemTime>,
    /// Letzter abgeschlossener Fetch-Cycle (Unix-Sekunden).
    last_updated_unix: Option<i64>,
}

fn panel_element(state: &BitcoinApplet) -> Element<'_, Message> {
    let line = format_panel_line(&state.values, state.cfg.panel_values());
    let h = state.core.applet.suggested_size(true).0 as f32;
    if state.core.applet.is_horizontal() {
        let mut r = row![]
            .align_y(Alignment::Center)
            .spacing(state.core.applet.suggested_padding(true).0 / 2);
        r = r.push(
            container(
                svg(svg::Handle::from_memory(LOGO_SVG.as_bytes().to_vec()))
                    .width(Length::Fixed(h))
                    .height(Length::Fixed(h)),
            )
            .align_y(Alignment::Center),
        );
        if !line.is_empty() {
            r = r.push(state.core.applet.text(line));
        }
        r.into()
    } else {
        column![state.core.applet.text(line)]
            .align_x(Alignment::Center)
            .into()
    }
}

fn popup_element(state: &BitcoinApplet) -> Element<'_, Message> {
    let Spacing {
        space_xxs,
        space_s,
        space_m,
        ..
    } = cosmic::theme::active().cosmic().spacing;

    let mut col = column![]
        .padding([space_s, 0, space_xxs, 0])
        .spacing(space_xxs);

    // Kopfzeile: Titel links, "Aktualisiert" rechts (Muster OpenCode-Widget).
    let head: Element<'_, Message> = match state.last_updated_unix {
        Some(unix) => row![
            text::heading(fl!("popup-title")).width(Length::Fill),
            text::body(fl!("updated-at", t = fmt_updated(unix))),
        ]
        .align_y(Alignment::Center)
        .padding([0, space_m])
        .into(),
        None => container(text::heading(fl!("popup-title")))
            .padding([0, space_m])
            .into(),
    };
    col = col.push(head);

    col = col.push(container(divider::horizontal::default()).padding([space_xxs, 0]));

    // Werte-Zeilen (Name links, Wert rechts).
    let rows = popup_rows(&state.values, &state.cfg.values);
    if rows.is_empty() {
        col = col.push(container(text::body(fl!("no-values"))).padding([0, space_m]));
    } else {
        for (name, value) in rows {
            col = col.push(
                row![text::body(name).width(Length::Fill), text::body(value)]
                    .padding([0, space_m])
                    .spacing(space_s),
            );
        }
    }

    // Trennlinie direkt über der Beenden-Zeile: kein unteres Padding,
    // damit der Abstand zur Menü-Zeile nicht zu groß wird.
    col = col.push(container(divider::horizontal::default()).padding([space_xxs, 0, 0, 0]));
    col = col.push(applet::menu_button(text::body(fl!("settings"))).on_press(Message::OpenSettings));
    col = col.push(applet::menu_button(text::body(fl!("quit"))).on_press(Message::Quit));
    container(col).into()
}

impl cosmic::Application for BitcoinApplet {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn init(core: Core, _flags: ()) -> (Self, Task<Message>) {
        let mut app = Self {
            core,
            popup: None,
            cfg: AppConfig::default(),
            values: ValueMap::new(),
            config_mtime: None,
            last_updated_unix: None,
        };
        app.cfg = crate::config::load();
        let cfg = app.cfg.clone();
        (
            app,
            Task::batch(vec![
                Task::perform(fetch_all(cfg), |r| {
                    cosmic::Action::App(Message::Fetched(r))
                }),
                Task::perform(wait_for_config_change(None), |t| {
                    cosmic::Action::App(Message::ConfigChanged(t))
                }),
            ]),
        )
    }

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TogglePopup(bounds) => {
                self.cfg = crate::config::load();
                if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = window::Id::unique();
                    self.popup = Some(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size = None;
                    // Anker = gesamtes Widget (nicht nur das Icon-Quadrat),
                    // damit das Popup zentriert darunter erscheint.
                    popup_settings.positioner.anchor_rect = Rectangle {
                        x: bounds.x as i32,
                        y: bounds.y as i32,
                        width: bounds.width.max(1.0) as i32,
                        height: bounds.height.max(1.0) as i32,
                    };
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
                Task::none()
            }
            Message::Fetched(results) => {
                if crate::applet::merge_fetched(&mut self.values, results) {
                    self.last_updated_unix = Some(chrono::Utc::now().timestamp());
                }
                let interval = crate::config::refresh_interval(&self.cfg);
                Task::perform(tokio::time::sleep(interval), move |_| {
                    cosmic::Action::App(Message::Tick)
                })
            }
            Message::Tick => {
                let cfg = self.cfg.clone();
                Task::perform(fetch_all(cfg), |r| {
                    cosmic::Action::App(Message::Fetched(r))
                })
            }
            Message::ConfigChanged(mtime) => {
                self.config_mtime = mtime;
                self.cfg = crate::config::load();
                let cfg = self.cfg.clone();
                Task::batch(vec![
                    Task::perform(fetch_all(cfg), |r| {
                        cosmic::Action::App(Message::Fetched(r))
                    }),
                    Task::perform(wait_for_config_change(mtime), |t| {
                        cosmic::Action::App(Message::ConfigChanged(t))
                    }),
                ])
            }
            Message::OpenSettings => {
                match crate::applet::settings_binary_path() {
                    Some(path) => {
                        let child = std::process::Command::new(path).spawn();
                        if let Err(e) = child {
                            tracing::error!("Settings-Fenster starten: {e}");
                        }
                        Task::none()
                    }
                    None => {
                        tracing::error!("Settings-Binary nicht gefunden");
                        Task::none()
                    }
                }
            }
            Message::Quit => {
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let padding = self.core.applet.suggested_padding(true).0;
        let btn = button::custom(panel_element(self))
            .padding(if self.core.applet.is_horizontal() {
                [0, padding]
            } else {
                [padding, 0]
            })
            .class(theme::Button::AppletIcon)
            .on_press_with_rectangle(move |offset, bounds| {
                Message::TogglePopup(Rectangle {
                    x: bounds.x - offset.x,
                    y: bounds.y - offset.y,
                    width: bounds.width,
                    height: bounds.height,
                })
            });

        autosize::autosize(btn, AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn view_window(&self, _id: window::Id) -> Element<'_, Message> {
        self.core
            .applet
            .popup_container(popup_element(self))
            .into()
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(applet::style())
    }
}
