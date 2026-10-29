use cosmic_applet_bitcoin::{fl, i18n};
use i18n_embed::LanguageLoader;
use std::sync::Once;

static DE_LOCALE: Once = Once::new();

/// Deterministische Sprache für diesen Testprozess: DE.
fn force_de() {
    DE_LOCALE.call_once(|| std::env::set_var("I18N_LANGUAGE", "de"));
}

#[test]
fn german_catalog_covers_all_messages() {
    force_de();
    assert_eq!(fl!("popup-title"), "Bitcoin");
    assert_eq!(fl!("quit"), "Beenden");
    assert_eq!(fl!("settings"), "Einstellungen");
    assert_eq!(fl!("no-values"), "Keine Werte konfiguriert");
    assert_eq!(fl!("instance-url"), "Instanz-URL");
    assert_eq!(fl!("refresh-interval"), "Aktualisierungsintervall");
    assert_eq!(fl!("save"), "Speichern");
    assert_eq!(fl!("value-height"), "Blockhöhe");
}

#[test]
fn loader_respects_i18n_language_override() {
    force_de();
    i18n::localize();
    let current = (*i18n::LANGUAGE_LOADER).current_language().to_string();
    assert!(current.starts_with("de"), "aktuelle Sprache: {current}");
}

#[test]
fn german_catalog_covers_value_names() {
    force_de();
    i18n::localize();
    assert_eq!(fl!("value-price", cur = "USD"), "Kurs USD");
    assert_eq!(fl!("value-fee-fastest"), "Gebühr (schnellste)");
    assert_eq!(fl!("value-diff-progress"), "Schwierigkeitsfortschritt");
    assert_eq!(fl!("value-diff-date"), "Geschätztes Retarget");
    assert_eq!(fl!("value-diff-blocks"), "Blöcke bis Retarget");
}
