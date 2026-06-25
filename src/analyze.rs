// `analyze` komutunun dosya işlemleri: outputs/ klasörünü bulur, verileri okur.
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

// fetch -a ile kaydedilen 6 dosyanın içeriği
pub struct FetchData {
    pub system: String,
    pub pppoe: String,
    pub ping: String,
    pub raw_log: String,
    pub wifi: String,
    pub mesh: String,
}

const OUTPUTS_DIR: &str = "outputs";

pub fn ensure_outputs_dir() -> Result<PathBuf> {
    let path = PathBuf::from(OUTPUTS_DIR);
    if path.exists() && !path.is_dir() {
        anyhow::bail!("'{OUTPUTS_DIR}' bir klasör değil. Dosyayı silin veya yeniden adlandırın.");
    }
    std::fs::create_dir_all(&path)
        .with_context(|| format!("'{OUTPUTS_DIR}' klasörü oluşturulamadı"))?;
    Ok(path)
}

// En son değiştirilen alt klasörü seçer (ör. outputs/2026-06-25_18-15-39/)
pub fn find_latest_fetch_dir() -> Result<PathBuf> {
    let base = ensure_outputs_dir()?;
    let entries = std::fs::read_dir(&base)
        .with_context(|| format!("'{}' klasörü okunamadı", base.display()))?;

    entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .max_by_key(|entry| entry.metadata().and_then(|meta| meta.modified()).ok())
        .map(|entry| entry.path())
        .with_context(|| {
            format!("'{OUTPUTS_DIR}' içinde analiz verisi yok. Önce 'keencli fetch -a' çalıştırın.")
        })
}

pub fn load_fetch_data(dir: &Path) -> Result<FetchData> {
    let read = |filename: &str, description: &str| -> Result<String> {
        let path = dir.join(filename);
        std::fs::read_to_string(&path).with_context(|| {
            format!(
                "'{}' okunamadı ({description}). Tam veri için 'keencli fetch -a' çalıştırın.",
                path.display()
            )
        })
    };

    Ok(FetchData {
        system: read("system.json", "sistem bilgisi")?,
        pppoe: read("interface_PPPoE0.json", "PPPoE arayüz bilgisi")?,
        ping: read("pingcheck.json", "ping kontrol verisi")?,
        raw_log: read("log.txt", "sistem günlüğü")?,
        wifi: read("wifi.json", "Wi-Fi durumu")?,
        mesh: read("mesh.json", "mesh durumu")?,
    })
}

pub fn folder_name(dir: &Path) -> Result<String> {
    dir.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .with_context(|| format!("Geçersiz klasör yolu: {}", dir.display()))
}

pub fn report_path(dir_name: &str, filename: &str) -> String {
    format!("{OUTPUTS_DIR}/{dir_name}/{filename}")
}
