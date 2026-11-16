# UsbLens

USBDeview'den ilham alan, modern arayüzlü, açık kaynak bir **Windows USB cihaz görüntüleyici**.
Bilgisayara şu an bağlı olan ve **geçmişte bağlanmış** tüm USB cihazlarını tarihleri, seri numaraları
ve sürücü harfleriyle listeler.

*A modern, open-source USB device viewer for Windows, inspired by NirSoft USBDeview.*

## Özellikler (v0.1)

- **Liste + arama + filtre:** Bağlı / geçmiş / devre dışı, cihaz türü (depolama, telefon, HID, kamera…), anlık arama (ad, seri no, `VID:PID`, sürücü harfi). Sütunlara tıklayarak sıralama.
- **Tarihler:** İlk kurulum, son takılma, son çıkarılma (Windows PnP özelliklerinden).
- **Zaman çizelgesi:** Hangi gün hangi cihazın takıldığını/çıkarıldığını gün gün gösterir.
- **Cihaz işlemleri:** Etkinleştir / devre dışı bırak, güvenli çıkar, kaldır (geçmiş kayıtlar dahil).
- **Canlı bildirim:** USB takılınca/çıkarılınca uygulama içi uyarı + Windows bildirimi, liste otomatik yenilenir.
- **Dışa aktarma:** HTML rapor (yazdır → PDF), CSV (Excel uyumlu, Türkçe karakterli) ve JSON. Rapora bilgisayar adı, kullanıcı, zaman damgası ve **SHA-256** özeti eklenir.
- **Ayarlar:** Her zaman üstte, kapatınca sistem tepsisine küçült, Windows ile başlat, küçültülmüş başlat; Windows bildirimi, takılınca/çıkarılınca sesli uyarı (kısa ton veya cihaz adını sesli okuma, ses düzeyi); depolama aygıtı takılınca uygulamayı öne getir / sürücüyü Gezgin'de aç; açılışta güncelleme denetimi.
- Sistem tepsisi simgesi (Göster / Ayarlar / Çıkış), tek örnek çalışma (ikinci kez açınca mevcut pencere öne gelir).
- Koyu/açık tema, Türkçe/İngilizce arayüz, `Ctrl+F` arama, `F5` yenile.

## Mimari

```
src/                 React + TypeScript arayüz (Vite)
src-tauri/src/
  lib.rs             Tauri komutları (list_devices, set_device_enabled, eject_device, export_devices…)
  model.rs           Ortak veri modeli, VID/PID/seri ayrıştırma, sınıflandırma (+ birim testleri)
  export.rs          CSV / JSON / HTML rapor üretimi
  platform/win.rs    Windows arka ucu: SetupAPI + CfgMgr32 (windows-sys)
  platform/mock.rs   Windows dışı sistemler için örnek veri (arayüz geliştirme)
```

Windows tarafında veri şu kaynaklardan gelir:

| Ne | Nasıl |
|---|---|
| Cihaz listesi (geçmiş dahil) | `SetupDiGetClassDevsW("USB" / "USBSTOR")`, `DIGCF_PRESENT` olmadan |
| Ad, sınıf, servis, tarihler | `SetupDiGetDevicePropertyW` + `DEVPKEY_Device_*` (`FirstInstallDate`, `LastArrivalDate`, `LastRemovalDate`, `IsPresent`, `ConfigFlags`…) |
| Sürücü harfi | `\\.\X:` → `IOCTL_STORAGE_GET_DEVICE_NUMBER` ↔ disk arayüzü → `CM_Get_Parent` ile USB üst cihazı |
| Devre dışı / etkin | `DIF_PROPERTYCHANGE` (`DICS_ENABLE` / `DICS_DISABLE`) |
| Kaldır | `DiUninstallDevice` |
| Güvenli çıkar | `CM_Request_Device_EjectW` |
| Canlı izleme | `CM_Register_Notification` (`GUID_DEVINTERFACE_USB_DEVICE`) → `usb-event` |

> Tarih özellikleri (`LastArrivalDate` vb.) Windows 8 ve sonrasında vardır. Uygulama Windows 10/11 hedefler.

## Gereksinimler

- Windows 10/11
- [Rust](https://rustup.rs) (stable, MSVC araç zinciri) ve Visual Studio C++ Build Tools
- Node.js 20+
- WebView2 (Windows 11'de hazır gelir)

## Çalıştırma

```bash
npm install
npm run tauri dev       # geliştirme
npm run tauri build     # kurulum dosyası: src-tauri/target/release/bundle/{nsis,msi}
```

Sadece arayüz üzerinde çalışmak için (Windows gerekmez, örnek veriyle açılır):

```bash
npm run dev             # http://localhost:1420
```

Testler ve Windows kodunu başka bir işletim sisteminde tip kontrolü:

```bash
cd src-tauri
cargo test
RUSTFLAGS="--cfg usblens_check" cargo check   # win.rs'i Linux/macOS'ta derler (bağlamadan)
```

## Güncelleme denetimi

Uygulama `https://yunuskaya.com.tr/usblens/latest.json` adresindeki küçük bir JSON dosyasını okur
(örnek: `site/usblens/latest.json`). Yeni sürüm çıkarınca:

1. Kurulum dosyasını `https://yunuskaya.com.tr/usblens/` altına yükleyin.
2. `latest.json` içindeki `version`, `url`, `notes`, `date` alanlarını güncelleyin.

Güvenlik için yalnızca `https://yunuskaya.com.tr/` ve `https://github.com/` bağlantıları açılır.
Ayarlar `%APPDATA%\com.yunus.usblens\settings.json` dosyasında saklanır.

## Yönetici yetkisi

Listeleme ve güvenli çıkarma standart kullanıcıyla çalışır. **Devre dışı bırakma ve kaldırma** yönetici
yetkisi ister. Alt çubuktaki "Yönetici olarak yeniden başlat" bağlantısı uygulamayı UAC ile yeniden açar.

## Yol haritası

- [ ] WPD (MTP telefon) ve Bluetooth cihazları için ek numaralandırma
- [ ] `setupapi.dev.log` ayrıştırma (kayıt defteri temizlenmiş olsa bile ilk takılma tarihi)
- [ ] Uzak bilgisayar / çevrimdışı kayıt defteri (`SYSTEM` hive) inceleme
- [ ] Beyaz liste: yalnızca izinli seri numaralarına izin ver, diğerlerinde uyar
- [ ] Doğrudan PDF dışa aktarma, imzalı rapor

## Geliştirici

Yunus Kaya — [yunuskaya.com.tr](https://yunuskaya.com.tr)

## Lisans

MIT
