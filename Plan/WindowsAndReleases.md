# خطة: ويندوز والـ Releases والتحديث

> القرار وأسبابه في `Plan/ADR/0003-windows-first-releases-and-updates.md`. الحالة: **اتقبلت. W1 اتنفّذ.**
>
> **قرارات المالك:** الريبو يفضل private دلوقتي. والتحديث: تنبيه، و`mirah update`، و**فحص أول ما مرآة تشتغل وزرار تثبيت**.

## الترتيب
1. **W1** ويندوز يتبني ويشتغل. ده أهم حاجة.
2. **W2** releases على GitHub.
3. **W3** `mirah update` وتنبيه التحديث.
4. **W4** تجربة على جهاز المالك.

---

## W1: دعم ويندوز

**W1.1 تغيير مقاس الشاشة على كل نظام**
- `crates/mirah/src/session/resize.rs`:
  - `#[cfg(unix)]`: يفضل SIGWINCH بـ `signal-hook`.
  - `#[cfg(windows)]`: thread بيقرا `crossterm::terminal::size()` كل `RESIZE_POLL_INTERVAL = 200ms`، ويبعت `Event::Resize` لما المقاس يتغير بس.
- `signal-hook` يبقى dependency لـ Unix بس.
- **حالات حرجة:** المقاس يرجع صفر وقت الـ minimize، فنتجاهله. والـ thread يقف لما الـ channel يتقفل.

**W1.2 الكيبورد على ويندوز**
- `crates/mirah/src/host/windows.rs`، وده الملف الوحيد اللي فيه `#[allow(unsafe_code)]`:
  - بيقرا `GetConsoleMode` بتاع stdin.
  - بيضيف `ENABLE_VIRTUAL_TERMINAL_INPUT`، ويرجّع الـ mode القديم في `restore()`، اللي بيتنادي من الـ Drop ومن الـ panic hook.
- `host.rs` يتحوّل لـ `host/mod.rs` + `host/windows.rs`.
- الـ workspace lint: `unsafe_code = "deny"` بدل `forbid`.
- **حالة حرجة:** stdin مش console (pipe مثلاً). وقتها نسيب الـ mode زي ما هو ونكمّل.

**W1.3 القيم الافتراضية لكل نظام**
- `command.rs`: الـ shell الافتراضي على ويندوز `%COMSPEC%`، ولو مش موجود `cmd.exe`. على Unix `$SHELL`، ولو مش موجود `/bin/sh`.
- `logging.rs`: على ويندوز `%LOCALAPPDATA%\mirah\mirah.log`.
- **حالة حرجة:** مسار فيه مسافات أو حروف عربي.

**W1.4 Windows Terminal و`WT_SESSION`**
- flag اسمه `--keep-env WT_SESSION`. الافتراضي يتحدد **بعد** تجربة W4، ومش هنخمّن.

**W1.5 الاختبارات والـ CI**
- `tests/run.rs`: على ويندوز يشغّل `cmd /c "echo مرحبا & exit 3"`.
- `.github/workflows/ci.yml`: `windows-latest` يتضاف للـ matrix.
- **التحقق:** الـ CI أخضر على التلات أنظمة.

---

## W2: الـ Releases

**W2.1 dist**
- نسطّب `dist` محلياً، ونشغّل `dist init` بالإعدادات دي:
  - **targets:** `x86_64-pc-windows-msvc`، و`aarch64-pc-windows-msvc`، و`x86_64-apple-darwin`، و`aarch64-apple-darwin`، و`x86_64-unknown-linux-gnu`، و`aarch64-unknown-linux-gnu`.
  - **installers:** `powershell` و`shell`.
  - `github-attestations = true`.
  - `install-updater = false`، لأننا هنستخدم المكتبة جوه مرآة نفسها.
- الـ workflow اللي بيطلع (`release.yml`): الـ actions تتثبت على SHA، و`permissions` أقل حاجة (`contents: write` و`id-token: write` و`attestations: write` بس).

**W2.2 النسخ**
- `version` في `[workspace.package]` يبقى `0.1.0`، وأول tag يبقى `v0.1.0` كـ pre-release.
- `CHANGELOG.md` بالعربي، وفيه الإنجليزي تحت كل قسم.

**W2.3 التوثيق**
- قسم التثبيت في `README.md` و`README.en.md`:
  - **ويندوز:** سطر PowerShell، أو ينزّل `mirah.exe` من صفحة الـ Releases.
  - **macOS وLinux:** سطر shell.
  - **إزاي تتأكد من الملف:** `gh attestation verify`.

---

## W3: التحديث

**W3.1 `mirah update`**
- `crates/mirah/src/update/mod.rs` بيستخدم `axoupdater`. بيبص على آخر release في `Khaymah/mirah`، ويقارن النسخ، ويتأكد من الـ checksum، ويركّب.
- **حالات حرجة:**
  - مفيش نت → رسالة واضحة، والـ exit code 1.
  - النسخة الحالية هي الأحدث → "إنت على آخر نسخة".
  - النسخة الحالية أحدث من آخر release → مفيش downgrade.
  - مرآة متركّبة من غير الـ installer (اتعملها build من الكود) → رسالة بتقول إزاي تحدّث.

**W3.2 التنبيه وزرار التثبيت (معدّل حسب قرار المالك)**
- مرآة بتفحص قبل ما البرنامج اللي جوه يشتغل. لو فيه نتيجة أقل من ساعة بتستخدمها، ولو لأ بتفحص بـ timeout ثانيتين. لو فيه نسخة جديدة بيظهر سطر فيه **[تثبيت]** و**[بعدين]**. ده في `crates/mirah/src/update/prompt.rs`.
- **الريبو private:** الـ token بييجي من `MIRAH_GITHUB_TOKEN` أو `gh auth token`. لو مفيش، الفحص بيتلغي في صمت.
- **حالات حرجة:** stdin مش ترمنال، أو المستخدم داس Ctrl-C على الزرار، أو التثبيت فشل → مرآة بتكمّل بالنسخة الحالية.

**النسخة الأولى من W3.2 (اتغيرت)**
- `crates/mirah/src/update/check.rs`:
  - thread جانبي بيشتغل مرة في اليوم بالكتير. وقت آخر فحص بيتحفظ في `state/last-update-check`.
  - timeout 3 ثواني.
  - لو فيه نسخة أحدث، بيطبع سطر واحد على stderr **بعد** ما الترمنال يرجع لحالته.
  - بيتقفل بـ `MIRAH_NO_UPDATE_CHECK=1`.
- **حالة حرجة:** مرآة خرجت قبل ما الفحص يخلص. نسيبه، ومانستناش.

**W3.3 الأمان**
- مفيش أي URL للتحميل غير GitHub Releases بتاع الريبو.
- قبل ما نشغّل أي installer، الـ SHA-256 لازم يطابق.
- كل حاجة بتتسجّل في الـ log: النسخة القديمة، والجديدة، ونتيجة الـ checksum.

---

## W4: تجربة على جهاز المالك (ويندوز)

| # | الخطوة | النتيجة المتوقعة |
|---|---|---|
| 1 | تثبيت من الـ release (`v0.1.0`) | `mirah --version` بيشتغل |
| 2 | `mirah claude` في Windows Terminal، ومن غير ما تحدد أي flag | العربي ظاهر صح، **مش متعكس مرتين** |
| 3 | لو اتعكس مرتين: `mirah --strip-env WT_SESSION claude` | لو اتحل بكده، يبقى ده الافتراضي على ويندوز |
| 4 | تكتب عربي وتحرّك بالأسهم | المؤشر في مكانه، والسطر سليم |
| 5 | تكبّر وتصغّر الشاشة | الرسم بيتظبط |
| 6 | `mirah update` (بعد ما ننزّل `v0.1.1`) | بيتحدّث |
| 7 | `MIRAH_LOG=debug` وتبعت الـ log | عشان نحلّل أي مشكلة |

## برا الخطة دي
- توقيع الـ exe (Azure Trusted Signing).
- winget وscoop وHomebrew.
