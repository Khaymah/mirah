# ADR-0004: مرآة بتشيل نسخة ConPTY الجديدة جواها على ويندوز

- **Status:** Accepted (2026-11-05)، ومستني تأكيد المالك في تجربة `0.1.0-alpha.4`
- **Date:** 2026-11-05
- **Deciders:** Sabry Dawood (المالك)
- **Project:** mirah
- **Tags:** windows, conpty, distribution, security

## السياق

- **تسجيل alpha.3 الحقيقي** (`mirah-session-2.vt`، 218,666 byte) فيه سطور بايظة، زي `│└──drwxr-xr-x┴1─PC─197121` و`قوسلعسوأو` بدل `قوس عسوأو`.
- **مرآة مش السبب.** الـ bytes البايظة موجودة في الـ stream نفسه اللي وصل لمرآة من الـ pty، قبل أي رسم. ولقيت أول byte بايظة بالـ bisect عند offset ‏139,083.
- **مصدرها ConPTY القديم اللي جوه ويندوز.** `portable-pty` بيستخدم `CreatePseudoConsole` من `kernel32.dll`. النسخة دي بتحتفظ بـ buffer خاص بيها، وبترسمه من جديد بـ VT من عندها. والأغلب إن عرض بعض الحروف عندها مختلف عن اللي Claude فاكره، فالنتيجة سطور متلخبطة.
- **Windows Terminal مابيقابلش المشكلة دي من غير مرآة**، لأنه بيشيل نسخته الجديدة من ConPTY (`OpenConsole.exe`)، واللي من 1.22 بقت بتعدّي الـ VT زي ما هو تقريباً (حسب release notes بتاعة Windows Terminal 1.22).
- **Microsoft بتنشر النسخة الجديدة** في NuGet package اسمها `Microsoft.Windows.Console.ConPTY` بترخيص MIT. فيها `conpty.dll` و`OpenConsole.exe` لـ x64 وARM64.
- **`portable-pty` بيدوّر على `conpty.dll` الأول** (`load_conpty` في `psuedocon.rs`)، ولو مالقاهوش بيرجع لـ `kernel32`. و`conpty.dll` بيدوّر على `OpenConsole.exe` جنبه.

## القرار

1. **النسخة `1.25.260930003`** من الـ NuGet package (أحدث نسخة stable) تتحط في `crates/mirah/vendor/conpty/{x64,arm64}/`، ومعاها `LICENSE` بتاع Microsoft و`SHA256SUMS`.
2. **الملفين بيتحطوا جوه `mirah.exe` نفسه** بـ `include_bytes!`، نسخة المعمارية بتاعة الـ build بس (حوالي 1.2 MB).
3. **أول ما مرآة تشتغل على ويندوز** (`host/windows/conpty.rs`):
   - بتكتب `conpty.dll` و`OpenConsole.exe` و`LICENSE.txt` في `%LOCALAPPDATA%\mirah\conpty\<version>-<arch>\`، **لو مش موجودين أو محتواهم مختلف** عن اللي جوه الـ binary. الكتابة بتحصل في ملف مؤقت وبعدين rename.
   - بتعمل `LoadLibraryW` بالمسار الكامل قبل أي pty. وبعد كده لما `portable-pty` يطلب `conpty.dll` بالاسم، ويندوز بيرجّع نفس الـ module اللي اتحمّل.
   - **لو أي خطوة فشلت**، بيتكتب warning في الـ log، ومرآة بتكمّل بـ ConPTY القديم. يعني الحل ده عمره ما يمنع مرآة إنها تشتغل.
4. **`MIRAH_CONPTY=system`** بيرجّع النسخة اللي جوه ويندوز، عشان لو حاجة باظت نقدر نقارن.
5. **اختبار ويندوز في الـ CI** (`tests/run.rs`) بيتأكد إن الـ log فيه `using the bundled conpty`.

## البدائل

| الخيار | المميزات | العيوب |
|---|---|---|
| **جوه الـ binary وتتفك في `%LOCALAPPDATA%`** ← المختار | ملف واحد، وبيشتغل مع `cargo install`، ومع أي updater بيبدّل `mirah.exe` بس | حجم الـ binary بيزيد 1.2 MB، وفيه كتابة على الديسك أول مرة |
| الملفين جنب `mirah.exe` في الـ zip | مفيش extraction | dist مابيدعمش ملفات مختلفة لكل target، ولو حد نقل `mirah.exe` لوحده الحل بيقع من غير ما يحس |
| تحميل الـ NuGet في الـ CI بدل الـ vendor | الريبو من غير binaries | الـ build من الكود من غير نت مش هيبقى فيه الحل، والـ release.yml متولّد من dist فالخطوة الزيادة صعبة |
| نسيب ConPTY القديم ونصلّح في مرآة | من غير ملفات Microsoft | الـ bytes بايظة قبل ما توصل لمرآة، فمفيش حاجة نصلّحها |

## Quality Gate

- **الأمان:**
  - الملفات جاية من nuget.org على HTTPS، والـ package عليها توقيع NuGet (`.signature.p7s`)، بس أنا ماتحققتش من التوقيع نفسه. الـ SHA-256 بتاع الـ package `02b07b349af66d801159bdf9e440d4a1ce78bb951f37fc8609731665afdae7ee`، وبتاع كل ملف في `SHA256SUMS`.
  - مرآة بتقارن المحتوى بالـ bytes اللي جواها كل مرة، فلو حد غيّر الملف على الديسك بيتكتب تاني.
  - `%LOCALAPPDATA%` خاص باليوزر. اللي يقدر يكتب فيه يقدر يغيّر `~\.mirah\bin\mirah.exe` نفسه، فمفيش صلاحية جديدة اتفتحت.
  - الـ `unsafe` الجديد هو `LoadLibraryW` بس، في ملف واحد ومعاه سبب.
- **الترخيص:** MIT. نص الترخيص بيتكتب جنب الملفات اللي اتفكّت، وموجود في الريبو.
- **التكلفة:** 2.4 MB في الريبو، و1.2 MB في كل binary لويندوز، وكتابة حوالي 1.2 MB على الديسك أول مرة بس.
- **الرجوع:** `MIRAH_CONPTY=system` من غير build جديد.

## النتايج

- ✅ الـ stream اللي بيوصل لمرآة على ويندوز هيبقى نفس اللي Claude كتبه تقريباً، زي ما بيحصل في Windows Terminal من غير مرآة.
- ⚠️ تحديث ConPTY بقى شغلنا: لما Microsoft تطلّع نسخة جديدة، نبدّل الملفات والـ `SHA256SUMS` و`VERSION` في `conpty.rs`.
- ⚠️ **ماتجربش على ويندوز حقيقي لسه.** الـ CI بيتأكد إن الـ module اتحمّل والبرنامج اشتغل وخرج صح بس. والتأكيد إن السطور البايظة اختفت محتاج تسجيل جديد من المالك.

## المخاطر

- 🚨 **النسخة الجديدة مختلفة في التصرف** (مثلاً بتعدّي الـ VT من غير ما ترسمه من جديد)، وده ممكن يغيّر حاجات زي الرد على `ESC[6n`. الحل: اختبار الـ CI على ويندوز، وتسجيل المالك، و`MIRAH_CONPTY=system`.
- ⚠️ **antivirus** ممكن يشتكي من exe بيتكتب في `%LOCALAPPDATA%`. الملف من Microsoft، فده مستبعد، بس لو حصل بيبان في الـ log ومرآة بتكمّل بالقديم.

## المراجع

- `microsoft/terminal`: `src/winconpty/winconpty.cpp` (بيدوّر على `OpenConsole.exe` جنب `conpty.dll`، وبعدين في `x64\` أو `arm64\`).
- `portable-pty 0.9`: `src/win/psuedocon.rs` (`load_conpty`).
- WezTerm بيعمل نفس الحكاية: بيشيل `conpty.dll` و`OpenConsole.exe` جنب الـ binary.
- `Plan/Progress/2026-11-05.md` قسم 23.
