//! Connection-independent helpers for editable settings. The UI owns widgets;
//! this module owns their catalog/validation and error-bearing file boundary.

use std::io::Write;
use std::path::Path;

pub fn merge_catalog(currencies: &[String], selected: &[String]) -> Vec<String> {
    let mut catalog = crate::mempool::known_value_ids(currencies);
    for value in selected {
        if !catalog.contains(value) {
            catalog.push(value.clone());
        }
    }
    catalog
}

pub fn parse_refresh_minutes(input: &str) -> Result<std::num::NonZeroU32, &'static str> {
    input
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|value| (1..=1440).contains(value))
        .and_then(std::num::NonZeroU32::new)
        .ok_or("refresh interval must be a whole number from 1 to 1440 minutes")
}

/// Write a new complete configuration before replacing the old one.
pub fn atomic_write(path: &Path, body: &[u8]) -> std::io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("config path has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let mut file = tempfile::Builder::new()
        .prefix(".config-")
        .tempfile_in(parent)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    file.write_all(body)?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|error| error.error)?;
    Ok(())
}
