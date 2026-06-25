# keencli

Keenetic router'dan tanı verisi toplayan, kaydeden ve isteğe bağlı AI ile analiz eden komut satırı aracı.

**Sürüm:** 1.0.1 · **Repo:** https://github.com/aethrox/keencli

## Hedef

Keenetic router'larda (Hopper, Giga, Speedster vb.) bağlantı sorunlarını **tek komutla** teşhis etmeyi kolaylaştırmak.

Manuel log okuma ve web arayüzü gezintisi yerine:

1. Router verisini yapılandırılmış şekilde toplamak
2. Gürültülü syslog'u AI için anlamlı satırlara indirmek
3. WAN/PPPoE/link olaylarını korele eden tanı raporu üretmek

Uzun vadede: ev ağı sorunlarında hızlı, tekrarlanabilir ve kayıt altına alınabilir bir tanı akışı sağlamak.

### v1.0 kapsamı (tamamlandı)

| Alan | Durum |
|------|-------|
| `fetch` / `analyze` / `status` | Tamam |
| Log filtresi (boot/WAN/DNS churn) | Tamam |
| Hassas veri maskeleme + `SecretString` | Tamam |
| Nix paketi (`nix build`) | Tamam |
| OpenRouter AI raporu | Tamam |

## Ne için?

Router arayüzüne girmeden şunları yapmak için:

- Sistem, PPPoE, ping, log, Wi-Fi ve mesh verisini çekmek
- Logları süzüp AI'ya uygun prompt üretmek
- OpenRouter üzerinden tanı raporu almak

Veriler `outputs/TARİH-SAAT/` altına kaydedilir; router'a tekrar bağlanmadan analiz edilebilir. `outputs/` yoksa `fetch` veya `analyze` sırasında otomatik oluşturulur.

## Nasıl çalışır?

```
keencli fetch -a          Router → JSON/log dosyaları
       ↓
keencli analyze           Log süzme → maskeleme → prompt → (opsiyonel) AI raporu
```

1. **fetch** — Router'a HTTP ile bağlanır (Keenetic auth), RCI endpoint'lerinden veri çeker. Kayıt öncesi IP/MAC/SSID maskelenir.
2. **analyze** — En son `outputs/` klasörünü okur, log'u filtreler (~3000 → ~60 satır), prompt üretir.
3. **AI** — `OPENROUTER_API_KEY` tanımlıysa prompt OpenRouter'a gönderilir; rapor aynı klasöre yazılır.

## Kurulum

**Gereksinimler:** Rust (edition 2024), router ile aynı ağ.

```bash
git clone https://github.com/aethrox/keencli.git
cd keencli

cp config.toml.example config.toml
# config.toml: ip ve username
# Şifreyi config.toml'a YAZMAYIN

export KEENETIC_PASSWORD='router_şifreniz'   # veya .env dosyası

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

```bash
export OPENROUTER_API_KEY='sk-or-...'
export LLM_MODEL='anthropic/claude-sonnet-4.6'

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

## Lisans

MIT — bkz. [LICENSE](LICENSE)