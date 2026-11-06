---
name: mirah-ui
description: Rules for mirah's own interface (the bar, tabs, settings menu, close dialog, program picker and status notices) - right-to-left order, Arabic wording, terminal-safe glyphs, palette, and narrow-width behaviour. Use before adding or changing anything mirah draws itself under crates/mirah/src/chrome/, any Arabic text shown to the user from crates/mirah/src/session/ or config/keys.rs, or when the owner asks to improve how mirah looks.
---

# mirah UI rules

mirah draws a little interface of its own around the program it runs. The program is the main task; mirah's interface stays out of its way.

## 1. Show only what the task needs

- The bar is one row. Buttons are icons only, never words.
- A button that cannot act right now is hidden, not greyed out (no ✕ or ● before a tab exists, no ↻ without an update).
- Settings the ordinary user changes sit in the basic part of the menu. Technical ones (letter joining, input order, mouse mapping) live under "خيارات متقدّمة", hidden until shown.
- A notice replaces nothing: it appears in the empty middle of the bar and fades after a few seconds.

## 2. Right-to-left order

- Read the bar from the right: brand, tabs (tab 1 rightmost), `+`, then the buttons. ✕ sits at the far left, like close in Arabic desktop apps.
- In lists and boxes, the label is on the right and its values follow to the left; the first value is the rightmost.
- Left arrow means "next" and right arrow means "previous" everywhere (bar, menu values, dialog buttons). Up and down keep their meaning.
- In a dialog, the action is on the right and "إلغاء" on its left. Focus starts on "إلغاء" for anything destructive.
- The focus marker `▌` sits on the left edge inside a box, and the row behind it is lighter.
- Mixed Arabic and Latin text (paths, program names, `Enter`, `Esc`) goes through `style::shaped`, which runs the bidi algorithm with an RTL base. Never assemble visual order by hand.

## 3. Arabic wording

Use simplified Modern Standard Arabic (فصحى مبسّطة): short, clear to any Arab reader, no colloquial words. Documentation outside the program stays in the owner's Egyptian Arabic.

- **Titles** are a short phrase or question: "اختر برنامجًا", "إغلاق claude؟".
- **Buttons** are verbs naming the result: "إغلاق" and "إلغاء", never "نعم" and "لا".
- **Hints** use "لـ" + verbal noun: "↑↓ للتنقل · Enter للفتح · Esc للرجوع".
- **On/off values** are "تشغيل" and "إيقاف". "فقط", not "بس".
- **Failures** say what failed, gently, with "تعذّر": "تعذّر تشغيل opencode", "تعذّر التحديث، راجع ملف السجل".
- **Done** uses the passive past: "حُفظ التسجيل", "ثُبّت 0.3.1، ويعمل في المرة القادمة".
- **Work in progress** uses "جارٍ": "جارٍ تثبيت 0.3.1…".
- **Counts** agree with the number (`launcher::ago`): دقيقة، دقيقتين، 3 دقائق، 11 دقيقة.
- Keep Latin digits, as the programs inside mirah do.
- Program names stay as the program spells them (`claude`, `OpenCode`).

| Instead of | Write |
|---|---|
| تفتح أنهي برنامج؟ | اختر برنامجًا |
| كمّل جلسة قبل كده | استئناف جلسة سابقة |
| مقدرتش أشغّل X | تعذّر تشغيل X |
| يتطبّق على: X بس | التطبيق على: X فقط |
| شغال / مقفول | تشغيل / إيقاف |
| نعم، اقفل / لا | إغلاق / إلغاء |

## 4. Glyphs that are safe in a terminal

Every glyph must take exactly one cell in Windows Terminal, Cascadia Mono, Kawkab Mono and DejaVu Sans Mono. Characters with an emoji presentation (such as ⚙ U+2699, ⏺, ▶, ✔) may be drawn two cells wide and break the row.

Allowed: `≡ ● ■ ✕ + ↻ ↺ ◆ ◉ ○ • ‹ › ▌ ▐ ┃ │ ─ ╭ ╮ ╰ ╯ ├ ┤ ↑ ↓ ← → · …`. The test `chrome::tests::every_glyph_is_terminal_safe` enforces this list; extend it only after checking the new glyph in those fonts.

## 5. Colour

Use the constants in `chrome/style.rs` only: a dark bar slightly lighter than the program, one accent (`ACCENT`) for the current choice and focus, `DIM` for secondary text, red only while recording, green only for an update. No new colours without a reason in `Plan/`.

## 6. Narrow terminals

The bar must keep ✕, `+` and the open tab at any width from 40 columns: tab names shorten first, then the bar shows only the open tab and `‹ n ›`. Boxes (menu, dialog, picker) must fit inside 60 columns, the narrowest width the agents themselves support (`chrome::tests::boxes_fit_inside_sixty_columns`). Add a test at 40 columns for anything new on the bar.

## Before finishing a UI change

1. Run `cargo test -p mirah chrome` and `cargo clippy --workspace --all-targets -- -D warnings`.
2. Look at it: drive mirah in a pty with fake agents and render the screens (see `Plan/Progress` section 36 for the method), at 100 and 40 columns.
3. Read every new Arabic string aloud against section 3.
