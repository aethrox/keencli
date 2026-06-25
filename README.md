# keencli

Keenetic router'dan tanı verisi toplayan, kaydeden ve isteğe bağlı AI ile analiz eden komut satırı aracı.

## Hedef

Keenetic router'larda (Hopper, Giga, Speedster vb.) bağlantı sorunlarını **tek komutla** teşhis etmeyi kolaylaştırmak.

Manuel log okuma ve web arayüzü gezintisi yerine:

1. Router verisini yapılandırılmış şekilde toplamak
2. Gürültülü syslog'u AI için anlamlı satırlara indirmek
3. WAN/PPPoE/link olaylarını korele eden tanı raporu üretmek

Uzun vadede: ev ağı sorunlarında hızlı, tekrarlanabilir ve kayıt altına alınabilir bir tanı akışı sağlamak.

## Ne için?

Router arayüzüne girmeden şunları yapmak için:

- Sistem, PPPoE, ping, log, Wi-Fi ve mesh verisini çekmek
- Logları süzüp AI'ya uygun prompt üretmek
- OpenRouter üzerinden tanı raporu almak

Veriler `outputs/TARİH-SAAT/` altına kaydedilir; router'a tekrar bağlanmadan analiz edilebilir.

## Nasıl çalışır?

```
keencli fetch -a          Router → JSON/log dosyaları
       ↓
keencli analyze           Log süzme → prompt → (opsiyonel) AI raporu
```

1. **fetch** — Router'a HTTP ile bağlanır (Keenetic auth), RCI endpoint'lerinden veri çeker.
2. **analyze** — En son `outputs/` klasörünü okur, log'u filtreler (~2800 → ~60 satır), hassas bilgileri maskeler, prompt üretir.
3. **AI** — `OPENROUTER_API_KEY` tanımlıysa prompt OpenRouter'a gönderilir; rapor aynı klasöre yazılır.

## Kurulum

**Gereksinimler:** Rust (edition 2024), router ile aynı ağ.

```bash
# config.toml
ip = "192.168.1.1"
username = "admin"

# Şifre (ortam değişkeni veya .env)
export KEENETIC_PASSWORD='router_şifreniz'

# Derle ve çalıştır
cargo build --release
./target/release/keencli --help
```

Nix kullanıyorsan: `nix develop` ile Rust ortamı hazır gelir.

## Komutlar

| Komut | Açıklama |
|-------|----------|
| `keencli fetch` | Yalnızca `system.json` |
| `keencli fetch -a` | Tüm veriler (analiz için gerekli) |
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

```bash
export OPENROUTER_API_KEY='sk-or-...'
export LLM_MODEL='anthropic/claude-sonnet-4.6'

keencli analyze
```

API key yoksa yalnızca `prompt_for_ai.txt` üretilir; komut hata vermez.

## Proje yapısı

```
src/
├── main.rs         CLI giriş noktası
├── api.rs          Router HTTP + Keenetic auth
├── config.rs       config.toml okuma
├── credentials.rs  Şifre ve API key kontrolleri
├── output.rs       Dosyaya kaydetme
├── analyze.rs      outputs/ okuma
├── log_filter.rs   Log süzme
├── prompt.rs       AI prompt + maskeleme
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
| JSON | serde / serde_json |
| Maskeleme | regex |
| Hata yönetimi | anyhow |

## Lisans

Kişisel proje — Kaan (Aethrox) Demirel