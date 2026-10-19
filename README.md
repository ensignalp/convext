<div align="center">

<p align="center">
  <img src="assets/logo.svg" alt="Convext Logo" width="160" height="160" />
</p>

<h1 align="center">⚡ Convext</h1>

<h3 align="center">
  <b>GNOME / Linux için %100 Çevrimdışı (Offline), Bağlam Duyarlı Sağ Tık Dosya Dönüştürücü, Medya İşleyici ve Libadwaita Araç Stüdyosu</b>
</h3>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2024_Edition_(1.98+)-F74C00?style=for-the-badge&logo=rust&logoColor=white" alt="Rust 2024" /></a>
  <a href="https://www.gtk.org/"><img src="https://img.shields.io/badge/GTK-4.12+-4A86CF?style=for-the-badge&logo=gtk&logoColor=white" alt="GTK4" /></a>
  <a href="https://gnome.pages.gitlab.gnome.org/libadwaita/"><img src="https://img.shields.io/badge/GNOME-Libadwaita_1.9-3584E4?style=for-the-badge&logo=gnome&logoColor=white" alt="Libadwaita" /></a>
  <a href="https://tokio.rs/"><img src="https://img.shields.io/badge/Async_Engine-Tokio_1.40-18181B?style=for-the-badge&logo=rust&logoColor=orange" alt="Tokio" /></a>
  <a href="#"><img src="https://img.shields.io/badge/Share-KDE_Connect_%26_LocalSend-1D99F3?style=for-the-badge&logo=kde&logoColor=white" alt="KDE Connect & LocalSend" /></a>
  <a href="#"><img src="https://img.shields.io/badge/Privacy-100%25_Offline-2EA44F?style=for-the-badge" alt="Offline" /></a>
</p>

<p align="center">
  <b>Dosya yöneticisinde sağ tıklayın $\rightarrow$ Dosya türünüze özel kademeli açılır menülerden (Dropdown) formatınızı veya aracınızı seçin $\rightarrow$ Arka planda sessizce tamamlansın!</b>
</p>

<p align="center">
  <a href="#-tek-komutla-hızlı-kurulum-one-line-install">🚀 Tek Komutla Kurulum</a> •
  <a href="#-desteklenen-kategoriler-dönüştürücüler-ve-araçlar">✨ Dönüştürücü & Araç Matrisi</a> •
  <a href="#-kde-connect--localsend-ortak-paylaşım-köprüsü">📲 KDE Connect & LocalSend</a> •
  <a href="#-gnome-gtk4--libadwaita-arayüz-mimarisi">🎨 Libadwaita & Kademeli Menü</a> •
  <a href="#️-sistem-mimarisi-ve-iş-akışı">🏗️ Sistem Mimarisi</a> •
  <a href="#-komut-satırı-cli-ve-soket-kullanımı">💻 CLI Kullanımı</a>
</p>

---

## 🌟 Neden Convext?

<p align="center">
  <b>Sıfır Bulut Yüklemesi, Sıfır Bekleme:</b> Görsellerinizi, videolarınızı, ses kayıtlarınızı, PDF belgelerinizi veya arşivlerinizi dönüştürmek için internete dosya yüklemenize veya karmaşık terminal komutları yazmanıza gerek yok. <b>Convext</b>, doğrudan GNOME Dosyalar (<b>Nautilus</b>) sağ tık menüsüne ve yerel <b>GTK4 + Libadwaita</b> arayüzüne entegre olur.
</p>

| 🎯 Akıllı Bağlam Algılama | 📑 Kademeli Açılır Menüler (`NESTED`) | 🦀 Rust 2024 + Tokio Async Motor |
| :---: | :---: | :---: |
| Bir `.png` görseline sağ tıkladığınızda yalnızca görsel araçları; bir `.mp4` videosuna sağ tıkladığınızda yalnızca video kırpma/sıkıştırma araçları; bir `.zip` arşivine sağ tıkladığınızda ise arşiv çıkarma ve dönüştürme araçları açılır. | Nautilus sağ tık menüsünde **`Convext Menü ▸`** üzerine geldiğinizde aynı menü içinde kaybolmaz; her alt kategori (**Format Dönüştür ▸**, **Araçlar ▸**, **Arşiv ▸**, **Cihaza Gönder ▸**) kendi ayrı dikey açılır penceresinde yan yana açılır. | Tüm ağır FFmpeg, QPDF, Pandoc, LibreOffice ve görüntü işleme görevleri `tokio` çoklu iş parçacığı havuzunda sessizce yürütülür; işlem bittiğinde masaüstü bildirimi gönderilir ve üretilen dosya otomatik vurgulanır. |

---

## 🚀 Tek Komutla Hızlı Kurulum (One-Line Install)

<p align="center">
  Sistem bağımlılıklarını (<code>Fedora</code>, <code>Ubuntu/Debian</code>, <code>Arch Linux</code>, <code>openSUSE</code>), Rust derleyicisini, <code>convext</code> arka plan motorunu, <code>systemd</code> kullanıcı servisini ve Nautilus sağ tık eklentisini <b>tek komutla</b> indirip kurmak için terminalinize yapıştırın:
</p>

```bash
curl -fsSL https://raw.githubusercontent.com/ensignalp/convext/main/install.sh | bash
```

<h3 align="center">Alternatif: Git ile Klonlayarak Kurulum</h3>

```bash
git clone https://github.com/ensignalp/convext.git
cd convext
chmod +x install.sh
./install.sh
```

<p align="center">
  <sub>💡 Kendi fork/depo adresinizden kurulum yapmak isterseniz: <code>CONVEXT_REPO_URL="https://github.com/&lt;KULLANICI_ADINIZ&gt;/convext.git" ./install.sh</code></sub>
</p>

---

## ✨ Desteklenen Kategoriler, Dönüştürücüler ve Araçlar

| Kategori & Modül | Algılanan Kaynak Uzantılar | 🔄 Hedef Dönüştürme Formatları (`Convert`) | 🛠️ Özel Medya & Belge Araçları (`Specialized Tools`) |
| :---: | :---: | :---: | :---: |
| <img src="https://img.shields.io/badge/🖼️_GÖRSEL-FF7800?style=for-the-badge" alt="Görsel" /><br/><sub><b>Image Engine</b></sub> | `png` `jpg` `jpeg`<br/>`webp` `avif` `heic`<br/>`heif` `gif` `bmp`<br/>`tiff` `svg` `ico` | ![PNG](https://img.shields.io/badge/PNG-242936?style=flat-square) ![JPG](https://img.shields.io/badge/JPG-242936?style=flat-square) ![WEBP](https://img.shields.io/badge/WEBP-242936?style=flat-square) ![AVIF](https://img.shields.io/badge/AVIF-242936?style=flat-square) ![HEIC](https://img.shields.io/badge/HEIC-242936?style=flat-square)<br/>![GIF](https://img.shields.io/badge/GIF-242936?style=flat-square) ![BMP](https://img.shields.io/badge/BMP-242936?style=flat-square) ![TIFF](https://img.shields.io/badge/TIFF-242936?style=flat-square) ![ICO](https://img.shields.io/badge/ICO-242936?style=flat-square) ![PDF](https://img.shields.io/badge/PDF-E66100?style=flat-square) | ⚡ **Hızlı Sıkıştırma:** `%85`, `%65`, `%40`<br/>🎯 **Hedef Boyut (`Binary Search`):** Tam `KB` / `MB` indirgeme<br/>🕵️ **Kalıcı Sansürleme & Kırpma:** `Pixelate`, `Blackout`, `Crop`<br/>🖼️ **Stüdyo Çerçeve:** Koyu arka plan dolgusu ekleme<br/>📑 **Çoklu Görsel $\rightarrow$ PDF:** Tek PDF'te birleştirme<br/>📱 **Çevrimdışı QR Okuyucu:** QR çözüp panoya kopyalama<br/>🧹 **Gizlilik:** EXIF / GPS üst verilerini sıfırlama |
| <img src="https://img.shields.io/badge/🎬_VİDEO-3584E4?style=for-the-badge" alt="Video" /><br/><sub><b>Video Engine</b></sub> | `mp4` `mov` `mkv`<br/>`webm` `avi` `wmv`<br/>`flv` `m4v` | ![MP4](https://img.shields.io/badge/MP4_H.264-242936?style=flat-square) ![MOV](https://img.shields.io/badge/MOV-242936?style=flat-square) ![MKV](https://img.shields.io/badge/MKV-242936?style=flat-square)<br/>![WEBM](https://img.shields.io/badge/WEBM_VP9-242936?style=flat-square) ![AVI](https://img.shields.io/badge/AVI-242936?style=flat-square) ![GIF](https://img.shields.io/badge/GIF_Animasyon-242936?style=flat-square)<br/>![MP3](https://img.shields.io/badge/MP3_Ses_Ayıkla-1C71D8?style=flat-square) | ⚡ **Hızlı Sıkıştırma:** `H.264 CRF 28` tek tıkla küçültme<br/>🎯 **Hedef Boyut (`Bitrate Solver`):** Tam `MB` / `KB` hedef boyutu<br/>🔗 **Video Birleştirici (`Join`):** Çoklu klibi tek MP4 yapma<br/>✂️ **Kırp & Hız (`Trim & Speed`):** Başlangıç/bitiş ve `0.5x–2.0x` hız<br/>➗ **İkiye Böl (`Split`):** Videoyu tam ortadan ikiye bölme<br/>📐 **Kadrajla (`Crop`) & Sessize Al (`Mute`)**<br/>📸 **Kare Yakala (`Snapshot PNG`) & Sansürle (`Redact`)** |
| <img src="https://img.shields.io/badge/🎵_SES-9141AC?style=for-the-badge" alt="Ses" /><br/><sub><b>Audio Engine</b></sub> | `mp3` `wav` `flac`<br/>`m4a` `aac` `ogg`<br/>`opus` `aiff` `wma` | ![MP3](https://img.shields.io/badge/MP3-242936?style=flat-square) ![WAV](https://img.shields.io/badge/WAV_PCM-242936?style=flat-square) ![FLAC](https://img.shields.io/badge/FLAC_Lossless-242936?style=flat-square)<br/>![M4A](https://img.shields.io/badge/M4A-242936?style=flat-square) ![AAC](https://img.shields.io/badge/AAC-242936?style=flat-square) ![OGG](https://img.shields.io/badge/OGG-242936?style=flat-square)<br/>![OPUS](https://img.shields.io/badge/OPUS-242936?style=flat-square) ![AIFF](https://img.shields.io/badge/AIFF-242936?style=flat-square) | 🎚️ **EBU R128 Ses Dengeleme:** Stüdyo standardı `-16 LUFS` normalize<br/>✂️ **Ses Kesici & Tempo:** Saniye bazlı kesme ve hızlandırma<br/>📊 **Ses Dalga Videosu (`Waveform MP4`):** `1280x720` görsel video<br/>🔊 **Kanal Dönüştürücü:** `Mono` $\leftrightarrow$ `Stereo` geçişi<br/>🔕 **Bleep Sansürleme:** Orta kesite `1000Hz` bip tonu uygulama<br/>🗜️ **Ses Sıkıştırma & ID3 Metadata Temizleme** |
| <img src="https://img.shields.io/badge/📄_PDF-E01B24?style=for-the-badge" alt="PDF" /><br/><sub><b>PDF Engine</b></sub> | `pdf` | ![DOCX](https://img.shields.io/badge/DOCX-242936?style=flat-square) ![PAGES](https://img.shields.io/badge/PAGES-242936?style=flat-square) ![TXT](https://img.shields.io/badge/TXT-242936?style=flat-square) ![MD](https://img.shields.io/badge/MD-242936?style=flat-square)<br/>![HTML](https://img.shields.io/badge/HTML-242936?style=flat-square) ![EPUB](https://img.shields.io/badge/EPUB-242936?style=flat-square) ![PNG](https://img.shields.io/badge/PNG_300DPI-242936?style=flat-square) ![JPG](https://img.shields.io/badge/JPG_300DPI-242936?style=flat-square) | 🗜️ **PDF Sıkıştırma:** Akış ve nesne sıkıştırma (`QPDF`)<br/>📑 **Sayfaları Ayır (`Split`):** Her sayfayı ayrı PDF dosyasına bölme<br/>🔗 **PDF Birleştir (`Merge`):** Çoklu PDF seçimini tek belgede birleştirme<br/>⚡ **Web Optimize (`Linearize`):** Hızlı görüntüleme için indeksleme<br/>🧹 **PDF Yazar / Üst Veri Temizleme** |
| <img src="https://img.shields.io/badge/📝_BELGE_%26_KİTAP-2EC27E?style=for-the-badge" alt="Belge" /><br/><sub><b>Doc & E-Book</b></sub> | `docx` `doc` `odt`<br/>`pages` `md` `txt`<br/>`rtf` `html` `epub` | ![PDF](https://img.shields.io/badge/PDF-E01B24?style=flat-square) ![DOCX](https://img.shields.io/badge/DOCX-242936?style=flat-square) ![ODT](https://img.shields.io/badge/ODT-242936?style=flat-square) ![PAGES](https://img.shields.io/badge/PAGES-242936?style=flat-square)<br/>![EPUB](https://img.shields.io/badge/EPUB-242936?style=flat-square) ![MD](https://img.shields.io/badge/MD-242936?style=flat-square) ![TXT](https://img.shields.io/badge/TXT-242936?style=flat-square) ![HTML](https://img.shields.io/badge/HTML-242936?style=flat-square)<br/>![PNG](https://img.shields.io/badge/PNG-242936?style=flat-square) ![JPG](https://img.shields.io/badge/JPG-242936?style=flat-square) | 🔄 **Çift Motorlu Dönüşüm:** `Pandoc` + `LibreOffice Headless`<br/>📸 **Belgeden Görsele:** Belgeleri doğrudan `PNG` / `JPG` sayfalara çevirme<br/>📦 **Hızlı Paketleme:** Tek tıkla `ZIP`, `TAR` veya `GZIP` arşivleme |
| <img src="https://img.shields.io/badge/📊_TABLO_%26_SUNUM-F5C211?style=for-the-badge" alt="Tablo ve Sunum" /><br/><sub><b>Sheet & Slides</b></sub> | `csv` `xlsx` `ods`<br/>`numbers` `tsv`<br/>`pptx` `odp` `key` | ![XLSX](https://img.shields.io/badge/XLSX-242936?style=flat-square) ![CSV](https://img.shields.io/badge/CSV-242936?style=flat-square) ![NUMBERS](https://img.shields.io/badge/NUMBERS-242936?style=flat-square)<br/>![PDF](https://img.shields.io/badge/PDF-E01B24?style=flat-square) ![PPTX](https://img.shields.io/badge/PPTX-242936?style=flat-square) ![KEY](https://img.shields.io/badge/KEY-242936?style=flat-square)<br/>![HTML](https://img.shields.io/badge/HTML_Tablo-242936?style=flat-square) ![MD](https://img.shields.io/badge/MD_Tablo-242936?style=flat-square) | 📊 **Akıllı Tablo Dönüşümü:** `CSV` $\leftrightarrow$ `XLSX` / `PDF` / `HTML` / `Markdown`<br/>📽️ **Slayt Dışa Aktarıcı:** Sunum slaytlarını tek tek `PNG` veya `JPG` görsellere dönüştürme |
| <img src="https://img.shields.io/badge/💬_ALTYAZI_%26_ARŞİV-1C71D8?style=for-the-badge" alt="Altyazı ve Arşiv" /><br/><sub><b>Subtitles & Archive</b></sub> | `srt` `vtt` `ass` `sub`<br/>`zip` `tar` `tar.gz`<br/>`tgz` `7z` `rar` `xz` | ![VTT](https://img.shields.io/badge/WebVTT-242936?style=flat-square) ![SRT](https://img.shields.io/badge/SubRip_SRT-242936?style=flat-square) ![TXT](https://img.shields.io/badge/Düz_Metin-242936?style=flat-square)<br/>![ZIP](https://img.shields.io/badge/ZIP-FF7800?style=flat-square) ![TAR](https://img.shields.io/badge/TAR-FF7800?style=flat-square) ![GZIP](https://img.shields.io/badge/TAR.GZ-FF7800?style=flat-square) | 💬 **Zaman Damgası Temizleyici:** `.srt` / `.vtt` dosyalarından saf metin (`.txt`) çıkarma<br/>📂 **Klasöre Çıkart (`Extract Here`):** Arşivleri otomatik alt klasöre açma<br/>🔄 **Arşiv Format Dönüştürücü:** `7z`/`RAR`/`TAR` $\leftrightarrow$ `ZIP`/`TAR.GZ` |

---

## 📲 KDE Connect & LocalSend Ortak Paylaşım Köprüsü

<p align="center">
  <b>Convext</b>, yerel ağınızdaki telefon ve bilgisayarlara sıfır tıklamayla dosya göndermek için <b>KDE Connect</b> (<code>kdeconnect-cli</code> / <code>kdeconnect-handler</code>) ve <b>LocalSend</b> (Flatpak <code>org.localsend.localsend_app</code> + Native) ile entegre çalışır:
</p>

| 🚀 Hızlı Gönder (Otomatik Köprü) | 📱 Canlı KDE Connect Cihaz Listesi | 📡 LocalSend (Flatpak & Native) |
| :---: | :---: | :---: |
| Eşleşmiş ve ulaşılabilir bir KDE Connect cihazınız (ör. *Galaxy S22*) varsa ek pencere açmadan dosyayı doğrudan telefonunuza gönderir; aktif cihaz yoksa otomatik olarak LocalSend'i başlatır. | Sağ tık menüsü açıldığı anda ağdaki aktif KDE Connect cihazlarınız (`KDE Connect: <Cihaz Adı> Cihazına Gönder`) menüde otomatik listelenir. | Seçili dosyaları Flatpak dosya yönlendirme (`--file-forwarding`) veya yerel ikili dosya üzerinden doğrudan LocalSend gönderim ekranına aktarır. |

---

## 🎨 GNOME GTK4 & Libadwaita Arayüz Mimarisi

<p align="center">
  <b>Convext</b>, GNOME İnsan Arayüzü Yönergelerine (<b>GNOME HIG</b>) tam uyumlu olarak yerel <b>GTK4 (<code>v4.12+</code>)</b> ve <b>Libadwaita (<code>v1.9</code>)</b> bileşenleriyle inşa edilmiştir:
</p>

| Modül / Pencere | Libadwaita Bileşenleri | Görevi |
| :---: | :---: | :---: |
| **Kademeli Sağ Tık Menüsü** | `Gtk.PopoverMenuFlags.NESTED` | **`Convext Menü ▸`** üzerine gelindiğinde menü aynı pencerede değişmez; her alt kategori kendi ayrı dikey açılır penceresinde (cascading dropdown) açılır. |
| **Hedef Boyut Sıkıştırıcı (`TargetSizeModal`)** | `adw::Window` + `adw::EntryRow` + `adw::ComboRow` | Görsel (`Binary Search`) ve videoları (`Bitrate Solver`) tam istediğiniz `KB` / `MB` dosya boyutuna indirger. |
| **Medya Kırpıcı & Tempo (`MediaTrimmerModal`)** | `adw::PreferencesGroup` + `gtk4::Scale` | Video ve ses dosyalarının başlangıç/bitiş saniyesini ve `0.5x – 2.0x` oynatma hızını ayarlar. |
| **Görsel Sansürleme & Kırpma (`ImageEditorModal`)** | `adw::ToolbarView` + `adw::HeaderBar` | Görsel üzerinde geri döndürülemez **Mozaik (`Pixelate`)**, **Karartma (`Blackout`)** ve **Kırpma (`Crop`)** uygular. |
| **Toplu Yeniden Adlandırıcı (`BatchRenameModal`)** | `adw::EntryRow` + `adw::ActionRow` | `{name}_{index}_{date}` şablonlarıyla canlı önizlemeli çoklu dosya adlandırma sağlar. |
| **Yüzen Dosya Rafı (`FloatShelfWindow`)** | `adw::Window` + `boxed-list` | Dosyaları geçici olarak toplayıp tek seferde sürükleme, birleştirme veya arşivleme imkanı sunar. |

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

<h3 align="center">📂 Proje Dizin Yapısı</h3>

</div>

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

<div align="center">

---

## 💻 Komut Satırı (CLI) ve Soket Kullanımı

<p align="center">
  Sağ tık menüsüne ek olarak <code>convext</code> ikili dosyasını doğrudan terminalden veya kendi betiklerinizden çağırabilirsiniz:
</p>

</div>

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

<div align="center">

---

## 🔧 Servis Yönetimi ve Kaldırma

<p align="center">
  Arka plan servisinin durumunu kontrol etmek, yeniden başlatmak veya sistemden tamamen kaldırmak için:
</p>

</div>

```bash
# Servis durumunu görüntüle ve yeniden başlat
systemctl --user status convext.service
systemctl --user restart convext.service

# Sistemden tamamen kaldır
chmod +x uninstall.sh && ./uninstall.sh
```

<div align="center">

---

<p align="center">
  <sub>⚡ <b>Convext</b> — GNOME & Linux için Rust 2024 + GTK4 + Libadwaita ile geliştirildi.</sub>
</p>

</div>
