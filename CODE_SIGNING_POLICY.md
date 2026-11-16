# Code signing policy

Windows releases of UsbLens are code-signed so that users can verify they come from this repository
and have not been modified.

> Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).
>
> *(This statement applies once the project is accepted into the SignPath Foundation program. Until then,
> release binaries are unsigned or signed with the maintainer's own certificate, as shown in each release.)*

## What is signed

- `usblens.exe` — the application binary (signed before it is packed into the installers)
- `UsbLens_<version>_x64-setup.exe` / `UsbLens-setup.exe` — installer

Only binaries built by the [GitHub Actions workflow](.github/workflows/build.yml) of this repository
from a commit on `main` or a release tag are signed. Locally built binaries are never signed.

## Team roles

| Role | Members |
|---|---|
| Committers and reviewers | [Yunus Kaya (@yunuskayax)](https://github.com/yunuskayax) |
| Approvers | [Yunus Kaya (@yunuskayax)](https://github.com/yunuskayax) |

Every signing request for a release must be approved manually by an approver.
Contributions from other people are only merged after review by a committer.

## Privacy policy

UsbLens reads USB device information **locally** from Windows. It does not send device lists, serial
numbers, reports or any personal data anywhere.

The only network connection the program makes on its own is the **update check**: it reads the latest
release information from the GitHub API (`api.github.com/repos/yunuskayax/USBLens/releases/latest`),
falling back to `https://usblens.com/latest.json`, to compare version numbers. The request contains only
the standard HTTP headers and the app version (`User-Agent: UsbLens/<version>`). The update check can be
turned off in *Settings → Updates*. Links (website, update download) open in the user's browser only
when the user clicks them.

---

## Kod imzalama politikası (Türkçe özet)

UsbLens'in Windows sürümleri, bu repodan geldiği ve değiştirilmediği doğrulanabilsin diye dijital olarak
imzalanır. Yalnızca bu reponun GitHub Actions iş akışında `main` dalından veya bir sürüm etiketinden
derlenen dosyalar imzalanır; her imzalama isteği onaylayıcı tarafından elle onaylanır.

**Gizlilik:** UsbLens USB bilgilerini yalnızca bilgisayarda okur; cihaz listesi, seri numarası veya kişisel
veri hiçbir yere gönderilmez. Kendiliğinden yaptığı tek bağlantı güncelleme denetimidir
(GitHub Releases, yedek olarak `usblens.com/latest.json`); *Ayarlar → Güncellemeler* bölümünden kapatılabilir.
