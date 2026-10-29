//! Konfiguration des Applets.
//!
//! Persistente Konfiguration liegt als TOML-Datei unter
//! `~/.config/cosmic-applet-bitcoin/config.toml` (Instanz-URL, Intervall,
//! Auswahl an Werten und Panel-Grenze). Fehlt das File komplett, greift
//! der Default (mempool.space, Preis USD + Blockhöhe). Eine gespeicherte
//! leere Liste ist bewusst erlaubt (Nutzer will nichts sehen).

pub use crate::mempool::ValueId;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Gesamte Applet-Konfiguration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    /// Basis-URL der Mempool-Instanz (ohne trailing slash); leer = default.
    #[serde(default)]
    pub instance_url: String,
    /// Ausgewählte Werte als IDs (`price:USD`, `fee:fastest`, `block:height`).
    #[serde(default)]
    pub values: Vec<String>,
    /// Wie viele der Werte (in Reihenfolge) im Panel neben dem Logo
    /// erscheinen. `None` = alle.
    #[serde(default)]
    pub panel_count: Option<usize>,
    /// Aktualisierungsintervall in Minuten (Minimum 1).
    #[serde(default = "default_refresh_minutes")]
    pub refresh_minutes: u32,
}

fn default_refresh_minutes() -> u32 {
    1
}

/// Default-Werte für den Erststart: Kurs USD + Blockhöhe.
#[must_use]
pub fn default_values() -> Vec<String> {
    vec!["price:USD".to_string(), "block:height".to_string()]
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            instance_url: String::new(),
            values: default_values(),
            panel_count: None,
            refresh_minutes: default_refresh_minutes(),
        }
    }
}

impl AppConfig {
    /// Die Werte, die im Panel neben dem Logo erscheinen: die ersten
    /// `panel_count` (geklemmt auf die Länge); `None` = alle.
    #[must_use]
    pub fn panel_values(&self) -> &[String] {
        let n = match self.panel_count {
            Some(n) => n.min(self.values.len()),
            None => self.values.len(),
        };
        &self.values[..n]
    }
}

/// Aktualisierungsintervall als Duration; immer mindestens 1 Minute.
#[must_use]
pub fn refresh_interval(cfg: &AppConfig) -> std::time::Duration {
    std::time::Duration::from_secs(u64::from(cfg.refresh_minutes.max(1)) * 60)
}

/// Pfad der persistenten Konfigurationsdatei.
pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("cosmic-applet-bitcoin").join("config.toml"))
}

/// Lädt die Konfiguration vom Standardpfad.
pub fn load() -> AppConfig {
    match config_path() {
        Some(path) => load_from(&path),
        None => AppConfig::default(),
    }
}

/// Lädt die Konfiguration aus einer Datei. Fehlt die Datei oder ist sie
/// ungültig → Default (mit Default-Liste); eine gültige Datei mit leer
/// Liste bleibt leer (Nutzer will nichts sehen).
pub fn load_from(path: &Path) -> AppConfig {
    let mut cfg = std::fs::read_to_string(path)
        .ok()
        .and_then(|s| toml::from_str::<AppConfig>(&s).ok())
        .unwrap_or_default();
    if cfg.refresh_minutes == 0 {
        cfg.refresh_minutes = default_refresh_minutes();
    }
    cfg
}

/// Speichert die Konfiguration am Standardpfad (Rechte `0600` unter Unix).
pub fn save(cfg: &AppConfig) -> std::io::Result<()> {
    let path = config_path().ok_or_else(|| {
        std::io::Error::other("kein Config-Verzeichnis ermittelbar")
    })?;
    save_to(&path, cfg)
}

/// Testbare Variante: speichert an einen expliziten Pfad.
pub fn save_to(path: &Path, cfg: &AppConfig) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = toml::to_string(cfg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, body)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Verschiebt eine Entity von Index `from` nach Index `to`
/// (Out-of-Bounds wird ignoriert — UI-Buttons können nicht "aus dem
/// Listenrand ziehen").
pub fn move_value(values: &mut Vec<String>, from: usize, to: usize) {
    if from >= values.len() || to >= values.len() || from == to {
        return;
    }
    let moved = values.remove(from);
    values.insert(to, moved);
}

/// Verschiebt einen Wert unter Beachtung der Panel-Grenze: Kreuzt der
/// Weg die Linie, wechselt GENAU dieser Wert die Gruppe — die Linie folgt,
/// sodass alle übrigen Werte in ihren Gruppen bleiben (internes
/// Verschieben lässt die Linie zwischen denselben Werten stehen).
pub fn move_value_across(
    values: &mut Vec<String>,
    boundary: &mut usize,
    from: usize,
    to: usize,
) {
    if from >= values.len() || to >= values.len() || from == to {
        return;
    }
    let was_in_panel = from < *boundary;
    let moved = values.remove(from);
    if was_in_panel {
        *boundary = boundary.saturating_sub(1);
    }
    values.insert(to, moved);
    if to < *boundary {
        *boundary += 1;
    }
}

/// Entfernt einen Wert und rückt die Panel-Grenze mit.
pub fn remove_row(values: &mut Vec<String>, boundary: &mut usize, i: usize) {
    if i >= values.len() {
        return;
    }
    values.remove(i);
    if i < *boundary {
        *boundary -= 1;
    }
    *boundary = (*boundary).min(values.len());
}
