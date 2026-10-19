# ⚡ Convext Menü (`citrus-daemon`)

**Linux (GNOME / Nautilus) için %100 Çevrimdışı (Offline), Bağlam Duyarlı Sağ Tık Dosya Dönüştürücü, Medya İşleyici, Arşiv ve Sansürleme Aracı.**

**Convext Menü**, dosya yöneticisinde (Nautilus) sağ tıkladığınız dosyanın türünü (Görsel, Video, Ses, PDF, Belge, E-Kitap, Tablo, Sunum, Altyazı veya Arşiv) anında algılar ve **yalnızca o dosya türüyle uyumlu** dönüştürme formatlarını, medya araçlarını ve dosya işlemlerini doğrudan sağ tık menüsünde (`Convext Menü`) listeler. Tüm dönüştürme ve sıkıştırma işlemleri ekranda kalabalık yapmadan **gizli arka planda (silent background)** Rust + Tokio motoruyla tamamlanır ve işlem bittiğinde yeni dosya otomatik olarak vurgulanır.

---

## 🚀 Tek Komutla Kurulum (One-Line Install)

Sistem bağımlılıklarını (Fedora, Ubuntu/Debian, Arch, openSUSE), Rust derleyicisini, `citrus-daemon` arka plan motorunu, `systemd` kullanıcı servisini ve Nautilus sağ tık eklentisini **tek komutla** indirip kurmak için:

```bash
curl -fsSL https://raw.githubusercontent.com/mustazafi/convext/main/install.sh | bash
```

### Alternatif: Depoyu Klonlayarak Tek Komutla Kurulum

```bash
git clone https://github.com/mustazafi/convext.git && cd convext && chmod +x install.sh && ./install.sh
```

> **Not:** Farklı bir GitHub kullanıcı adı/depo adresiniz varsa kurulum sırasında `CONVEXT_REPO_URL` değişkenini verebilirsiniz:
> ```bash
> CONVEXT_REPO_URL="https://github.com/<KULLANICI_ADINIZ>/<REPO_ADINIZ>.git" ./install.sh
> ```

---

## ✨ Özellikler ve Sağ Tık Menü Yapısı (`Convext Menü`)

Dosya yöneticisinde herhangi bir dosyaya (veya çoklu dosya seçimine) **Sağ Tık $\rightarrow$ `Convext Menü`** yaptığınızda dosya kategorisine göre aşağıdaki dinamik menüler otomatik açılır:

| Kategori | Format Dönüştürme (`Convert`) | Özel Araçlar (`Tools`) |
| :--- | :--- | :--- |
| **🖼️ Görsel**<br>(`png`, `jpg`, `webp`, `avif`, `heic`, `gif`, `bmp`, `tiff`, `svg`, `ico`) | `PNG`, `JPG`, `WEBP`, `AVIF`, `HEIC`, `GIF`, `BMP`, `TIFF`, `ICO`, `PDF` | • Kaliteye Göre Sıkıştır (`%85`, `%65`, `%40`)<br>• Hedef Boyuta Sıkıştır (`MB / KB`)<br>• Görsel Kırp & Sansürle (`Crop / Blackout / Pixelate`)<br>• Koyu Arka Plan Çerçevesi Ekle<br>• Çoklu Görseli Tek PDF Yap<br>• QR Kodu Oku ve Panoya Kopyala<br>• EXIF / Metadata Temizle |
| **🎬 Video**<br>(`mp4`, `mov`, `mkv`, `webm`, `avi`, `wmv`, `flv`, `m4v`) | `MP4`, `MOV`, `MKV`, `WEBM`, `AVI`, `GIF`, `MP3 (Sesi Ayıkla)` | • Videoyu Hızlı Sıkıştır (`H.264 CRF 28`)<br>• Hedef Boyuta Sıkıştır (`MB / KB`)<br>• Videoları Birleştir (`Join Videos`)<br>• Videoyu Kırp / Kes (`Trim Video`)<br>• Videoyu İkiye Böl (`Split Video`)<br>• Videoyu Kadrajla (`Crop Video`)<br>• Oynatma Hızını Değiştir (`0.5x – 2.0x`)<br>• Sesi Kaldır (`Remove Audio / Mute`)<br>• Videodan Kare Yakala (`Save Frame PNG`)<br>• Videoyu Sansürle (`Redact Video`)<br>• Metadata Temizle |
| **🎵 Ses**<br>(`mp3`, `wav`, `flac`, `m4a`, `aac`, `ogg`, `opus`, `aiff`) | `MP3`, `WAV`, `FLAC`, `M4A`, `AAC`, `OGG`, `OPUS`, `AIFF` | • Sesi Sıkıştır (`Compress Audio`)<br>• Sesi Kırp & Hız Değiştir (`Trim & Speed`)<br>• Ses Seviyesini Dengele (`EBU R128 Normalize`)<br>• Ses Dalga Videosu Oluştur (`Waveform MP4`)<br>• Mono / Stereo Kanal Değiştir<br>• Orta Bölümü Sansürle (`Bleep`)<br>• Metadata Temizle |
| **📄 PDF**<br>(`pdf`) | `DOCX`, `PAGES`, `TXT`, `MD`, `HTML`, `EPUB`, `PNG (Sayfalar)`, `JPG (Sayfalar)` | • PDF Sıkıştır (`Compress PDF`)<br>• PDF Sayfalarını Tek Tek Ayır (`Split PDF`)<br>• Seçili PDF'leri Birleştir (`Merge PDFs`)<br>• PDF Yapısını Optimize Et (`Linearize`)<br>• PDF Metadata Temizle |
| **📝 Belge & E-Kitap**<br>(`docx`, `odt`, `pages`, `md`, `txt`, `rtf`, `html`, `epub`) | `PDF`, `DOCX`, `ODT`, `PAGES`, `EPUB`, `MD`, `TXT`, `HTML`, `PNG`, `JPG` | • Arşiv Oluştur (`ZIP`, `TAR`, `GZIP`)<br>• EXIF / Metadata Temizle |
| **📊 Tablo / Veri**<br>(`csv`, `xlsx`, `ods`, `numbers`, `tsv`) | `XLSX`, `CSV`, `NUMBERS`, `PDF`, `HTML Tablo`, `Markdown Tablo` | • Arşiv Oluştur (`ZIP`, `TAR`, `GZIP`) |
| **📽️ Sunum**<br>(`pptx`, `odp`, `key`) | `PDF`, `PPTX`, `KEY`, `PNG (Slaytlar)`, `JPG (Slaytlar)` | • Arşiv Oluştur (`ZIP`, `TAR`, `GZIP`) |
| **💬 Altyazı**<br>(`srt`, `vtt`, `ass`, `sub`) | `VTT`, `SRT`, `TXT (Zaman Damgasız Düz Metin)` | • Arşiv Oluştur (`ZIP`, `TAR`, `GZIP`) |
| **📦 Arşiv**<br>(`zip`, `tar`, `tar.gz`, `tgz`, `7z`, `rar`) | `ZIP`, `TAR`, `GZIP (.tar.gz)` | • Klasöre Çıkart (`Extract Here`) |

### Ortak Dosya İşlemleri (`File Actions`)
Her dosya ve klasör için **`Convext Menü` $\rightarrow$ `Dosya İşlemleri`** altında:
* **Toplu Yeniden Adlandır (`Batch Rename`):** `{name}_{index}_{date}` şablonlarıyla çoklu adlandırma penceresi.
* **Float Rafına Ekle (`Floating Shelf`):** Dosyaları ekran üstünde yüzen geçici rafa (Dropover tarzı) sabitleme.
* **LocalSend / AirDrop ile Gönder:** Yerel ağdaki cihazlara dosya gönderimi.
* **Dosyayı Çoğalt (`Duplicate`)** ve **Organized Klasörüne Taşı**.
* **Tam Dosya Yolunu Kopyala (`Copy Path`)** ve **Burada Terminal Aç**.

---

## 🛠️ Mimari Yapı

```text
citrus-daemon/
├── Cargo.toml                  # Rust bağımlılıkları (GTK4, Cairo, Tokio, image, zip, tar, flate2, rqrr)
├── install.sh                  # Tek komutla tüm bağımlılıkları ve eklentiyi kuran betik
├── uninstall.sh                # Sistemden tamamen kaldırma betiği
├── run_tangerine.sh            # Geliştirici test ve örnek dosya oluşturma betiği
├── assets/
│   ├── convext_menu.py         # Nautilus 4.0 / 4.1 dinamik sağ tık menü eklentisi
│   └── style.css               # Koyu tema (Dark Frosted) GTK4 modal stilleri
└── src/
    ├── main.rs                 # Arka plan servisi, Unix soket & CLI (--convext) yönlendiricisi
    ├── config.rs               # Yapılandırma ve harici ikili (binary) yetenek tespiti
    ├── error.rs                # Hata yönetimi (thiserror)
    ├── daemon/                 # Unix kontrol soketi ve pencere köprüsü
    ├── desktop/                # Dosya yöneticisi (DBus ShowItems) ve pano/terminal eylemleri
    ├── engine/                 # Görsel, Video, Ses, PDF, Belge, Arşiv ve QR/EXIF motorları
    └── ui/                     # Kırpma/Sansürleme, Video/Ses Kesici, Hedef Boyut ve Float Shelf modalları
```

---

## 💻 Komut Satırı (CLI) Kullanımı

Sağ tık menüsünün yanı sıra doğrudan terminalden de çağırabilirsiniz:

```bash
# Bir görseli arka planda WEBP formatına dönüştür
~/.local/bin/citrus-daemon --convext IMG_CONVERT:webp /path/to/image.png

# Bir arşivi bulunduğu klasöre çıkart
~/.local/bin/citrus-daemon --convext ARC_EXTRACT /path/to/archive.zip

# Birden fazla videoyu tek MP4 dosyasında birleştir
~/.local/bin/citrus-daemon --convext VID_JOIN video1.mp4 video2.mp4
```

---

## 🔄 Servis Yönetimi ve Kaldırma

Arka plan servisinin durumunu kontrol etmek veya yeniden başlatmak için:

```bash
systemctl --user status convext.service
systemctl --user restart convext.service
```

Sistemden tamamen kaldırmak için:

```bash
chmod +x uninstall.sh && ./uninstall.sh
```
