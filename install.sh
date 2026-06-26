#!/usr/bin/env bash
# keencli kurulum script'i — https://github.com/aethrox/keencli
set -euo pipefail

REPO_URL="https://github.com/aethrox/keencli.git"
BIN_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/keencli"
DATA_DIR="${HOME}/.local/share/keencli"
SRC_DIR="${KEENETIC_SRC_DIR:-${DATA_DIR}/src}"

rule() {
    echo "────────────────────────────────────────────────"
}

heading() {
    echo ""
    rule
    echo "  $1"
    rule
    echo ""
}

step() {
    printf "  %s) %s\n" "$1" "$2"
}

cmd() {
    echo "     $1"
}

echo ""
echo "  keencli kurulumu"
echo ""

if ! command -v cargo >/dev/null 2>&1; then
    echo "  Hata: cargo bulunamadı."
    echo "  Rust kurulumu: https://rustup.rs"
    exit 1
fi

if ! command -v git >/dev/null 2>&1; then
    echo "  Hata: git bulunamadı."
    exit 1
fi

mkdir -p "$BIN_DIR" "$CONFIG_DIR" "$DATA_DIR/outputs"

if [[ -d "$SRC_DIR/.git" ]]; then
    echo "  >> Kaynak güncelleniyor"
    echo "     $SRC_DIR"
    git -C "$SRC_DIR" pull --ff-only
else
    echo "  >> Kaynak indiriliyor"
    echo "     $SRC_DIR"
    mkdir -p "$(dirname "$SRC_DIR")"
    git clone "$REPO_URL" "$SRC_DIR"
fi

echo ""
echo "  >> Derleniyor (release)..."
cargo build --release --manifest-path "$SRC_DIR/Cargo.toml"

install -m 755 "$SRC_DIR/target/release/keencli" "$BIN_DIR/keencli"

if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
    cp "$SRC_DIR/config.toml.example" "$CONFIG_DIR/config.toml"
    echo "  >> Örnek config oluşturuldu"
    echo "     $CONFIG_DIR/config.toml"
fi

if [[ ! -f "$CONFIG_DIR/.env.example" ]]; then
    cp "$SRC_DIR/.env.example" "$CONFIG_DIR/.env.example"
fi

heading "Kurulum tamamlandı"

echo "  Dosyalar"
echo ""
printf "    %-10s %s\n" "Binary" "$BIN_DIR/keencli"
printf "    %-10s %s\n" "Config" "$CONFIG_DIR/config.toml"
printf "    %-10s %s\n" "Şifre" "$CONFIG_DIR/.env"
printf "    %-10s %s\n" "Veri" "$DATA_DIR/outputs/"
echo ""

if [[ ":${PATH}:" != *":${BIN_DIR}:"* ]]; then
    heading "PATH uyarısı"
    echo "  $BIN_DIR şu an PATH'te değil."
    echo "  ~/.bashrc veya ~/.zshrc dosyana ekle, sonra yeni terminal aç:"
    echo ""
    cmd 'export PATH="$HOME/.local/bin:$PATH"'
    echo ""
fi

heading "İlk kullanım"

step 1 "Router bilgilerini ayarla (şifre yazma)"
cmd "${EDITOR:-nano} $CONFIG_DIR/config.toml"
echo ""

step 2 "Router şifresini kalıcı olarak tanımla"
cmd "cp $CONFIG_DIR/.env.example $CONFIG_DIR/.env"
cmd "${EDITOR:-nano} $CONFIG_DIR/.env"
echo ""
echo "     Geçici alternatif (yalnızca bu terminal):"
cmd "export KEENETIC_PASSWORD='router_şifreniz'"
echo ""

step 3 "Bağlantıyı test et"
cmd "keencli status"
echo ""

step 4 "Veri çek ve analiz et"
cmd "keencli fetch -a"
cmd "keencli analyze"
echo ""

echo "  Opsiyonel — AI raporu için .env dosyasına ekleyin:"
echo ""
cmd "OPENROUTER_API_KEY=sk-or-..."
cmd "LLM_MODEL=anthropic/claude-sonnet-4.6   # önerilen"
echo "     Alternatif: deepseek/deepseek-v4-pro (ekonomik)"
echo "     Test özeti: AI_MODELS.md"
echo ""

heading "Diğer"

echo "  Güncelleme"
cmd "curl -fsSL https://raw.githubusercontent.com/aethrox/keencli/main/install.sh | bash"
echo ""
echo "  Kaldırma"
cmd "curl -fsSL https://raw.githubusercontent.com/aethrox/keencli/main/uninstall.sh -o uninstall.sh"
cmd "bash uninstall.sh"
echo ""
echo "  Belgeler"
cmd "https://github.com/aethrox/keencli#kurulum-sonrası"
echo ""
