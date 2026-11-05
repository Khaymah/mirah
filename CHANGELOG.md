# سجل التغييرات · Changelog

العربي هو الأساس، والإنجليزي تحت كل قسم.

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
