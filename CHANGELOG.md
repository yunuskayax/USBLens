# Değişiklik günlüğü / Changelog

Bu dosya `src/changelog.json` dosyasından üretilir (`node scripts/changelog.mjs md`).

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
