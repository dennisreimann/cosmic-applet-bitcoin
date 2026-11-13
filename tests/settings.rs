use cosmic_applet_bitcoin::settings::merge_catalog;

#[test]
fn saved_currency_survives_incomplete_live_catalog() {
    let catalog = merge_catalog(&["EUR".to_string()], &["price:USDT".to_string()]);
    assert!(catalog.contains(&"price:EUR".to_string()));
    assert!(catalog.contains(&"price:USDT".to_string()));
    assert_eq!(catalog.iter().filter(|id| *id == "price:USDT").count(), 1);
}
