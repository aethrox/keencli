// config.toml dosyasını okur ve doğrular.
use crate::paths;
use anyhow::{Context, Result};
use serde::Deserialize;

// `#[derive(Deserialize)]`: TOML alanlarını otomatik struct'a çevirir.
// Şifre bilinçli olarak yok — yalnızca KEENETIC_PASSWORD ortam değişkeni kullanılır.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouterConfig {
    pub ip: String,
    pub username: String,
}

pub fn load() -> Result<RouterConfig> {
    // Yalnızca config.toml — KEENETIC_PASSWORD ayrı okunur (credentials.rs).
    // Environment::with_prefix("KEENETIC") password alanını struct'a sızdırır; deny_unknown_fields reddeder.
    let config_path = paths::config_file();
    let config = config::Config::builder()
        .add_source(config::File::from(config_path.as_path()))
        .build()
        .with_context(|| {
            format!(
                "config.toml okunamadı (aranan: {}). \
                 Kurulum için: ~/.config/keencli/config.toml — geliştirme için: ./config.toml",
                config_path.display()
            )
        })?;

    let mut config: RouterConfig = config
        .try_deserialize()
        .context("config.toml formatı hatalı. 'ip' ve 'username' alanları gerekli.")?;

    if let Ok(ip) = std::env::var("KEENETIC_IP")
        && !ip.trim().is_empty()
    {
        config.ip = ip;
    }
    if let Ok(username) = std::env::var("KEENETIC_USERNAME")
        && !username.trim().is_empty()
    {
        config.username = username;
    }

    if config.ip.trim().is_empty() {
        anyhow::bail!("config.toml: 'ip' alanı boş olamaz.");
    }
    if config.username.trim().is_empty() {
        anyhow::bail!("config.toml: 'username' alanı boş olamaz.");
    }

    Ok(config)
}
