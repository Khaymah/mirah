# ADR-0003: ويندوز أولوية، والتوزيع بـ dist على GitHub Releases، والتحديث بإذن المستخدم

- **Status:** Proposed
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
   - **تنبيه مرة في اليوم:** thread جانبي بيسأل GitHub، ولو فيه نسخة أحدث بيطبع سطر واحد **بعد** ما مرآة تخرج وترجّع الترمنال لحالته. بيتقفل بـ `MIRAH_NO_UPDATE_CHECK=1`.

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
