// Keenetic router ile HTTP haberleşmesi.
// Tüm veri çekme işlemleri bu modül üzerinden yapılır.
use anyhow::{Context, Result};
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};

// `cookie_store(true)`: login sonrası oturum çerezlerini saklar.
pub struct ApiClient {
    client: Client,
    base_url: String,
    username: String,
    password: SecretString,
}

impl ApiClient {
    pub fn new(ip: String, username: String, password: SecretString) -> Result<Self> {
        let client = Client::builder()
            .cookie_store(true)
            .build()
            .context("HTTP istemcisi oluşturulamadı")?;

        Ok(Self {
            client,
            base_url: format!("http://{ip}"),
            username,
            password,
        })
    }

    // Keenetic kimlik doğrulama: GET /auth → md5(user:realm:pass) → sha256(challenge+md5)
    async fn login(&self) -> Result<()> {
        let url = format!("{}/auth", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Router'a bağlanılamadı ({})", self.base_url))?;

        // Zaten oturum açıksa 200 döner, tekrar giriş gerekmez.
        if resp.status().is_success() {
            return Ok(());
        }

        let realm = resp
            .headers()
            .get("X-NDM-Realm")
            .context(
                "Router kimlik doğrulama yanıtı eksik (X-NDM-Realm). IP adresini kontrol edin.",
            )?
            .to_str()
            .context("X-NDM-Realm header geçersiz")?;

        let challenge = resp
            .headers()
            .get("X-NDM-Challenge")
            .context("Router kimlik doğrulama yanıtı eksik (X-NDM-Challenge).")?
            .to_str()
            .context("X-NDM-Challenge header geçersiz")?;

        // Adım 1: md5(kullanıcı:realm:şifre)
        let step1 = format!(
            "{}:{}:{}",
            self.username,
            realm,
            self.password.expose_secret()
        );
        let md5_hash = format!("{:x}", md5::compute(&step1));
        // Adım 2: sha256(challenge + md5_hash)
        let step2 = format!("{challenge}{md5_hash}");
        let password_hash = hex_hash(Sha256::digest(step2));

        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "login": self.username,
                "password": password_hash,
            }))
            .send()
            .await
            .context("Router giriş isteği gönderilemedi")?;

        if !resp.status().is_success() {
            anyhow::bail!(
                "Router girişi başarısız (HTTP {}). Kullanıcı adı veya şifreyi kontrol edin.",
                resp.status()
            );
        }

        Ok(())
    }

    // Ortak HTTP GET: tüm endpoint'ler bu fonksiyonu kullanır.
    async fn fetch_text(&self, path: &str, label: &str) -> Result<String> {
        let url = format!("{}/{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("{label} isteği gönderilemedi"))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .with_context(|| format!("{label} yanıtı okunamadı"))?;

        validate_response(label, status, &body)?;
        Ok(body)
    }

    pub async fn get_system_info(&self) -> Result<String> {
        self.login().await?;
        self.fetch_text("rci/show/system", "Sistem bilgisi").await
    }

    pub async fn get_interface(&self, name: &str) -> Result<String> {
        self.login().await?;
        self.fetch_text(
            &format!("rci/show/interface/{name}"),
            &format!("'{name}' arayüz bilgisi"),
        )
        .await
    }

    pub async fn get_ping_check(&self) -> Result<String> {
        self.login().await?;
        self.fetch_text("rci/show/ping-check", "Ping kontrol").await
    }

    pub async fn get_log(&self) -> Result<String> {
        self.login().await?;
        let body = self.fetch_text("ci/log", "Sistem günlüğü").await?;

        if body.trim().is_empty() {
            anyhow::bail!("Sistem günlüğü boş döndü.");
        }

        Ok(body.replace("\r\n", "\n"))
    }

    pub async fn get_wifi(&self) -> Result<String> {
        self.login().await?;
        self.fetch_text("rci/show/interface/WifiMaster0", "Wi-Fi durumu")
            .await
    }

    pub async fn get_mesh(&self) -> Result<String> {
        self.login().await?;
        self.fetch_text("rci/show/mws/member", "Mesh durumu").await
    }

    // Tek login + iki istek (get_system_info gibi her seferinde login yapmaz).
    pub async fn get_status(&self) -> Result<String> {
        self.login().await?;

        let system = self.fetch_text("rci/show/system", "Sistem bilgisi").await?;
        let pppoe = self
            .fetch_text("rci/show/interface/PPPoE0", "PPPoE durumu")
            .await?;

        let hostname = parse_hostname(&system);
        let uptime = parse_uptime(&system);
        let pppoe_status = if pppoe.contains("\"up\"") || pppoe.contains("running") {
            "Bağlı"
        } else {
            "Bağlı değil"
        };

        Ok(format!(
            "Router: {hostname}\nUptime: {uptime}\nPPPoE Durumu: {pppoe_status}"
        ))
    }
}

// Router HTML döndürürse (oturum bitmiş veya hatalı yanıt) hata verir.
fn validate_response(label: &str, status: reqwest::StatusCode, body: &str) -> Result<()> {
    if !status.is_success() {
        anyhow::bail!("{label} alınamadı (HTTP {status}).");
    }
    if body.trim_start().starts_with('<') {
        anyhow::bail!(
            "{label} geçersiz yanıt döndü. Oturum süresi dolmuş veya router erişilemiyor olabilir."
        );
    }
    Ok(())
}

fn hex_hash(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

fn parse_uptime(system_json: &str) -> String {
    let value: serde_json::Value = match serde_json::from_str(system_json) {
        Ok(v) => v,
        Err(_) => return "bilinmiyor".to_string(),
    };

    value
        .get("uptime")
        .and_then(parse_uptime_seconds)
        .map(|seconds| {
            let hours = seconds / 3600;
            let minutes = (seconds % 3600) / 60;
            format!("{hours} saat {minutes} dakika")
        })
        .unwrap_or_else(|| "bilinmiyor".to_string())
}

// Keenetic uptime'ı bazen sayı, bazen string döndürür — ikisini de dene.
fn parse_uptime_seconds(value: &serde_json::Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
        .or_else(|| value.as_i64().and_then(|n| u64::try_from(n).ok()))
}

fn parse_hostname(system_json: &str) -> String {
    let value: serde_json::Value = match serde_json::from_str(system_json) {
        Ok(v) => v,
        Err(_) => return "Keenetic".to_string(),
    };

    value
        .get("hostname")
        .and_then(|h| h.as_str())
        .unwrap_or("Keenetic")
        .to_string()
}
