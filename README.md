# keencli

Keenetic router'dan tanı verisi toplayan, kaydeden ve isteğe bağlı AI ile analiz eden komut satırı aracı.

**Sürüm:** 1.0.2 · **Repo:** https://github.com/aethrox/keencli

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

Veriler `outputs/TARİH-SAAT/` altına kaydedilir (kurulum sonrası: `~/.local/share/keencli/outputs/`); router'a tekrar bağlanmadan analiz edilebilir. `outputs/` yoksa `fetch` veya `analyze` sırasında otomatik oluşturulur.

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

Kurulumdan sonra şu dosyalar kullanılır:

| Dosya | Açıklama |
|-------|----------|
| `~/.config/keencli/config.toml` | Router IP ve kullanıcı adı |
| `~/.config/keencli/.env` | Router şifresi (ve opsiyonel AI anahtarları) |
| `~/.local/share/keencli/outputs/` | Fetch ve analiz çıktıları |

**1. Config düzenle** — şifre yazmayın:

```bash
nano ~/.config/keencli/config.toml
```

**2. Şifreyi tanımla** — önerilen yol (her terminalde geçerli):

```bash
cp ~/.config/keencli/.env.example ~/.config/keencli/.env
nano ~/.config/keencli/.env
```

`.env` içeriği örneği:

```
KEENETIC_PASSWORD=router_şifreniz
```

Geçici alternatif: `export KEENETIC_PASSWORD='...'` (yalnızca o shell oturumu).

**3. Bağlantıyı test et:**

```bash
keencli status
```

**4. Veri çek ve analiz et:**

```bash
keencli fetch -a
keencli analyze
```

**5. (Opsiyonel) AI raporu** — `.env` dosyasına ekle:

```
OPENROUTER_API_KEY=sk-or-...
LLM_MODEL=anthropic/claude-sonnet-4.6
```

`~/.local/bin` PATH'te değilse shell profiline ekle ve yeni terminal aç:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

**Güncelleme:** Kurulum script'ini tekrar çalıştır (`curl ... | bash` veya `./install.sh`).

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

## Lisans

MIT — bkz. [LICENSE](LICENSE)