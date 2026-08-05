# keencli

A CLI that pulls diagnostic data from Keenetic routers (system, PPPoE, ping, logs, Wi-Fi, mesh), filters and masks it, and optionally turns it into an AI-written diagnostic report.

> [!CAUTION]
> AI-generated reports can be wrong. Verify findings against the raw logs or a human expert before treating a diagnosis as certain or changing anything on the router.

Useful when a Hopper/Giga/Speedster-class Keenetic router has PPPoE drops, WAN issues, or flaky connectivity and you'd rather diagnose from a terminal than the web UI. Output is saved to timestamped folders, so you can analyze it without staying connected to the router.

**Version:** 1.0.6 · **Repo:** https://github.com/aethrox/keencli

## Quick start

```bash
curl -fsSL https://raw.githubusercontent.com/aethrox/keencli/main/install.sh | bash
```

This downloads and builds the source under `~/.local/share/keencli/src`, installs the binary to `~/.local/bin/keencli`, writes a starter config to `~/.config/keencli/config.toml`, and stores fetch output under `~/.local/share/keencli/outputs/`. If `~/.local/bin` isn't on your `PATH`, the script warns you, so add it:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Then set up the router IP/username and password (see Configuration below) and run:

```bash
keencli status
keencli fetch -a
keencli analyze
```

## What it does

| Command | What it collects/does |
|---|---|
| `keencli fetch` | Pulls `system.json` only |
| `keencli fetch -a` | Pulls all data (system, PPPoE, ping, logs, Wi-Fi, mesh), required for `analyze` |
| `keencli analyze` | Filters the log (~3000 lines down to ~60), masks sensitive fields, builds an AI prompt, and writes an AI report if an API key is configured |
| `keencli status` | Live hostname, uptime, and PPPoE status |

Each command has `--help` for the full flag list.

## How it works

```
keencli fetch -a     router -> JSON + log files
       |
       v
keencli analyze      filter -> mask -> prompt -> (optional) AI report
```

1. **fetch**: authenticates with the router over Keenetic's own auth scheme and pulls data from its RCI endpoints.
2. **analyze**: reads the most recent fetch folder, filters the log (boot noise, WAN flaps, DNS churn), and writes `prompt_for_ai.txt`.
3. **AI report**: if `OPENROUTER_API_KEY` is set, the prompt is sent to OpenRouter and the report is saved next to the fetch data. Without a key, `analyze` still runs and just skips this step.

## Configuration

| File | Purpose |
|---|---|
| `~/.config/keencli/config.toml` | Router IP and username |
| `~/.config/keencli/.env` | Router password, AI API key |
| `~/.local/share/keencli/outputs/` | Fetch and analysis output |

**1. Config** (edit `ip` and `username`, never put the password here):

```bash
nano ~/.config/keencli/config.toml
```

**2. Password** (persistent, recommended):

```bash
cp ~/.config/keencli/.env.example ~/.config/keencli/.env
nano ~/.config/keencli/.env
```

```env
KEENETIC_PASSWORD=your_router_password
```

Or set it just for the current shell: `export KEENETIC_PASSWORD='...'`.

**3. AI report (optional)**, add to `.env`:

```env
OPENROUTER_API_KEY=sk-or-...
LLM_MODEL=anthropic/claude-sonnet-4.6   # recommended
LLM_TEMPERATURE=0.3                     # optional, defaults to 0.3
```

See [AI_MODELS.md](AI_MODELS.md) for model recommendations tested against real router data. `anthropic/claude-sonnet-4.6` gives the most complete reports; `deepseek/deepseek-v4-pro` is a cheaper reliable second choice. `openai/gpt-4.1`, `qwen/qwen3-235b-a22b`, `x-ai/grok-4.20`, and `x-ai/grok-4.3` are not recommended for this task: they tend to miss log flaps or deny route loss in testing.

**Update:** re-run `install.sh`.

## Uninstall

Fully interactive, with separate confirmation for the binary, config, and data, defaulting to no on each:

```bash
curl -fsSL https://raw.githubusercontent.com/aethrox/keencli/main/uninstall.sh -o uninstall.sh
bash uninstall.sh
```

This deliberately can't be piped through `curl | bash`; download it and run it locally to avoid accidental deletion.

## Manual / development setup

```bash
git clone https://github.com/aethrox/keencli.git
cd keencli
cp config.toml.example config.toml   # fill in ip and username, not the password
cp .env.example .env                 # KEENETIC_PASSWORD goes here
cargo build --release
./target/release/keencli --help
```

Nix is also supported: `nix develop` (dev shell), `nix build` (package, binary at `./result/bin/keencli`), `nix run` (run via flake app).

## Output files

After `fetch -a`:

```
outputs/2026-06-25_18-15-39/
├── system.json
├── interface_PPPoE0.json
├── pingcheck.json
├── log.txt
├── wifi.json
└── mesh.json
```

`analyze` adds:

```
├── prompt_for_ai.txt
└── ai_report_anthropic-claude-sonnet-4.6.md   # only if an API key is set
```

## Security

- The router password is read only from `KEENETIC_PASSWORD` (env or `.env`); it can't be written into `config.toml` (`deny_unknown_fields` rejects it).
- Output files and the AI prompt have IP/MAC/SSID masked before they're written.
- `.env` and `outputs/` are git-ignored.
- Found a vulnerability? See [SECURITY.md](SECURITY.md) to report it privately through GitHub.

## Project structure

```
src/
├── main.rs         CLI entry point
├── api.rs          Router HTTP client + Keenetic auth
├── config.rs       config.toml parsing
├── paths.rs        XDG config/output paths
├── credentials.rs  Password and API key handling (SecretString)
├── output.rs       Writes masked output files
├── analyze.rs      Reads outputs/, builds the report
├── log_filter.rs   Log filtering (WAN, ping-check, DNS churn)
├── mask.rs         IP/MAC/SSID masking
├── prompt.rs       AI prompt generation
└── llm.rs          OpenRouter client
```

## Built with

| Area | Library |
|---|---|
| CLI | clap |
| HTTP | reqwest (cookies) |
| Async | tokio |
| Config | config + TOML |
| Auth | md5 + sha2 (Keenetic challenge) |
| Secrets | secrecy |
| JSON | serde / serde_json |
| Masking | regex |
| Errors | anyhow |

## Limitations

- Tested against Keenetic Hopper/Giga/Speedster-class routers only; other Keenetic models or firmware versions are untested.
- AI reports are advisory only; they can be wrong or incomplete, and OpenRouter receives the (masked) prompt regardless of which model is chosen. See Disclaimer below.

## Disclaimer

- **Independent project**: keencli is not affiliated with, endorsed by, or an official product of Keenetic or OpenRouter.
- **No warranty**: provided "as is", without warranty of any kind; the author is not liable for damages arising from use.
- **AI output**: reports generated via OpenRouter are advisory only and may contain errors, omissions, or incorrect information; verify independently before changing any network or router setting.
- **Third-party services**: AI analysis sends data to OpenRouter, subject to its own terms; data is transmitted to a third party even after masking.
- **Your responsibility**: keeping router credentials and API keys secure, and complying with local law and service terms, is on you.

## License

MIT, see [LICENSE](LICENSE). The disclaimer above supplements, and does not replace, the MIT license's "as is" terms.
