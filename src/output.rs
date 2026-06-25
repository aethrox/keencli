// Router'dan çekilen veriyi outputs/TARİH/ altına kaydeder.
// IP, MAC ve SSID diske yazılmadan önce maskelenir.
use crate::mask;
use anyhow::Context;
use chrono::Local;
use serde_json::json;
use std::fs;

pub fn save_system_info_to_dir(dir: &str, data: String) -> anyhow::Result<()> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let data = mask::mask_sensitive(&data);

    let filename = format!("{}/system.json", dir);
    let output = json!({
        "timestamp": timestamp,
        "type": "system_info",
        "data": data
    });

    fs::write(&filename, serde_json::to_string_pretty(&output)?)
        .with_context(|| format!("'{filename}' yazılamadı"))?;
    println!("Sistem bilgisi kaydedildi: {filename}");
    Ok(())
}

pub fn save_interface_to_dir(dir: &str, name: &str, data: String) -> anyhow::Result<()> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let data = mask::mask_sensitive(&data);

    let filename = format!("{}/interface_{}.json", dir, name);
    let output = json!({
        "timestamp": timestamp,
        "type": "interface",
        "name": name,
        "data": data
    });
    fs::write(&filename, serde_json::to_string_pretty(&output)?)
        .with_context(|| format!("'{filename}' yazılamadı"))?;
    Ok(())
}

pub fn save_ping_check_to_dir(dir: &str, data: String) -> anyhow::Result<()> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let data = mask::mask_sensitive(&data);

    let filename = format!("{}/pingcheck.json", dir);
    let output = json!({
        "timestamp": timestamp,
        "type": "ping_check",
        "data": data
    });
    fs::write(&filename, serde_json::to_string_pretty(&output)?)
        .with_context(|| format!("'{filename}' yazılamadı"))?;
    Ok(())
}

// Log hem düz metin (log.txt) hem özet JSON (log.json) olarak kaydedilir.
pub fn save_log_to_dir(dir: &str, data: String) -> anyhow::Result<()> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let data = mask::mask_sensitive(&data);
    let line_count = data.lines().count();

    let txt_path = format!("{}/log.txt", dir);
    fs::write(&txt_path, &data).with_context(|| format!("'{txt_path}' yazılamadı"))?;

    let json_path = format!("{}/log.json", dir);
    let output = json!({
        "timestamp": timestamp,
        "type": "log",
        "line_count": line_count,
        "byte_count": data.len(),
        "data": data
    });
    fs::write(&json_path, serde_json::to_string_pretty(&output)?)
        .with_context(|| format!("'{json_path}' yazılamadı"))?;
    println!(
        "Log kaydedildi: {line_count} satır, {} byte → {txt_path}",
        data.len()
    );
    Ok(())
}

pub fn save_wifi_to_dir(dir: &str, data: String) -> anyhow::Result<()> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let data = mask::mask_sensitive(&data);

    let filename = format!("{}/wifi.json", dir);
    let output = json!({
        "timestamp": timestamp,
        "type": "wifi",
        "data": data
    });
    fs::write(&filename, serde_json::to_string_pretty(&output)?)
        .with_context(|| format!("'{filename}' yazılamadı"))?;
    Ok(())
}

pub fn save_mesh_to_dir(dir: &str, data: String) -> anyhow::Result<()> {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let data = mask::mask_sensitive(&data);

    let filename = format!("{}/mesh.json", dir);
    let output = json!({
        "timestamp": timestamp,
        "type": "mesh",
        "data": data
    });
    fs::write(&filename, serde_json::to_string_pretty(&output)?)
        .with_context(|| format!("'{filename}' yazılamadı"))?;
    Ok(())
}
