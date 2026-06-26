// Kurulum ve geliştirme ortamları için config/outputs yolları.
use std::path::{Path, PathBuf};

const LOCAL_CONFIG: &str = "config.toml";

/// `KEENETIC_CONFIG` → `./config.toml` (varsa) → `~/.config/keencli/config.toml` → `./config.toml`
pub fn config_file() -> PathBuf {
    if let Ok(path) = std::env::var("KEENETIC_CONFIG") {
        return PathBuf::from(path);
    }
    if uses_dev_layout() {
        return PathBuf::from(LOCAL_CONFIG);
    }
    if let Some(path) = xdg_config_file()
        && path.exists()
    {
        return path;
    }
    PathBuf::from(LOCAL_CONFIG)
}

pub fn config_dir() -> Option<PathBuf> {
    let file = config_file();
    file.parent().map(|parent| parent.to_path_buf())
}

/// `KEENETIC_OUTPUTS_DIR` → `./outputs` (geliştirme) → `~/.local/share/keencli/outputs` (kurulum)
pub fn outputs_base() -> PathBuf {
    if let Ok(path) = std::env::var("KEENETIC_OUTPUTS_DIR") {
        return PathBuf::from(path);
    }
    if uses_dev_layout() || !uses_installed_layout() {
        PathBuf::from("outputs")
    } else {
        xdg_data_base().join("outputs")
    }
}

pub fn load_dotenv() {
    if let Some(dir) = config_dir() {
        let env_file = dir.join(".env");
        if env_file.is_file() {
            let _ = dotenv::from_path(env_file);
        }
    }
    let _ = dotenv::dotenv();
}

/// Repoda `config.toml` varsa geliştirme modu; kurulum yollarını geçersiz kılar.
fn uses_dev_layout() -> bool {
    Path::new(LOCAL_CONFIG).is_file()
}

fn uses_installed_layout() -> bool {
    xdg_config_file().is_some_and(|path| path.exists())
}

fn xdg_config_file() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("keencli/config.toml"))
}

fn xdg_data_base() -> PathBuf {
    if let Some(data) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(data).join("keencli");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".local/share/keencli");
    }
    PathBuf::from("outputs")
}

pub fn display_path(path: &Path) -> String {
    path.display().to_string()
}
