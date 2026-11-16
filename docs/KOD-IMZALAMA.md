# Kod imzalama kurulumu

İmzasız dosyalarda Windows "Bilinmeyen yayımcı" uyarısı gösterir. GitHub Actions iş akışı
(`.github/workflows/build.yml`) iki imzalama yöntemini destekler. Hangisi ayarlanmışsa onu
otomatik kullanır; hiçbiri yoksa imzasız derler.

| Yöntem | Ücret | Yayımcı adı | Not |
|---|---|---|---|
| **SignPath Foundation** | Ücretsiz (açık kaynak) | SignPath Foundation | Başvuru ve onay gerekir |
| **Kendi sertifikan (PFX)** | ~150–300 $/yıl | Yunus Kaya | Bulut imzalama veya PFX dışa aktarılabilen sertifika gerekir |

> İmzalı dosyalarda da SmartScreen, dosya yeterince indirilip itibar kazanana kadar bir süre uyarı
> gösterebilir. Uyarının hiç çıkmaması için Microsoft Store (MSIX) yolu gerekir.

---

## A) SignPath Foundation (önerilen, ücretsiz)

### 1. Başvuru
1. <https://signpath.org> → **Apply**. Proje: `https://github.com/yunuskayax/USBLens`
2. Başvuruda istenenler bu repoda hazır:
   - OSI onaylı lisans: **MIT** (`LICENSE`)
   - Kod imzalama politikası: [`CODE_SIGNING_POLICY.md`](../CODE_SIGNING_POLICY.md) (ekip rolleri + gizlilik)
   - Derleme GitHub'ın kendi sunucularında yapılıyor (GitHub-hosted runner)
3. Onaylanınca README'deki "Free code signing provided by SignPath.io…" cümlesi zorunludur (README'de hazır).

### 2. SignPath'te proje ayarları
Onaydan sonra SignPath panelinde:
1. **Project** oluştur, slug: `usblens`
2. **Trusted build system** olarak GitHub.com'u ekle ve projeye bağla
3. İki **Artifact configuration** oluştur, içeriklerini bu repodan kopyala:
   - slug `exe` → [`.signpath/artifact-configurations/exe.xml`](../.signpath/artifact-configurations/exe.xml)
   - slug `installers` → [`.signpath/artifact-configurations/installers.xml`](../.signpath/artifact-configurations/installers.xml)
4. **Signing policy** slug: `release-signing` (SignPath Foundation sertifikası, elle onay)
5. Bir **CI user** oluşturup **API token** al

### 3. GitHub ayarları
Repo → **Settings → Secrets and variables → Actions**:

| Tür | Ad | Değer |
|---|---|---|
| Secret | `SIGNPATH_API_TOKEN` | SignPath CI kullanıcısının API token'ı |
| Variable | `SIGNPATH_ORGANIZATION_ID` | SignPath organizasyon kimliği (panel adresindeki GUID) |
| Variable | `SIGNPATH_PROJECT_SLUG` | `usblens` *(farklıysa)* |
| Variable | `SIGNPATH_POLICY_SLUG` | `release-signing` *(farklıysa)* |

Bundan sonra her derlemede exe ve kurulum dosyaları imzalama için SignPath'e gönderilir.
SignPath panelinden isteği **onayladığında** iş akışı devam eder.

---

## B) Kendi sertifikan (PFX)

1. Bir sertifika sağlayıcısından **OV kod imzalama sertifikası** al.
   2023'ten beri anahtarlar donanım anahtarında veya bulut HSM'de tutulmak zorunda. GitHub Actions'ta
   kullanabilmek için sağlayıcının **bulut imzalama** veya **PFX olarak teslim** seçeneğini seç.
2. PFX dosyasını base64'e çevir (PowerShell):
   ```powershell
   [Convert]::ToBase64String([IO.File]::ReadAllBytes("sertifika.pfx")) | Set-Clipboard
   ```
3. Repo → **Settings → Secrets and variables → Actions → New repository secret**:
   - `WINDOWS_CERTIFICATE` → panodaki base64 metni
   - `WINDOWS_CERTIFICATE_PASSWORD` → PFX şifresi

İş akışı exe'yi ve kurulum dosyalarını `signtool` ile imzalar ve zaman damgası ekler.

---

## Sürüm yayınlama

```bash
git tag v0.2.1
git push origin v0.2.1
```
Etiket gönderilince iş akışı imzalı kurulum dosyalarıyla **taslak bir Release** oluşturur.
Release sayfasında kontrol edip **Publish** demen yeterli.

> Güvenlik: Sertifika dosyalarını, şifreleri ve token'ları asla repoya koyma; yalnızca GitHub Secrets kullan.
