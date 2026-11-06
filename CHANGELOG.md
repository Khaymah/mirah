# سجل التغييرات · Changelog

العربي هو الأساس، والإنجليزي تحت كل قسم.

## 0.1.0-alpha.10 - 2026-11-06

**الريبو بقى public.**

- **GitHub attestations:** كل ملف في الـ release معاه attestation بيثبت إنه اتبنى من `Khaymah/mirah` في GitHub Actions. ممكن تتأكد بـ `gh attestation verify <الملف> --repo Khaymah/mirah`. والـ action اللي بيعمل ده (`actions/attest`) متثبّت على commit بعينه، زي كل الـ actions التانية.
- **التثبيت بأمر واحد** من غير `gh`: الأمر موجود في الـ README وفي كل release. وبعدها `mirah update` بيكمّل.
- **أول نسخة تقدر تحدّث ليها من alpha.9** بـ `mirah update` أو بزرار Install.

**English:** The repository is public. Every release file now has a GitHub attestation proving it was built from `Khaymah/mirah` in GitHub Actions; check it with `gh attestation verify <file> --repo Khaymah/mirah`. The attest action is pinned to a commit like every other action. Installing takes one command with no `gh` (in the README and in each release), and `mirah update` takes over from there. This is the first version alpha.9 can update to with `mirah update` or the Install button.

## 0.1.0-alpha.9 - 2026-11-06

**`mirah update`.**

- **`mirah update`:**
  - بيدوّر على آخر نسخة، والـ alpha من ضمنها.
  - بينزّلها، **وبيتأكد من الـ SHA-256**، وبيبدّل `mirah` بيها.
  - ولو الـ checksum مش متطابق، مفيش حاجة بتتغير.
  - على ويندوز، الـ exe الشغال بيتغيّر اسمه لـ `.old` لحد المرة الجاية، فالتحديث بيشتغل وانت جوه مرآة.
- **الفحص لما مرآة تبدأ:**
  - النتيجة بتتحفظ ساعة، والفحص ليه timeout ثانيتين.
  - لو فيه نسخة أحدث، بيظهر سطر فيه **Install** و**Later**.
  - Later بتسكّت السؤال عن نفس النسخة 24 ساعة.
  - بيتقفل بـ `MIRAH_NO_UPDATE_CHECK=1`، ومابيظهرش لو stdin أو stdout مش ترمنال.
- **`mirah -- PROGRAM`:** بيشغّل برنامج اسمه زي أمر من أوامر مرآة، زي برنامج اسمه `update`.
- **من غير مكتبات HTTP:** التحميل بـ `curl`، والفك بـ `tar`، وفي ويندوز بالـ `tar.exe` بتاع ويندوز نفسه، لأن الـ GNU tar اللي بييجي مع Git مابيفكش zip. والـ binary على لينكس لسه 2.3 MB.

**English:** `mirah update` finds the newest release (alphas included), downloads it, checks its SHA-256 and replaces `mirah` with it; nothing changes if the checksum does not match. On Windows the running exe is moved to `.old` until the next start, so updating works from inside mirah. At startup mirah checks for a newer version (result kept for an hour, 2-second timeout) and offers **Install** or **Later**; Later quiets that version for 24 hours. `MIRAH_NO_UPDATE_CHECK=1` turns the check off, and it never appears when stdin or stdout is not a terminal. `mirah -- PROGRAM` runs a program named like a mirah command. Downloads use `curl` and unpacking uses `tar` (Windows' own `tar.exe`, since Git's GNU tar cannot unpack zip), with no HTTP library inside mirah; the Linux binary is still 2.3 MB.

## 0.1.0-alpha.8 - 2026-11-06

**frames أقل، ومفيش شاشة نص مرسومة.**

- **Synchronized output:** لما البرنامج يقول إنه بيحدّث الشاشة (`?2026h`)، مرآة بتستنى لحد ما يخلّص (`?2026l`) قبل ما ترسم. Claude بيعمل ده مع كل تحديث. قبل كده، لو التحديث وصل على أكتر من قراية من الـ pty، مرآة كانت بترسم كل حتة لوحدها، فممكن تظهر شاشة نص مرسومة. ولو البرنامج مقفلهوش، مرآة بترسم بعد 150ms على الأكتر.
- **frame كل 8ms على الأكتر:** لو الـ output جاي ورا بعضه بسرعة، مرآة بتجمعه في frame واحد. في الاختبار، 3000 سطر ورا بعض اترسموا في 3 frames بس. ومفيش تأخير زيادة لو الشاشة كانت هادية.
- **الردود على أسئلة البرنامج** (زي `ESC[6n`) بتتبعت على طول، من غير ما تستنى الـ frame. **والـ frame الأخير بيترسم دايماً** قبل ما مرآة تخرج.
- **لو مفيش حاجة اتغيرت، مفيش frame.**

**English:** Fewer frames and no half-drawn screens. When the program announces a synchronized update (`?2026h`), which Claude does for every update, mirah waits for its end (`?2026l`) before drawing, so an update split across several pty reads no longer shows a half-drawn screen; an update that never ends is drawn after at most 150ms. Frames are at most one every 8ms, so a burst of output is coalesced (3000 lines in a row drew 3 frames in testing) with no added delay when the screen was idle. Replies to the program's queries are still sent immediately, the last frame is always drawn before exit, and nothing is drawn when nothing changed.

## 0.1.0-alpha.7 - 2026-11-06

**أسرع.** نفس الشكل بالظبط، بس كل frame بقى بياخد وقت أقل.

- على تسجيل حقيقي لـ Claude على ويندوز (51×183)، الوضع `visual` بقى بيستخدم **نص التعليمات** اللي كان بيستخدمها: 928 مليون بقوا 456 مليون. ومتوسط الـ frame نزل من 3.9ms لـ 1.6ms، وأبطأ frame من 7.5ms لـ 2.4ms. والوضع العادي بقى أقل بـ 18%.
- **cache لكل سطر:** السطر اللي مااتغيرش من الـ frame اللي فات مابيتحسبش تاني، حتى لو اتحرك لمكان تاني في الشاشة. والسطور اللي مفيهاش عربي مش بتدخل الـ cache أصلاً، لأن حسابها أرخص من البحث عنها.
- **إرجاع الترتيب المنطقي أخف:** القرايات المكررة مابتتجربش مرتين، واختيار القراية اللي بتحط `│` في مكانها بقى من غير ما ينسخ السطر.
- **`Cell.text` بقى `CompactString`** بدل `String`، فالحرف الواحد مابيحجزش memory. ودي تغيير في الـ API بتاع `mirah-bidi`، و`shape` بقت بتقبل أي `AsRef<str>`.
- اتأكدت إن الشاشات طالعة **زي ما هي بالظبط** على 4 تسجيلات حقيقية، في الوضعين، وبنوعين التشكيل.

**English:** Faster, with identical output. On a real recording of Claude on Windows (51×183), visual mode now uses half the instructions (928M to 456M), the average frame dropped from 3.9ms to 1.6ms and the slowest from 7.5ms to 2.4ms; logical mode uses 18% fewer. Rows that did not change since the last frame are not computed again, even when they moved on screen; rows without Arabic skip the cache because computing them is cheaper than looking them up. Un-reordering skips duplicate readings and checks table frames without copying the row. `Cell.text` is now a `CompactString`, so a single character does not allocate; this changes the `mirah-bidi` API, and `shape` now accepts any `AsRef<str>`. Screens were verified identical on four real recordings, in both input orders and both shaping modes.

## 0.1.0-alpha.6 - 2026-11-06

- **`mirah --record claude`:** بيحفظ `mirah-session-N.vt` و`mirah-session-N.log` في الفولدر اللي انت فيه، بالرقم اللي بعد آخر جلسة، وبيكتب مكانهم لما تخرج. مش محتاج `MIRAH_RECORD` ولا `MIRAH_LOG` ولا تدوّر في `%LOCALAPPDATA%`.
- **ويندوز: البرامج اللي متسطّبة بـ npm** (`opencode` و`gemini` وغيرهم) كانت بتفشل بـ `%1 is not a valid Win32 application`، لأن مرآة كانت بتلاقي ملف الـ shell script اللي من غير امتداد قبل الـ `.cmd`. مرآة بقت بتدوّر بنفسها في الـ PATH بالامتدادات اللي في `PATHEXT`، وبتشغّل الـ `.cmd` و`.bat` عن طريق `cmd.exe /d /c`.
- **وضع `visual` بيتحدد من اسم البرنامج اللي انت كتبته**، فلو Claude متسطّب بـ npm (`claude.cmd`) لسه بيتعرف عليه.

**English:** `mirah --record claude` saves `mirah-session-N.vt` and `mirah-session-N.log` in the current folder, numbered after the last session there, and prints their location on exit. On Windows, programs installed with npm (`opencode`, `gemini` and others) failed with `%1 is not a valid Win32 application` because the extensionless shell script was found before the `.cmd` shim; mirah now searches PATH with `PATHEXT` itself and runs `.cmd`/`.bat` files through `cmd.exe /d /c`. Visual input order is now detected from the program name as typed, so Claude installed through npm is still recognized.

## 0.1.0-alpha.5 - 2026-11-05

تصليحات من أول تسجيل لـ alpha.4 على ويندوز.

- **الكلام اللي Claude بيرتّبه بنفسه على ويندوز:** مرآة بقت بترجّع كل سطر للترتيب المنطقي، **وبتتأكد** إن تشغيل نفس خوارزمية Claude على النتيجة بيطلّع بالظبط اللي اترسم. ولو مش متأكدة، بتسيب السطر زي ما Claude رسمه وبتصلّح الأقواس المعكوسة بس.
  - **الجداول بقت بتتعكس كلها صح.** Claude بيعكس كل صف لوحده حسب أول حرف فيه، فالجدول كان بيطلع نصه معكوس ونصه لأ. مرآة بقت بتاخد أماكن الأعمدة من الإطار اللي فوق الجدول، وبتختار القراية اللي بتحط `│` في مكانها.
  - **الأقواس** بقت بتطلع صح، حتى حوالين كلام إنجليزي جوه العربي، زي `(mirah)`.
- **في كل الترمنالات:**
  - **الـ emoji جوه الجملة** بقى بيتحرك معاها، بعد ما كان بيقسمها نصين.
  - **علامات الـ markdown والأقواس اللازقة في أول كلمة** (`**`، `` ` ``، `(`، `«`) بقت بتتحرك مع الجملة. اللي بيفضل مكانه هو العلامات بس (`●`، `-`، `>`، `1.`).
- **README:** صف `MIRAH_LOG` في جدول الإعدادات كان ناقصه `|`، فالجملة اللي بعده كانت داخلة في الجدول. Claude نفسه هو اللي لقاه وهو شغال جوه مرآة.

**English:** Fixes from the first alpha.4 recording on Windows. For programs that reorder Arabic themselves (Claude Code on Windows), mirah now puts each row back in logical order and verifies that running the program's own algorithm on the result reproduces exactly what was drawn; when it cannot confirm that, it keeps the row as drawn and only fixes reversed brackets. Tables now mirror consistently: the program reverses each row on its own, and mirah uses the column positions of the table's top border to choose the reading that puts every `│` in place. Brackets now come out right, including around Latin words inside Arabic. On every terminal, an emoji inside a sentence now moves with it instead of splitting it, and markdown marks or brackets attached to the first word move with the sentence; only list and prompt markers stay in place. The `MIRAH_LOG` row in the README settings table was missing its closing `|`; Claude found it while running inside mirah.

## 0.1.0-alpha.4 - 2026-11-05

تصليحات من تجربة alpha.3 على ويندوز.

- **السطور البايظة على ويندوز** (زي `│└──drwxr-xr-x┴1─PC`) كان مصدرها ConPTY القديم اللي جوه ويندوز، مش مرآة. مرآة بقت بتشيل جواها النسخة الأحدث من ConPTY (`conpty.dll` و`OpenConsole.exe` من Microsoft، بترخيص MIT)، وبتفكّها في `%LOCALAPPDATA%\mirah\conpty\` أول مرة. ولو عايز ترجع للقديم: `MIRAH_CONPTY=system`. التفاصيل في [ADR-0004](Plan/ADR/0004-bundle-conpty-on-windows.md).
- **Claude Code على ويندوز بيرتّب العربي بنفسه دايماً**، حتى من غير `WT_SESSION`. فمرآة بقت بتعرف إن الكلام جاي بترتيب مرئي، وبترجّعه للترتيب المنطقي الأول، وبعدين بتحاذيه يمين زي أي ترمنال تاني. وكمان الأقواس بقت بتتعكس صح. الوضع ده بيتختار لوحده لـ Claude Code على ويندوز، وممكن تغيّره بـ `MIRAH_INPUT=logical|visual`.
- **اتشال إخفاء `WT_SESSION`** و`MIRAH_PASS_WT_SESSION`، لأنهم كانوا مبنيين على فرضية غلط.

**English:** Fixes from the alpha.3 Windows test. The broken rows on Windows came from the old ConPTY that ships with Windows, not from mirah, so mirah now bundles Microsoft's newer ConPTY (`conpty.dll` and `OpenConsole.exe`, MIT licensed) and extracts it to `%LOCALAPPDATA%\mirah\conpty\` on first run (`MIRAH_CONPTY=system` goes back to the old one). Claude Code on Windows always reorders Arabic itself, so mirah now recognizes visual-order input, puts it back in logical order, then right-aligns it as on any other terminal, with brackets mirrored correctly. This is picked automatically for Claude Code on Windows and can be set with `MIRAH_INPUT=logical|visual`. Hiding `WT_SESSION` and `MIRAH_PASS_WT_SESSION` were removed because they rested on a wrong assumption.

## 0.1.0-alpha.3 - 2026-11-05

تصليحات من أول تجربة حقيقية على Windows Terminal.

- **Windows Terminal:** وضع `plain` بدل `words`. مرآة بتبعت ترتيب مرئي كامل بحروف عربي عادية، وWindows Terminal بيشكّلها صح. اللي اكتشفناه من تسجيل حقيقي إن Windows Terminal مابيعكسش حاجة، بس مابيعرفش يرسم الـ presentation forms.
- **Claude Code في Windows Terminal كان بيرتّب العربي ويعكس الجداول بنفسه**، فالنتيجة كانت بتتعكس مرتين. مرآة بقت بتخفي `WT_SESSION` عن البرنامج اللي جواها وبتحط `TERM=xterm-256color` لو مش متحدد. ولو عايز تمنع ده: `MIRAH_PASS_WT_SESSION=1`.
- **أي سطر فيه عربي اتجاهه يمين-شمال**، حتى لو بيبدأ بإنجليزي (زي `- Go: interfaces ضمنية...`)، زي إضافة RTL في VS Code.
- **العناوين** (bold وأبيض) بقت بتتحاذى يمين، بعد ما كانت بتتحسب كود.
- **output الكود** اللي لازق في بلوك الكود بيفضل مكانه.

**English:** Fixes from the first real Windows Terminal test. Windows Terminal gets a `plain` mode instead of `words`: mirah sends full visual order with plain Arabic letters and Windows Terminal joins them; a real recording showed Windows Terminal reorders nothing but cannot draw presentation forms. Claude Code reorders Arabic and mirrors tables itself when it sees Windows Terminal, which reversed everything twice, so mirah now hides `WT_SESSION` from the program and sets `TERM=xterm-256color` if unset (`MIRAH_PASS_WT_SESSION=1` keeps it). Any row containing Arabic uses a right-to-left base direction even when it starts with Latin text. Bold white headings now align right instead of counting as code, and uncolored output inside a code block stays in place.

## 0.1.0-alpha.2 - 2026-11-05

العرض بقى من اليمين للشمال.

- **الكلام يتحاذى يمين** (عربي أو إنجليزي)، و**علامات أول السطر** (`●` و`•` و`-` و`1.`) بتتنقل معاه. الـ indentation بقى هامش على اليمين.
- **الكود، والـ diff، وoutput الأوامر (`⎿`)، وخانة الكتابة، واللي تحتها بيفضلوا مكانهم.** والجلسات اللي مفيهاش عربي خالص مابتتلمسش.
- **الجداول اللي فيها عربي بتتعكس:** أول عمود على اليمين.
- **العربي جوه الإطارات** بيتحاذى يمين جوه الإطار.
- **Windows Terminal:** وضع `words`. مرآة بترتّب الكلمات بس، والترمنال بيقلب الحروف ويشكّلها بنفسه. ده بيصلّح العكس مرتين اللي كان في alpha.1.
- **المحررات** (زي vim) مابتتحاذاش، عشان السطور ماتنطّش مع المؤشر.
- الإعدادات: `MIRAH_ALIGN=prose|arabic|off`، و`MIRAH_SHAPING=full|words`، و`MIRAH_RECORD=<ملف>` لتسجيل output البرنامج عشان الـ bug reports.

**English:** Display is now right to left. Prose (Arabic or English) aligns right and its list markers move with it; indentation becomes a right margin. Code, diffs, tool output (`⎿`), the input box and everything below it stay in place, and screens with no Arabic are untouched. Tables that contain Arabic are mirrored so the first column is on the right. Arabic inside frames aligns right inside the frame. Windows Terminal gets a `words` mode: mirah orders words only and the terminal reverses and joins the letters itself, which fixes the double reversal in alpha.1. Editors such as vim are not aligned so rows do not jump with the cursor. New settings: `MIRAH_ALIGN`, `MIRAH_SHAPING`, and `MIRAH_RECORD` to save a program's output for bug reports.

## 0.1.0-alpha.1 - 2026-11-05

أول نسخة تجريبية.

- مرآة بتشغّل أي برنامج terminal جواها، وبتصلّح العربي والفارسي والعبري: بتقسّم السطر، وتطبّق UAX #9، وتوصّل الحروف ومعاها "لا".
- الإطارات والأعمدة بتفضل في مكانها، والإنجليزي والأرقام جوه الكلام العربي بيفضلوا من الشمال لليمين.
- المؤشر الوهمي اللي Claude Code وبرامج Ink بيرسموه بيظهر في مكانه الصح، والسطر بيفضل سليم.
- بتقفل الـ bidi بتاع الترمنال بـ BDSM، وترجّعه وانت خارج.
- بتشتغل على Linux وmacOS وWindows.
- `MIRAH_LOG=debug` بيكتب log في ملف.

**English:** First alpha. mirah runs any terminal program and fixes Arabic, Persian and Hebrew: row segmentation, UAX #9 reordering and Arabic shaping including lam-alef. Frames and columns stay in place, and Latin text and numbers stay left to right inside Arabic. The fake cursor that Claude Code and Ink programs draw lands in the right place without scrambling the row. mirah turns off the terminal's own bidi with BDSM and restores it on exit. It runs on Linux, macOS and Windows. `MIRAH_LOG=debug` writes a log file.
