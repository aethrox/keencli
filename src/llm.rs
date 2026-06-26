// OpenRouter API üzerinden AI analizi.
// Timeout yok: uzun süren modeller erken kesilmesin diye bilerek eklenmedi.
use anyhow::{Context, Result};
use chrono::Local;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;

fn format_api_error(error: &serde_json::Value) -> String {
    let message = error["message"].as_str().unwrap_or("bilinmeyen API hatası");
    let code = error
        .get("code")
        .map(|value| value.to_string())
        .unwrap_or_default();
    let provider = error["metadata"]["provider_name"]
        .as_str()
        .or_else(|| error["metadata"]["raw"].as_str())
        .unwrap_or("");

    if provider.is_empty() && code.is_empty() {
        return message.to_string();
    }

    let mut detail = message.to_string();
    if !code.is_empty() {
        detail.push_str(&format!(" (kod: {code})"));
    }
    if !provider.is_empty() {
        detail.push_str(&format!("\n  Sağlayıcı: {provider}"));
    }
    detail
}

const DEFAULT_TEMPERATURE: f64 = 0.3;

/// `LLM_TEMPERATURE` → varsayılan 0.3 (tanı raporu için önerilen)
fn resolve_temperature() -> Result<f64> {
    let Some(raw) = std::env::var("LLM_TEMPERATURE")
        .ok()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(DEFAULT_TEMPERATURE);
    };

    let value: f64 = raw
        .trim()
        .parse()
        .with_context(|| format!("LLM_TEMPERATURE geçersiz: '{raw}'. Örnek: 0.3"))?;

    if !(0.0..=2.0).contains(&value) {
        anyhow::bail!("LLM_TEMPERATURE 0.0–2.0 arasında olmalı (girilen: {value}).");
    }

    Ok(value)
}

pub struct LLMClient {
    client: Client,
    api_key: SecretString,
    model: String,
}

impl LLMClient {
    pub fn new(api_key: SecretString, model: String) -> Result<Self> {
        let client = Client::builder()
            .build()
            .context("OpenRouter HTTP istemcisi oluşturulamadı")?;

        Ok(Self {
            client,
            api_key,
            model,
        })
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    // Prompt'u OpenRouter'a gönderir, dönen metin raporu oluşturur.
    pub async fn analyze(&self, prompt: String) -> Result<String> {
        let mut payload = json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 4096,
        });
        // Sakana Fugu temperature desteklemiyor.
        if !self.model.starts_with("sakana/") {
            payload["temperature"] = json!(resolve_temperature()?);
        }

        let resp = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.expose_secret()),
            )
            .header("HTTP-Referer", "https://github.com/aethrox/keencli")
            .header("X-OpenRouter-Title", "keencli")
            .json(&payload)
            .send()
            .await
            .context("OpenRouter API'ye bağlanılamadı")?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await.context("OpenRouter yanıtı okunamadı")?;

        if let Some(error) = body.get("error") {
            anyhow::bail!("OpenRouter API hatası: {}", format_api_error(error));
        }

        if !status.is_success() {
            anyhow::bail!("OpenRouter isteği başarısız (HTTP {status}).");
        }

        body["choices"][0]["message"]["content"]
            .as_str()
            .map(str::to_string)
            .context("OpenRouter yanıtında metin içerik bulunamadı")
    }

    // Model adındaki / karakterini - yapar: anthropic/claude → ai_report_anthropic-claude.md
    pub fn report_filename(model: &str) -> String {
        format!("ai_report_{}.md", model.replace('/', "-"))
    }

    pub fn format_report(model: &str, content: &str) -> String {
        let generated = Local::now().format("%Y-%m-%d %H:%M:%S");
        format!(
            "<!-- model: {model} -->\n<!-- generated: {generated} -->\n\n\
             **Model:** `{model}`\n\n\
             {content}\n\n\
             ---\n\n\
             *Bu rapor otomatik üretilmiştir; %100 doğruluk garantisi yoktur. \
             Yapılandırma değişikliği veya kesin teşhis öncesinde bulguları \
             bağımsız olarak doğrulamanız gerekir.*"
        )
    }
}
