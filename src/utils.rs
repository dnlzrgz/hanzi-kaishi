use std::{fs, path::Path};

use anyhow::Result;
use reqwest::blocking::Client;

pub fn download_media(client: &Client, url: &str, path: &Path) -> Result<()> {
    let bytes = client.get(url).send()?.error_for_status()?.bytes()?;
    fs::write(path, &bytes)?;
    Ok(())
}
