<div align="center">

<img src="assets/logo.svg" alt="Convext Logo" width="148" height="148" />

# ⚡ Convext

### **GNOME / Linux için %100 Çevrimdışı (Offline), Bağlam Duyarlı Sağ Tık Dosya Dönüştürücü, Medya İşleyici ve Libadwaita Araç Stüdyosu**

[![Rust 2024](https://img.shields.io/badge/Rust-2024_Edition_(1.98+)-F74C00?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![GTK4](https://img.shields.io/badge/GTK-4.12+-4A86CF?style=for-the-badge&logo=gtk&logoColor=white)](https://www.gtk.org/)
[![Libadwaita](https://img.shields.io/badge/GNOME-Libadwaita_1.9-3584E4?style=for-the-badge&logo=gnome&logoColor=white)](https://gnome.pages.gitlab.gnome.org/libadwaita/)
[![Tokio](https://img.shields.io/badge/Async_Engine-Tokio_1.40-18181B?style=for-the-badge&logo=rust&logoColor=orange)](https://tokio.rs/)
[![KDE Connect & LocalSend](https://img.shields.io/badge/Share-KDE_Connect_%26_LocalSend-1D99F3?style=for-the-badge&logo=kde&logoColor=white)](#)
[![Offline](https://img.shields.io/badge/Privacy-100%25_Offline-2EA44F?style=for-the-badge)](#)

<p align="center">
  <b>Dosya yöneticisinde sağ tıklayın $\rightarrow$ Dosya türünüze özel kademeli açılır menülerden (Dropdown) formatınızı veya aracınızı seçin $\rightarrow$ Arka planda sessizce tamamlansın!</b>
</p>

---

[🚀 Tek Komutla Kurulum](#-tek-komutla-hızlı-kurulum-one-line-install) •
[✨ Dönüştürücü & Araç Matrisi](#-desteklenen-kategoriler-dönüştürücüler-ve-araçlar) •
[🎨 GNOME Libadwaita & Kademeli Menü](#-gnome-gtk4--libadwaita-arayüz-mimarisi) •
[📲 KDE Connect & LocalSend](#-kde-connect--localsend-ortak-paylaşım-köprüsü) •
[🏗️ Sistem Mimarisi](#️-sistem-mimarisi-ve-iş-akışı) •
[💻 CLI Kullanımı](#-komut-satırı-cli-ve-soket-kullanımı)

</div>

---

## 🌟 Neden Convext?

> [!IMPORTANT]
> **Sıfır Bulut Yüklemesi, Sıfır Bekleme:** Görsellerinizi, videolarınızı, ses kayıtlarınızı, PDF belgelerinizi veya arşivlerinizi dönüştürmek için internete dosya yüklemenize veya karmaşık terminal komutları yazmanıza gerek yok. **Convext**, doğrudan GNOME Dosyalar (**Nautilus**) sağ tık menüsüne ve yerel **GTK4 + Libadwaita** arayüzüne entegre olur.

* **🎯 Akıllı Bağlam Algılama (Context-Aware):** Bir `.png` görseline sağ tıkladığınızda yalnızca görsel dönüştürme/sıkıştırma/sansürleme araçları; bir `.mp4` videosuna sağ tıkladığınızda yalnızca video kırpma/birleştirme/ses ayıklama araçları; bir `.zip` arşivine sağ tıkladığınızda ise arşiv çıkarma ve dönüştürme araçları açılır.
* **📑 Kademeli Açılır Menüler (`Gtk.PopoverMenuFlags.NESTED`):**
  * Nautilus sağ tık menüsünde **`Convext Menü ▸`** üzerine geldiğinizde aynı menü sayfasının içinde kaybolmaz; her alt kategori (**Format Dönüştür ▸**, **Araçlar ▸**, **Arşiv Araçları ▸**, **Cihaza Gönder ▸**, **Dosya İşlemleri ▸**) **kendi ayrı dikey açılır penceresinde (cascading dropdown)** yan yana açılır.
* **🦀 Rust 2024 Edition + Tokio Çoklu İş Parçacığı Motoru:** Tüm ağır FFmpeg, QPDF, Pandoc, LibreOffice ve görüntü işleme görevleri `tokio` multi-threaded çalışma zamanında sessizce yürütülür; işlem bittiğinde masaüstü bildirimi gönderilir ve üretilen dosya otomatik olarak dosya yöneticisinde vurgulanır.

---

## 🚀 Tek Komutla Hızlı Kurulum (One-Line Install)

Sistem bağımlılıklarını (`Fedora`, `Ubuntu/Debian`, `Arch Linux`, `openSUSE`), Rust derleyicisini, `convext` arka plan motorunu, `systemd` kullanıcı servisini ve Nautilus sağ tık eklentisini **tek komutla** indirip kurmak için terminalinize yapıştırın:

```bash
curl -fsSL https://raw.githubusercontent.com/mustazafi/convext/main/install.sh | bash
```

### Alternatif: Git ile Klonlayarak Kurulum

```bash
git clone https://github.com/mustazafi/convext.git
cd convext
chmod +x install.sh
./install.sh
```

> [!TIP]
> Kendi fork/depo adresinizden kurulum yapmak isterseniz `CONVEXT_REPO_URL` ortam değişkenini kullanabilirsiniz:
> ```bash
> CONVEXT_REPO_URL="https://github.com/<KULLANICI_ADINIZ>/convext.git" ./install.sh
> ```

---

## ✨ Desteklenen Kategoriler, Dönüştürücüler ve Araçlar

<div align="center">

| Kategori & Modül | Algılanan Kaynak Uzantılar | 🔄 Hedef Dönüştürme Formatları (`Convert`) | 🛠️ Özel Medya & Belge Araçları (`Specialized Tools`) |
| :---: | :--- | :--- | :--- |
| <img src="https://img.shields.io/badge/🖼️_GÖRSEL-FF7800?style=for-the-badge" alt="Görsel" /><br/><sub><b>Image Engine</b></sub> | `png` `jpg` `jpeg`<br/>`webp` `avif` `heic`<br/>`heif` `gif` `bmp`<br/>`tiff` `svg` `ico` | ![PNG](https://img.shields.io/badge/PNG-242936?style=flat-square&logoColor=white) ![JPG](https://img.shields.io/badge/JPG-242936?style=flat-square) ![WEBP](https://img.shields.io/badge/WEBP-242936?style=flat-square) ![AVIF](https://img.shields.io/badge/AVIF-242936?style=flat-square) ![HEIC](https://img.shields.io/badge/HEIC-242936?style=flat-square)<br/>![GIF](https://img.shields.io/badge/GIF-242936?style=flat-square) ![BMP](https://img.shields.io/badge/BMP-242936?style=flat-square) ![TIFF](https://img.shields.io/badge/TIFF-242936?style=flat-square) ![ICO](https://img.shields.io/badge/ICO-242936?style=flat-square) ![PDF](https://img.shields.io/badge/PDF-E66100?style=flat-square) | ⚡ **Hızlı Sıkıştırma:** `%85 (Yüksek)`, `%65 (Dengeli)`, `%40 (Maks)`<br/>🎯 **Hedef Boyut (`Binary Search`):** Tam `KB` / `MB` boyutuna indirgeme<br/>🕵️ **Kalıcı Sansürleme & Kırpma:** `Pixelate`, `Blackout`, `Crop`<br/>🖼️ **Stüdyo Çerçeve:** Koyu arka plan dolgusu ekleme<br/>📑 **Çoklu Görsel $\rightarrow$ PDF:** Seçili görselleri tek PDF'te birleştirme<br/>📱 **Çevrimdışı QR Okuyucu:** QR içeriğini çözüp panoya kopyalama<br/>🧹 **Gizlilik:** EXIF / GPS / Kamera üst verilerini sıfırlama |
| <img src="https://img.shields.io/badge/🎬_VİDEO-3584E4?style=for-the-badge" alt="Video" /><br/><sub><b>Video Engine</b></sub> | `mp4` `mov` `mkv`<br/>`webm` `avi` `wmv`<br/>`flv` `m4v` | ![MP4](https://img.shields.io/badge/MP4_H.264-242936?style=flat-square) ![MOV](https://img.shields.io/badge/MOV-242936?style=flat-square) ![MKV](https://img.shields.io/badge/MKV-242936?style=flat-square)<br/>![WEBM](https://img.shields.io/badge/WEBM_VP9-242936?style=flat-square) ![AVI](https://img.shields.io/badge/AVI-242936?style=flat-square) ![GIF](https://img.shields.io/badge/GIF_Animasyon-242936?style=flat-square)<br/>![MP3](https://img.shields.io/badge/MP3_Ses_Ayıkla-1C71D8?style=flat-square) | ⚡ **Hızlı Sıkıştırma:** `H.264 CRF 28` tek tıkla küçültme<br/>🎯 **Hedef Boyut (`Bitrate Solver`):** Tam `MB` / `KB` hedef boyutu<br/>🔗 **Video Birleştirici (`Join`):** Çoklu klibi kayıpsız tek MP4 yapma<br/>✂️ **Kırp & Hız Değiştir (`Trim & Speed`):** Başlangıç/bitiş ve `0.5x–2.0x` hız<br/>➗ **İkiye Böl (`Split`):** Videoyu tam ortadan iki ayrı dosyaya bölme<br/>📐 **Kadrajla (`Crop`) & Sessize Al (`Mute`):** Ses kanalını kaldırma<br/>📸 **Kare Yakala (`Snapshot PNG`) & Sansürle (`Redact Video`)** |
| <img src="https://img.shields.io/badge/🎵_SES-9141AC?style=for-the-badge" alt="Ses" /><br/><sub><b>Audio Engine</b></sub> | `mp3` `wav` `flac`<br/>`m4a` `aac` `ogg`<br/>`opus` `aiff` `wma` | ![MP3](https://img.shields.io/badge/MP3-242936?style=flat-square) ![WAV](https://img.shields.io/badge/WAV_PCM-242936?style=flat-square) ![FLAC](https://img.shields.io/badge/FLAC_Lossless-242936?style=flat-square)<br/>![M4A](https://img.shields.io/badge/M4A-242936?style=flat-square) ![AAC](https://img.shields.io/badge/AAC-242936?style=flat-square) ![OGG](https://img.shields.io/badge/OGG-242936?style=flat-square)<br/>![OPUS](https://img.shields.io/badge/OPUS-242936?style=flat-square) ![AIFF](https://img.shields.io/badge/AIFF-242936?style=flat-square) | 🎚️ **EBU R128 Ses Dengeleme:** Stüdyo standardı `-16 LUFS` normalize<br/>✂️ **Ses Kesici & Tempo:** Saniye bazlı kesme ve hızlandırma<br/>📊 **Ses Dalga Videosu (`Waveform MP4`):** Sesten `1280x720` görsel video<br/>🔊 **Kanal Dönüştürücü:** `Mono` $\leftrightarrow$ `Stereo` geçişi<br/>🔕 **Bleep Sansürleme:** Orta kesite `1000Hz` bip tonu uygulama<br/>🗜️ **Ses Sıkıştırma & ID3 Metadata Temizleme** |
| <img src="https://img.shields.io/badge/📄_PDF-E01B24?style=for-the-badge" alt="PDF" /><br/><sub><b>PDF Engine</b></sub> | `pdf` | ![DOCX](https://img.shields.io/badge/DOCX-242936?style=flat-square) ![PAGES](https://img.shields.io/badge/PAGES-242936?style=flat-square) ![TXT](https://img.shields.io/badge/TXT-242936?style=flat-square) ![MD](https://img.shields.io/badge/MD-242936?style=flat-square)<br/>![HTML](https://img.shields.io/badge/HTML-242936?style=flat-square) ![EPUB](https://img.shields.io/badge/EPUB-242936?style=flat-square) ![PNG](https://img.shields.io/badge/PNG_300DPI-242936?style=flat-square) ![JPG](https://img.shields.io/badge/JPG_300DPI-242936?style=flat-square) | 🗜️ **PDF Sıkıştırma:** Akış ve nesne sıkıştırma (`QPDF`)<br/>📑 **Sayfaları Ayır (`Split`):** Her sayfayı ayrı PDF dosyasına bölme<br/>🔗 **PDF Birleştir (`Merge`):** Çoklu PDF seçimini tek belgede birleştirme<br/>⚡ **Web Optimize (`Linearize`):** Hızlı görüntüleme için yeniden indeksleme<br/>🧹 **PDF Yazar / Üst Veri Temizleme** |
| <img src="https://img.shields.io/badge/📝_BELGE_%26_KİTAP-2EC27E?style=for-the-badge" alt="Belge" /><br/><sub><b>Doc & E-Book</b></sub> | `docx` `doc` `odt`<br/>`pages` `md` `txt`<br/>`rtf` `html` `epub` | ![PDF](https://img.shields.io/badge/PDF-E01B24?style=flat-square) ![DOCX](https://img.shields.io/badge/DOCX-242936?style=flat-square) ![ODT](https://img.shields.io/badge/ODT-242936?style=flat-square) ![PAGES](https://img.shields.io/badge/PAGES-242936?style=flat-square)<br/>![EPUB](https://img.shields.io/badge/EPUB-242936?style=flat-square) ![MD](https://img.shields.io/badge/MD-242936?style=flat-square) ![TXT](https://img.shields.io/badge/TXT-242936?style=flat-square) ![HTML](https://img.shields.io/badge/HTML-242936?style=flat-square)<br/>![PNG](https://img.shields.io/badge/PNG-242936?style=flat-square) ![JPG](https://img.shields.io/badge/JPG-242936?style=flat-square) | 🔄 **Çift Motorlu Dönüşüm:** `Pandoc` + `LibreOffice Headless`<br/>📸 **Belgeden Görsele:** Belgeleri doğrudan yüksek çözünürlüklü `PNG`/`JPG` sayfalara çevirme<br/>📦 **Hızlı Paketleme:** Tek tıkla `ZIP`, `TAR` veya `GZIP` arşivleme |
| <img src="https://img.shields.io/badge/📊_TABLO_%26_SUNUM-F5C211?style=for-the-badge" alt="Tablo ve Sunum" /><br/><sub><b>Sheet & Slides</b></sub> | `csv` `xlsx` `ods`<br/>`numbers` `tsv`<br/>`pptx` `odp` `key` | ![XLSX](https://img.shields.io/badge/XLSX-242936?style=flat-square) ![CSV](https://img.shields.io/badge/CSV-242936?style=flat-square) ![NUMBERS](https://img.shields.io/badge/NUMBERS-242936?style=flat-square)<br/>![PDF](https://img.shields.io/badge/PDF-E01B24?style=flat-square) ![PPTX](https://img.shields.io/badge/PPTX-242936?style=flat-square) ![KEY](https://img.shields.io/badge/KEY-242936?style=flat-square)<br/>![HTML](https://img.shields.io/badge/HTML_Tablo-242936?style=flat-square) ![MD](https://img.shields.io/badge/MD_Tablo-242936?style=flat-square) | 📊 **Akıllı Tablo Dönüşümü:** `CSV` $\leftrightarrow$ `XLSX` / `PDF` / `HTML` / `Markdown`<br/>📽️ **Slayt Dışa Aktarıcı:** Sunum slaytlarını tek tek `PNG` veya `JPG` görsellere dönüştürme |
| <img src="https://img.shields.io/badge/💬_ALTYAZI_%26_ARŞİV-1C71D8?style=for-the-badge" alt="Altyazı ve Arşiv" /><br/><sub><b>Subtitles & Archive</b></sub> | `srt` `vtt` `ass` `sub`<br/>`zip` `tar` `tar.gz`<br/>`tgz` `7z` `rar` `xz` | ![VTT](https://img.shields.io/badge/WebVTT-242936?style=flat-square) ![SRT](https://img.shields.io/badge/SubRip_SRT-242936?style=flat-square) ![TXT](https://img.shields.io/badge/Düz_Metin-242936?style=flat-square)<br/>![ZIP](https://img.shields.io/badge/ZIP-FF7800?style=flat-square) ![TAR](https://img.shields.io/badge/TAR-FF7800?style=flat-square) ![GZIP](https://img.shields.io/badge/TAR.GZ-FF7800?style=flat-square) | 💬 **Zaman Damgası Temizleyici:** `.srt` / `.vtt` dosyalarından zaman kodlarını söküp saf metin (`.txt`) çıkarma<br/>📂 **Klasöre Çıkart (`Extract Here`):** Arşivleri otomatik alt klasöre açma<br/>🔄 **Arşiv Format Dönüştürücü:** `7z`/`RAR`/`TAR` $\leftrightarrow$ `ZIP`/`TAR.GZ` |

</div>

---

## 📲 KDE Connect & LocalSend Ortak Paylaşım Köprüsü

**Convext**, yerel ağınızdaki telefon ve bilgisayarlara sıfır tıklamayla dosya göndermek için **KDE Connect** (`kdeconnect-cli` / `kdeconnect-handler`) ve **LocalSend** (Flatpak `org.localsend.localsend_app` + Native) ile entegre çalışır:

* **`Hızlı Gönder (KDE Connect / LocalSend Otomatik)`:** Eşleşmiş ve ulaşılabilir bir KDE Connect cihazınız (ör. *Galaxy S22*) varsa ek pencere açmadan dosyayı doğrudan telefonunuza gönderir; aktif KDE Connect cihazı yoksa otomatik olarak LocalSend'i başlatır.
* **Canlı Cihaz Listesi:** Sağ tık menüsü açıldığı anda ağdaki aktif KDE Connect cihazlarınız (`KDE Connect: <Cihaz Adı> Cihazına Gönder`) menüde otomatik listelenir.

---

## 🎨 GNOME GTK4 & Libadwaita Arayüz Mimarisi

**Convext**, GNOME İnsan Arayüzü Yönergelerine (**GNOME HIG**) tam uyumlu olarak yerel **GTK4 (`v4.12+`)** ve **Libadwaita (`v1.9`)** bileşenleriyle inşa edilmiştir:

1. **Kademeli Sağ Tık Menüsü (`Gtk.PopoverMenuFlags.NESTED`):**
   * Nautilus'un `Gtk.PopoverMenu` nesnelerine çalışma zamanında `NESTED` bayrağı uygulanır. Böylece **`Convext Menü ▸`** üzerine gelindiğinde menü aynı pencerede değişmez, her alt kategori kendi ayrı dikey açılır penceresinde (cascading dropdown) açılır.
2. **Yerel Libadwaita Yardımcı Modalları:**
   * **Hedef Boyut Sıkıştırıcı (`TargetSizeModal`):** `adw::EntryRow` ve `adw::ComboRow` (`KB` / `MB`) ile görsel (Binary Search) ve videoları (Bitrate Solver) tam istediğiniz dosya boyutuna indirger.
   * **Medya Kırpıcı & Tempo (`MediaTrimmerModal`):** `adw::ActionRow` + `gtk4::Scale` ile başlangıç/bitiş saniyesini ve `0.5x – 2.0x` oynatma hızını ayarlar.
   * **Görsel Sansürleme & Kırpma (`ImageEditorModal`):** `adw::ToolbarView` + `adw::HeaderBar` üzerinde geri döndürülemez **Mozaik (`Pixelate`)**, **Karartma (`Blackout`)** ve **Kırpma (`Crop`)** uygular.
   * **Toplu Yeniden Adlandırıcı (`BatchRenameModal`):** `{name}_{index}_{date}` şablonlarıyla canlı önizlemeli (`adw::ActionRow`) çoklu dosya adlandırma.
   * **Yüzen Dosya Rafı (`FloatShelfWindow`):** `boxed-list` stilinde `adw::ActionRow` satırlarıyla dosyaları geçici olarak toplayıp tek seferde sürükleme veya işleme.

---

## 🏗️ Sistem Mimarisi ve İş Akışı

```mermaid
flowchart LR
    A["📂 Nautilus Dosya Yöneticisi<br/>(Sağ Tık -> Convext Menü ▸)"] -->|Gtk.PopoverMenuFlags.NESTED<br/>Kademeli Dikey Dropdownlar| B("⚡ Convext Daemon<br/>Rust 2024 + GTK4 + Libadwaita")
    B -->|Unix Domain Socket<br/>convext-daemon.sock| C["⚙️ Tokio Async İş Havuzu<br/>(Sessiz Arka Plan Motoru)"]
    C --> D["🖼️ Image / Video / Audio / PDF / Archive"]
    C --> E["📲 KDE Connect & LocalSend Bridge"]
    D --> F["🔔 Masaüstü Bildirimi &<br/>Dosya Yöneticisinde Vurgula"]
```

### Proje Dizin Yapısı

```text
convext/
├── Cargo.toml                  # Rust 2024 Edition yapılandırması (convext, gtk4, libadwaita, tokio)
├── install.sh                  # Tek komutla tüm bağımlılıkları, servisi ve eklentiyi kuran betik
├── uninstall.sh                # Sistemden tamamen kaldırma betiği
├── run_convext.sh              # Geliştirici derleme, örnek dosya üretme ve test betiği
├── assets/
│   ├── logo.svg                # Convext vektörel GNOME squircle logosu
│   ├── convext_menu.py         # Nautilus 4.0 / 4.1 kademeli (NESTED) sağ tık menü eklentisi
│   └── style.css               # GTK4 / Libadwaita tamamlayıcı stiller
└── src/
    ├── main.rs                 # Libadwaita başlatıcı, Unix soket & CLI (--convext) yönlendiricisi
    ├── config.rs               # ConvextConfig (~/.config/convext/config.toml) ve binary tespiti
    ├── error.rs                # ConvextError & ConvextResult hata yönetimi
    ├── daemon/                 # Unix soket sunucusu ($XDG_RUNTIME_DIR/convext-daemon.sock)
    ├── desktop/                # KDE Connect, LocalSend, pano, terminal ve DBus ShowItems köprüsü
    ├── engine/                 # Görsel, Video, Ses, PDF, Belge, Arşiv ve QR/EXIF motorları
    └── ui/                     # Libadwaita pencereleri (Hedef Boyut, Kırpıcı, Sansürleyici, Float Rafı)
```

---

## 💻 Komut Satırı (CLI) ve Soket Kullanımı

Sağ tık menüsüne ek olarak `convext` ikili dosyasını doğrudan terminalden veya kendi betiklerinizden çağırabilirsiniz:

```bash
# Bir görseli arka planda sessizce WEBP formatına dönüştür
~/.local/bin/convext --convext IMG_CONVERT:webp /path/to/gorsel.png

# Bir videoyu arka planda hızlı sıkıştır
~/.local/bin/convext --convext VID_QUICK_COMPRESS /path/to/video.mp4

# Birden fazla videoyu tek MP4 dosyasında birleştir
~/.local/bin/convext --convext VID_JOIN klip1.mp4 klip2.mp4

# Dosyayı KDE Connect veya LocalSend üzerinden hızlı gönder
~/.local/bin/convext --convext ACT_AIRDROP /path/to/dosya.pdf

# Bir arşivi bulunduğu klasöre çıkart
~/.local/bin/convext --convext ARC_EXTRACT /path/to/arsiv.zip
```

---

## 🔧 Servis Yönetimi ve Kaldırma

Arka plan servisinin durumunu kontrol etmek veya yeniden başlatmak için:

```bash
systemctl --user status convext.service
systemctl --user restart convext.service
```

Sistemden tamamen kaldırmak için:

```bash
chmod +x uninstall.sh
./uninstall.sh
```
