use cosmic_applet_bitcoin::config::{self, AppConfig};

#[test]
fn app_config_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let cfg = AppConfig {
        instance_url: "https://mempool.local:8080".into(),
        values: vec!["price:EUR".into(), "block:height".into()],
        panel_count: Some(1),
        refresh_minutes: 5,
    };

    config::save_to(&path, &cfg).unwrap();

    let loaded = config::load_from(&path);
    assert_eq!(loaded, cfg);
}

#[test]
fn load_invalid_toml_falls_back_to_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "not valid toml [[[[").unwrap();

    assert_eq!(config::load_from(&path), AppConfig::default());
}

#[test]
fn load_missing_file_falls_back_to_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing.toml");

    // Kein File → Default-Liste (Preis USD + Blockhöhe) für den Erststart.
    assert_eq!(config::load_from(&path), AppConfig::default());
    assert_eq!(config::default_values(), ["price:USD", "block:height"]);
}

#[test]
fn saved_empty_list_stays_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let cfg = AppConfig {
        values: vec![],
        panel_count: None,
        ..AppConfig::default()
    };
    config::save_to(&path, &cfg).unwrap();
    assert!(config::load_from(&path).values.is_empty());
}

#[test]
fn refresh_interval_defaults_to_one_minute() {
    assert_eq!(
        config::refresh_interval(&AppConfig::default()),
        std::time::Duration::from_secs(60)
    );
}

#[test]
fn refresh_interval_minimum_is_one_minute() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "refresh_minutes = 0\n").unwrap();
    assert_eq!(config::load_from(&path).refresh_minutes, 1);

    std::fs::write(&path, "refresh_minutes = 5\n").unwrap();
    let cfg = config::load_from(&path);
    assert_eq!(
        config::refresh_interval(&cfg),
        std::time::Duration::from_secs(300)
    );
}

#[test]
fn panel_values_none_means_all() {
    let cfg = AppConfig {
        values: vec!["price:USD".into(), "block:height".into()],
        panel_count: None,
        ..AppConfig::default()
    };
    assert_eq!(cfg.panel_values().len(), 2);
}

#[test]
fn panel_values_some_means_first_n_clamped() {
    let cfg = AppConfig {
        values: vec!["price:USD".into(), "block:height".into()],
        panel_count: Some(1),
        ..AppConfig::default()
    };
    assert_eq!(cfg.panel_values(), ["price:USD"]);
    assert_eq!(
        AppConfig { panel_count: Some(99), ..cfg }.panel_values().len(),
        2
    );
}

#[test]
fn move_value_reorders_within_bounds() {
    let mut v = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    config::move_value(&mut v, 0, 2);
    assert_eq!(v, ["b", "c", "a"]);
    config::move_value(&mut v, 2, 0);
    assert_eq!(v, ["a", "b", "c"]);
}

#[test]
fn move_value_ignores_out_of_bounds() {
    let mut v = vec!["a".to_string()];
    config::move_value(&mut v, 5, 0);
    config::move_value(&mut v, 0, 5);
    assert_eq!(v, ["a"]);
}

#[test]
fn move_value_across_keeps_other_members() {
    let mut v = ["a", "b", "c", "d", "e"].iter().map(|s| s.to_string()).collect();
    let mut boundary = 3; // a,b,c im Panel

    // d (unten) nach oben (idx 3 → 2): nur d wechselt ins Panel.
    config::move_value_across(&mut v, &mut boundary, 3, 2);
    assert_eq!(boundary, 4);
    assert_eq!(v, ["a", "b", "d", "c", "e"]);

    // b (oben) nach unten (idx 1 → 3): nur b verlässt das Panel.
    config::move_value_across(&mut v, &mut boundary, 1, 3);
    assert_eq!(boundary, 3);
    assert_eq!(v, ["a", "d", "c", "b", "e"]);
}

#[test]
fn move_value_across_internal_moves_follow_line() {
    let mut v = ["a", "b", "c", "d", "e"].iter().map(|s| s.to_string()).collect();
    let mut boundary = 2;
    config::move_value_across(&mut v, &mut boundary, 1, 0);
    assert_eq!(boundary, 2);
    assert_eq!(v, ["b", "a", "c", "d", "e"]);
    config::move_value_across(&mut v, &mut boundary, 3, 4);
    assert_eq!(boundary, 2);
}

#[test]
fn move_value_across_ignores_out_of_bounds() {
    let mut v = vec!["a".to_string()];
    let mut b = 1;
    config::move_value_across(&mut v, &mut b, 5, 0);
    config::move_value_across(&mut v, &mut b, 0, 5);
    assert_eq!(v, ["a"]);
    assert_eq!(b, 1);
}

#[test]
fn remove_row_above_boundary_shifts_boundary() {
    let mut v = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
    let mut b = 2;
    config::remove_row(&mut v, &mut b, 0);
    assert_eq!(v, ["b", "c"]);
    assert_eq!(b, 1);
    config::remove_row(&mut v, &mut b, 1);
    assert_eq!(v, ["b"]);
    assert_eq!(b, 1);
}

#[test]
fn remove_row_clamps_boundary_to_length() {
    let mut v = vec!["a".to_string()];
    let mut b = 1;
    config::remove_row(&mut v, &mut b, 0);
    assert!(v.is_empty());
    assert_eq!(b, 0);
    config::remove_row(&mut v, &mut b, 0);
}
