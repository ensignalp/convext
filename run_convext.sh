#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_PATH="$PROJECT_DIR/target/release/convext"
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp}"
SOCK_PATH="$RUNTIME_DIR/convext-daemon.sock"
DEMO_DIR="$PROJECT_DIR/demo_test_files"
NAUTILUS_EXT_DIR="$HOME/.local/share/nautilus-python/extensions"

prepare_demo_files() {
    mkdir -p "$DEMO_DIR"
    echo "[*] Tüm kategoriler için örnek test dosyaları hazırlanıyor: $DEMO_DIR"

    python3 - "$DEMO_DIR" <<'PYEOF'
import os, sys, zlib, struct, zipfile

demo_dir = sys.argv[1]

def make_png(path, width, height, rgb_fn):
    raw = bytearray()
    for y in range(height):
        raw.append(0)
        for x in range(width):
            r, g, b = rgb_fn(x, y, width, height)
            raw.extend((r, g, b))
    def chunk(tag, data):
        return struct.pack("!I", len(data)) + tag + data + struct.pack("!I", zlib.crc32(tag + data) & 0xffffffff)
    ihdr = struct.pack("!IIBBBBB", width, height, 8, 2, 0, 0, 0)
    idat = zlib.compress(bytes(raw), 9)
    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", idat) + chunk(b"IEND", b""))

# 1. Image (JPG/PNG)
make_png(
    os.path.join(demo_dir, "1_gorsel_ornek.png"),
    640, 400,
    lambda x, y, w, h: (45, int(135 * (x / w)), int(225 * (y / h)))
)

# 2. Document (Markdown / DOCX)
md_path = os.path.join(demo_dir, "2_belge_ornek.md")
with open(md_path, "w", encoding="utf-8") as f:
    f.write("# Convext Menü Test Belgesi\n\nPAGES, PDF, TXT, MD, HTML, EPUB, PNG, JPG donusum testi.\n")

# 3. EPUB
epub_path = os.path.join(demo_dir, "3_kitap_ornek.epub")
with zipfile.ZipFile(epub_path, "w", zipfile.ZIP_DEFLATED) as zf:
    zf.writestr("mimetype", "application/epub+zip")
    zf.writestr("content.html", "<html><body><h1>Convext EPUB</h1><p>EPUB -> PDF, DOCX, PAGES, TXT, MD, HTML</p></body></html>")

# 4. Spreadsheet (CSV)
csv_path = os.path.join(demo_dir, "4_tablo_ornek.csv")
with open(csv_path, "w", encoding="utf-8") as f:
    f.write("Urun,Kategori,Durum\nConvext,Linux,Aktif\nLibadwaita,UI,Hazir\n")

# 5. Subtitles (SRT)
srt_path = os.path.join(demo_dir, "5_altyazi_ornek.srt")
with open(srt_path, "w", encoding="utf-8") as f:
    f.write("1\n00:00:01,000 --> 00:00:03,500\nConvext Sağ Tık Menüsü Aktif!\n\n2\n00:00:04,000 --> 00:00:06,500\nVTT ve TXT donusumu hazir.\n")

# 6. Archive (ZIP)
zip_path = os.path.join(demo_dir, "6_arsiv_ornek.zip")
with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as zf:
    zf.write(md_path, arcname="belge.md")
    zf.write(csv_path, arcname="tablo.csv")

# 7. PDF
pdf_path = os.path.join(demo_dir, "7_dokuman_ornek.pdf")
pdf_bytes = (
    b"%PDF-1.4\n"
    b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n"
    b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n"
    b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 400 250] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n"
    b"4 0 obj\n<< /Length 55 >>\nstream\nBT /F1 18 Tf 40 130 Td (Convext PDF Test) Tj ET\nendstream\nendobj\n"
    b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n"
    b"xref\n0 6\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000241 00000 n \n0000000346 00000 n \n"
    b"trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n414\n%%EOF\n"
)
with open(pdf_path, "wb") as f:
    f.write(pdf_bytes)
PYEOF

    if command -v ffmpeg >/dev/null 2>&1; then
        if [ ! -f "$DEMO_DIR/8_video_ornek.mp4" ]; then
            ffmpeg -y -f lavfi -i testsrc=duration=3:size=640x360:rate=25 \
                   -f lavfi -i sine=frequency=440:duration=3 \
                   -c:v libx264 -pix_fmt yuv420p -c:a aac \
                   "$DEMO_DIR/8_video_ornek.mp4" >/dev/null 2>&1 || true
        fi
        if [ ! -f "$DEMO_DIR/9_ses_ornek.mp3" ]; then
            ffmpeg -y -f lavfi -i sine=frequency=520:duration=3 \
                   "$DEMO_DIR/9_ses_ornek.mp3" >/dev/null 2>&1 || true
        fi
    fi
}

install_convext_context_menu() {
    echo "[*] 'Convext Menü' sağ tık eklentisi kuruluyor..."
    mkdir -p "$NAUTILUS_EXT_DIR" "$HOME/.local/bin"
    rm -f "$NAUTILUS_EXT_DIR/citrus_selection_bridge.py"
    cp "$PROJECT_DIR/assets/convext_menu.py" "$NAUTILUS_EXT_DIR/convext_menu.py"
}

restart_daemon() {
    echo "[*] Convext arka plan motoru (convext) derleniyor ve başlatılıyor..."
    (cd "$PROJECT_DIR" && cargo build --release)

    pkill -x "convext" >/dev/null 2>&1 || true
    pkill -x "citrus-daemon" >/dev/null 2>&1 || true
    rm -f "$SOCK_PATH"
    sleep 0.3

    cp "$BIN_PATH" "$HOME/.local/bin/convext"
    chmod +x "$HOME/.local/bin/convext"

    setsid "$BIN_PATH" >/tmp/convext.log 2>&1 < /dev/null &
    for _ in {1..30}; do
        if [ -S "$SOCK_PATH" ]; then
            break
        fi
        sleep 0.2
    done
    sleep 0.2
    echo "[*] Convext arka plan servisi aktif (GNOME GTK4 + Libadwaita)!"
}

reload_nautilus() {
    if command -v nautilus >/dev/null 2>&1; then
        echo "[*] Dosya yöneticisi (Nautilus) 'Convext Menü' için yenileniyor..."
        nautilus -q >/dev/null 2>&1 || true
        sleep 0.4
        nohup nautilus "$DEMO_DIR" >/dev/null 2>&1 &
    fi
}

echo "========================================================"
echo "   Convext (GNOME Libadwaita & Sağ Tık Sistemi)         "
echo "========================================================"

prepare_demo_files
install_convext_context_menu
restart_daemon
reload_nautilus
