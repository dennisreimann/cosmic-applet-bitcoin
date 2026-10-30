use cosmic_applet_bitcoin::applet::format_panel_line_lang;
use std::collections::HashMap;

fn values(entries: Vec<(&str, Result<f64, ()>)>) -> HashMap<String, Result<f64, ()>> {
    entries
        .into_iter()
        .map(|(id, r)| (id.to_string(), r))
        .collect()
}

fn selected() -> Vec<String> {
    vec!["price:USD".to_string(), "block:height".to_string()]
}

#[test]
fn empty_selection_renders_empty_line() {
    assert_eq!(format_panel_line_lang(true, &HashMap::new(), &[]), "");
}

#[test]
fn values_in_selection_order_join_with_middle_dot() {
    let v = values(vec![("block:height", Ok(973692.0)), ("price:USD", Ok(79941.0))]);
    let sel = ["block:height", "price:USD"];
    let sel: Vec<String> = sel.iter().map(|s| s.to_string()).collect();
    assert_eq!(format_panel_line_lang(true, &v, &sel), "973 692 · 79.941 $");
    assert_eq!(format_panel_line_lang(true, &v, &selected()), "79.941 $ · 973 692");
}

#[test]
fn failed_value_renders_dash() {
    let v = values(vec![("price:USD", Err(())), ("block:height", Ok(973692.0))]);
    assert_eq!(format_panel_line_lang(true, &v, &selected()), "— · 973 692");
}

#[test]
fn missing_value_renders_dash() {
    assert_eq!(format_panel_line_lang(true, &HashMap::new(), &selected()), "— · —");
}

#[test]
fn merge_err_overwrites_for_dash_and_skips_clock_without_ok() {
    // Err-Einträge stammen immer aus einer erfolgreichen Antwort (Wert
    // fehlte darin) → sie überschreiben („—“). Ganz fehlgeschlagene
    // Gruppen liefern von fetch_values gar keine Einträge
    // (tests/mempool.rs) — deshalb bleibt die Anzeige dort unverändert.
    let mut m = values(vec![("price:USD", Ok(79941.0))]);
    let results = vec![
        ("price:USD".to_string(), Err(())),
        ("block:height".to_string(), Err(())),
    ];
    assert!(!cosmic_applet_bitcoin::applet::merge_fetched(&mut m, results));
    assert_eq!(m.get("price:USD"), Some(&Err(())));
    assert_eq!(m.get("block:height"), Some(&Err(())));
}

#[test]
fn merge_no_entries_changes_nothing() {
    // Gruppe fehlgeschlagen → keine Einträge → alte Werte bleiben stehen.
    let mut m = values(vec![("price:USD", Ok(79941.0))]);
    assert!(!cosmic_applet_bitcoin::applet::merge_fetched(&mut m, vec![]));
    assert_eq!(m.get("price:USD"), Some(&Ok(79941.0)));
}

#[test]
fn merge_ok_updates_values_and_ticks_clock() {
    let mut m = values(vec![("price:USD", Ok(79941.0))]);
    let results = vec![
        ("price:USD".to_string(), Ok(80221.0)),
        ("block:height".to_string(), Err(())),
    ];
    assert!(cosmic_applet_bitcoin::applet::merge_fetched(&mut m, results));
    assert_eq!(m.get("price:USD"), Some(&Ok(80221.0)));
}

#[test]
fn merge_missing_value_in_ok_group_shows_dash() {
    // Request ok, Wert fehlt in der Antwort → Err überschreibt („—“).
    let mut m = values(vec![("price:EUR", Ok(71784.0))]);
    let results = vec![("price:EUR".to_string(), Err(()))];
    assert!(!cosmic_applet_bitcoin::applet::merge_fetched(&mut m, results));
    assert!(m.get("price:EUR").is_some_and(|r| r.is_err()));
}

#[test]
fn merge_empty_results_is_no_update() {
    let mut m = values(vec![]);
    assert!(!cosmic_applet_bitcoin::applet::merge_fetched(&mut m, vec![]));
}
