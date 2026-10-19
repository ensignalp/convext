import os
import shutil
import socket
import subprocess
import gi

try:
    gi.require_version("Nautilus", "4.1")
except ValueError:
    try:
        gi.require_version("Nautilus", "4.0")
    except ValueError:
        pass

try:
    gi.require_version("Gtk", "4.0")
except ValueError:
    pass

from gi.repository import Nautilus, GObject, Gtk, GLib


def enable_cascading_dropdowns_on_widget(widget) -> None:
    """
    Walks the GTK4 widget tree inside Nautilus and sets Gtk.PopoverMenuFlags.NESTED
    on every Gtk.PopoverMenu so submenus cascade out as separate dropdown popovers
    instead of replacing the menu page in-place.
    """
    if widget is None:
        return
    try:
        if isinstance(widget, Gtk.PopoverMenu):
            if widget.get_flags() != Gtk.PopoverMenuFlags.NESTED:
                widget.set_flags(Gtk.PopoverMenuFlags.NESTED)
        child = widget.get_first_child()
        while child is not None:
            enable_cascading_dropdowns_on_widget(child)
            child = child.get_next_sibling()
    except Exception:
        pass


def patch_all_nautilus_menus() -> bool:
    try:
        for toplevel in Gtk.Window.list_toplevels():
            enable_cascading_dropdowns_on_widget(toplevel)
    except Exception:
        pass
    return False


def list_kdeconnect_devices() -> list[tuple[str, str]]:
    devices = []
    try:
        out = subprocess.check_output(
            ["kdeconnect-cli", "-a", "--id-name-only"],
            stderr=subprocess.DEVNULL,
            timeout=0.8,
        ).decode("utf-8", errors="ignore")
        for line in out.splitlines():
            s = line.strip()
            if " " in s:
                dev_id, dev_name = s.split(" ", 1)
                devices.append((dev_id.strip(), dev_name.strip()))
    except Exception:
        pass
    return devices


def resolve_daemon_binary() -> str:
    candidates = [
        os.path.expanduser("~/.local/bin/convext"),
        shutil.which("convext") or "",
        "/usr/local/bin/convext",
        "/home/mustazafi/YAZILIM/convext/target/release/convext",
        "/home/mustazafi/YAZILIM/citrus-daemon/target/release/convext",
    ]
    for c in candidates:
        if c and os.path.exists(c) and os.access(c, os.X_OK):
            return c
    return os.path.expanduser("~/.local/bin/convext")


IMAGE_EXTS = {"png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff", "tif", "svg", "heic", "heif", "avif", "ico"}
VIDEO_EXTS = {"mp4", "mov", "mkv", "webm", "avi", "wmv", "flv", "m4v"}
AUDIO_EXTS = {"mp3", "wav", "flac", "m4a", "aac", "ogg", "opus", "aiff", "wma"}
PDF_EXTS = {"pdf"}
DOC_EXTS = {"docx", "doc", "odt", "pages", "md", "markdown", "txt", "rtf", "html", "htm", "epub"}
SHEET_EXTS = {"csv", "xlsx", "xls", "ods", "numbers", "tsv"}
SLIDES_EXTS = {"pptx", "ppt", "odp", "key"}
SUB_EXTS = {"srt", "vtt", "ass", "sub"}
ARCHIVE_EXTS = {"zip", "tar", "gz", "tgz", "7z", "rar", "bz2", "xz"}


def detect_category(path: str) -> str:
    lower = path.lower()
    if lower.endswith(".tar.gz") or lower.endswith(".tar.xz") or lower.endswith(".tar.bz2"):
        return "archive"
    ext = os.path.splitext(lower)[1].lstrip(".")
    if ext in IMAGE_EXTS:
        return "image"
    if ext in VIDEO_EXTS:
        return "video"
    if ext in AUDIO_EXTS:
        return "audio"
    if ext in PDF_EXTS:
        return "pdf"
    if ext in DOC_EXTS:
        return "doc"
    if ext in SHEET_EXTS:
        return "sheet"
    if ext in SLIDES_EXTS:
        return "slides"
    if ext in SUB_EXTS:
        return "sub"
    if ext in ARCHIVE_EXTS:
        return "archive"
    return "generic"


def get_ext(path: str) -> str:
    lower = path.lower()
    if lower.endswith(".tar.gz"):
        return "gzip"
    return os.path.splitext(lower)[1].lstrip(".")


def send_convext_command(action_code: str, paths: list[str]) -> None:
    runtime_dir = os.environ.get("XDG_RUNTIME_DIR", "/tmp")
    sock_path = os.path.join(runtime_dir, "convext-daemon.sock")
    payload = "CONVEXT:" + action_code + "\t" + "\t".join(paths) + "\n"

    try:
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
            s.settimeout(1.5)
            s.connect(sock_path)
            s.sendall(payload.encode("utf-8"))
            return
    except Exception:
        pass

    bin_path = resolve_daemon_binary()
    if os.path.exists(bin_path):
        subprocess.Popen(
            [bin_path, "--convext", action_code] + paths,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
        )


class ConvextMenuExtension(GObject.GObject, Nautilus.MenuProvider):
    def __init__(self):
        super().__init__()
        self._counter = 0
        GLib.idle_add(patch_all_nautilus_menus)

    def _make_item(self, label: str, action_code: str, paths: list[str], tip: str = "") -> Nautilus.MenuItem:
        self._counter += 1
        safe_id = f"Convext::Item_{self._counter}_{action_code.replace(':', '_')}"
        item = Nautilus.MenuItem(name=safe_id, label=label, tip=tip or label)
        item.connect("activate", self._on_activate, action_code, paths)
        return item

    def _make_submenu(self, label: str) -> tuple[Nautilus.MenuItem, Nautilus.Menu]:
        self._counter += 1
        parent = Nautilus.MenuItem(
            name=f"Convext::Sub_{self._counter}",
            label=label,
            tip=label,
        )
        sub = Nautilus.Menu()
        parent.set_submenu(sub)
        return parent, sub

    def _on_activate(self, _menu_item, action_code: str, paths: list[str]):
        send_convext_command(action_code, paths)

    def get_file_items(self, *args):
        # Ensure all Nautilus context menu popovers use Gtk.PopoverMenuFlags.NESTED
        # so every submenu opens in its own separate cascading dropdown!
        patch_all_nautilus_menus()
        GLib.idle_add(patch_all_nautilus_menus)

        files = args[-1] if args else []
        if not files:
            return []

        paths = []
        for f in files:
            try:
                loc = f.get_location()
                if loc is not None:
                    p = loc.get_path()
                    if p:
                        paths.append(p)
            except Exception:
                pass

        if not paths:
            return []

        first_path = paths[0]
        cat = detect_category(first_path)
        curr_ext = get_ext(first_path)

        # Single top-level "Convext Menü" item in the right-click context menu.
        # Because Gtk.PopoverMenuFlags.NESTED is enabled, hovering "Convext Menü"
        # opens a separate cascading dropdown, and hovering any category inside it
        # opens another separate cascading dropdown!
        self._counter += 1
        root_item = Nautilus.MenuItem(
            name=f"Convext::RootMenu_{self._counter}",
            label="Convext Menü",
            tip="Convext Dönüştürme, Medya ve Arşiv Menüsü",
        )
        root_menu = Nautilus.Menu()
        root_item.set_submenu(root_menu)

        # 1. ARCHIVE FILES
        if cat == "archive":
            root_menu.append_item(self._make_item("Klasöre Çıkart (Extract Here)", "ARC_EXTRACT", paths))
            conv_parent, conv_sub = self._make_submenu("Arşivi Dönüştür (Archive Convert)")
            for fmt, lbl in [
                ("zip", "ZIP Arşivine Dönüştür (.zip)"),
                ("tar", "TAR Arşivine Dönüştür (.tar)"),
                ("gzip", "GZIP Arşivine Dönüştür (.tar.gz)"),
            ]:
                if fmt != curr_ext:
                    conv_sub.append_item(self._make_item(lbl, f"ARC_CONVERT:{fmt}", paths))
            root_menu.append_item(conv_parent)

        # 2. IMAGE FILES
        elif cat == "image":
            conv_parent, conv_sub = self._make_submenu("Format Dönüştür (Convert Image)")
            for fmt, lbl in [
                ("png", "PNG Formatına Dönüştür (.png)"),
                ("jpg", "JPG Formatına Dönüştür (.jpg)"),
                ("webp", "WEBP Formatına Dönüştür (.webp)"),
                ("avif", "AVIF Formatına Dönüştür (.avif)"),
                ("heic", "HEIC Formatına Dönüştür (.heic)"),
                ("gif", "GIF Formatına Dönüştür (.gif)"),
                ("bmp", "BMP Formatına Dönüştür (.bmp)"),
                ("tiff", "TIFF Formatına Dönüştür (.tiff)"),
                ("ico", "ICO İkona Dönüştür (.ico)"),
                ("pdf", "PDF Belgesine Dönüştür (.pdf)"),
            ]:
                if fmt != curr_ext:
                    conv_sub.append_item(self._make_item(lbl, f"IMG_CONVERT:{fmt}", paths))
            root_menu.append_item(conv_parent)

            tools_parent, tools_sub = self._make_submenu("Görsel Araçları (Image Tools)")
            tools_sub.append_item(self._make_item("Sıkıştır — Yüksek Kalite (%85)", "IMG_COMPRESS_85", paths))
            tools_sub.append_item(self._make_item("Sıkıştır — Dengeli (%65)", "IMG_COMPRESS_65", paths))
            tools_sub.append_item(self._make_item("Sıkıştır — Maksimum Küçültme (%40)", "IMG_COMPRESS_40", paths))
            tools_sub.append_item(self._make_item("Hedef Boyuta Sıkıştır (MB / KB)...", "IMG_TARGET_SIZE", paths))
            tools_sub.append_item(self._make_item("Görsel Kırp & Sansürle (Crop / Redact)...", "IMG_EDITOR", paths))
            tools_sub.append_item(self._make_item("Koyu Arka Plan Çerçevesi Ekle", "IMG_ADD_BG", paths))
            if len(paths) > 1:
                tools_sub.append_item(self._make_item("Seçili Görselleri Tek PDF Yap", "IMG_COLLAGE_PDF", paths))
            tools_sub.append_item(self._make_item("QR Kodu Oku ve Panoya Kopyala", "IMG_QR_DECODE", paths))
            tools_sub.append_item(self._make_item("EXIF / Metadata Temizle", "STRIP_METADATA", paths))
            root_menu.append_item(tools_parent)

        # 3. VIDEO FILES
        elif cat == "video":
            conv_parent, conv_sub = self._make_submenu("Format Dönüştür (Convert Video)")
            for fmt, lbl in [
                ("mp4", "MP4 Formatına Dönüştür (.mp4)"),
                ("mov", "MOV Formatına Dönüştür (.mov)"),
                ("mkv", "MKV Formatına Dönüştür (.mkv)"),
                ("webm", "WEBM Formatına Dönüştür (.webm)"),
                ("avi", "AVI Formatına Dönüştür (.avi)"),
                ("gif", "GIF Animasyona Dönüştür (.gif)"),
                ("mp3", "Sesi Ayıkla — MP3 (.mp3)"),
            ]:
                if fmt != curr_ext:
                    conv_sub.append_item(self._make_item(lbl, f"VID_CONVERT:{fmt}", paths))
            root_menu.append_item(conv_parent)

            tools_parent, tools_sub = self._make_submenu("Video Araçları (Video Tools)")
            tools_sub.append_item(self._make_item("Videoyu Hızlı Sıkıştır (Compress Video)", "VID_QUICK_COMPRESS", paths))
            tools_sub.append_item(self._make_item("Hedef Boyuta Sıkıştır (MB / KB)...", "VID_TARGET_SIZE", paths))
            tools_sub.append_item(self._make_item("Videoları Birleştir (Join Videos)", "VID_JOIN", paths))
            tools_sub.append_item(self._make_item("Videoyu Kırp / Kes (Trim Video)...", "VID_TRIM", paths))
            tools_sub.append_item(self._make_item("Videoyu İkiye Böl (Split Video)", "VID_SPLIT", paths))
            tools_sub.append_item(self._make_item("Videoyu Kadrajla (Crop Video)", "VID_CROP", paths))
            tools_sub.append_item(self._make_item("Oynatma Hızını Değiştir (Playback Speed)...", "VID_TRIM", paths))
            tools_sub.append_item(self._make_item("Sesi Kaldır / Sessize Al (Remove Audio)", "VID_MUTE", paths))
            tools_sub.append_item(self._make_item("Videodan Kare Yakala (Save Frame PNG)", "VID_SNAPSHOT", paths))
            tools_sub.append_item(self._make_item("Videoyu Sansürle (Redact Video)", "VID_REDACT", paths))
            tools_sub.append_item(self._make_item("Metadata Bilgilerini Temizle (Edit Metadata)", "STRIP_METADATA", paths))
            root_menu.append_item(tools_parent)

        # 4. AUDIO FILES
        elif cat == "audio":
            conv_parent, conv_sub = self._make_submenu("Format Dönüştür (Convert Audio)")
            for fmt, lbl in [
                ("mp3", "MP3 Formatına Dönüştür (.mp3)"),
                ("wav", "WAV Formatına Dönüştür (.wav)"),
                ("flac", "FLAC Formatına Dönüştür (.flac)"),
                ("m4a", "M4A Formatına Dönüştür (.m4a)"),
                ("aac", "AAC Formatına Dönüştür (.aac)"),
                ("ogg", "OGG Formatına Dönüştür (.ogg)"),
                ("opus", "OPUS Formatına Dönüştür (.opus)"),
                ("aiff", "AIFF Formatına Dönüştür (.aiff)"),
            ]:
                if fmt != curr_ext:
                    conv_sub.append_item(self._make_item(lbl, f"AUD_CONVERT:{fmt}", paths))
            root_menu.append_item(conv_parent)

            tools_parent, tools_sub = self._make_submenu("Ses Araçları (Audio Tools)")
            tools_sub.append_item(self._make_item("Sesi Sıkıştır (Compress Audio)", "AUD_COMPRESS", paths))
            tools_sub.append_item(self._make_item("Sesi Kırp & Hız Değiştir (Trim & Speed)...", "AUD_TRIM", paths))
            tools_sub.append_item(self._make_item("Ses Seviyesini Dengele (EBU R128 Normalize)", "AUD_NORMALIZE", paths))
            tools_sub.append_item(self._make_item("Ses Dalga Videosu Oluştur (Waveform MP4)", "AUD_VISUALIZE", paths))
            tools_sub.append_item(self._make_item("Mono / Stereo Kanal Değiştir", "AUD_CHANNELS", paths))
            tools_sub.append_item(self._make_item("Orta Bölümü Sansürle (Bleep)", "AUD_BLEEP", paths))
            tools_sub.append_item(self._make_item("Metadata Bilgilerini Temizle", "STRIP_METADATA", paths))
            root_menu.append_item(tools_parent)

        # 5. PDF FILES
        elif cat == "pdf":
            conv_parent, conv_sub = self._make_submenu("Format Dönüştür (Convert PDF)")
            for fmt, lbl in [
                ("docx", "Word Belgesine Dönüştür (.docx)"),
                ("pages", "Apple Pages'e Dönüştür (.pages)"),
                ("txt", "Düz Metne Dönüştür (.txt)"),
                ("md", "Markdown'a Dönüştür (.md)"),
                ("html", "HTML Sayfasına Dönüştür (.html)"),
                ("epub", "EPUB E-Kitaba Dönüştür (.epub)"),
                ("png", "Sayfaları PNG Görsele Dönüştür (.png)"),
                ("jpg", "Sayfaları JPG Görsele Dönüştür (.jpg)"),
            ]:
                conv_sub.append_item(self._make_item(lbl, f"PDF_CONVERT:{fmt}", paths))
            root_menu.append_item(conv_parent)

            tools_parent, tools_sub = self._make_submenu("PDF Araçları (PDF Tools)")
            tools_sub.append_item(self._make_item("PDF Sıkıştır (Compress PDF)", "PDF_COMPRESS", paths))
            tools_sub.append_item(self._make_item("PDF Sayfalarını Tek Tek Ayır (Split PDF)", "PDF_SPLIT", paths))
            if len(paths) > 1:
                tools_sub.append_item(self._make_item("Seçili PDF'leri Birleştir (Merge PDFs)", "PDF_MERGE", paths))
            tools_sub.append_item(self._make_item("PDF Yapısını Optimize Et (Organize PDF)", "PDF_ORGANIZE", paths))
            tools_sub.append_item(self._make_item("PDF Metadata Temizle", "STRIP_METADATA", paths))
            root_menu.append_item(tools_parent)

        # 6. DOCUMENTS & E-BOOKS
        elif cat == "doc":
            conv_parent, conv_sub = self._make_submenu("Format Dönüştür (Convert Document)")
            for fmt, lbl in [
                ("pdf", "PDF Belgesine Dönüştür (.pdf)"),
                ("docx", "Word Belgesine Dönüştür (.docx)"),
                ("odt", "LibreOffice ODT'ye Dönüştür (.odt)"),
                ("pages", "Pages Formatına Dönüştür (.pages)"),
                ("epub", "EPUB E-Kitaba Dönüştür (.epub)"),
                ("md", "Markdown'a Dönüştür (.md)"),
                ("txt", "Düz Metne Dönüştür (.txt)"),
                ("html", "HTML Sayfasına Dönüştür (.html)"),
                ("png", "PNG Görsele Dönüştür (.png)"),
                ("jpg", "JPG Görsele Dönüştür (.jpg)"),
            ]:
                if fmt != curr_ext:
                    conv_sub.append_item(self._make_item(lbl, f"DOC_CONVERT:{fmt}", paths))
            root_menu.append_item(conv_parent)

        # 7. SPREADSHEETS
        elif cat == "sheet":
            conv_parent, conv_sub = self._make_submenu("Tablo Dönüştür (Convert Spreadsheet)")
            for fmt, lbl in [
                ("xlsx", "Excel Tablosuna Dönüştür (.xlsx)"),
                ("csv", "CSV Tablosuna Dönüştür (.csv)"),
                ("numbers", "Numbers Tablosuna Dönüştür (.numbers)"),
                ("pdf", "PDF Belgesine Dönüştür (.pdf)"),
                ("html", "HTML Tablosuna Dönüştür (.html)"),
                ("md", "Markdown Tablosuna Dönüştür (.md)"),
            ]:
                if fmt != curr_ext:
                    conv_sub.append_item(self._make_item(lbl, f"SHEET_CONVERT:{fmt}", paths))
            root_menu.append_item(conv_parent)

        # 8. SLIDES / PRESENTATIONS
        elif cat == "slides":
            conv_parent, conv_sub = self._make_submenu("Sunum Dönüştür (Convert Slides)")
            for fmt, lbl in [
                ("pdf", "PDF Belgesine Dönüştür (.pdf)"),
                ("pptx", "PowerPoint'e Dönüştür (.pptx)"),
                ("key", "Keynote'a Dönüştür (.key)"),
                ("png", "Slaytları PNG Görsele Dönüştür (.png)"),
                ("jpg", "Slaytları JPG Görsele Dönüştür (.jpg)"),
            ]:
                if fmt != curr_ext:
                    conv_sub.append_item(self._make_item(lbl, f"SLIDES_CONVERT:{fmt}", paths))
            root_menu.append_item(conv_parent)

        # 9. SUBTITLES
        elif cat == "sub":
            conv_parent, conv_sub = self._make_submenu("Altyazı Dönüştür (Convert Subtitles)")
            for fmt, lbl in [
                ("vtt", "WebVTT Formatına Dönüştür (.vtt)"),
                ("srt", "SubRip Formatına Dönüştür (.srt)"),
                ("txt", "Düz Metne Dönüştür (.txt)"),
            ]:
                if fmt != curr_ext:
                    conv_sub.append_item(self._make_item(lbl, f"SUB_CONVERT:{fmt}", paths))
            root_menu.append_item(conv_parent)

        # ARCHIVE CREATION SUBMENU (for non-archive files)
        if cat != "archive":
            arc_parent, arc_sub = self._make_submenu("Arşiv Araçları (Archive Tools)")
            arc_sub.append_item(self._make_item("ZIP Arşivi Oluştur (.zip)", "ARC_CREATE_ZIP", paths))
            arc_sub.append_item(self._make_item("TAR Arşivi Oluştur (.tar)", "ARC_CREATE_TAR", paths))
            arc_sub.append_item(self._make_item("GZIP Arşivi Oluştur (.tar.gz)", "ARC_CREATE_GZIP", paths))
            root_menu.append_item(arc_parent)

        # SHARE & SEND SUBMENU (KDE Connect + LocalSend)
        share_parent, share_sub = self._make_submenu("Cihaza Gönder (KDE Connect & LocalSend)")
        share_sub.append_item(
            self._make_item(
                "Hızlı Gönder (KDE Connect / LocalSend Otomatik)",
                "ACT_AIRDROP",
                paths,
            )
        )
        for dev_id, dev_name in list_kdeconnect_devices():
            share_sub.append_item(
                self._make_item(
                    f"KDE Connect: {dev_name} Cihazına Gönder",
                    f"ACT_KDECONNECT_DEV:{dev_id}",
                    paths,
                )
            )
        share_sub.append_item(
            self._make_item("KDE Connect Cihaz Seçici ile Gönder...", "ACT_KDECONNECT", paths)
        )
        share_sub.append_item(
            self._make_item("LocalSend ile Gönder", "ACT_LOCALSEND", paths)
        )
        root_menu.append_item(share_parent)

        # FILE ACTIONS SUBMENU
        act_parent, act_sub = self._make_submenu("Dosya İşlemleri (File Actions)")
        act_sub.append_item(self._make_item("Toplu Yeniden Adlandır (Batch Rename)...", "ACT_BATCH_RENAME", paths))
        act_sub.append_item(self._make_item("Float Rafına Ekle (Floating Shelf)", "ACT_FLOAT", paths))
        act_sub.append_item(self._make_item("Dosyayı Çoğalt (Duplicate)", "ACT_COPY_FILES", paths))
        act_sub.append_item(self._make_item("Convext_Organized Klasörüne Taşı", "ACT_MOVE_ORGANIZED", paths))
        act_sub.append_item(self._make_item("Tam Dosya Yolunu Kopyala (Copy Path)", "ACT_COPY_PATH", paths))
        act_sub.append_item(self._make_item("Burada Terminal Aç", "ACT_TERMINAL", paths))
        root_menu.append_item(act_parent)

        return [root_item]
