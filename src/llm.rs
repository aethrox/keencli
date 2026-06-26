// OpenRouter API üzerinden AI analizi.
// Timeout yok: uzun süren modeller erken kesilmesin diye bilerek eklenmedi.
use anyhow::{Context, Result};
use chrono::Local;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;

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
        let resp = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.expose_secret()),
            )
            .header("HTTP-Referer", "http://localhost")
            .json(&json!({
                "model": self.model,
                "messages": [{"role": "user", "content": prompt}],
                "temperature": 0.7
            }))
            .send()
            .await
            .context("OpenRouter API'ye bağlanılamadı")?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await.context("OpenRouter yanıtı okunamadı")?;

        if let Some(error) = body.get("error") {
            let message = error["message"].as_str().unwrap_or("bilinmeyen API hatası");
            anyhow::bail!("OpenRouter API hatası: {message}");
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
