use std::sync::Once;

static DE_LOCALE: Once = Once::new();

fn force_de() {
    DE_LOCALE.call_once(|| std::env::set_var("I18N_LANGUAGE", "de"));
}
use cosmic_applet_bitcoin::applet::popup_rows_lang;
use std::collections::HashMap;

fn values(entries: Vec<(&str, Result<f64, ()>)>) -> HashMap<String, Result<f64, ()>> {
    entries
        .into_iter()
        .map(|(id, r)| (id.to_string(), r))
        .collect()
}

fn selected() -> Vec<String> {
    vec!["price:USD".to_string(), "block:height".to_string(), "fee:fastest".to_string()]
}

#[test]
fn rows_in_selection_order_with_units() {
    let v = values(vec![
        ("fee:fastest", Ok(2.909)),
        ("block:height", Ok(973692.0)),
        ("price:USD", Ok(79941.0)),
    ]);
    let rows = popup_rows_lang(true, &v, &selected());
    assert_eq!(rows[0], ("Kurs USD".to_string(), "79.941 $".to_string()));
    assert_eq!(rows[1], ("Blockhöhe".to_string(), "973 692".to_string()));
    assert_eq!(rows[2], ("Gebühr (schnellste)".to_string(), "2,91 sat/vB".to_string()));
}

#[test]
fn failed_value_shows_dash() {
    let v = values(vec![("fee:hour", Err(()))]);
    let sel = ["fee:hour".to_string()];
    assert_eq!(popup_rows_lang(true, &v, &sel), vec![("Gebühr (Stunde)".to_string(), "—".to_string())]);
}

#[test]
fn missing_value_shows_dash() {
    let sel = ["price:EUR".to_string()];
    assert_eq!(popup_rows_lang(true, &HashMap::new(), &sel), vec![("Kurs EUR".to_string(), "—".to_string())]);
}

#[test]
fn empty_selection_gives_no_rows() {
    assert!(popup_rows_lang(true, &HashMap::new(), &[]).is_empty());
}

#[test]
fn difficulty_values_display_human_names() {
    force_de();
    for (id, expected) in [
        ("diff:progress", "Schwierigkeitsfortschritt"),
        ("diff:change", "Schwierigkeitsänderung"),
        ("diff:date", "Geschätztes Retarget"),
        ("diff:blocks", "Blöcke bis Retarget"),
    ] {
        assert_eq!(
            cosmic_applet_bitcoin::applet::value_display_name(id),
            expected
        );
    }
}
