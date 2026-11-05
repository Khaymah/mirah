# سجل التغييرات · Changelog

العربي هو الأساس، والإنجليزي تحت كل قسم.

## 0.1.0-alpha.1 - 2026-11-05

أول نسخة تجريبية.

- مرآة بتشغّل أي برنامج terminal جواها، وبتصلّح العربي والفارسي والعبري: بتقسّم السطر، وتطبّق UAX #9، وتوصّل الحروف ومعاها "لا".
- الإطارات والأعمدة بتفضل في مكانها، والإنجليزي والأرقام جوه الكلام العربي بيفضلوا من الشمال لليمين.
- المؤشر الوهمي اللي Claude Code وبرامج Ink بيرسموه بيظهر في مكانه الصح، والسطر بيفضل سليم.
- بتقفل الـ bidi بتاع الترمنال بـ BDSM، وترجّعه وانت خارج.
- بتشتغل على Linux وmacOS وWindows.
- `MIRAH_LOG=debug` بيكتب log في ملف.

**English:** First alpha. mirah runs any terminal program and fixes Arabic, Persian and Hebrew: row segmentation, UAX #9 reordering and Arabic shaping including lam-alef. Frames and columns stay in place, and Latin text and numbers stay left to right inside Arabic. The fake cursor that Claude Code and Ink programs draw lands in the right place without scrambling the row. mirah turns off the terminal's own bidi with BDSM and restores it on exit. It runs on Linux, macOS and Windows. `MIRAH_LOG=debug` writes a log file.
