//! Mempool.space-REST-Zugriff.
//!
//! Die anzeigbaren Werte sind IDs: `price:<CODE>` (Kurs in der Währung),
//! `fee:<tier>` (Gebühr in sat/vB) und `block:height`. Pro Zyklus werden
//! nur die Endpunkte geladen, die zur Auswahl beitragen:
//! `/api/v1/prices`, `/api/v1/fees/precise`, `/api/blocks/tip/height`.

use chrono::{DateTime, Local};
use i18n_embed::LanguageLoader;
use std::collections::BTreeMap;

/// Basis-Instanz, wenn in der Konfiguration nichts steht.
pub const DEFAULT_INSTANCE: &str = "https://mempool.space";

pub type ValueId = String;

/// Gebühren-IDs (`/api/v1/fees/precise`) in Anzeigereihenfolge.
pub const FEE_IDS: &[&str] = &[
    "fee:fastest",
    "fee:halfHour",
    "fee:hour",
    "fee:economy",
    "fee:minimum",
];

/// Währungen, die mempool.space im Free-Tier liefert — Fallback, wenn die
/// konfigurierte Instanz im Einstellungsfenster (noch) nicht antwortet.
pub const DEFAULT_CURRENCIES: &[&str] = &["USD", "EUR", "GBP", "CAD", "CHF", "AUD", "JPY"];

/// Der Katalog aller auswählbaren Werte: Preise aller Currencies, Fees,
/// Blockhöhe. Reihenfolge: Preise, Fees, Blockhöhe.
#[must_use]
pub fn known_value_ids(currencies: &[String]) -> Vec<ValueId> {
    let mut ids: Vec<ValueId> = currencies
        .iter()
        .map(|c| format!("price:{c}"))
        .collect();
    ids.extend(FEE_IDS.iter().map(|s| s.to_string()));
    ids.push("block:height".to_string());
    ids.extend(
        [
            "diff:progress",
            "diff:change",
            "diff:date",
            "diff:blocks",
        ]
        .iter()
        .map(|s| s.to_string()),
    );
    ids
}

/// Parst `/api/v1/prices`; Währungen deterministisch sortiert, `time` weg.
pub fn parse_prices(body: &[u8]) -> Result<Vec<(String, f64)>, String> {
    let v: serde_json::Value =
        serde_json::from_slice(body).map_err(|e| format!("ungültiges JSON: {e}"))?;
    let obj = v.as_object().ok_or("preise: kein JSON-Objekt")?;
    let mut out: Vec<(String, f64)> = obj
        .iter()
        .filter(|(k, _)| *k != "time")
        .filter_map(|(k, v)| v.as_f64().map(|val| (k.clone(), val)))
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    if out.is_empty() {
        return Err("preise: keine Währungen in der Antwort".to_string());
    }
    Ok(out)
}

/// Parst `/api/v1/fees/precise` in eine `fee:<tier> → sat/vB`-Map.
pub fn parse_fees(body: &[u8]) -> Result<BTreeMap<ValueId, f64>, String> {
    let v: serde_json::Value =
        serde_json::from_slice(body).map_err(|e| format!("ungültiges JSON: {e}"))?;
    let obj = v.as_object().ok_or("fees: kein JSON-Objekt")?;
    let mut out = BTreeMap::new();
    for id in FEE_IDS {
        let key = format!("{}Fee", id.strip_prefix("fee:").unwrap_or(""));
        out.insert(
            id.to_string(),
            obj.get(&key)
                .and_then(|v| v.as_f64())
                .ok_or_else(|| format!("fees: {key} fehlt"))?,
        );
    }
    Ok(out)
}

/// Parst `/api/v1/difficulty-adjustment` in `diff:<feld> → f64`.
/// `estimatedRetargetDate` (ms) wird auf Sekunden нормiert.
pub fn parse_difficulty(
    body: &[u8],
) -> Result<BTreeMap<ValueId, f64>, String> {
    let v: serde_json::Value =
        serde_json::from_slice(body).map_err(|e| format!("ungültiges JSON: {e}"))?;
    let obj = v.as_object().ok_or("difficulty: kein JSON-Objekt")?;
    let f = |key: &str| -> Result<f64, String> {
        obj.get(key)
            .and_then(|x| x.as_f64())
            .ok_or_else(|| format!("difficulty: {key} fehlt"))
    };
    let mut out = BTreeMap::new();
    out.insert("diff:progress".to_string(), f("progressPercent")?);
    out.insert("diff:change".to_string(), f("difficultyChange")?);
    out.insert("diff:blocks".to_string(), f("remainingBlocks")?);
    out.insert("diff:date".to_string(), f("estimatedRetargetDate")? / 1000.0);
    Ok(out)
}

/// Parst `/api/blocks/tip/height` (reine Ganzzahl im Body).
pub fn parse_height(body: &[u8]) -> Result<i64, String> {
    let s = std::str::from_utf8(body).map_err(|e| format!("höhe: nicht-utf8: {e}"))?;
    s.trim()
        .parse::<i64>()
        .map_err(|_| format!("höhe: unerwartete Antwort: {s}"))
}

/// Tausendertrenner (Leerzeichen) mit optionalem Nachkommaanteil.
#[must_use]
pub fn thousands(n: f64) -> String {
    let negative = n < 0.0;
    let abs = n.abs();
    let int_part = abs.trunc();
    let int_s = format!("{}", int_part as u64);
    let grouped: String = {
        let mut s = String::new();
        for (c, ch) in int_s.chars().rev().enumerate() {
            if c > 0 && c % 3 == 0 {
                s.push(' ');
            }
            s.push(ch);
        }
        s.chars().rev().collect()
    };
    let frac = abs - int_part;
    let mut out = String::new();
    if negative {
        out.push('-');
    }
    out.push_str(&grouped);
    if frac >= 0.005 {
        out.push_str(&format!(" {}", format!("{frac:.2}").trim_start_matches('0')));
    }
    out
}

/// Währungssymbole je Code; Fallback: der Code selbst.
#[must_use]
fn currency_symbol(code: &str) -> &str {
    match code {
        "USD" => "$",
        "CAD" => "CA$",
        "EUR" => "€",
        "GBP" => "£",
        "CHF" => "Fr",
        "AUD" => "A$",
        "JPY" | "CNY" | "KRW" => "¥",
        "RUB" => "₽",
        "BRL" => "R$",
        _ => code,
    }
}

/// Gepunktete (de) bzw. kommagetrennte (en) Zahlendarstellung.
#[must_use]
fn group_locale(n: f64, de: bool) -> String {
    let negative = n < 0.0;
    let abs = n.abs();
    let int_s = format!("{}", abs.trunc() as u64);
    let sep = if de { '.' } else { ',' };
    let grouped: String = {
        let mut s = String::new();
        for (c, ch) in int_s.chars().rev().enumerate() {
            if c > 0 && c % 3 == 0 {
                s.push(sep);
            }
            s.push(ch);
        }
        s.chars().rev().collect()
    };
    let frac = abs - abs.trunc();
    let mut out = String::new();
    if negative {
        out.push('-');
    }
    out.push_str(&grouped);
    if frac >= 0.005 {
        let dec = if de { ',' } else { '.' };
        let frac_s = format!("{frac:.2}")
            .trim_start_matches('0')
            .replace('.', &dec.to_string());
        out.push(dec);
        out.push_str(&frac_s);
    }
    out
}

/// Anzeigbarer Wert einer ID, in der App-Sprache formatiert:
/// Preise mit Währungssymbol (de nach der Zahl, en davor), Fees
/// `x.xx sat/vB`, Blockhöhe mit Leerzeichen-Tausender, Difficulty menschenlesbar.
#[must_use]
pub fn format_value_lang(is_de: bool, id: &str, value: f64) -> String {
    if let Some(code) = id.strip_prefix("price:") {
        let symbol = currency_symbol(code);
        let number = group_locale(value, is_de);
        return if is_de {
            format!("{number} {symbol}")
        } else {
            format!("{symbol}{number}")
        };
    }
    if id.starts_with("fee:") {
        return format!("{} sat/vB", fmt2(value, is_de, 2));
    }
    if id == "block:height" {
        return thousands(value);
    }
    match id {
        "diff:progress" => {
            if is_de {
                format!("{} %", fmt2(value, is_de, 1))
            } else {
                format!("{}%", fmt2(value, is_de, 1))
            }
        }
        "diff:change" => {
            let sign = if value >= 0.0 { '+' } else { '-' };
            let v = value.abs();
            let formatted = fmt2(v, is_de, 2);
            if is_de {
                format!("{sign}{formatted} %")
            } else {
                format!("{sign}{formatted}%")
            }
        }
        "diff:blocks" => group_locale(value, is_de),
        "diff:date" => {
            let secs = value as i64;
            match DateTime::from_timestamp(secs, 0) {
                Some(d) => {
                    let local = d.with_timezone(&Local);
                    if is_de {
                        local.format("%d.%m.%Y %H:%M").to_string()
                    } else {
                        local.format("%d %b %Y, %H:%M").to_string()
                    }
                }
                None => format!("{secs}"),
            }
        }
        _ => thousands(value),
    }
}

/// Zahl mit n Dezimalstellen, de: Komma/en: Punkt, keine Gruppierung.
#[must_use]
pub fn fmt2(value: f64, is_de: bool, decimals: usize) -> String {
    let dec = if is_de { ',' } else { '.' };
    format!("{value:.decimals$}").replace('.', &dec.to_string())
}

/// Ist die App-Sprache Deutsch (bestimmt Format von Zahlen/Datumsangaben)?
#[must_use]
pub fn is_app_language_de() -> bool {
    crate::i18n::localize();
    (*crate::i18n::LANGUAGE_LOADER)
        .current_language()
        .language
        .as_str()
        == "de"
}

/// Anzeige in der App-Sprache: de → deutsche Formatierung, sonst en.
#[must_use]
pub fn format_value(id: &str, value: f64) -> String {
    format_value_lang(is_app_language_de(), id, value)
}

/// „Timeout“-konstante Dauer für requests.
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Holt die Währungen von `/api/v1/prices` (für den Settings-Katalog).
pub async fn fetch_prices(base_url: &str) -> Result<Vec<(String, f64)>, String> {
    let t = get_text(base_url, "api/v1/prices").await?;
    parse_prices(t.as_bytes())
}

async fn get_text(base_url: &str, path: &str) -> Result<String, String> {
    let url = format!("{}/{}", base_url.trim_end_matches('/'), path);
    let client = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .build()
        .map_err(|e| format!("HTTP-Client: {e}"))?;
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("{url}: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("{url}: Status {}", status.as_u16()));
    }
    resp.text().await.map_err(|e| format!("{url}: {e}"))
}

/// Holt die angewählten Werte parallel — nur der jeweils nötige Endpunkt
/// je ID-Gruppe: Preise einmal für alle `price:*`, Fees einmal für
/// `fee:*`, Blockhöhe einmal.
/// Parst `/api/v1/prices` in eine `price:<CODE> → f64`-Map.
fn parse_prices_map(body: &[u8]) -> Result<BTreeMap<ValueId, f64>, String> {
    Ok(parse_prices(body)?
        .into_iter()
        .map(|(c, v)| (format!("price:{c}"), v))
        .collect())
}

/// Parst `/api/blocks/tip/height` in eine Eintrag-Map.
fn parse_height_map(body: &[u8]) -> Result<BTreeMap<ValueId, f64>, String> {
    Ok(BTreeMap::from([(
        "block:height".to_string(),
        parse_height(body)? as f64,
    )]))
}

/// Führt eine Wertegruppe aus: fragt den Endpunkt ab, parst die Antwort und
/// liefert Einträge für die angefragten IDs.
///
/// `None` = Gruppe nicht angefragt **oder** fehlgeschlagen (Netzwerk, 5xx,
/// ungültiges JSON) — das Applet behält in dem Fall die alten Werte. Ein
/// `Err` in der Liste bedeutet dagegen: Antwort ok, aber genau dieser Wert
/// fehlte darin (→ „—“-Platzhalter).
async fn fetch_group<F>(
    base_url: &str,
    path: &str,
    ids: &[ValueId],
    parse: F,
) -> Option<Vec<(ValueId, Result<f64, String>)>>
where
    F: FnOnce(&[u8]) -> Result<BTreeMap<ValueId, f64>, String>,
{
    if ids.is_empty() {
        return None;
    }
    let body = get_text(base_url, path).await.ok()?;
    let map = parse(body.as_bytes()).ok()?;
    Some(
        ids.iter()
            .map(|id| {
                (
                    id.clone(),
                    map.get(id)
                        .copied()
                        .ok_or_else(|| "wert fehlt in antwort".to_string()),
                )
            })
            .collect(),
    )
}

/// Holt die angewählten Werte parallel — pro ID-Gruppe nur der jeweils
/// nötige Endpunkt: Preise einmal für alle `price:*`, Fees einmal für
/// `fee:*`, Blockhöhe und Difficulty-Adjustment je einmal.
pub async fn fetch_values(
    base_url: &str,
    ids: &[ValueId],
) -> Vec<(ValueId, Result<f64, String>)> {
    let base = base_url.trim_end_matches('/');
    let group = |prefix: &str| -> Vec<ValueId> {
        ids.iter()
            .filter(|id| id.starts_with(prefix))
            .cloned()
            .collect()
    };
    let prices_ids = group("price:");
    let fees_ids = group("fee:");
    let height_ids = group("block:");
    let diff_ids = group("diff:");

    let (prices, fees, height, diff) = futures_util::future::join4(
        fetch_group(base, "api/v1/prices", &prices_ids, parse_prices_map),
        fetch_group(base, "api/v1/fees/precise", &fees_ids, parse_fees),
        fetch_group(base, "api/blocks/tip/height", &height_ids, parse_height_map),
        fetch_group(base, "api/v1/difficulty-adjustment", &diff_ids, parse_difficulty),
    )
    .await;

    [prices, fees, height, diff]
        .into_iter()
        .flatten()
        .flatten()
        .collect()
}
