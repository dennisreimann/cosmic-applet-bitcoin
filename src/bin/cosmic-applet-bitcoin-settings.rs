//! Eigenständiges Einstellungsfenster für das Applet.
//!
//! Das Applet selbst kann aus dem Panel-Applet-Kontext keine normalen
//! Fenster öffnen (privilegierter Panel-Wayland-Socket, nur
//! Layer-Surfaces). Das Applet spawnt daher dieses Binary
//! (`Message::OpenSettings` → `settings_binary_path()`).

use cosmic::app::{Core, Task};
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::{Alignment, Length, window};
use cosmic::widget::{
    button, column, container, divider, dropdown, row, scrollable, text, text_input,
};
use cosmic::{Application, Element};
use cosmic_applet_bitcoin::applet::value_display_name;
use cosmic_applet_bitcoin::config::{self, AppConfig};
use cosmic_applet_bitcoin::fl;
use cosmic_applet_bitcoin::mempool;

fn main() -> cosmic::iced::Result {
    cosmic::app::run::<SettingsApp>(
        cosmic::app::Settings::default()
            .size(cosmic::iced::Size::new(550.0, 640.0))
            // Breite fix (nur in der Höhe skalierbar). Height-Grenzen
            // explizit endlich — max-Höhe INFINITY erzeugt beim xdg_toplevel
            // einen negativen Wert und killt das Fenster.
            .size_limits(
                cosmic::iced::Limits::NONE
                    .min_width(550.0)
                    .min_height(400.0)
                    .max_width(550.0)
                    .max_height(1200.0),
            ),
        (),
    )
}

/// Feste Breite aller Input-Felder im Settings-Fenster.
const INPUT_WIDTH: f32 = 170.0;

#[derive(Debug, Clone)]
enum Message {
    UrlChanged(String),
    RefreshMinutesChanged(String),
    /// Währungscodes der konfigurierten Instanz (Fallback: default-Liste).
    CurrenciesLoaded(Vec<String>),
    RowValueSelected(usize, usize),
    MoveRowUp(usize),
    MoveRowDown(usize),
    BoundaryUp,
    BoundaryDown,
    AddRow,
    RemoveRow(usize),
    Save,
}

#[derive(Default)]
struct SettingsApp {
    core: Core,
    url: String,
    refresh_minutes: String,
    /// Grenzindex: rows[..boundary] erscheint im Panel. rows.len() = alle.
    boundary: usize,
    /// Ausgewählte Werte (IDs) je Zeile.
    rows: Vec<String>,
    /// Katalog der wählbaren Werte (IDs): Preise der Instanz + Fees + Höhe.
    catalog: Vec<mempool::ValueId>,
}

impl SettingsApp {
    /// Katalog neu aufbauen (nach Währungs-Fetch): Preise vor Fees/height.
    fn rebuild_catalog(&mut self, currencies: Vec<String>) {
        self.catalog = mempool::known_value_ids(&currencies);
    }

    fn catalog_index(&self, id: &str) -> Option<usize> {
        self.catalog.iter().position(|s| s == id)
    }
}

impl Application for SettingsApp {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "net.d11n.CosmicBitcoinSettings";

    fn init(core: Core, _flags: ()) -> (Self, Task<Message>) {
        let cfg: AppConfig = cosmic_applet_bitcoin::config::load();
        let main_id = core.main_window_id().unwrap();
        let mut app = Self {
            core,
            url: cfg.instance_url,
            refresh_minutes: cfg.refresh_minutes.to_string(),
            boundary: match cfg.panel_count {
                Some(n) => n.min(cfg.values.len()),
                None => cfg.values.len(),
            },
            rows: cfg.values.clone(),
            catalog: mempool::known_value_ids(mempool::DEFAULT_CURRENCIES.iter().map(|s| s.to_string()).collect::<Vec<_>>().as_slice()),
        };
        app.core.set_header_title(fl!("settings-title"));
        let title_task = cosmic::ApplicationExt::set_window_title(
            &mut app,
            fl!("settings-title"),
            main_id,
        );

        // Währungen der konfigurierten Instanz laden (fallback: default).
        let base = if app.url.is_empty() {
            mempool::DEFAULT_INSTANCE.to_string()
        } else {
            app.url.clone()
        };
        (
            app,
            Task::perform(
                async move {
                    cosmic_applet_bitcoin::mempool::fetch_prices(&base)
                        .await
                        .map(|p| p.into_iter().map(|(c, _)| c).collect::<Vec<_>>())
                        .unwrap_or_else(|_| {
                            mempool::DEFAULT_CURRENCIES.iter().map(|s| s.to_string()).collect()
                        })
                },
                |r| cosmic::Action::App(Message::CurrenciesLoaded(r)),
            )
            .chain(title_task),
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
            Message::UrlChanged(v) => self.url = v.trim().to_string(),
            Message::RefreshMinutesChanged(v) => self.refresh_minutes = v,
            Message::CurrenciesLoaded(currencies) => {
                self.rebuild_catalog(currencies);
                // Ungültige Selektionen entfernen: Zeilen mit IDs, die im
                // Katalog nicht mehr existieren, aus der Auswahl werfen.
                self.rows = self
                    .rows
                    .iter()
                    .filter(|id| self.catalog_index(id).is_some())
                    .cloned()
                    .collect();
                self.boundary = self.boundary.min(self.rows.len());
            }
            Message::RowValueSelected(i, idx) => {
                if let Some(id) = self.catalog.get(idx) {
                    if let Some(row) = self.rows.get_mut(i) {
                        *row = id.clone();
                    }
                }
            }
            Message::MoveRowUp(i) => {
                if i > 0 {
                    config::move_value_across(&mut self.rows, &mut self.boundary, i, i - 1);
                }
            }
            Message::MoveRowDown(i) => {
                if i + 1 < self.rows.len() {
                    config::move_value_across(&mut self.rows, &mut self.boundary, i, i + 1);
                }
            }
            Message::BoundaryUp => {
                if self.boundary > 0 {
                    self.boundary -= 1;
                }
            }
            Message::BoundaryDown => {
                if self.boundary < self.rows.len() {
                    self.boundary += 1;
                }
            }
            Message::AddRow => {
                self.rows
                    .push(mempool::known_value_ids(&[]).into_iter().last().unwrap_or_else(|| "block:height".to_string()));
            }
            Message::RemoveRow(i) => config::remove_row(&mut self.rows, &mut self.boundary, i),
            Message::Save => {
                let refresh_minutes: u32 =
                    self.refresh_minutes.trim().parse().unwrap_or(1);
                let cfg = AppConfig {
                    instance_url: self.url.trim().trim_end_matches('/').to_string(),
                    panel_count: (self.boundary != self.rows.len()).then_some(self.boundary),
                    refresh_minutes: refresh_minutes.clamp(1, 1440),
                    values: self.rows.clone(),
                };
                if let Err(e) = config::save(&cfg) {
                    eprintln!("Config speichern fehlgeschlagen: {e}");
                }
                return window::close(self.core.main_window_id().unwrap());
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        self.view_window(self.core.main_window_id().unwrap())
    }

    fn view_window(&self, _id: window::Id) -> Element<'_, Message> {
        let binding = cosmic::theme::active();
        let Spacing {
            space_xxxs,
            space_xxs,
            space_s,
            space_m,
            ..
        } = binding.cosmic().spacing;

        let mut col = column![].padding(space_m).spacing(space_s).width(Length::Fill);

        // Label + Input als Gruppe: Abstand halbiert (space_xxxs statt
        // space_s), damit das Label optisch am Input hängt.
        fn labeled<'a>(
            label: String,
            input: cosmic::widget::TextInput<'a, Message>,
            spacing: f32,
        ) -> cosmic::widget::Column<'a, Message, cosmic::Theme> {
            cosmic::widget::column![text::body(label), input]
                .spacing(spacing)
                .width(Length::Fill)
        }

        col = col.push(labeled(
            fl!("instance-url"),
            text_input::text_input(mempool::DEFAULT_INSTANCE, &self.url)
                .on_input(Message::UrlChanged)
                .width(Length::Fill),
            space_xxxs as f32,
        ));

        col = col.push(
            column![
                text::body(fl!("refresh-interval")),
                row![
                    text_input::text_input(fl!("refresh-interval"), &self.refresh_minutes)
                        .on_input(Message::RefreshMinutesChanged)
                        .width(Length::Fixed(INPUT_WIDTH)),
                    text::body(fl!("minutes")),
                ]
                .align_y(Alignment::Center)
                .spacing(space_xxs),
            ]
            .spacing(space_xxxs),
        );

        let labels: Vec<String> = self.catalog.iter().map(|id| value_display_name(id)).collect();

        let trash_button = |i: usize| {
            // Entfernen-Kreuz (X) in kräftigem Rot, quadratisch in
            // Input-Höhe, runde Ecken via Button-Style.
            let close_icon = cosmic::widget::icon::icon(
                cosmic::widget::icon::Handle::from(
                    cosmic::widget::icon::from_name("window-close-symbolic"),
                ),
            )
            .size(16)
            .class(cosmic::theme::Svg::custom(move |_| {
                cosmic::iced::widget::svg::Style {
                    color: Some(cosmic::iced::Color::from_rgb(0.83, 0.11, 0.11)),
                }
            }));
            button::custom(close_icon)
                .padding(f32::from(space_xxs) + 2.0)
                .on_press(Message::RemoveRow(i))
        };
        let arrow = |name: &str, msg: Message| {
            button::icon(cosmic::widget::icon::Handle::from(
                cosmic::widget::icon::from_name(name),
            ))
            .on_press(msg)
        };

        // Zeile mit der Trennlinie: Divider füllt bis zum Button-Cluster;
        // Pfeile + Platzhalter (≈ Trash-Breite) spiegeln das Muster der
        // Entity-Zeilen, damit die Pfeile exakt übereinanderstehen.
        let boundary_row = || {
            let ind = "  ";
            let _ = ind;
            row![
                divider::horizontal::default().width(Length::Fill),
                arrow("go-up-symbolic", Message::BoundaryUp),
                arrow("go-down-symbolic", Message::BoundaryDown),
                container(row![]).width(Length::Fixed(36.0)),
            ]
            .spacing(space_xxs)
            .align_y(Alignment::Center)
            .width(Length::Fill)
        };

        col = col.push(
            container(text::heading(fl!("values-header")))
                .padding([space_xxs, 0, 0, 0]),
        );

        for (i, row_id) in self.rows.iter().enumerate() {
            if i == self.boundary {
                // Trennlinie: darüber steht, was im Panel erscheint. Immer
                // sichtbar — auch an erster/letzter Position, sonst lässt
                // sie sich nicht zurückholen.
                col = col.push(boundary_row());
            }
            col = col.push(
                row![
                    dropdown::dropdown::<String, Message>(
                        labels.to_vec(),
                        self.catalog_index(row_id),
                        move |idx| Message::RowValueSelected(i, idx),
                    )
                    .width(Length::Fill),
                    // Button-Cluster rechtsbündig, damit er bei allen
                    // Zeilen auf derselben x-Position endet.
                    arrow("go-up-symbolic", Message::MoveRowUp(i)),
                    arrow("go-down-symbolic", Message::MoveRowDown(i)),
                    trash_button(i),
                ]
                .spacing(space_xxs)
                .align_y(Alignment::Center)
                .width(Length::Fill),
            );
        }
        if !self.rows.is_empty() && self.boundary == self.rows.len() {
            // Linie an letzter Position: hinter allen Zeilen ausgeben.
            col = col.push(boundary_row());
        }
        col = col.push(
            row![
                button::standard(fl!("value-add")).on_press(Message::AddRow),
                // Speichern rechtsbündig
                container(row![]).width(Length::Fill),
                button::suggested(fl!("save")).on_press(Message::Save),
            ]
            .spacing(space_s)
            .width(Length::Fill),
        );

        container(scrollable(col))
            .width(Length::Fill)
            .into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        None
    }
}
