// Keenetic syslog'unu LLM için süzer (~2789 → ~66 satır).
//
// Çıkarılanlar: kernel boot gürültüsü, driver spam, ilk WAN olayından önceki provisioning
// Tutulanlar: PPPoE, Modem hangup, ping-check, port 0/1 link, IP/route/DNS değişimleri, [E]/[C]
pub fn filter_log_lines(raw: &str) -> String {
    let lines: Vec<&str> = raw.lines().collect();
    let original_count = lines.len();
    let wan_start = find_wan_start_index(&lines);

    let kept: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let parsed = parse_syslog_line(line);

            if wan_start.is_some_and(|start| index < start) {
                return keep_pre_wan_line(line, parsed.as_ref()).then_some(*line);
            }

            (!should_filter_out(line, parsed.as_ref()) && should_keep(line, parsed.as_ref()))
                .then_some(*line)
        })
        .collect();

    let filtered_count = kept.len();
    eprintln!("log filter: {original_count} → {filtered_count} lines");

    kept.join("\n")
}

// İlk WAN hazır olayı: bundan önceki satırlar boot/provisioning sayılır.
fn find_wan_start_index(lines: &[&str]) -> Option<usize> {
    lines.iter().position(|line| is_wan_ready_marker(line))
}

fn is_wan_ready_marker(line: &str) -> bool {
    if line.contains("PPPoE0") && line.contains("IP address is") {
        return true;
    }
    if line.contains("ping-check") && line.contains("running") {
        return true;
    }
    if line.contains("default route") && line.contains("added") {
        return true;
    }

    false
}

// WAN ayağa kalkmadan önce yalnızca gerçek [E]/[C] hataları tutulur.
fn keep_pre_wan_line(line: &str, parsed: Option<&ParsedLine<'_>>) -> bool {
    if is_provisioning_boilerplate(line) || is_bridge_churn(line) {
        return false;
    }

    let Some(parsed) = parsed else {
        return false;
    };

    if parsed.level != "E" && parsed.level != "C" {
        return false;
    }

    !should_filter_out(line, Some(parsed))
}

#[derive(Debug, Clone, Copy)]
struct ParsedLine<'a> {
    level: &'a str,
    month: &'a str,
    day: u32,
    time_secs: u32,
    source: &'a str,
}

fn parse_syslog_line(line: &str) -> Option<ParsedLine<'_>> {
    let line = line.trim_end();
    let after_level = line.strip_prefix('[')?;
    let (level, rest) = after_level.split_once(']')?;
    let rest = rest.trim_start();

    if rest.len() < 14 {
        return None;
    }

    let month = &rest[0..3];
    let mut i = 3;
    while i < rest.len() && rest.as_bytes()[i] == b' ' {
        i += 1;
    }

    let day_start = i;
    while i < rest.len() && rest.as_bytes()[i].is_ascii_digit() {
        i += 1;
    }
    let day: u32 = rest[day_start..i].parse().ok()?;

    while i < rest.len() && rest.as_bytes()[i] == b' ' {
        i += 1;
    }

    if i + 8 > rest.len() {
        return None;
    }
    let time_str = &rest[i..i + 8];
    let time_secs = parse_hms(time_str)?;
    i += 8;

    if rest.as_bytes().get(i) != Some(&b' ') {
        return None;
    }
    i += 1;

    let source_start = i;
    let colon = rest[i..].find(": ")?;
    let source = &rest[source_start..i + colon];

    Some(ParsedLine {
        level,
        month,
        day,
        time_secs,
        source,
    })
}

fn parse_hms(time: &str) -> Option<u32> {
    let bytes = time.as_bytes();
    if bytes.len() != 8 || bytes[2] != b':' || bytes[5] != b':' {
        return None;
    }
    let h: u32 = time[0..2].parse().ok()?;
    let m: u32 = time[3..5].parse().ok()?;
    let s: u32 = time[6..8].parse().ok()?;
    Some(h * 3600 + m * 60 + s)
}

fn is_bridge_churn(line: &str) -> bool {
    let mentions_bridge =
        line.contains("Bridge0") || line.contains("Bridge1") || line.contains("Bridge5");
    if !mentions_bridge {
        return false;
    }

    line.contains("interface is up")
        || line.contains("interface is down")
        || line.contains("IP address is")
        || line.contains("IP address cleared")
}

// true dönerse satır atılır (gürültü, boot, driver spam vb.)
fn should_filter_out(line: &str, parsed: Option<&ParsedLine<'_>>) -> bool {
    if line.contains("DriverManager: loading") {
        return true;
    }
    if line.contains("ensoc_dmt") {
        return true;
    }
    if line.contains("atm0") || line.contains("ptm0") {
        return true;
    }
    if line.contains("not found: \"show/log\"")
        || line.contains("not found: \"components/list\"")
        || line.contains("not found: \"log\"")
    {
        return true;
    }
    if line.contains("last message repeated") {
        return true;
    }

    let Some(parsed) = parsed else {
        return false;
    };

    // Keenetic bazen syslog tarihini Jan 1 olarak yazar — kernel boot gürültüsü.
    if parsed.month == "Jan" && parsed.day == 1 && parsed.source == "kernel" {
        return true;
    }

    if parsed.month == "Jan" && parsed.day == 1 && parsed.time_secs <= 60 {
        return true;
    }

    if is_bridge_churn(line) {
        return true;
    }

    if is_provisioning_boilerplate(line) {
        return true;
    }

    false
}

fn is_provisioning_boilerplate(line: &str) -> bool {
    if is_dns_session_churn(line) {
        return false;
    }

    const PATTERNS: &[&str] = &[
        "interface created",
        "description saved",
        "identity saved",
        "password saved",
        "Interface::Repository:",
        "LCP echo parameters",
        "TCP-MSS adjustment",
        "assigned role",
        "security level set",
        "static MTU is",
        "global priority",
        "accept IPv4 name servers",
        "accept IPv6 name servers",
        "accept prefixes",
        "accept address provided",
        "DHCP client hostname",
        "DHCP client DNS",
        "started DHCP client on",
        "renamed to",
        "MAC address is",
        "backed up the default address",
        "network MTU reset to default",
        "default MTU is",
        "enabled connection via",
        "connection service standby",
        "NDM DHCP Client, v",
        "created PID file",
        "Igmp::Proxy",
        "PingCheck::Client: set",
        "adding name server",
        "adding a host route",
        "host route for name server",
        "added, domain",
    ];

    PATTERNS.iter().any(|pattern| line.contains(pattern))
}

// true dönerse satır tutulur (WAN, PPPoE, link, hata seviyesi vb.)
fn should_keep(line: &str, parsed: Option<&ParsedLine<'_>>) -> bool {
    if line.contains("PPPoE0") {
        return true;
    }
    if line.contains("Modem hangup") {
        return true;
    }
    if line.contains("ping-check") {
        return true;
    }
    if line.contains("Vlan35") {
        return true;
    }
    if ge_port_link_state_change(line) {
        return true;
    }
    if line.contains("IP address is") || line.contains("IP address cleared") {
        return true;
    }
    if line.contains("default route") {
        return true;
    }
    if is_dns_session_churn(line) {
        return true;
    }

    if let Some(parsed) = parsed {
        if parsed.level == "E" || parsed.level == "C" {
            return true;
        }
    }

    false
}

// WAN oturumu sırasında ISP DNS sunucusu eklenmesi/silinmesi (ping-check ile korele).
fn is_dns_session_churn(line: &str) -> bool {
    line.contains("Dns::Manager:")
        && line.contains("name server")
        && (line.contains("deleted") || line.contains("added,"))
}

fn ge_port_link_state_change(line: &str) -> bool {
    let wan_port = line.contains("GigabitEthernet0/0") || line.contains("GigabitEthernet0/1");
    if !wan_port {
        return false;
    }

    line.contains("switch link down")
        || line.contains("switch link up")
        || line.contains("interface is up")
        || line.contains("interface is down")
        || line.contains("changed \"link\" layer state")
}
