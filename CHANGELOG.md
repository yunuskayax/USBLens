# Değişiklik günlüğü / Changelog

Bu dosya `src/changelog.json` dosyasından üretilir (`node scripts/changelog.mjs md`).

## 0.3.2 — 2026-11-23

- **Yeni:** Kopyalanabilir etiket: depolama aygıtları için "Kingston 64 GB USB Bellek (S.No: …)" biçiminde otomatik etiket, tek tıkla kopyalama
- **Yeni:** "Tümünü kopyala": cihazın tüm bilgileri düz metin olarak panoya
- **Yeni:** Ayarlar → Depolama aygıtları: etiket şablonu ({marka}, {model}, {kapasite}, {tür}, {seri}, {sürücü}…) ve canlı önizleme
- **Yeni:** Depolama aygıtlarında marka, model ve kapasite (kutudaki boyut ve kullanılabilir alan) gösterilir; USB bellek ile harici disk ayrılır

<details><summary>English</summary>

- **New:** Copyable label: storage devices get an automatic label like "Kingston 64 GB USB Flash Drive (S/N: …)", copied with one click
- **New:** "Copy all": every device property as plain text to the clipboard
- **New:** Settings → Storage devices: label template ({brand}, {model}, {capacity}, {type}, {serial}, {drive}…) with live preview
- **New:** Storage devices show brand, model and capacity (size on the box and usable space); flash drives and external disks are told apart

</details>

## 0.3.1 — 2026-11-17

- **Düzeltme:** Her cihaz bir kez listelenir: USB bellek, disk kaydı ve bileşik cihaz arayüzleri tek satırda birleştirildi
- **Yeni:** Ayrıntı panelinde cihazın alt bileşenleri (klavye, disk, ses arayüzü…) listelenir
- **Düzeltme:** Uygulama içi güncellemede "sunucuya ulaşılamadı" hatası: indirme artık üç farklı yolla denenir (Windows sertifika deposu, yerleşik sertifikalar, Windows indiricisi)
- **İyileştirme:** Güncelleme başarısız olursa hatanın ayrıntısı gösterilir
- **Yeni:** Ayarlar → Genel: alt bileşenleri ayrı satırlarda gösterme seçeneği (ham görünüm)

<details><summary>English</summary>

- **Fixed:** Each device is listed once: USB drive, disk entry and composite interfaces are merged into one row
- **New:** Device details list the device's sub-components (keyboard, disk, audio interface…)
- **Fixed:** In-app update "could not reach the server" error: the download now tries three methods (Windows certificate store, built-in roots, Windows downloader)
- **Improved:** If an update fails, the error details are shown
- **New:** Settings → General: option to show sub-components as separate rows (raw view)

</details>

## 0.3.0 — 2026-11-17

- **Yeni:** BadUSB uyarısı: hem klavye hem depolama gibi davranan cihazlar ve klavye taklidi yapabilen programlanabilir kartlar (Arduino, Digispark, Pico…) işaretlenir
- **Yeni:** Yeni klavye uyarısı: bilgisayara ilk kez takılan klavyeler için uyarı
- **Yeni:** Beyaz liste: yalnızca izin verilen cihazlara güven; listede olmayanlar için uyar veya otomatik engelle
- **Yeni:** Olay günlüğü: her takma ve çıkarma; zaman, kullanıcı ve uyarılarla kaydedilir, CSV/JSON olarak dışa aktarılır
- **Yeni:** "Uyarılar" filtresi, tabloda risk simgeleri ve ayrıntı panelinde güvenlik kutusu
- **İyileştirme:** Güvenlik uyarıları uygulama tepsideyken de Windows bildirimi olarak gelir

<details><summary>English</summary>

- **New:** BadUSB warning: flags devices acting as both keyboard and storage, and programmable boards that can pretend to be keyboards (Arduino, Digispark, Pico…)
- **New:** New keyboard warning for keyboards connected for the first time
- **New:** Allowlist: trust only approved devices; warn about or automatically block the rest
- **New:** Event log: every plug and unplug recorded with time, user and alerts; export to CSV/JSON
- **New:** "Alerts" filter, risk icons in the table and a security box in device details
- **Improved:** Security alerts arrive as Windows notifications even while the app is in the tray

</details>

## 0.2.5 — 2026-11-16

- **Yeni:** Uygulama içinden tek tıkla güncelleme: indirir, SHA-256 ile doğrular, kurar ve yeniden açılır
- **Yeni:** Güncelleme indirilirken ilerleme çubuğu
- **İyileştirme:** Güncelleme ayarları ve kısayollar korunarak kurulur
- **İyileştirme:** Kurum ve okul ağlarında güvenli bağlantı için Windows sertifika deposu kullanılır

<details><summary>English</summary>

- **New:** One-click in-app updates: downloads, verifies (SHA-256), installs and restarts
- **New:** Progress bar while the update downloads
- **Improved:** Updates keep your settings and shortcuts
- **Improved:** Uses the Windows certificate store, so updates also work behind corporate / school network proxies

</details>

## 0.2.4 — 2026-11-16

- **Yeni:** Sürüm notları: Ayarlar → Güncellemeler bölümünde tüm değişiklikler listelenir
- **Yeni:** Güncellemeden sonraki ilk açılışta "Yenilikler" penceresi gösterilir
- **İyileştirme:** Tüm bağlantılar ve kurulum sihirbazı usblens.com adresine taşındı
- **İyileştirme:** Kurulum tek dosya: yalnızca setup.exe (MSI kaldırıldı, çift kurulum sorunu önlendi)

<details><summary>English</summary>

- **New:** Release notes: every change is listed in Settings → Updates
- **New:** A "What's new" window appears on the first launch after an update
- **Improved:** All links and the setup wizard now point to usblens.com
- **Improved:** Single installer: setup.exe only (MSI removed to prevent duplicate installs)

</details>

## 0.2.3 — 2026-11-16

- **İyileştirme:** Güncelleme denetimi doğrudan GitHub Releases üzerinden yapılır
- **Yeni:** usblens.com: İndir düğmesi en son sürümü doğrudan indirir
- **Düzeltme:** "Güncelleme sunucusuna ulaşılamadı" hatası giderildi

<details><summary>English</summary>

- **Improved:** Update check now reads GitHub Releases directly
- **New:** usblens.com: the Download button downloads the latest version directly
- **Fixed:** Fixed the "Could not reach the update server" error

</details>

## 0.2.2 — 2026-11-16

- **Yeni:** Kenar çubuğu simge moduna daraltılabilir (Ctrl+B)
- **Yeni:** Modern, Türkçe kurulum sihirbazı; kurulum yönetici izni istemez
- **Yeni:** Kod imzalama altyapısı (SignPath Foundation / kendi sertifikan)

<details><summary>English</summary>

- **New:** The side panel can collapse to an icon rail (Ctrl+B)
- **New:** Modern setup wizard in Turkish and English; no admin rights needed to install
- **New:** Code signing support (SignPath Foundation / own certificate)

</details>

## 0.2.1 — 2026-11-16

- **Düzeltme:** Alt çubuğun yukarı kayıp altta boşluk kalması düzeltildi

<details><summary>English</summary>

- **Fixed:** Fixed the status bar floating up and leaving a gap at the bottom

</details>

## 0.2.0 — 2026-11-16

- **Yeni:** Ayarlar: her zaman üstte, tepsiye küçült, Windows ile başlat, küçültülmüş başlat
- **Yeni:** Takılma / çıkarılma için sesli uyarı (kısa ton veya cihaz adını sesli okuma)
- **Yeni:** Depolama aygıtı takılınca uygulamayı öne getir / sürücüyü Gezgin'de aç
- **Yeni:** Sistem tepsisi simgesi, güncelleme denetimi ve Hakkında sayfası

<details><summary>English</summary>

- **New:** Settings: always on top, minimize to tray, start with Windows, start minimized
- **New:** Sound alerts on plug / unplug (chime or the device name read aloud)
- **New:** Bring the app to front / open the drive in Explorer when storage is plugged in
- **New:** Tray icon, update check and About page

</details>

## 0.1.0 — 2026-11-16

- **Yeni:** İlk sürüm: bağlı ve geçmişte bağlanmış USB cihazları, arama, filtreler, zaman çizelgesi
- **Yeni:** Etkinleştir / devre dışı bırak, güvenli çıkar, kaldır
- **Yeni:** Canlı takılma bildirimi; HTML, CSV ve JSON rapor (SHA-256)

<details><summary>English</summary>

- **New:** First release: current and past USB devices, search, filters, timeline
- **New:** Enable / disable, safely remove, uninstall
- **New:** Live plug-in alerts; HTML, CSV and JSON reports (SHA-256)

</details>
