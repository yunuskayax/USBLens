# usblens.com — tanıtım sitesi

Site `site/` klasöründe. `main` dalına her gönderimde **GitHub Pages** ile otomatik yayınlanır
(`.github/workflows/pages.yml`). GitHub Pages ücretsizdir ve HTTPS sertifikasını kendisi verir.

## 1. Pages'i bir kez aç
Repo → **Settings → Pages → Build and deployment → Source: GitHub Actions**.
Sonra **Actions → site → Run workflow**. Site şu adreste açılır:
`https://yunuskayax.github.io/USBLens/`

## 2. Alan adını bağla (usblens.com alınınca)
1. Alan adı sağlayıcının DNS ayarlarına şu kayıtları ekle:

   | Tür | Ad | Değer |
   |---|---|---|
   | A | `@` | `185.199.108.153` |
   | A | `@` | `185.199.109.153` |
   | A | `@` | `185.199.110.153` |
   | A | `@` | `185.199.111.153` |
   | CNAME | `www` | `yunuskayax.github.io` |

2. Repo → **Settings → Pages → Custom domain** → `usblens.com` → **Save**.
3. DNS yayılınca (birkaç dakika–birkaç saat) **Enforce HTTPS** kutusunu işaretle.

> İsteğe bağlı: GitHub hesabında **Settings → Pages → Verified domains** bölümünden alan adını
> doğrulamak, başkasının bu alan adını kendi Pages sitesine bağlamasını engeller.

## Site içeriği
- İndir düğmesi, GitHub'daki son sürümün kurulum dosyasına otomatik bağlanır.
- Ekran görüntüleri `docs/screenshots/` klasöründen alınır; README ile aynıdır.
- Türkçe / İngilizce: ziyaretçinin tarayıcı diline göre açılır, sağ üstten değiştirilebilir.
