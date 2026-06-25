// AI'ya gönderilecek prompt'u oluşturur.
// IP, MAC ve SSID bilgileri gönderilmeden önce maskelenir.
use crate::mask;

// 6 veri kaynağını birleştirip analiz talimatı + verileri döndürür.
pub fn generate_prompt(
    system: String,
    pppoe: String,
    ping: String,
    log: String,
    wifi: String,
    mesh: String,
) -> String {
    let system = mask::mask_sensitive(&system);
    let pppoe = mask::mask_sensitive(&pppoe);
    let ping = mask::mask_sensitive(&ping);
    let log = mask::mask_sensitive(&log);
    let wifi = mask::mask_sensitive(&wifi);
    let mesh = mask::mask_sensitive(&mesh);

    format!(
        r#"You are a network engineer. Analyze the router diagnostics below.

Logs are pre-filtered: boot noise and pre-WAN provisioning are already removed.

Rules:
- Only report issues clearly supported by the data below. Do not invent events.
- Single Modem hangup that recovers without ping-check or route loss: MEDIUM at most.
- HIGH/CRITICAL require correlated evidence (repeated hangups, ping-check flaps, or IP/route loss).
- List each distinct issue as one bullet under its severity. If none, write "none".
- Healthy mesh (online members, stable uplink) is not an issue — report mesh only for anomalies.
- ndnproxy/WSD [E] lines are LOW LAN noise, not WAN outages.

Output format:

## Router Diagnostic Report
**Snapshot time:** <from data>
**System uptime:** <human-readable>
**PPPoE session uptime:** <human-readable>
**PPPoE IP:** <address> | **Session ID:** <id> | **Remote AC:** <ac-mac>

### Issues by Severity

**[CRITICAL]**
- none | <one issue per bullet>

**[HIGH]**
- none | <one issue per bullet>

**[MEDIUM]**
- none | <one issue per bullet>

**[LOW]**
- none | <one issue per bullet>

### Root Cause Summary
<2-3 sentences, evidence-based>

### Recommended Actions
- <action>

---

**System Info:**
{}

**PPPoE0 Interface:**
{}

**Ping Check:**
{}

**Recent Logs:**
{}

**WiFi Status:**
{}

**Mesh Status:**
{}"#,
        system, pppoe, ping, log, wifi, mesh
    )
}
