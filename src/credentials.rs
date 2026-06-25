// Hassas bilgilerin (şifre, API key) tek yerden okunması ve kontrol edilmesi.
use anyhow::{Context, Result};
use secrecy::SecretString;

/// Router şifresini yalnızca ortam değişkeninden okur (.env dahil).
pub fn resolve_password() -> Result<SecretString> {
    let password = std::env::var("KEENETIC_PASSWORD").context(
        "Router şifresi bulunamadı. KEENETIC_PASSWORD ortam değişkenini tanımlayın \
         (veya .env dosyası kullanın). Şifreyi config.toml'a yazmayın.",
    )?;

    if password.trim().is_empty() {
        anyhow::bail!("KEENETIC_PASSWORD boş. Router şifrenizi ortam değişkeni olarak tanımlayın.");
    }

    Ok(SecretString::from(password))
}

/// OpenRouter API anahtarı ve model adını kontrol eder.
pub fn resolve_llm_config() -> Result<(SecretString, String)> {
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .context("OPENROUTER_API_KEY tanımlı değil. OpenRouter hesabınızdan API anahtarı alın.")?;

    if api_key.trim().is_empty() {
        anyhow::bail!("OPENROUTER_API_KEY boş. Geçerli bir API anahtarı girin.");
    }

    let model = std::env::var("LLM_MODEL").context(
        "LLM_MODEL tanımlı değil. Örnek: export LLM_MODEL='anthropic/claude-sonnet-4.6'",
    )?;

    if model.trim().is_empty() {
        anyhow::bail!("LLM_MODEL boş. Kullanmak istediğiniz model adını girin.");
    }

    Ok((SecretString::from(api_key), model))
}
