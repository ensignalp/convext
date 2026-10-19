#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# Convext Menü — Tek Komutla Otomatik Kurulum Betiği (Linux)
# ==============================================================================

REPO_URL="${CONVEXT_REPO_URL:-https://github.com/mustazafi/convext.git}"
INSTALL_SRC_DIR="${HOME}/.local/share/convext-src"
BIN_DIR="${HOME}/.local/bin"
NAUTILUS_EXT_DIR="${HOME}/.local/share/nautilus-python/extensions"
SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"
AUTOSTART_DIR="${HOME}/.config/autostart"

echo "================================================================"
echo "   🚀 Convext Menü — Otomatik Kurulum Başlatılıyor              "
echo "================================================================"

# 1. Sistem Paket Yöneticisini Algıla ve Bağımlılıkları Kur
install_system_dependencies() {
    echo "[1/6] Sistem paketleri ve bağımlılıklar kontrol ediliyor..."
    if command -v dnf >/dev/null 2>&1; then
        sudo dnf install -y \
            gcc pkg-config git curl \
            gtk4-devel cairo-devel cairo-gobject-devel pango-devel gdk-pixbuf2-devel glib2-devel libX11-devel \
            nautilus-python ffmpeg-free qpdf pandoc poppler-utils perl-Image-ExifTool p7zip p7zip-plugins wl-clipboard xclip || true
    elif command -v apt-get >/dev/null 2>&1; then
        sudo apt-get update -y
        sudo apt-get install -y \
            build-essential pkg-config git curl \
            libgtk-4-dev libcairo2-dev libpango1.0-dev libgdk-pixbuf-2.0-dev libglib2.0-dev libx11-dev \
            python3-nautilus ffmpeg qpdf pandoc poppler-utils libimage-exiftool-perl p7zip-full wl-clipboard xclip || true
    elif command -v pacman >/dev/null 2>&1; then
        sudo pacman -Sy --needed --noconfirm \
            base-devel pkgconf git curl \
            gtk4 cairo pango gdk-pixbuf2 glib2 libx11 \
            nautilus-python ffmpeg qpdf pandoc poppler perl-image-exiftool p7zip wl-clipboard xclip || true
    elif command -v zypper >/dev/null 2>&1; then
        sudo zypper install -y \
            gcc pkg-config git curl \
            gtk4-devel cairo-devel pango-devel gdk-pixbuf-devel glib2-devel libX11-devel \
            python-nautilus ffmpeg qpdf pandoc poppler-tools exiftool p7zip wl-clipboard xclip || true
    else
        echo "  [!] Paket yöneticisi otomatik algılanamadı, mevcut sistem kütüphaneleri kullanılacak."
    fi
}

# 2. Rust / Cargo Kontrolü ve Kurulumu
ensure_rust_toolchain() {
    echo "[2/6] Rust derleme ortamı (cargo) kontrol ediliyor..."
    if ! command -v cargo >/dev/null 2>&1; then
        if [ -f "${HOME}/.cargo/env" ]; then
            # shellcheck disable=SC1091
            source "${HOME}/.cargo/env"
        fi
    fi

    if ! command -v cargo >/dev/null 2>&1; then
        echo "  -> Rust bulunamadı, rustup ile otomatik kuruluyor..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # shellcheck disable=SC1091
        source "${HOME}/.cargo/env"
    fi
}

# 3. Kaynak Kod Dizinini Belirle (Yerel Dizin veya Git Clone)
prepare_source_directory() {
    echo "[3/6] Kaynak kod hazırlanıyor..."
    if [ -f "./Cargo.toml" ] && [ -f "./assets/convext_menu.py" ]; then
        WORK_DIR="$(pwd)"
        echo "  -> Yerel proje dizini algılandı: ${WORK_DIR}"
    else
        echo "  -> Proje GitHub üzerinden indiriliyor: ${REPO_URL}"
        rm -rf "${INSTALL_SRC_DIR}"
        git clone --depth 1 "${REPO_URL}" "${INSTALL_SRC_DIR}"
        WORK_DIR="${INSTALL_SRC_DIR}"
    fi
}

# 4. Release Derlemesi ve Binary Kurulumu
build_and_install_binary() {
    echo "[4/6] Convext motoru (citrus-daemon) derleniyor (Release)..."
    mkdir -p "${BIN_DIR}"
    (
        cd "${WORK_DIR}"
        cargo build --release
    )
    pkill -x "citrus-daemon" >/dev/null 2>&1 || true
    cp "${WORK_DIR}/target/release/citrus-daemon" "${BIN_DIR}/citrus-daemon"
    chmod +x "${BIN_DIR}/citrus-daemon"
    echo "  -> Binary kuruldu: ${BIN_DIR}/citrus-daemon"
}

# 5. Nautilus Sağ Tık Eklentisi Kurulumu
install_nautilus_extension() {
    echo "[5/6] 'Convext Menü' sağ tık eklentisi kuruluyor..."
    mkdir -p "${NAUTILUS_EXT_DIR}"
    rm -f "${NAUTILUS_EXT_DIR}/citrus_selection_bridge.py"
    cp "${WORK_DIR}/assets/convext_menu.py" "${NAUTILUS_EXT_DIR}/convext_menu.py"
    echo "  -> Eklenti kuruldu: ${NAUTILUS_EXT_DIR}/convext_menu.py"
}

# 6. Arka Plan Servisi (systemd --user & Autostart) ve Nautilus Yenileme
configure_services_and_start() {
    echo "[6/6] Arka plan servisi etkinleştiriliyor ve Nautilus yenileniyor..."
    mkdir -p "${SYSTEMD_USER_DIR}" "${AUTOSTART_DIR}"

    cat > "${SYSTEMD_USER_DIR}/convext.service" <<EOF
[Unit]
Description=Convext Menu Background Media & File Engine
After=graphical-session.target

[Service]
Type=simple
ExecStart=${BIN_DIR}/citrus-daemon
Restart=on-failure
RestartSec=2

[Install]
WantedBy=default.target
EOF

    cat > "${AUTOSTART_DIR}/convext-daemon.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Convext Daemon
Comment=Convext Right-Click Context Menu Background Engine
Exec=${BIN_DIR}/citrus-daemon
Terminal=false
X-GNOME-Autostart-enabled=true
EOF

    if command -v systemctl >/dev/null 2>&1; then
        systemctl --user daemon-reload >/dev/null 2>&1 || true
        systemctl --user enable --now convext.service >/dev/null 2>&1 || true
    fi

    if ! pgrep -x "citrus-daemon" >/dev/null 2>&1; then
        setsid "${BIN_DIR}/citrus-daemon" >/tmp/citrus-daemon.log 2>&1 < /dev/null &
    fi

    if command -v nautilus >/dev/null 2>&1; then
        nautilus -q >/dev/null 2>&1 || true
    fi
}

install_system_dependencies
ensure_rust_toolchain
prepare_source_directory
build_and_install_binary
install_nautilus_extension
configure_services_and_start

echo ""
echo "================================================================"
echo " ✅ KURULUM TAMAMLANDI! Convext Menü Kullanıma Hazır."
echo "================================================================"
echo " • Dosya yöneticisinde (Nautilus) herhangi bir dosyaya veya"
echo "   çoklu dosya seçimine SAĞ TIKLAYIP 'Convext Menü'yü seçin."
echo " • Kaldırmak için: ./uninstall.sh"
echo "================================================================"
