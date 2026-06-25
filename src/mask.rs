// IP, MAC ve SSID maskeleme — prompt ve outputs/ için ortak.
use regex::Regex;

pub fn mask_sensitive(text: &str) -> String {
    let mut masked = text.to_string();

    if let Ok(re) = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b") {
        masked = re.replace_all(&masked, "[IP_MASKED]").to_string();
    }

    if let Ok(re) = Regex::new(r"([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})") {
        masked = re.replace_all(&masked, "[MAC_MASKED]").to_string();
    }

    if let Ok(re) = Regex::new(r#""ssid":\s*"[^"]+""#) {
        masked = re
            .replace_all(&masked, r#""ssid": "[SSID_MASKED]""#)
            .to_string();
    }

    masked
}
