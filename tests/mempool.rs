use std::collections::HashMap;
use cosmic_applet_bitcoin::mempool;

#[test]
fn parse_prices_skips_time_and_sorts_by_code() {
    let body = br#"{"time":1793278509,"USD":79941,"EUR":71784,"CHF":66196,"JPY":12646148}"#;
    let prices = mempool::parse_prices(body).unwrap();
    let codes: Vec<&str> = prices.iter().map(|(c, _)| c.as_str()).collect();
    assert_eq!(codes, ["CHF", "EUR", "JPY", "USD"]);
    assert_eq!(prices.iter().find(|(c, _)| c == "EUR").unwrap().1, 71784.0);
}

#[test]
fn parse_prices_fails_without_usd_like_fields() {
    assert!(mempool::parse_prices(br#"{"time":123}"#).is_err());
    assert!(mempool::parse_prices(br#"not json"#).is_err());
}

#[test]
fn parse_fees_maps_tiers() {
    let body = br#"{"fastestFee":2.909,"halfHourFee":1.519,"hourFee":0.698,"economyFee":0.2,"minimumFee":0.1}"#;
    let fees = mempool::parse_fees(body).unwrap();
    assert_eq!(fees.get("fee:fastest"), Some(&2.909));
    assert_eq!(fees.get("fee:halfHour"), Some(&1.519));
    assert_eq!(fees.get("fee:hour"), Some(&0.698));
    assert_eq!(fees.get("fee:economy"), Some(&0.2));
    assert_eq!(fees.get("fee:minimum"), Some(&0.1));
    assert_eq!(fees.get("fee:unknown"), None);
}

#[test]
fn parse_height_integer_body() {
    assert_eq!(mempool::parse_height(b"973692").unwrap(), 973692);
    assert!(mempool::parse_height(b"<html>404</html>").is_err());
}

#[test]
fn format_block_height_still_space_separator() {
    assert_eq!(mempool::format_value_lang(true, "block:height", 973692.0), "973 692");
    assert_eq!(mempool::format_value_lang(false, "block:height", 973692.0), "973 692");
}

#[test]
fn known_value_ids_price_fees_height() {
    let ids = mempool::known_value_ids(&["USD".to_string(), "EUR".to_string()]);
    assert!(ids.contains(&"price:USD".to_string()));
    assert!(ids.contains(&"price:EUR".to_string()));
    assert!(ids.contains(&"block:height".to_string()));
    assert!(ids.contains(&"fee:fastest".to_string()));
    assert!(ids.contains(&"fee:minimum".to_string()));
}

#[tokio::test]
async fn fetch_values_hits_only_needed_endpoints() {
    let server = httpmock::MockServer::start();
    server.mock(|when, then| {
        when.method(httpmock::Method::GET).path("/api/v1/prices");
        then.status(200).body(r#"{"time":1,"EUR":71784}"#);
    });
    server.mock(|when, then| {
        when.method(httpmock::Method::GET).path("/api/blocks/tip/height");
        then.status(200).body("973692");
    });

    let vals =
        mempool::fetch_values(&server.base_url(), &["price:EUR".into(), "block:height".into()])
            .await;
    let got: HashMap<_, _> = vals
        .iter()
        .map(|(id, r)| (id.clone(), r.clone().map_err(|_| ())))
        .collect::<std::collections::HashMap<String, Result<f64, ()>>>();
    assert_eq!(got.get("price:EUR"), Some(&Ok(71784.0)));
    assert_eq!(got.get("block:height"), Some(&Ok(973692.0)));
}

#[test]
fn parse_difficulty_maps_fields() {
    let body = br#"{"progressPercent":98.2639,"difficultyChange":5.5267,"estimatedRetargetDate":1793301079205,"remainingBlocks":35,"remainingTime":19910205,"previousRetarget":5.92}"#;
    let d = mempool::parse_difficulty(body).unwrap();
    assert_eq!(d.get("diff:progress"), Some(&98.2639_f64));
    assert_eq!(d.get("diff:change"), Some(&5.5267_f64));
    assert_eq!(d.get("diff:blocks"), Some(&35.0));
    // estimatedRetargetDate (ms) bleibt als Sekunden im F64-Map.
    assert_eq!(d.get("diff:date"), Some(&(1793301079.205_f64)));
}

#[test]
fn parse_difficulty_missing_field_is_err() {
    assert!(mempool::parse_difficulty(br#"{"progressPercent":1}"#).is_err());
}

#[test]
fn known_value_ids_includes_difficulty() {
    let ids = mempool::known_value_ids(&["USD".to_string()]);
    for id in ["diff:progress", "diff:change", "diff:date", "diff:blocks"] {
        assert!(ids.contains(&id.to_string()), "fehlend: {id}");
    }
}

#[test]
fn format_price_german_locale_symbol_after() {
    assert_eq!(mempool::format_value_lang(true, "price:EUR", 71784.0), "71.784 €");
    assert_eq!(mempool::format_value_lang(true, "price:USD", 79941.0), "79.941 $");
    assert_eq!(mempool::format_value_lang(true, "price:CHF", 66196.0), "66.196 Fr");
    assert_eq!(mempool::format_value_lang(true, "price:JPY", 12646148.0), "12.646.148 ¥");
}

#[test]
fn format_price_english_locale_symbol_before() {
    assert_eq!(mempool::format_value_lang(false, "price:EUR", 71784.0), "€71,784");
    assert_eq!(mempool::format_value_lang(false, "price:USD", 79941.0), "$79,941");
}

#[test]
fn format_difficulty_values_human_readable() {
    assert_eq!(mempool::format_value_lang(true, "diff:progress", 98.264), "98,3 %");
    assert_eq!(mempool::format_value_lang(false, "diff:progress", 98.264), "98.3%");
    assert_eq!(mempool::format_value_lang(true, "diff:change", 5.5267), "+5,53 %");
    assert_eq!(mempool::format_value_lang(false, "diff:change", -0.5), "-0.50%");
    assert_eq!(mempool::format_value_lang(true, "diff:blocks", 35.0), "35");
    assert_eq!(mempool::format_value_lang(true, "diff:blocks", 1112.0), "1.112");
}

#[test]
fn format_difficulty_date_locale_and_german_style() {
    // Parser normalisiert ms → s: 1793301079.205 s ≙ 29.10.2026
    let s = mempool::format_value_lang(true, "diff:date", 1793301079.205);
    assert!(s.starts_with("29.10.2026"), "de-Format: {s}");
    let s = mempool::format_value_lang(false, "diff:date", 1793301079.205);
    assert!(s.contains("2026"), "en-Format: {s}");
}
