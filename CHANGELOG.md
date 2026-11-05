# سجل التغييرات · Changelog

العربي هو الأساس، والإنجليزي تحت كل قسم.

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
