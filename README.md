# keencli

Keenetic router'lardan CLI ile tanı verisi toplayan, logları süzen ve isteğe bağlı AI raporu üreten komut satırı aracı.

> [!CAUTION]
> **Yapay zeka çıktıları hata içerebilir. Kesin teşhis ve router değişikliklerinden önce bulguları loglar üzerinden veya bir uzmanla doğrulayın.**

Hopper, Giga, Speedster vb. modellerde PPPoE kopması, WAN sorunları veya bağlantı dalgalanması yaşandığında web arayüzüne girmeden terminalden teşhis koymanızı sağlar. Veriler zaman damgalı klasörlere kaydedilir; router'a yeniden bağlanmadan analiz edebilirsiniz.

**Sürüm:** 1.0.6 · **Repo:** https://github.com/aethrox/keencli

```bash
curl -fsSL https://raw.githubusercontent.com/aethrox/keencli/main/install.sh | bash
```

## Ne yapar?

| Adım | Açıklama |
|------|----------|
| `fetch` | Sistem, PPPoE, ping, log, Wi-Fi ve mesh verisini router'dan çeker |
| `analyze` | Log'u süzer (~3000 → ~60 satır), maskeler, AI prompt'u üretir |
| `status` | Canlı hostname, uptime ve PPPoE durumunu gösterir |

- Kayıt öncesi IP, MAC ve SSID maskelenir; şifre yalnızca `.env` veya ortam değişkeninden okunur
- Log filtresi (boot, WAN, DNS churn); `install.sh` / `uninstall.sh`; Nix paketi (`nix build`)
- `OPENROUTER_API_KEY` tanımlıysa OpenRouter üzerinden tanı raporu yazılır
- Kurulum sonrası çıktılar `~/.local/share/keencli/outputs/TARİH-SAAT/` altında saklanır

## Nasıl çalışır?

```
keencli fetch -a     Router → JSON ve log dosyaları
       ↓
keencli analyze      Süzme → maskeleme → prompt → (opsiyonel) AI raporu
```

1. **fetch** — Keenetic auth ile router'a bağlanır, RCI endpoint'lerinden veri çeker
2. **analyze** — En son fetch klasörünü okur, log'u filtreler, `prompt_for_ai.txt` üretir
3. **AI** — API key varsa prompt OpenRouter'a gönderilir; rapor aynı klasöre kaydedilir

## Kurulum

**Gereksinimler:** Rust (edition 2024), router ile aynı ağ.

### Tek komut (önerilen)

```bash
curl -fsSL https://raw.githubusercontent.com/aethrox/keencli/main/install.sh | bash
```

Script şunları yapar:

- Kaynağı `~/.local/share/keencli/src` altına indirir ve derler
- Binary'yi `~/.local/bin/keencli` konumuna kurar
- Örnek config'i `~/.config/keencli/config.toml` olarak oluşturur
- Fetch çıktıları `~/.local/share/keencli/outputs/` altına yazılır

`~/.local/bin` PATH'te değilse script uyarı verir; shell profiline ekleyin:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Yerel script ile kurulum:

```bash
./install.sh
```

Kurulum script'i bitince config, şifre ve ilk kullanım adımlarını ekranda gösterir.

### Kurulum sonrası

| Dosya | Ne için |
|-------|---------|
| `~/.config/keencli/config.toml` | Router IP ve kullanıcı adı |
| `~/.config/keencli/.env` | Router şifresi, AI anahtarları |
| `~/.local/share/keencli/outputs/` | Fetch ve analiz çıktıları |

**1 — Config** (`ip`, `username`; şifre yazmayın):

```bash
nano ~/.config/keencli/config.toml
```

**2 — Şifre** (kalıcı, önerilen):

```bash
cp ~/.config/keencli/.env.example ~/.config/keencli/.env
nano ~/.config/keencli/.env
```

```env
KEENETIC_PASSWORD=router_şifreniz
```

Geçici: `export KEENETIC_PASSWORD='...'` (yalnızca o terminal).

**3 — Test:**

```bash
keencli status
```

**4 — Kullanım:**

```bash
keencli fetch -a
keencli analyze
```

**5 — AI (opsiyonel)** — `.env` dosyasına ekleyin:

```env
OPENROUTER_API_KEY=sk-or-...
LLM_MODEL=anthropic/claude-sonnet-4.6   # önerilen
LLM_TEMPERATURE=0.3                     # opsiyonel; varsayılan 0.3
```

`LLM_TEMPERATURE` tanımlı değilse **0.3** kullanılır (tanı raporu için önerilen). Deneme için `0.7` verebilirsiniz; `sakana/*` modellerinde gönderilmez.

**Güncelleme:** `install.sh` script'ini tekrar çalıştırın.

### Kaldırma

Tamamen etkileşimli — binary, config ve veri için ayrı onay; varsayılan yanıt hayır.

```bash
curl -fsSL https://raw.githubusercontent.com/aethrox/keencli/main/uninstall.sh -o uninstall.sh
bash uninstall.sh
```

`curl | bash` çalışmaz; önce indirip terminalden çalıştırın. Yanlışlıkla silmeyi önlemek için kasıtlıdır.

### Manuel (geliştirme)

```bash
git clone https://github.com/aethrox/keencli.git
cd keencli

cp config.toml.example config.toml
# config.toml: ip ve username
# Şifreyi config.toml'a YAZMAYIN

cp .env.example .env                        # KEENETIC_PASSWORD burada

cargo build --release
./target/release/keencli --help
```

### Nix

```bash
nix develop          # geliştirme ortamı
nix build            # paket derle
./result/bin/keencli --version

nix run              # flake app ile çalıştır
```

`cargo build` sonrası güncel binary: `./target/debug/keencli` veya `./target/release/keencli`.  
`./result/bin/keencli` yalnızca `nix build` sonrası günceldir.

## Komutlar

| Komut | Açıklama |
|-------|----------|
| `keencli fetch` | Yalnızca `system.json` |
| `keencli fetch -a` | Tüm veriler (analyze için gerekli) |
| `keencli analyze` | Prompt + opsiyonel AI raporu |
| `keencli status` | Canlı hostname, uptime, PPPoE |

Detay: `keencli <komut> --help`

## Çıktı dosyaları

`fetch -a` sonrası:

```
outputs/2026-06-25_18-15-39/
├── system.json
├── interface_PPPoE0.json
├── pingcheck.json
├── log.txt
├── wifi.json
└── mesh.json
```

`analyze` sonrası eklenenler:

```
├── prompt_for_ai.txt
└── ai_report_anthropic-claude-sonnet-4.6.md   # API key varsa
```

## AI analizi (opsiyonel)

### Önerilen modeller

Gerçek router verisi üzerinde test edilmiş öneriler — ayrıntılı karşılaştırma: [AI_MODELS.md](AI_MODELS.md)

| Öncelik | Model | Ne zaman? |
|---------|-------|-----------|
| **1 — Önerilen** | `anthropic/claude-sonnet-4.6` | En eksiksiz tanı raporu; varsayılan tercih |
| **2 — Ekonomik** | `deepseek/deepseek-v4-pro` | Düşük maliyet; testte güvenilir ikinci seçenek |
| **3 — Alternatif** | `google/gemini-2.5-pro` | Farklı sağlayıcı; severity bazen şişkin |
| **4 — Bütçe** | `qwen/qwen3.5-plus-02-15` | Ucuz; DNS iyi, bazı WAN olayları atlanabilir |

Bu görevde **önerilmez** (test): `openai/gpt-4.1`, `qwen/qwen3-235b-a22b`, `x-ai/grok-4.20`, `x-ai/grok-4.3` — log flap'lerini sık kaçırır veya route kaybını inkâr eder.

Model adlarını [OpenRouter model listesinden](https://openrouter.ai/models) doğrulayın.

```bash
export OPENROUTER_API_KEY='sk-or-...'
export LLM_MODEL='anthropic/claude-sonnet-4.6'   # önerilen
export LLM_TEMPERATURE='0.3'                     # opsiyonel

keencli analyze
```

API key yoksa yalnızca `prompt_for_ai.txt` üretilir; komut hata vermez.  
`analyze` sırasında verilerin OpenRouter'a gönderileceği konusunda uyarı gösterilir.

## Güvenlik

- Router şifresi yalnızca `KEENETIC_PASSWORD` ortam değişkeni (veya `.env`)
- `config.toml`'a şifre yazılamaz (`deny_unknown_fields`)
- `outputs/` kayıtları ve AI prompt'u IP/MAC/SSID maskelenmiş halde
- `.env` ve `outputs/` git'e girmez

## Proje yapısı

```
src/
├── main.rs         CLI giriş noktası
├── api.rs          Router HTTP + Keenetic auth
├── config.rs       config.toml okuma
├── paths.rs        XDG config/outputs yolları
├── credentials.rs  Şifre ve API key (SecretString)
├── output.rs       Maskelenmiş dosyaya kaydetme
├── analyze.rs      outputs/ okuma ve oluşturma
├── log_filter.rs   Log süzme (WAN, ping-check, DNS churn)
├── mask.rs         IP/MAC/SSID maskeleme
├── prompt.rs       AI prompt üretimi
└── llm.rs          OpenRouter istemcisi
```

## Kullanılan teknolojiler

| Alan | Kütüphane |
|------|-----------|
| CLI | clap |
| HTTP | reqwest (cookies) |
| Async | tokio |
| Config | config + TOML |
| Auth | md5 + sha2 (Keenetic challenge) |
| Gizli veri | secrecy |
| JSON | serde / serde_json |
| Maskeleme | regex |
| Hata yönetimi | anyhow |

## Yasal uyarı

- **Bağımsız proje** — keencli, Keenetic veya OpenRouter ile bağlantılı değildir; resmi bir ürün değildir.
- **Garanti yok** — Yazılım «olduğu gibi» (as-is) sunulur; açık veya zımni garanti verilmez. Kullanımdan doğan zararlardan yazar sorumlu tutulamaz.
- **AI çıktıları** — OpenRouter üzerinden üretilen raporlar yalnızca yardımcı öneridir; hata, eksik veya yanlış bilgi içerebilir. Ağ veya router ayarı değiştirmeden önce sonuçları bağımsız olarak doğrulamanız gerekir.
- **Üçüncü taraf hizmetler** — AI analizi OpenRouter'a veri gönderir; kullanımınız kendi API koşullarına tabidir. Veriler, maskeleme sonrası bile üçüncü tarafa iletilir.
- **Kullanıcı sorumluluğu** — Router kimlik bilgilerinizi ve API anahtarlarınızı güvenli tutmak, yerel mevzuata ve hizmet koşullarına uymak sizin sorumluluğunuzdadır.

## Lisans

MIT — ayrıntılar için [LICENSE](LICENSE). Yukarıdaki sorumluluk reddi, MIT lisansının «as-is» koşullarını tamamlar.
