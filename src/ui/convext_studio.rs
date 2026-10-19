use crate::engine::mime_router::{MediaCategory, MimeRouter};
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

pub struct ConvextStudioWindow;

impl ConvextStudioWindow {
    pub fn show<F>(app: &gtk4::Application, files: Vec<PathBuf>, on_execute: F)
    where
        F: Fn(String, Vec<PathBuf>) + 'static,
    {
        if files.is_empty() {
            return;
        }

        let first = &files[0];
        let category = MimeRouter::inspect_file(first);
        let file_count = files.len();
        let file_title = if file_count == 1 {
            first
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Dosya")
                .to_string()
        } else {
            format!("{} Dosya Seçildi", file_count)
        };

        let cat_badge = match category {
            MediaCategory::Image => "Görsel (Image)",
            MediaCategory::Video => "Video Medya",
            MediaCategory::Audio => "Ses (Audio)",
            MediaCategory::Pdf => "PDF Belgesi",
            MediaCategory::Document | MediaCategory::Epub => "Belge / E-Kitap",
            MediaCategory::Spreadsheet => "Tablo (Spreadsheet)",
            MediaCategory::Slides => "Sunum (Slides)",
            MediaCategory::Subtitles => "Altyazı (Subtitles)",
            MediaCategory::Archive => "Arşiv Paketi",
            MediaCategory::Generic => "Genel Dosya",
        };

        let window = adw::Window::builder()
            .application(app)
            .title("Convext Menü")
            .default_width(500)
            .default_height(620)
            .resizable(true)
            .build();

        let toolbar_view = adw::ToolbarView::new();
        let header_bar = adw::HeaderBar::new();
        let win_title = adw::WindowTitle::new("Convext Menü", &format!("{} • {}", file_title, cat_badge));
        header_bar.set_title_widget(Some(&win_title));
        toolbar_view.add_top_bar(&header_bar);

        let page = adw::PreferencesPage::new();
        let on_exec_rc = Rc::new(on_execute);

        // 1. FORMAT CONVERSION GROUP (Vertical AdwComboRow Dropdown)
        let (conv_formats, conv_prefix): (Vec<(&str, &str)>, &str) = match category {
            MediaCategory::Image => (
                vec![
                    ("png", "PNG — Kayıpsız Görsel (.png)"),
                    ("jpg", "JPG — Standart Fotoğraf (.jpg)"),
                    ("webp", "WEBP — Modern Web Görseli (.webp)"),
                    ("avif", "AVIF — Yeni Nesil Sıkıştırma (.avif)"),
                    ("heic", "HEIC — Apple HEIF Formatı (.heic)"),
                    ("gif", "GIF — Animasyon / Grafik (.gif)"),
                    ("bmp", "BMP — Ham Bitmap (.bmp)"),
                    ("tiff", "TIFF — Baskı Kalitesi (.tiff)"),
                    ("ico", "ICO — Masaüstü İkonu (.ico)"),
                    ("pdf", "PDF — Görselden PDF Belgesi (.pdf)"),
                ],
                "IMG_CONVERT:",
            ),
            MediaCategory::Video => (
                vec![
                    ("mp4", "MP4 — H.264 / AAC Standart Video (.mp4)"),
                    ("mov", "MOV — Apple QuickTime Video (.mov)"),
                    ("mkv", "MKV — Matroska Video Kapsayıcı (.mkv)"),
                    ("webm", "WEBM — VP9 / Opus Web Video (.webm)"),
                    ("avi", "AVI — Klasik Video Formatı (.avi)"),
                    ("gif", "GIF — Hareketli GIF Animasyon (.gif)"),
                    ("mp3", "MP3 — Videodan Sesi Ayıkla (.mp3)"),
                ],
                "VID_CONVERT:",
            ),
            MediaCategory::Audio => (
                vec![
                    ("mp3", "MP3 — Evrensel Ses Formatı (.mp3)"),
                    ("wav", "WAV — Sıkıştırmasız PCM Ses (.wav)"),
                    ("flac", "FLAC — Kayıpsız Stüdyo Ses (.flac)"),
                    ("m4a", "M4A — Apple AAC Ses (.m4a)"),
                    ("aac", "AAC — Gelişmiş Ses Kodlama (.aac)"),
                    ("ogg", "OGG — Ogg Vorbis Ses (.ogg)"),
                    ("opus", "OPUS — Yüksek Verimli Ses (.opus)"),
                    ("aiff", "AIFF — Apple Stüdyo Ses (.aiff)"),
                ],
                "AUD_CONVERT:",
            ),
            MediaCategory::Pdf => (
                vec![
                    ("docx", "DOCX — Microsoft Word Belgesi (.docx)"),
                    ("pages", "PAGES — Apple Pages Belgesi (.pages)"),
                    ("txt", "TXT — Düz Metin Çıktısı (.txt)"),
                    ("md", "MD — Markdown Belgesi (.md)"),
                    ("html", "HTML — Web Sayfası (.html)"),
                    ("epub", "EPUB — E-Kitap Formatı (.epub)"),
                    ("png", "PNG — Sayfaları Görsele Çevir (.png)"),
                    ("jpg", "JPG — Sayfaları Fotoğrafa Çevir (.jpg)"),
                ],
                "PDF_CONVERT:",
            ),
            MediaCategory::Document | MediaCategory::Epub => (
                vec![
                    ("pdf", "PDF — Taşınabilir Belge Formatı (.pdf)"),
                    ("docx", "DOCX — Microsoft Word Belgesi (.docx)"),
                    ("odt", "ODT — LibreOffice Metin Belgesi (.odt)"),
                    ("pages", "PAGES — Apple Pages Belgesi (.pages)"),
                    ("epub", "EPUB — E-Kitap Formatı (.epub)"),
                    ("md", "MD — Markdown Belgesi (.md)"),
                    ("txt", "TXT — Düz Metin (.txt)"),
                    ("html", "HTML — Web Sayfası (.html)"),
                    ("png", "PNG — Sayfaları Görsele Çevir (.png)"),
                    ("jpg", "JPG — Sayfaları Fotoğrafa Çevir (.jpg)"),
                ],
                "DOC_CONVERT:",
            ),
            MediaCategory::Spreadsheet => (
                vec![
                    ("xlsx", "XLSX — Microsoft Excel Tablosu (.xlsx)"),
                    ("csv", "CSV — Virgülle Ayrılmış Veri (.csv)"),
                    ("numbers", "NUMBERS — Apple Numbers Tablosu (.numbers)"),
                    ("pdf", "PDF — Tablodan PDF Raporu (.pdf)"),
                    ("html", "HTML — Web Tablosu (.html)"),
                    ("md", "MD — Markdown Tablosu (.md)"),
                ],
                "SHEET_CONVERT:",
            ),
            MediaCategory::Slides => (
                vec![
                    ("pdf", "PDF — Sunumu PDF'e Dönüştür (.pdf)"),
                    ("pptx", "PPTX — PowerPoint Sunumu (.pptx)"),
                    ("key", "KEY — Apple Keynote Sunumu (.key)"),
                    ("png", "PNG — Slaytları PNG Görsele Çevir (.png)"),
                    ("jpg", "JPG — Slaytları JPG Görsele Çevir (.jpg)"),
                ],
                "SLIDES_CONVERT:",
            ),
            MediaCategory::Subtitles => (
                vec![
                    ("vtt", "VTT — WebVTT Altyazı Formatı (.vtt)"),
                    ("srt", "SRT — SubRip Altyazı Formatı (.srt)"),
                    ("txt", "TXT — Zaman Damgasız Düz Metin (.txt)"),
                ],
                "SUB_CONVERT:",
            ),
            MediaCategory::Archive => (
                vec![
                    ("zip", "ZIP — Standart ZIP Arşivi (.zip)"),
                    ("tar", "TAR — Paket Arşivi (.tar)"),
                    ("gzip", "GZIP — Sıkıştırılmış TAR.GZ Arşivi (.tar.gz)"),
                ],
                "ARC_CONVERT:",
            ),
            MediaCategory::Generic => (vec![], ""),
        };

        if !conv_formats.is_empty() {
            let group_conv = adw::PreferencesGroup::builder()
                .title("Format Dönüştürücü (Convert Format)")
                .description("Açılır menüden hedef dosya formatını seçip arka planda dönüştürün")
                .build();

            let labels: Vec<&str> = conv_formats.iter().map(|(_, l)| *l).collect();
            let codes: Vec<String> = conv_formats
                .iter()
                .map(|(fmt, _)| format!("{}{}", conv_prefix, fmt))
                .collect();

            let combo_conv = adw::ComboRow::builder()
                .title("Hedef Format Seçin")
                .subtitle("Dikey açılır menü (Dropdown)")
                .model(&gtk4::StringList::new(&labels))
                .selected(0)
                .build();

            let btn_conv = gtk4::Button::builder()
                .label("Dönüştür")
                .valign(gtk4::Align::Center)
                .build();
            btn_conv.add_css_class("suggested-action");
            btn_conv.add_css_class("pill");

            let combo_ref = combo_conv.clone();
            let files_ref = files.clone();
            let win_ref = window.clone();
            let exec_ref = on_exec_rc.clone();
            btn_conv.connect_clicked(move |_| {
                let idx = combo_ref.selected() as usize;
                if let Some(code) = codes.get(idx) {
                    exec_ref(code.clone(), files_ref.clone());
                    win_ref.close();
                }
            });

            combo_conv.add_suffix(&btn_conv);
            group_conv.add(&combo_conv);
            page.add(&group_conv);
        }

        // 2. SPECIALIZED TOOLS GROUP (Vertical AdwComboRow Dropdown)
        let tools_list: Vec<(&str, &str)> = match category {
            MediaCategory::Image => {
                let mut v = vec![
                    ("IMG_COMPRESS_85", "Sıkıştır — Yüksek Kalite (%85)"),
                    ("IMG_COMPRESS_65", "Sıkıştır — Dengeli Küçültme (%65)"),
                    ("IMG_COMPRESS_40", "Sıkıştır — Maksimum Küçültme (%40)"),
                    ("IMG_TARGET_SIZE", "Hedef Boyuta Sıkıştır (MB / KB)..."),
                    ("IMG_EDITOR", "Görsel Kırp & Sansürle (Crop / Redact)..."),
                    ("IMG_ADD_BG", "Koyu Arka Plan Çerçevesi Ekle"),
                    ("IMG_QR_DECODE", "QR Kodu Oku ve Panoya Kopyala"),
                    ("STRIP_METADATA", "EXIF / Metadata Bilgilerini Temizle"),
                ];
                if file_count > 1 {
                    v.insert(6, ("IMG_COLLAGE_PDF", "Seçili Görselleri Tek PDF Yap"));
                }
                v
            }
            MediaCategory::Video => vec![
                ("VID_QUICK_COMPRESS", "Videoyu Hızlı Sıkıştır (Compress Video)"),
                ("VID_TARGET_SIZE", "Hedef Boyuta Sıkıştır (MB / KB)..."),
                ("VID_JOIN", "Videoları Birleştir (Join Videos)"),
                ("VID_TRIM", "Videoyu Kırp / Kes (Trim Video)..."),
                ("VID_SPLIT", "Videoyu İkiye Böl (Split Video)"),
                ("VID_CROP", "Videoyu Kadrajla (Crop Video)"),
                ("VID_TRIM", "Oynatma Hızını Değiştir (Playback Speed)..."),
                ("VID_MUTE", "Sesi Kaldır / Sessize Al (Remove Audio)"),
                ("VID_SNAPSHOT", "Videodan Kare Yakala (Save Frame PNG)"),
                ("VID_REDACT", "Videoyu Sansürle (Redact Video)"),
                ("STRIP_METADATA", "Metadata Bilgilerini Temizle (Edit Metadata)"),
            ],
            MediaCategory::Audio => vec![
                ("AUD_COMPRESS", "Sesi Sıkıştır (Compress Audio)"),
                ("AUD_TRIM", "Sesi Kırp & Hız Değiştir (Trim & Speed)..."),
                ("AUD_NORMALIZE", "Ses Seviyesini Dengele (EBU R128 Normalize)"),
                ("AUD_VISUALIZE", "Ses Dalga Videosu Oluştur (Waveform MP4)"),
                ("AUD_CHANNELS", "Mono / Stereo Kanal Değiştir"),
                ("AUD_BLEEP", "Orta Bölümü Sansürle (Bleep)"),
                ("STRIP_METADATA", "Metadata Bilgilerini Temizle"),
            ],
            MediaCategory::Pdf => {
                let mut v = vec![
                    ("PDF_COMPRESS", "PDF Boyutunu Sıkıştır (Compress PDF)"),
                    ("PDF_SPLIT", "PDF Sayfalarını Tek Tek Ayır (Split PDF)"),
                    ("PDF_ORGANIZE", "PDF Yapısını Optimize Et (Linearize)"),
                    ("STRIP_METADATA", "PDF Metadata Bilgilerini Temizle"),
                ];
                if file_count > 1 {
                    v.insert(2, ("PDF_MERGE", "Seçili PDF Dosyalarını Birleştir (Merge)"));
                }
                v
            }
            MediaCategory::Archive => vec![
                ("ARC_EXTRACT", "Arşivi Klasöre Çıkart (Extract Here)"),
            ],
            _ => vec![
                ("STRIP_METADATA", "EXIF / Metadata Bilgilerini Temizle"),
            ],
        };

        if !tools_list.is_empty() {
            let group_tools = adw::PreferencesGroup::builder()
                .title("Medya & Belge Araçları (Specialized Tools)")
                .description("Dosya türüne özel işleme, sıkıştırma ve düzenleme araçları")
                .build();

            let tool_labels: Vec<&str> = tools_list.iter().map(|(_, l)| *l).collect();
            let tool_codes: Vec<String> = tools_list.iter().map(|(c, _)| (*c).to_string()).collect();

            let combo_tools = adw::ComboRow::builder()
                .title("Araç Seçin")
                .subtitle("Dikey açılır menü (Dropdown)")
                .model(&gtk4::StringList::new(&tool_labels))
                .selected(0)
                .build();

            let btn_tool = gtk4::Button::builder()
                .label("Çalıştır")
                .valign(gtk4::Align::Center)
                .build();
            btn_tool.add_css_class("suggested-action");
            btn_tool.add_css_class("pill");

            let combo_ref = combo_tools.clone();
            let files_ref = files.clone();
            let win_ref = window.clone();
            let exec_ref = on_exec_rc.clone();
            btn_tool.connect_clicked(move |_| {
                let idx = combo_ref.selected() as usize;
                if let Some(code) = tool_codes.get(idx) {
                    exec_ref(code.clone(), files_ref.clone());
                    win_ref.close();
                }
            });

            combo_tools.add_suffix(&btn_tool);
            group_tools.add(&combo_tools);
            page.add(&group_tools);
        }

        // 3. ARCHIVE TOOLS GROUP (Vertical AdwComboRow Dropdown)
        let arc_list: Vec<(&str, &str)> = if category == MediaCategory::Archive {
            vec![
                ("ARC_EXTRACT", "Klasöre Çıkart (Extract Here)"),
                ("ARC_CONVERT:zip", "ZIP Arşivine Dönüştür (.zip)"),
                ("ARC_CONVERT:tar", "TAR Arşivine Dönüştür (.tar)"),
                ("ARC_CONVERT:gzip", "GZIP Arşivine Dönüştür (.tar.gz)"),
            ]
        } else {
            vec![
                ("ARC_CREATE_ZIP", "Seçili Dosyalardan ZIP Arşivi Oluştur (.zip)"),
                ("ARC_CREATE_TAR", "Seçili Dosyalardan TAR Arşivi Oluştur (.tar)"),
                ("ARC_CREATE_GZIP", "Seçili Dosyalardan GZIP Arşivi Oluştur (.tar.gz)"),
            ]
        };

        let group_arc = adw::PreferencesGroup::builder()
            .title("Arşiv Araçları (Archive Tools)")
            .description("Sıkıştırılmış paket oluşturma ve arşiv çıkarma işlemleri")
            .build();

        let arc_labels: Vec<&str> = arc_list.iter().map(|(_, l)| *l).collect();
        let arc_codes: Vec<String> = arc_list.iter().map(|(c, _)| (*c).to_string()).collect();

        let combo_arc = adw::ComboRow::builder()
            .title("Arşiv İşlemi Seçin")
            .subtitle("Dikey açılır menü (Dropdown)")
            .model(&gtk4::StringList::new(&arc_labels))
            .selected(0)
            .build();

        let btn_arc = gtk4::Button::builder()
            .label("Uygula")
            .valign(gtk4::Align::Center)
            .build();
        btn_arc.add_css_class("pill");

        let combo_arc_ref = combo_arc.clone();
        let files_arc_ref = files.clone();
        let win_arc_ref = window.clone();
        let exec_arc_ref = on_exec_rc.clone();
        btn_arc.connect_clicked(move |_| {
            let idx = combo_arc_ref.selected() as usize;
            if let Some(code) = arc_codes.get(idx) {
                exec_arc_ref(code.clone(), files_arc_ref.clone());
                win_arc_ref.close();
            }
        });

        combo_arc.add_suffix(&btn_arc);
        group_arc.add(&combo_arc);
        page.add(&group_arc);

        // 4. FILE ACTIONS GROUP (Vertical AdwComboRow Dropdown)
        let act_list: Vec<(&str, &str)> = vec![
            ("ACT_BATCH_RENAME", "Toplu Yeniden Adlandır (Batch Rename)..."),
            ("ACT_FLOAT", "Float Rafına Sabitle (Floating Shelf)"),
            ("ACT_AIRDROP", "LocalSend / AirDrop ile Cihaza Gönder"),
            ("ACT_COPY_FILES", "Dosyayı Aynı Klasörde Çoğalt (Duplicate)"),
            ("ACT_MOVE_ORGANIZED", "Convext_Organized Klasörüne Taşı"),
            ("ACT_COPY_PATH", "Tam Dosya Yolunu Panoya Kopyala (Copy Path)"),
            ("ACT_TERMINAL", "Bu Klasörde Terminal Aç"),
        ];

        let group_act = adw::PreferencesGroup::builder()
            .title("Dosya İşlemleri (File Actions)")
            .description("Hızlı dosya yönetimi, paylaşım ve terminal işlemleri")
            .build();

        let act_labels: Vec<&str> = act_list.iter().map(|(_, l)| *l).collect();
        let act_codes: Vec<String> = act_list.iter().map(|(c, _)| (*c).to_string()).collect();

        let combo_act = adw::ComboRow::builder()
            .title("Dosya Eylemi Seçin")
            .subtitle("Dikey açılır menü (Dropdown)")
            .model(&gtk4::StringList::new(&act_labels))
            .selected(0)
            .build();

        let btn_act = gtk4::Button::builder()
            .label("Çalıştır")
            .valign(gtk4::Align::Center)
            .build();
        btn_act.add_css_class("pill");

        let combo_act_ref = combo_act.clone();
        let files_act_ref = files.clone();
        let win_act_ref = window.clone();
        let exec_act_ref = on_exec_rc.clone();
        btn_act.connect_clicked(move |_| {
            let idx = combo_act_ref.selected() as usize;
            if let Some(code) = act_codes.get(idx) {
                exec_act_ref(code.clone(), files_act_ref.clone());
                win_act_ref.close();
            }
        });

        combo_act.add_suffix(&btn_act);
        group_act.add(&combo_act);
        page.add(&group_act);

        toolbar_view.set_content(Some(&page));
        window.set_content(Some(&toolbar_view));
        window.present();
    }
}
