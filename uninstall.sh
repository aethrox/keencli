#!/usr/bin/env bash
# keencli kaldırma script'i — https://github.com/aethrox/keencli
set -euo pipefail

BIN_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/keencli"
DATA_DIR="${HOME}/.local/share/keencli"
BINARY="${BIN_DIR}/keencli"

rule() {
    echo "────────────────────────────────────────────────"
}

usage() {
    cat <<'EOF'

  keencli kaldırma

  Kurulumu adım adım siler. Her öğe için ayrı onay sorulur.
  Varsayılan yanıt hayırdır — Enter veya n ile iptal edilir.

  Kullanım:
    bash uninstall.sh

  curl ile:
    curl -fsSL .../uninstall.sh -o uninstall.sh
    bash uninstall.sh

EOF
}

confirm() {
    local answer
    read -r -p "  Kaldırılsın mı? [y/N] " answer
    [[ "${answer,,}" == "y" || "${answer,,}" == "yes" ]]
}

ask_remove() {
    local title="$1"
    local detail="$2"
    local path="$3"

    echo ""
    rule
    echo "  $title"
    echo ""
    echo "  $detail"
    echo "  Konum: $path"
    echo ""

    if confirm; then
        echo "  >> Kaldırılıyor..."
        return 0
    fi
    echo "  >> Korundu."
    return 1
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
    exit 0
fi

if [[ $# -gt 0 ]]; then
    echo ""
    echo "  Bu script yalnızca etkileşimli çalışır; bayrak kabul etmez."
    usage
    exit 1
fi

if [[ ! -t 0 ]]; then
    echo ""
    echo "  Hata: Etkileşimli terminal gerekli."
    echo ""
    echo "  Önce indir, sonra çalıştır:"
    echo "    curl -fsSL https://raw.githubusercontent.com/aethrox/keencli/main/uninstall.sh -o uninstall.sh"
    echo "    bash uninstall.sh"
    echo ""
    exit 1
fi

echo ""
echo "  keencli kaldırma"
echo ""
echo "  Her adım için ayrı onay sorulur."
echo "  Enter veya n = silme yok."
echo ""

found=0
[[ -x "$BINARY" || -f "$BINARY" ]] && found=1
[[ -d "$CONFIG_DIR" ]] && found=1
[[ -d "$DATA_DIR" ]] && found=1

if [[ "$found" -eq 0 ]]; then
    echo "  Kurulum bulunamadı. Aranan konumlar:"
    echo ""
    echo "    $BINARY"
    echo "    $CONFIG_DIR"
    echo "    $DATA_DIR"
    echo ""
    exit 0
fi

echo "  Bulunan öğeler:"
echo ""
[[ -x "$BINARY" || -f "$BINARY" ]] && echo "    • Binary  $BINARY"
[[ -d "$CONFIG_DIR" ]] && echo "    • Config  $CONFIG_DIR"
[[ -d "$DATA_DIR" ]] && echo "    • Veri    $DATA_DIR"

removed=0

if [[ -x "$BINARY" || -f "$BINARY" ]]; then
    if ask_remove \
        "Binary" \
        "keencli komut satırı aracı." \
        "$BINARY"; then
        rm -f "$BINARY"
        removed=1
    fi
fi

if [[ -d "$CONFIG_DIR" ]]; then
    if ask_remove \
        "Config" \
        "Router ayarları, .env ve şifre burada saklanır." \
        "$CONFIG_DIR"; then
        rm -rf "$CONFIG_DIR"
        removed=1
    fi
fi

if [[ -d "$DATA_DIR" ]]; then
    if ask_remove \
        "Veri" \
        "Fetch çıktıları, analiz raporları ve kaynak kod." \
        "$DATA_DIR"; then
        rm -rf "$DATA_DIR"
        removed=1
    fi
fi

echo ""
rule
if [[ "$removed" -eq 1 ]]; then
    echo "  Seçilen öğeler kaldırıldı."
else
    echo "  Hiçbir öğe kaldırılmadı."
fi
rule
echo ""

echo "  Not: Shell profiline PATH eklediysen elle kaldırabilirsin:"
echo '    export PATH="$HOME/.local/bin:$PATH"'
echo ""
echo "  Yeniden kurulum:"
echo "    curl -fsSL https://raw.githubusercontent.com/aethrox/keencli/main/install.sh | bash"
echo ""
