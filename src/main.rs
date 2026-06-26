// Program giriş noktası.
// Her `mod` satırı src/ altındaki bir dosyayı dahil eder.
// Örnek: `mod api` → src/api.rs
mod analyze; // analyze komutu: outputs/ klasöründen veri okur
mod api; // Router HTTP istekleri (login, veri çekme)
mod config; // config.toml okuma
mod credentials; // Şifre ve API key kontrolleri
mod llm; // OpenRouter AI çağrıları
mod log_filter; // Log süzme (~3000 → ~60 satır)
mod mask; // IP, MAC, SSID maskeleme
mod output; // Çekilen veriyi dosyaya yazma
mod paths; // Config ve outputs yolları (kurulum / geliştirme)
mod prompt; // AI prompt üretimi

use chrono::Local;
use clap::{Parser, Subcommand};
use tokio::fs;

const SETUP_HELP: &str = "\
\n\
Yapılandırma\n\
\n\
[Kurulum — install.sh sonrası]\n\
- config  ~/.config/keencli/config.toml\n\
- şifre   ~/.config/keencli/.env\n\
- veri    ~/.local/share/keencli/outputs/\n\
\n\
[Geliştirme — repo içinden]\n\
- config  ./config.toml\n\
- şifre   ./.env\n\
- veri    ./outputs/\n\
\n\
Şifreyi config.toml dosyasına yazmayın.\n\
Komut ayrıntısı: keencli <komut> --help\n";

#[derive(Parser)]
#[command(
    author,
    version = env!("CARGO_PKG_VERSION"),
    about = "Keenetic router bilgi toplama aracı",
    long_about = "\
Keenetic router'dan veri çeker, outputs/ klasörüne kaydeder ve analiz eder.",
    after_help = SETUP_HELP,
    help_template = "\
{version} — {about-with-newline}\n\
\n\
Kullanım: {usage}\n\
\n\
{subcommands}\n\
\n\
Seçenekler:\n\
{options}{after-help}\
",
    subcommand_help_heading = "Komutlar",
    subcommand_value_name = "KOMUT",
    arg_required_else_help = true,
    disable_help_subcommand = true,
    disable_version_flag = false,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Router'dan veri çeker → outputs/TARİH/
    #[command(
        about = "Router'dan veri çeker",
        long_about = "\
Router'a bağlanır ve veriyi zaman damgalı klasöre kaydeder.\n\
\n\
[Kullanım]\n\
- keencli fetch      → system.json\n\
- keencli fetch -a   → analyze için tüm dosyalar\n\
\n\
[fetch -a dosyaları]\n\
- system.json\n\
- interface_PPPoE0.json\n\
- pingcheck.json\n\
- log.txt\n\
- wifi.json\n\
- mesh.json\n\
\n\
[Çıktı konumu]\n\
- Kurulum:    ~/.local/share/keencli/outputs/TARİH-SAAT/\n\
- Geliştirme: ./outputs/TARİH-SAAT/",
        help_template = "\
{about-with-newline}\n\
\n\
Kullanım: {usage}\n\
\n\
Seçenekler:\n\
{options}\
"
    )]
    Fetch {
        /// PPPoE, log, Wi-Fi ve mesh dahil tüm verileri çeker
        #[arg(short, long)]
        all: bool,
    },

    /// Kayıtlı veriyi analiz eder, AI raporu üretir
    #[command(
        about = "Kayıtlı veriyi analiz eder",
        long_about = "\
En son fetch klasöründeki kayıtlı veriyi analiz eder.\n\
Fetch verisi yoksa hata verir.\n\
\n\
[Adımlar]\n\
1. Kayıtlı verileri okur, log'u süzer (~3000 → ~60 satır)\n\
2. IP/MAC/SSID maskeler, prompt_for_ai.txt üretir\n\
3. API key varsa OpenRouter ile AI raporu yazar\n\
\n\
[Girdi — fetch -a ile oluşmuş olmalı]\n\
- system.json, interface_PPPoE0.json, pingcheck.json\n\
- log.txt, wifi.json, mesh.json\n\
\n\
[Çıktı]\n\
- prompt_for_ai.txt   (her zaman)\n\
- ai_report_MODEL.md  (API key varsa)\n\
\n\
[AI — opsiyonel ortam değişkenleri]\n\
- OPENROUTER_API_KEY\n\
- LLM_MODEL        önerilen: anthropic/claude-sonnet-4.6\n\
- LLM_TEMPERATURE  opsiyonel, varsayılan 0.3 (0.0–2.0)\n\
             ekonomik: deepseek/deepseek-v4-pro\n\
\n\
[Örnek akış]\n\
keencli fetch -a\n\
export OPENROUTER_API_KEY='...'\n\
export LLM_MODEL='anthropic/claude-sonnet-4.6'\n\
keencli analyze",
        help_template = "\
{about-with-newline}\n\
\n\
Kullanım: {usage}\n\
\n\
Seçenekler:\n\
{options}\
"
    )]
    Analyze,

    /// Canlı router durumu (hostname, uptime, PPPoE)
    #[command(
        about = "Canlı router durumu",
        long_about = "\
Router'a bağlanır ve özet durum gösterir. Veri kaydetmez.\n\
\n\
[Gösterilen bilgiler]\n\
- hostname\n\
- uptime\n\
- PPPoE bağlantı durumu\n\
\n\
Bağlantı testi için kullanışlıdır.",
        help_template = "\
{about-with-newline}\n\
\n\
Kullanım: {usage}\n\
\n\
Seçenekler:\n\
{options}\
"
    )]
    Status,
}

// `fetch` ve `fetch -a` komutunun tüm adımları.
// `?` işareti: hata olursa fonksiyondan çıkar ve kullanıcıya mesaj gösterir.
async fn run_fetch(all: bool) -> anyhow::Result<()> {
    let config = config::load()?;
    let password = credentials::resolve_password()?;
    let api = api::ApiClient::new(config.ip.clone(), config.username.clone(), password)?;

    println!("Router IP: {}", config.ip);
    println!("Username: {}", config.username);
    println!("Veri çekme işlemi başlatılıyor...");

    let outputs_base = analyze::ensure_outputs_dir()?;
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let dir = outputs_base.join(&timestamp);
    fs::create_dir_all(&dir).await?;
    let dir = paths::display_path(&dir);

    let info = api.get_system_info().await?;
    output::save_system_info_to_dir(&dir, info)?;

    // `-a` bayrağı yoksa yalnızca system.json kaydedilir.
    if all {
        let pppoe = api.get_interface("PPPoE0").await?;
        output::save_interface_to_dir(&dir, "PPPoE0", pppoe)?;

        let ping = api.get_ping_check().await?;
        output::save_ping_check_to_dir(&dir, ping)?;

        let log = api.get_log().await?;
        output::save_log_to_dir(&dir, log)?;

        let wifi = api.get_wifi().await?;
        output::save_wifi_to_dir(&dir, wifi)?;

        let mesh = api.get_mesh().await?;
        output::save_mesh_to_dir(&dir, mesh)?;

        println!("Tüm veriler başarıyla çekildi ve kaydedildi.");
    } else {
        println!("Temel veri çekildi. Tam çekim için: keencli fetch -a");
    }

    Ok(())
}

// `analyze` komutu: kayıtlı veriden prompt üretir, API key varsa AI raporu da yazar.
async fn run_analyze() -> anyhow::Result<()> {
    println!("Analiz için son fetch verileri kullanılıyor...");

    let dir_path = analyze::find_latest_fetch_dir()?;
    println!("Kullanılan klasör: {}", dir_path.display());

    let data = analyze::load_fetch_data(&dir_path)?;
    let log = log_filter::filter_log_lines(&data.raw_log);
    let prompt = prompt::generate_prompt(
        data.system,
        data.pppoe,
        data.ping,
        log,
        data.wifi,
        data.mesh,
    );

    let dir_name = analyze::folder_name(&dir_path)?;
    let prompt_file = analyze::report_path(&dir_name, "prompt_for_ai.txt");
    std::fs::write(&prompt_file, &prompt)?;
    println!("Prompt hazırlandı: {prompt_file}");

    // API key yoksa hata vermeyiz; prompt yine kaydedilir.
    match credentials::resolve_llm_config() {
        Ok((api_key, model)) => {
            println!(
                "Not: Tanılama verileri OpenRouter üzerinden '{model}' modeline gönderilecek."
            );
            println!("Not: AI raporu otomatik üretilir; %100 doğruluk garantisi yoktur.");
            println!("AI analizi yapılıyor...");
            let llm = llm::LLMClient::new(api_key, model)?;
            let result = llm.analyze(prompt).await?;
            let report_name = llm::LLMClient::report_filename(llm.model());
            let report_file = analyze::report_path(&dir_name, &report_name);
            let report = llm::LLMClient::format_report(llm.model(), &result);

            std::fs::write(&report_file, report)?;
            println!("AI raporu kaydedildi: {report_file} ({})", llm.model());
        }
        Err(error) => {
            println!("AI analizi atlandı: {error:#}");
            println!(
                "Yalnızca prompt üretildi. AI için OPENROUTER_API_KEY ve LLM_MODEL tanımlayın."
            );
        }
    }

    Ok(())
}

async fn run_status() -> anyhow::Result<()> {
    let config = config::load()?;
    let password = credentials::resolve_password()?;
    let api = api::ApiClient::new(config.ip, config.username, password)?;
    let status = api.get_status().await?;
    println!("{status}");
    Ok(())
}

// `#[tokio::main]`: async fonksiyonların çalışması için gerekli (HTTP istekleri).
// `paths::load_dotenv`: ~/.config/keencli/.env veya ./.env
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    paths::load_dotenv();
    let cli = Cli::parse(); // terminalde yazdığın komutu okur (fetch, analyze, status)

    match cli.command {
        Commands::Fetch { all } => run_fetch(all).await?,
        Commands::Analyze => run_analyze().await?,
        Commands::Status => run_status().await?,
    }

    Ok(())
}
