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
