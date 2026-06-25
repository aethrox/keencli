#!/usr/bin/env bash
# keencli kurulum script'i — https://github.com/aethrox/keencli
set -euo pipefail

REPO_URL="https://github.com/aethrox/keencli.git"
BIN_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/keencli"
DATA_DIR="${HOME}/.local/share/keencli"
SRC_DIR="${KEENETIC_SRC_DIR:-${DATA_DIR}/src}"

echo "==> keencli kurulumu"
echo ""

if ! command -v cargo >/dev/null 2>&1; then
    echo "Hata: cargo bulunamadı."
    echo "Rust kur: https://rustup.rs"
    exit 1
fi

if ! command -v git >/dev/null 2>&1; then
    echo "Hata: git bulunamadı."
    exit 1
fi

mkdir -p "$BIN_DIR" "$CONFIG_DIR" "$DATA_DIR/outputs"

if [[ -d "$SRC_DIR/.git" ]]; then
    echo "==> Kaynak güncelleniyor: $SRC_DIR"
    git -C "$SRC_DIR" pull --ff-only
else
    echo "==> Kaynak indiriliyor: $SRC_DIR"
    mkdir -p "$(dirname "$SRC_DIR")"
    git clone "$REPO_URL" "$SRC_DIR"
fi

echo "==> Derleniyor (release)..."
cargo build --release --manifest-path "$SRC_DIR/Cargo.toml"

install -m 755 "$SRC_DIR/target/release/keencli" "$BIN_DIR/keencli"

if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
    cp "$SRC_DIR/config.toml.example" "$CONFIG_DIR/config.toml"
    echo "==> Örnek config oluşturuldu: $CONFIG_DIR/config.toml"
fi

if [[ ! -f "$CONFIG_DIR/.env.example" ]]; then
    cp "$SRC_DIR/.env.example" "$CONFIG_DIR/.env.example"
fi

echo ""
echo "============================================"
echo "  Kurulum tamamlandı"
echo "============================================"
echo ""
echo "  Binary:  $BIN_DIR/keencli"
echo "  Config:  $CONFIG_DIR/config.toml"
echo "  Şifre:   $CONFIG_DIR/.env  (henüz yoksa oluştur)"
echo "  Veri:    $DATA_DIR/outputs/"
echo ""

if [[ ":${PATH}:" != *":${BIN_DIR}:"* ]]; then
    echo "!! PATH uyarısı"
    echo "   $BIN_DIR şu an PATH'te değil. Shell profiline ekle"
    echo "   (~/.bashrc veya ~/.zshrc), sonra yeni terminal aç:"
    echo ""
    echo '   export PATH="$HOME/.local/bin:$PATH"'
    echo ""
fi

echo "--------------------------------------------"
echo "  Kurulum sonrası (ilk kullanım)"
echo "--------------------------------------------"
echo ""
echo "1) Router bilgilerini ayarla:"
echo "   $EDITOR $CONFIG_DIR/config.toml"
echo "   (ip ve username — şifreyi bu dosyaya YAZMAYIN)"
echo ""
echo "2) Router şifresini kalıcı olarak tanımla:"
echo "   cp $CONFIG_DIR/.env.example $CONFIG_DIR/.env"
echo "   $EDITOR $CONFIG_DIR/.env"
echo ""
echo "   Alternatif (geçici, yalnızca o terminal için):"
echo "   export KEENETIC_PASSWORD='router_şifreniz'"
echo ""
echo "3) Bağlantıyı test et:"
echo "   keencli status"
echo ""
echo "4) Veri çek ve analiz et:"
echo "   keencli fetch -a"
echo "   keencli analyze"
echo ""
echo "Opsiyonel — AI raporu için $CONFIG_DIR/.env dosyasına ekle:"
echo "   OPENROUTER_API_KEY=sk-or-..."
echo "   LLM_MODEL=anthropic/claude-sonnet-4.6"
echo ""
echo "Güncelleme: install.sh script'ini tekrar çalıştır."
echo "Detay: https://github.com/aethrox/keencli#kurulum-sonrası"
echo ""
