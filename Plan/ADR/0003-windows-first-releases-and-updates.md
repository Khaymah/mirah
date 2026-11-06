# ADR-0003: ويندوز أولوية، والتوزيع بـ dist على GitHub Releases، والتحديث بإذن المستخدم

- **Status:** Accepted (2026-11-05)
- **Date:** 2026-11-05
- **Deciders:** المالك (Sabry Dawood)
- **Project:** mirah
- **Tags:** windows, distribution, security, updates

## السياق

- **جهاز المالك ويندوز**، فويندوز بقى أولوية بعد ما كان في ADR-0002 "برا نطاق v1".
- **الكود دلوقتي مش هيتعمله build على ويندوز:** `signal-hook` (SIGWINCH) شغال على Unix بس، ومفيش طريقة نعرف بيها إن مقاس الشاشة اتغيّر على ويندوز.
- **على ويندوز الكيبورد مابيطلّعش escape sequences من نفسه.** من غير `ENABLE_VIRTUAL_TERMINAL_INPUT` الأسهم مش هتوصل للبرنامج اللي جوه مرآة كـ `ESC [ D`.
- **مفيش binary جاهز.** `target/release/mirah` بيطلع لما تعمل build على جهازك بس، ومش في الريبو. المالك عايز build على GitHub مع **auto update**.
- **الريبو private.** ملفات GitHub Releases في ريبو private محتاجة تسجيل دخول عشان تتحمّل.
- **مخاطرة لازم نجرّبها على ويندوز:** Claude Code بيعرف إنه شغال في Windows Terminal عن طريق `WT_SESSION` (اتأكدت من الـ binary v2.1.318)، والتحديث 2.1.74 "صلّح العربي في Windows Terminal". لو Claude Code بيعكس النص بنفسه هناك، ومرآة عكسته تاني، النص هيتعكس مرتين.

## القرار

1. **ويندوز:**
   - مقاس الشاشة: نقراه كل 200ms في thread على ويندوز، وSIGWINCH يفضل على Unix.
   - نشغّل `ENABLE_VIRTUAL_TERMINAL_INPUT` على stdin.
   - الـ shell الافتراضي `%COMSPEC%`، والـ log في `%LOCALAPPDATA%\mirah\`.
   - `windows-latest` يتضاف للـ CI.
2. **التوزيع:** [dist](https://axodotdev.github.io/cargo-dist/) (cargo-dist) بيعمل `release.yml`:
   - أي tag `v*` بيبني Windows (x64 وARM64) وmacOS (x64 وARM64) وLinux (x64 وARM64).
   - بيعمل installer بـ PowerShell لويندوز وبـ shell للباقي.
   - كل ملف معاه SHA-256 و**GitHub artifact attestation**.
3. **التحديث:** ماينزلش أي binary جديد إلا بإذن المستخدم:
   - `mirah update`: بيدوّر على آخر release، ويتأكد من الـ checksum، ويركّبه. بنستخدم مكتبة [axoupdater](https://docs.rs/crate/axoupdater/latest) اللي معمولة لـ dist.
   - **فحص أول ما مرآة تشتغل وزرار تثبيت** (قرار المالك): قبل ما البرنامج اللي جوه يشتغل، مرآة بتفحص لو فيه نسخة أحدث. النتيجة بتتحفظ ساعة، والفحص ليه timeout ثانيتين. لو فيه نسخة جديدة بيظهر سطر فيه زرارين: **[تثبيت]** و**[بعدين]**، ونتحرّك بينهم بالأسهم أو بـ Enter/Esc. لو المستخدم اختار تثبيت، بيحصل نفس اللي `mirah update` بيعمله. بيتقفل بـ `MIRAH_NO_UPDATE_CHECK=1`، ومش بيظهر لو stdin مش ترمنال.
   - **الريبو private** (قرار المالك): الفحص والتحميل محتاجين token. مرآة بتاخده من `MIRAH_GITHUB_TOKEN`، ولو مش موجود من `gh auth token`. ولو مفيش الاتنين، الفحص بيتلغي من غير ما يطبع أي حاجة. الـ token مابيتكتبش في الـ log ولا في أي ملف.

## البدائل

### التوزيع
| الخيار | المميزات | العيوب |
|---|---|---|
| **dist** ← المختار | معمول للـ Rust، وبيعمل installer لويندوز، وفيه checksums وattestations، ومعاه مكتبة تحديث جاهزة | أداة كمان نعتمد عليها. الـ updater بتاعه مكتوب عليه "experimental" في الـ docs |
| workflow نكتبه بإيدنا + `self_update` | تحكّم كامل | هنكتب بنفسنا كل حاجة dist بيعملها: installers وchecksums وattestations وmatrix |
| winget أو scoop بس | ده المكان الطبيعي لأي حد على ويندوز | محتاجين releases الأول أصلاً، فده خطوة بعدين |

### التحديث
| الخيار | المميزات | العيوب |
|---|---|---|
| **تنبيه + `mirah update`** ← المختار | المستخدم شايف إيه اللي بيتغير، ولو حصلت مشكلة في الـ supply chain مش هتوصل للأجهزة لوحدها | المستخدم لازم يكتب أمر |
| تحديث صامت لوحده | مفيش أي مجهود من المستخدم | لو release اتخرّق، كل الأجهزة هتتأثر من غير ما حد يحس. وعلى ويندوز مينفعش نبدّل exe وهو شغال، فهيحتاج حيل إضافية |

### طريقة تشغيل الأسهم على ويندوز
| الخيار | المميزات | العيوب |
|---|---|---|
| **`windows-sys` + ملف `unsafe` واحد** ← المختار | الـ bytes بتوصل زي ما هي، ومعاها bracketed paste والماوس وkitty keyboard | لازم نغيّر الـ lint من `forbid` لـ `deny` ونسمح بـ `unsafe` في الملف ده بس |
| نقرا الـ events بـ crossterm ونرجّعها VT | مفيش `unsafe` خالص | بنخسر دقة: الـ paste والماوس وkitty keyboard هيتحوّلوا من غير ما نحس، وفيه bugs محتملة |

## Quality Gate

| المحور | حمّال؟ | |
|---|---|---|
| 🔒 **Security** | ✅ الأساسي | **التحديث يعني كود جديد بيشتغل على جهاز المستخدم.**<br>• التحميل HTTPS من `github.com/Khaymah/mirah` بس، ومفيش أي مصدر تاني.<br>• SHA-256 لازم يطابق قبل التركيب.<br>• attestations يقدر أي حد يتأكد منها بـ `gh attestation verify`.<br>• مفيش تحديث صامت.<br>• مفيش رجوع لنسخة أقدم من غير أمر صريح.<br>• الـ `GITHUB_TOKEN` في الـ workflow صلاحياته أقل حاجة، والـ actions متثبتة على SHA.<br>• `unsafe` في ملف واحد عليه review، وبينادي 2 Win32 API بس (`GetConsoleMode` و`SetConsoleMode`) |
| 🛡️ **Reliability** | ✅ | مود الـ console بيرجع لحالته الأصلية عند الخروج وعند الـ panic، زي raw mode بالظبط. لو فحص التحديث فشل (مفيش نت، أو GitHub واقع) مش بيأثر على التشغيل، ولا بيعطّل ولا بيطبع حاجة |
| 🔧 **Maintainability** | ✅ | كود ويندوز معزول ورا `#[cfg(windows)]` في `host/windows.rs` و`session/resize.rs`. الـ CI على 3 أنظمة بيمنع أي حد يكسر ويندوز من غير ما يحس |
| 🚀 Performance | جزئي | قراية المقاس كل 200ms مالهاش تكلفة تقريباً. فحص التحديث في thread جانبي، مرة في اليوم بالكتير، ومش في الطريق اللي بيرسم |
| ⚡ Scalability | N/A | أداة محلية |
| 💰 Cost | جزئي | GitHub Actions ببلاش للريبوهات الـ public. لو الريبو فضل private، دقايق build ويندوز وماك بتتحسب بمضاعف. توقيع الـ exe بـ Azure Trusted Signing بفلوس، فبيتأجّل |
| 👁️ Observability | جزئي | فحص التحديث ونتيجته بيتسجّلوا في الـ log |
| 🟢 Availability | N/A | |

**الـ trade-off:** بنتنازل عن **التحديث الصامت المريح** مقابل **إن الـ supply chain يبقى آمن**. وبنتنازل عن **`forbid(unsafe)` في كل حتة** مقابل **إن الكيبورد على ويندوز يوصل صح**، والـ `unsafe` محصور في ملف واحد.

## تعديل بعد القرار (2026-11-05)

- **(اتحلّت في 2026-11-06: الريبو بقى public والـ attestations اشتغلت من `0.1.0-alpha.10`.)** الـ attestations مقفولة طول ما الريبو private. [وثائق GitHub](https://docs.github.com/de/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations) بتقول إنها في الريبوهات الـ private محتاجة GitHub Enterprise Cloud، ومقدرتش أتأكد من خطة Khaymah. التحقق دلوقتي بالـ SHA-256 checksum بس. **لما الريبو يبقى public:** `github-attestations = true` في `dist-workspace.toml`، وبعدها `dist generate`.

## تعديل تاني: أداة التحديث (2026-11-06)

- **المالك:** الريبو هيبقى public، فالتحديث يتعمل على أساس إنه public، ومن غير token.
- **`axoupdater` اتشال من القرار.** قسته في مشروع لوحده:
  - الـ binary بقى 7.4 MB، ومعاه 288 crate، منهم tokio وreqwest. مرآة كلها دلوقتي حوالي 3 MB.
  - بيشغّل الـ installer بتاع dist، والـ installer بتاع PowerShell مابيتأكدش من الـ SHA-256 (بتاع الـ shell بيتأكد). وويندوز هو الأولوية.
  - وجرّبت كمان `ureq` مع rustls: ‏3.5 MB و57 crate، ومعظمهم TLS.
- **القرار الجديد:** `mirah update` بيعمل الخطوات بنفسه، بأدوات موجودة على كل نظام:
  1. `curl` بيجيب قايمة الـ releases من GitHub API، ومعاها الـ prereleases.
  2. `curl` بينزّل الأرشيف والـ `.sha256` بتاعه من `github.com/.../releases/download/`.
  3. مرآة بتتأكد من الـ SHA-256 بنفسها بـ `sha2`. ولو مش متطابق، مفيش تثبيت.
  4. `tar` بيفك الأرشيف. على ويندوز 10 و11 بيفك الـ zip كمان.
  5. الـ exe الشغال بيتغيّر اسمه لـ `.old`، والجديد بيتنسخ مكانه، ولو أي حاجة فشلت القديم بيرجع. وأول ما مرآة تشتغل المرة الجاية، بتمسح الـ `.old`.
  - **الـ dependencies الجديدة:** `semver` و`sha2` بس، ومفيش HTTP ولا TLS جوه مرآة.
  - **المخاطرة:** لازم `curl` و`tar` يبقوا موجودين. ويندوز 10 من نسخة 1803 بيشيلهم، وكمان macOS وLinux. ولو مش موجودين، الرسالة بتقول كده وبتشرح إزاي تحدّث بإيدك.
- **كل باقي القرار زي ما هو:** الفحص أول ما مرآة تشتغل، والزرارين، وcache لمدة ساعة، وtimeout ثانيتين، و`MIRAH_NO_UPDATE_CHECK=1`.

## النتايج
- ✅ مستخدم ويندوز يقدر يثبّت بسطر PowerShell واحد، أو ينزّل `mirah.exe` من صفحة الـ Releases.
- ✅ `mirah update` بيشتغل على الأنظمة التلاتة.
- ⚠️ **أول ما الـ exe ينزل، ويندوز هيطلّع تحذير SmartScreen** لأنه مش موقّع. التوقيع مؤجّل.
- ⚠️ لو الريبو فضل private، الـ installer وفحص التحديث **مش هيشتغلوا** إلا لو المستخدم متسجّل دخول بـ `gh`.

## المخاطر
- 🚨 **Claude Code يعكس النص بنفسه في Windows Terminal، ومرآة تعكسه تاني.** الحل: أول اختبار على جهاز المالك. لو حصل، مرآة تشيل `WT_SESSION` من الـ environment بتاع البرنامج اللي جواها، ويبقى ليه flag يرجّعه.
- 🚨 **ConPTY بيعيد رسم الـ output** قبل ما يوصل لينا. محاكي الشاشة بيفهم الـ VT اللي بيطلع من ConPTY عادي، بس لازم نجرّب. الـ CI بيشغّل اختبار التشغيل الكامل (`tests/run.rs`) على ويندوز.

## خطة التحقق
- الـ CI أخضر على Linux وmacOS وWindows.
- أول release (`v0.1.0`، pre-release) طلع فيه 6 ملفات، والـ checksums والـ attestations بتاعتهم سليمة.
- المالك شغّل `mirah claude` على ويندوز، والعربي ظاهر صح والأسهم شغالة.
- `mirah update` حدّث من `v0.1.0` لـ `v0.1.1` على ويندوز.
- **Review date:** بعد أول release.

## المراجع
- `Plan/WindowsAndReleases.md` (المهام)
- ADR-0002
- [dist: updater](https://axodotdev.github.io/cargo-dist/book/installers/updater.html) · [axoupdater](https://docs.rs/crate/axoupdater/latest) · [dist changelog](https://axodotdev.github.io/cargo-dist/changelog)
