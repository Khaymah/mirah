# سجل التغييرات · Changelog

العربي هو الأساس، والإنجليزي تحت كل قسم.

## 0.4.0-alpha.5 - 2026-11-06

**ألوان للماركداون، والجلسات جنب البرنامج، والنسخ من Claude على ويندوز.**

- **تلوين Markdown** (إعداد جديد، شغال افتراضياً)، زي opencode:
  - العناوين، وعلامات القوايم، والكلمات العريضة جوه الكلام (زي **Rust** و**Go**)، وصف عناوين الجدول بلون مميز.
  - إطار الجداول وخطوط الفصل `---` بلون أهدى. مرآة بتلوّن الكلام اللي بلون الكلام العادي بس، فالبرامج اللي بتلوّن بنفسها مابتتأثرش.
- **الجداول العربي اللي Claude بيعكسها بنفسه** (على ويندوز) بقت بتروح على يمين الشاشة هي كمان.
- **الجلسات بقت في فولدر `sessions` جنب `mirah.exe`.** ولو مرآة مش قادرة تكتب هناك، بترجع لـ `%LOCALAPPDATA%\mirah\sessions`. و`sessions_dir` في `config.toml` بيختار مكان تاني. ومرآة بتكتب مكان الفولدر وهي خارجة.
- **النسخ من Claude على ويندوز:** Claude الظاهر بيكتب في حافظة ويندوز بنفسه كمان، فالنص المقلوب كان بيفضل. دلوقتي بعد أي نسخة من برنامج بيقلب العربي، مرآة بتحط النص بالترتيب الصح في حافظة ويندوز بنفسها، مرتين (بعد ربع ثانية، وبعد حوالي ثانية).

**English:** Markdown colours, sessions next to the program, and copying from Claude on Windows. A new "highlight Markdown" setting (on by default) colours headings, list markers, bold words in prose and table header rows with the accent and quiets table borders and `---` rules, like opencode, touching only text in the normal text colour. Arabic tables that Claude mirrors itself on Windows now move to the right as well. Sessions are kept in a `sessions` folder next to `mirah.exe`, falling back to `%LOCALAPPDATA%\mirah\sessions` when that is not writable; `sessions_dir` in `config.toml` picks another place, and mirah prints the folder on exit. Claude on Windows apparently writes the Windows clipboard itself as well, so the reversed text stayed; after a copy from a program that reverses Arabic, mirah now puts the corrected text on the Windows clipboard itself, twice (after a quarter second and after about a second).

## 0.4.0-alpha.4 - 2026-11-06

**تشغيل التسجيلات كان بيعيد نفسه من غير نهاية.**

- **التسجيل فيه أسئلة للترمنال** (زي "المؤشر فين؟" و"إنت أنهي ترمنال؟")، كان البرنامج اللي اتسجّل بيسألها. ومرآة كانت بترد عليها، والرد بيوصل للمشغّل كأنه زراير، وآخر حرف في الرد `R` اللي هو "من الأول". فالتسجيل كان بيبدأ من الأول من غير نهاية، ومرآة بتبان معلّقة، والتسجيل نفسه بيكبر جداً. دلوقتي المشغّل بيشيل من التسجيل:
  - أسئلة الترمنال.
  - أوضاع الماوس والكيبورد وإشارات التركيز، عشان التسجيل مايغيّرش طريقة الإدخال.
  - النسخ للحافظة (`OSC 52`)، عشان تشغيل تسجيل مايغيّرش الحافظة.
- **عارض Markdown** مابقاش بيرسم الشاشة كل نص ثانية لما مفيش حاجة اتغيرت.

**English:** Playing a recording restarted forever. Recordings hold the terminal questions the recorded program asked (cursor position, device attributes); mirah answered them, the answers reached the player as keys, and their final `R` meant "restart", so playback looped, mirah looked frozen and recordings of it grew huge. The player now strips terminal questions, mouse, keyboard and focus modes, and clipboard writes from the recording. The Markdown viewer no longer redraws every half second when nothing changed.

## 0.4.0-alpha.3 - 2026-11-06

**تشغيل التسجيلات، والماوس بعد ما مرآة تخرج، وجداول Claude.**

- **`mirah play FILE.vt` أو `mirah play <ID>`:**
  - بيشغّل التسجيل في تاب، وكأن البرنامج شغال قدامك.
  - المسافة بتوقف، و`→` أسرع، و`End` لآخره، و`R` من الأول، و`Esc` بيقفل.
  - والتسجيلات اللي في الفولدر بتظهر في شاشة الاختيار تحت "ملفات للعرض".
- **الترمنال كان بيفضل يبعت حركة الماوس** (`[555;207;45M…`) و`[I` و`[O` بعد ما مرآة تخرج:
  - مرآة بقت بتقفل الماوس كله (`1000`، و`1002`، و`1003`، و`1005`، و`1006`، و`1015`) وإشارات التركيز (`1004`) أول ما تبدأ.
  - ووهي خارجة، بتقفلهم في الشاشتين: قبل ما ترجع للشاشة العادية وبعدها.
  - قبل كده `1004` و`1005` و`1015` ماكانوش بيتقفلوا خالص.
- **التاب مابياخدش ماوس ولا إشارات تركيز إلا لو البرنامج اللي فيه طالبهم.** فالـ shell اللي في تاب مابقاش بيظهر فيه كلام غريب لما تحرّك الماوس. ولما تتنقل بين التابات، إشارات التركيز بترجع زي ما التاب طالبها.
- **الرعشة وقت التحديد في Claude:** تحويل مكان الماوس بقى بيتثبّت من أول ما تدوس لحد ما تسيب، فالتحديد مابقاش بيتحرك رايح جاي وClaude بيرسمه.
- **جداول Claude العربي بقت على يمين الشاشة**، بنفس المسافة اللي كانت على الشمال، لو الجزء اللي يمينها فاضي.
- **النسخ:** التنوين والشدة بيفضلوا على حرفهم في النص اللي Claude بينسخه ("فعلاً" مابقتش "فعلًا").

**English:** Recording playback, the mouse after mirah exits, and Claude's tables. `mirah play FILE.vt` or `mirah play <ID>` replays a recording in a tab as if the program were running (space pauses, `→` speeds up, `End` jumps to the end, `R` restarts, `Esc` closes), and recordings in the folder appear in the picker under "files to view". The terminal could keep sending mouse motion and focus events after mirah exited: mirah now turns off every mouse mode (1000, 1002, 1003, 1005, 1006, 1015) and focus reporting (1004) when it starts and on both screens when it leaves; 1004, 1005 and 1015 were never turned off before. A tab only receives mouse and focus events when its program asked for them, so a shell tab no longer fills with garbage when the mouse moves, and switching tabs restores each tab's focus reporting. While dragging, the mouse mapping is held from press to release, so Claude's selection no longer jumps back and forth as it redraws. Mirrored Arabic tables from Claude move to the right edge, keeping their margin, when the space is free. Marks such as tanween stay on their letter in text Claude copies.

## 0.4.0-alpha.2 - 2026-11-06

**الزراير والكيبورد على العربي.**

- **`S` و`Q` في عارض Markdown مكانوش بيشتغلوا** لو الكيبورد على العربي، لأن الزرار بيبعت `س` و`ض`. دلوقتي العارض بيفهم الحرف من مكانه في الكيبورد العربي كمان. و`Esc` بيقفل في أي حالة، وشريط الحالة بقى بيقول `Esc للإغلاق`.
- **نفس التصليح في الشريط وشاشة الاختيار:** `Ctrl+]` وبعدين `ف` بيفتح تاب جديد زي `t`، والأرقام العربي (`١`–`٩`) بتشتغل زي `1`–`9`.
- **`MIRAH_LOG=debug`** بقى بيسجّل الزراير اللي العارض بيستقبلها.

**English:** Keys with an Arabic keyboard layout. `S` and `Q` in the Markdown viewer did nothing with an Arabic layout, because those keys send `س` and `ض`; the viewer now maps Arabic-layout letters to their keys, `Esc` always closes, and the status line says so. The bar and the picker do the same: `Ctrl+]` then `ف` opens a tab like `t`, and Arabic digits `١`–`٩` work like `1`–`9`. `MIRAH_LOG=debug` now logs the keys the viewer receives.

## 0.4.0-alpha.1 - 2026-11-06

**عارض ملفات Markdown، والكود اللي فيه عربي.**

- **`mirah view FILE.md`**، أو من شاشة الاختيار تحت "ملفات Markdown" (آخر 5 ملفات في الفولدر): الملف بيتفتح في تاب.
  - العناوين، والقوايم، والاقتباسات على اليمين.
  - الجداول بإطار، ومعكوسة، وعلى يمين الشاشة.
  - الكود على الشمال، والسطور الطويلة بتلف.
  - التنقل: الأسهم، وعجلة الماوس، و`PgUp` و`PgDn`، و`Home` و`End`.
  - **`S` بيحفظ نسخة في التنزيلات**، باسم جديد لو الملف موجود. و`Q` بيقفل.
- **سطر الكود اللي فيه كلمة عربي** (زي `println!("مرحبا");`) بقى بيتقري من الشمال لليمين، بدل ما كان بيتقلب. ده بيحصل بس لو أغلب حروف السطر لاتيني، عشان الكلام العربي مايتأثرش.
- **الخط اللي على الشمال** (`┃` أو `│`) في رسالتك في opencode وفي الاقتباسات بقى بيروح على يمين الكلام، زي علامة القايمة.

**English:** A Markdown viewer and code with Arabic in it. `mirah view FILE.md`, or the picker's "Markdown files" section (the newest five in the folder), opens the file in a tab: headings, lists and quotes on the right, tables with borders mirrored to the right, code on the left, long lines wrapped; arrows, the mouse wheel, PgUp/PgDn and Home/End move, `S` saves a copy to Downloads under a new name if one exists, and `Q` closes it. A code line holding an Arabic word, such as `println!("مرحبا");`, now reads left to right instead of reversed, only when most of its letters are Latin. The left bar (`┃`, `│`) of opencode's user message and of quotes moves to the right of the text, like a list marker.

## 0.3.0-alpha.4 - 2026-11-06

**النسخ، وREADME جديد.**

- **النسخ من Claude Code بقى بالترتيب الصح.** Claude على ويندوز بيقلب العربي بنفسه، فلما بينسخ اللي انت محدده (`OSC 52`)، كان بيوصل للحافظة مقلوب: "5. الأداء" كانت بتتنسخ "ءادألا .5". دلوقتي مرآة بترجّع كل سطر للترتيب الصح قبل ما يوصل للحافظة، لأي برنامج "ترتيب نصه" مرئي.
- **إعداد "النسخ عند التحديد"** في القايمة: Claude بينسخ أي حاجة تحددها لوحده، ولو قفلت الإعداد، التحديد مابيغيّرش الحافظة.
- **README اتكتب من الأول** بترتيب منطقي: مرآة بتعمل إيه، وإزاي تسطّبها، وأول استخدام، والشريط والتابات، والإعدادات، والجلسات، والحدود. وأوامر التثبيت بقت على آخر نسخة. ومن هنا ورايح أي تغيير بيتكتب في الـ README في نفس الـ commit.

**English:** Copying and a new README. Claude Code on Windows reverses Arabic itself, so its copy-on-select (`OSC 52`) reached the clipboard reversed ("5. الأداء" became "ءادألا .5"); mirah now restores each line to reading order for any program whose text order is visual. A new menu setting, "copy on select", stops selection from changing the clipboard. The README is rewritten in a logical order (what mirah does, install, first run, the bar and tabs, settings, sessions, limits) with install commands for the current version, and from now on every user-visible change updates it in the same commit.

## 0.3.0-alpha.3 - 2026-11-06

**المحاذاة.** من تسجيل opencode اللي المالك بعته، ومقارنة قبل وبعد على كل تسجيلات Claude.

- **الكلام العربي اللي فيه كود صغير** (زي "Rust: بيستخدم `Result<T, E>`") بقى بيتحاذى يمين. قبل كده opencode بيلوّن الكود جوه الجملة، فلو السطر قصير، نص حروفه بتبقى ملوّنة، ومرآة كانت بتفتكره كود. دلوقتي السطر اللي حروفه العربي بلون الكلام العادي بيتحسب كلام. ونسبة رموز الكود المسموحة في السطر العربي بقت 25%، بعد ما كانت 12%.
- **الكلام العربي الرمادي** (زي "Thought" في opencode) بيتحاذى يمين.
- **رسالتك في opencode** (الصندوق اللي ليه خط على الشمال بس `┃`)، والاقتباسات اللي جنبها `│`: الكلام اللي جواهم بقى يمين.
- **سطر الكتابة في Claude لما يكون فيه عربي** بقى بيتحاذى يمين زي ما المالك طلب، بعد ما كانت علامة `>` بتخلّيه يتحسب كود.
- **ملاحظة:** لما يبقى فيه عربي على الشاشة، الكلام الإنجليزي اللي جنبه بيتحاذى يمين كمان، زي القاعدة القديمة. فالسطر "Resume this session with:" اللي Claude بيكتبه وهو بيقفل بقى يمين.

**English:** Alignment fixes from the owner's opencode recording, checked before and after on every Claude recording. Arabic sentences with a little inline code, which opencode colours, are prose again: a row whose Arabic letters are in the text colour is not treated as code, and Arabic rows allow 25% code characters instead of 12%. Grey Arabic such as opencode's "Thought" aligns right. Text after a single left bar (`┃`, `│`), as in opencode's user message and in quotes, aligns right. Claude's input row aligns right when it holds Arabic, as asked; before, its `>` made it look like code. English prose next to Arabic still aligns right, so "Resume this session with:" at Claude's exit does too.

## 0.3.0-alpha.2 - 2026-11-06

**نصوص أوضح، وقايمة أبسط.** على حسب المقال اللي بعته المالك عن تصميم الواجهات العربي.

- **النصوص جوه مرآة بقت فصحى مبسّطة:** "اختر برنامجًا"، و"استئناف جلسة سابقة"، و"إغلاق claude؟"، و"تعذّر تشغيل opencode". والأزرار بقت أفعال بتقول هتعمل إيه ("إغلاق" و"إلغاء")، بدل "نعم" و"لا". والوقت بيتكتب بالعدد الصح: "منذ دقيقتين"، و"منذ 5 دقائق".
- **القايمة:** الإعدادات اللي اليوزر العادي بيغيّرها بس هي اللي ظاهرة (المحاذاة، وعكس الجداول، والتسجيل، والتحديث). والتقنية (وصل الحروف، وترتيب نص البرنامج، وتحويل نقرات الماوس) تحت "خيارات متقدّمة".
- **زرار الإعدادات بقى `≡` بدل `⚙`**، لأن `⚙` ممكن يتعرض في Windows Terminal كـ emoji بعرض خانتين ويلخبط الشريط.
- **الشاشات الضيقة:** في 40 عمود، الشريط بيصغّر الأزرار واسم مرآة الأول، فـ ✕ و`+` والتاب المفتوح بيفضلوا ظاهرين ومابيركبوش على بعض.
- **قواعد الواجهة** بقت Skill جوه الريبو (`.claude/skills/mirah-ui/`)، ومعاها اختبارات بتتأكد إن كل رمز بياخد خانة واحدة، وإن الصناديق بتكفي في 60 عمود.

**English:** Clearer wording and a simpler menu, following Arabic interface guidance the owner shared. mirah's own text is now simplified Modern Standard Arabic; buttons name their action ("إغلاق", "إلغاء") and ages agree with their number. The menu shows the everyday settings and keeps letter joining, input order and mouse mapping under "advanced options". The settings button is `≡` instead of `⚙`, which Windows Terminal may draw as a two-cell emoji. At 40 columns the bar shrinks its buttons and the brand first, so ✕, `+` and the open tab stay visible without overlapping. The interface rules live in a repository skill (`.claude/skills/mirah-ui/`) with tests for single-cell glyphs and boxes that fit in 60 columns.

## 0.3.0-alpha.1 - 2026-11-06

**شاشة اختيار البرنامج، والتابات، والجلسات، وشكل جديد.**

- **`mirah` لوحده** بيفتح شاشة اختيار: مرآة بتدوّر في الـ PATH على Claude Code، وOpenCode، وCodex، وGemini، وQwen، وAider، وCrush، وCursor Agent، وAmp، وCopilot، وGoose، وفي الآخر الـ shell بتاعك. وآخر برنامج استخدمته بيبقى هو المختار. وتقدر تضيف برامج بـ `[[agents]]` في `config.toml`. و`mirah claude` بيشتغل زي الأول.
- **التابات جوه الشريط:** `+` أو `Ctrl+]` وبعدين `t` بيفتح تاب جديد، و`Ctrl+]` وبعدين رقم بيروح للتاب، والأسهم بتتنقل بينهم، والضغط على اسم التاب بيفتحه. والتابات التانية بتفضل شغالة، ولو فيها حاجة جديدة بيظهر عليها `•`. و✕ بيقفل التاب اللي انت عليه، ولما آخر تاب يتقفل، مرآة بتخرج.
- **الجلسات:** كل تاب جلسة ليها ID (زي `1106-0457-k3f2`) وفولدر في `%LOCALAPPDATA%\mirah\sessions\` أو `~/.local/state/mirah/sessions/`، فيه `session.toml` والتسجيل والـ log. فالتسجيلات مابقتش بتتكتب جنب البرنامج.
  - **`mirah sessions`** بيعرض آخر الجلسات.
  - **`mirah resume <ID>`** بيكمّل نفس المحادثة بطريقة كل برنامج (`claude --continue`، و`opencode --continue`، و`codex resume --last`، و`gemini --resume`)، في نفس الفولدر، والتسجيل بيكمّل في نفس الملف. وشاشة الاختيار بتعرض آخر الجلسات اللي ينفع تتكمّل.
  - مرآة بتحتفظ بآخر 50 جلسة أو آخر 14 يوم.
  - **"سجّل كل جلسة"** بقى إعداد في القايمة. و`--record` لسه شغال، وبيكتب في فولدر الجلسة، وبيطبع الـ ID.
- **شكل أهدى:** ألوان غامقة بلون مميز واحد، والتاب المفتوح بخلفية زي البرنامج، والأزرار أيقونات بس، وفواصل `┃` رفيعة. والقايمة، وسؤال القفل، وشاشة الاختيار بقوا صناديق بحواف مدوّرة. وقيم الإعدادات بتظهر جنب بعض، واللي شغالة عليها لون، والضغط على أي قيمة بيختارها.
- **علامات التشكيل** (زي الشدّة) في الشريط والقوايم بقت على حرفها، بعد ما كانت بتاخد خانة لوحدها.

**English:** A program picker, tabs, stored sessions and a calmer look. `mirah` on its own opens a picker of the agents found on PATH (Claude Code, OpenCode, Codex, Gemini, Qwen, Aider, Crush, Cursor Agent, Amp, Copilot, Goose, then your shell), with the last one preselected; `[[agents]]` in `config.toml` adds more, and `mirah claude` still starts directly. Tabs live in the bar: `+` or `Ctrl+]` then `t` opens one, `Ctrl+]` then a digit jumps to it, arrows move between them and a click opens one; background tabs keep running and show `•` when they have new output; ✕ closes the open tab and closing the last one exits. Every tab is a session with an ID and a folder under `%LOCALAPPDATA%\mirah\sessions\` or `~/.local/state/mirah/sessions/` holding `session.toml`, the recording and the log, so recordings no longer land next to the program. `mirah sessions` lists them, and `mirah resume <ID>` continues the same conversation with each program's own option, in the same folder, appending to the same recording; the picker lists recent sessions too. The newest 50 sessions or the last 14 days are kept. "Record every session" is a menu setting, and `--record` still works, writing to the session folder and printing the ID. The bar, the menu, the close question and the picker use a calmer palette with one accent, icon-only buttons and rounded boxes; menu values sit side by side with the current one highlighted and can be clicked. Arabic marks such as shadda stay on their letter in the bar and menus.

## 0.2.0-alpha.2 - 2026-11-06

تصليحات من أول تجربة للشريط على ويندوز.

- **التسجيل بقى باسم البرنامج:** `claude-session-1.vt`، و`opencode-session-1.vt`، والرقم لكل برنامج لوحده. والـ `.log` بتاع `--record` بقى بنفس الاسم.
- **إيقاف التسجيل وتشغيله تاني بيكمّل في نفس الملف**، والساعة اللي في الشريط بتكمّل من المدة اللي وقفت عندها. قبل كده كان بيعمل ملف جديد، وممكن يكتب فوق ملف موجود.
- **الجداول اللي فيها emoji** (زي `Rust 🦀`) بقت بتتعكس كلها، بعد ما كان صف العناوين بس هو اللي بيتعكس. والـ emoji بيفضل كامل.

**English:** Fixes from the first test of the bar on Windows. Recordings are named after the program (`claude-session-1.vt`, `opencode-session-1.vt`, numbered per program), and `--record` names its log the same way. Stopping and starting the recording again continues the same file, and the bar's clock continues from where it stopped; before, it started a new file and could write over an existing one. Tables with emoji such as `Rust 🦀` now mirror as a whole instead of only the header row, and each emoji stays whole.

## 0.2.0-alpha.1 - 2026-11-06

**شريط مرآة، وملف إعدادات.**

- **شريط بالعربي في أول سطر:** فيه اسم البرنامج، وأزرار للإعدادات والتسجيل والتحديث والقفل. والبرنامج بيشتغل في باقي الشاشة، يعني بيخسر سطر واحد.
- **`Ctrl+]`** بيفتح الشريط: الأسهم بتتنقل، وEnter بيختار، وEsc بيرجع. والضغطتين ورا بعض بيوصّلوه للبرنامج. وبيشتغل في Windows Terminal (win32-input-mode)، وفي kitty (CSI-u)، وفي أي ترمنال تاني.
- **الماوس على الشريط** بيشتغل لما البرنامج طالب الماوس. والضغطات اللي تحت الشريط بتتطرح منها سطر وبتوصل للبرنامج.
- **قايمة الإعدادات** فيها المحاذاة، وتشكيل الحروف، وترتيب البرنامج، وعكس الجداول، وتحويل الماوس، وفحص التحديث. والتغيير بيبان على طول، ومن غير ما تقفل البرنامج. وكل إعداد بيتطبّق على كل البرامج أو على البرنامج ده بس.
- **`config.toml`** في `%APPDATA%\mirah\` أو `~/.config/mirah/`، وفيه إعدادات عامة وقسم لكل برنامج. ومتغيرات `MIRAH_*` بتكسب الاتنين. والتعليقات اللي بتكتبها بتفضل زي ما هي. ولو الملف بايظ، مرآة مابتكتبش فوقه.
- **التسجيل من الشريط:** بيبدأ ويقف وانت شغال، وبيوري المدة.
- **التحديث:** مابقاش بيسألك قبل ما البرنامج يبدأ. الفحص بقى في الخلفية، والنتيجة بتظهر علامة ↻ على الشريط، والتثبيت من الشريط بيتأكد من الـ SHA-256 زي `mirah update`. ولو الشريط مقفول، السؤال القديم بيرجع.
- **✕** بيسأل الأول، وبعدين بيقفل البرنامج ومرآة.
- **`MIRAH_BAR=0`** بيخفي الشريط.

**English:** The mirah bar and a settings file. An Arabic bar on the first row shows the program and buttons for settings, recording, updating and closing; the program runs below it. `Ctrl+]` opens the bar (arrows move, Enter chooses, Esc returns, pressing it twice sends it to the program) and is recognised in plain, win32-input-mode and kitty CSI-u input. Clicks on the bar work when the program asks for the mouse; clicks below it reach the program one row up. The settings menu changes alignment, letter joining, input order, table mirroring, mouse mapping and the update check live, for every program or just this one, and saves to `config.toml` (`%APPDATA%\mirah\` or `~/.config/mirah/`), keeping your comments and never writing over a broken file; `MIRAH_*` variables still win. Recording starts and stops from the bar with a clock. The update check now runs in the background and shows a ↻ badge instead of asking before the program starts; installing from the bar uses the same SHA-256 check as `mirah update`, and the old question returns when the bar is off. ✕ asks first, then closes the program and mirah. `MIRAH_BAR=0` hides the bar.

## 0.1.0-alpha.12 - 2026-11-06

تصليحات opencode، من تسجيلين حقيقيين (`mirah-session-6` لـ Claude و`mirah-session-7` لـ opencode).

- **أعمدة الشاشة:** opencode بيقسم الشاشة لعمودين بالخلفية: الشات، والـ sidebar على اليمين. مرآة بقت بتلاقي أي عمود الخلفية بتتغير عنده في 75% من السطور على الأقل، **وبتتعامل مع كل عمود لوحده**: الترتيب، وتصنيف كل سطر كلام ولا كود، والمحاذاة، والجداول. فالشات بيتحاذى يمين جوه عموده، ومابقاش بيعدّي فوق الـ sidebar. والمستطيلات الغامقة اللي كانت بتبان اختفت.
- **التحديد في opencode:** opencode بيلوّن التحديد بخلفية بيضا، وبيلوّن الكلمات المهمة جواه برتقالي. **تغيير الخلفية جوه السطر مابقاش بيقسم الجملة**، والكلام اللي عليه تحديد مابقاش بيتحسب في ألوان الكود.
- **لون الكلام العادي** بيتحسب من الحروف اللي على الخلفية العادية بس. قبل كده، تحديد كبير كان بيخلّي لون التحديد هو "اللون العادي"، فباقي الشاشة كله كان بيتحسب كود.
- **العناوين** اللي كلها bold وبلون واحد ومفيها عربي بقت بتتحاذى، زي عناوين opencode البرتقاني.
- **الـ emoji جوه الجملة** مابقاش بيخلّي السطر يبان كأن فيه فراغ كبير.

**English:** opencode fixes from two real recordings. Columns where the background changes on at least 75% of rows (opencode's chat and sidebar) are now laid out separately, so the chat aligns right inside its own column instead of over the sidebar, and the dark rectangles are gone. A background change inside a row no longer splits a sentence, and highlighted cells are left out of the code-colour test, so opencode selections keep their order and alignment. The usual text colour is taken only from letters on the usual background, so a large selection no longer turns the rest of the screen into "code". Bold, single-colour Arabic rows (opencode's headings) align as headings, and a wide emoji no longer looks like a gap.

## 0.1.0-alpha.11 - 2026-11-06

تصليحات من تجربة المالك (4 screenshots وتسجيل `mirah-session-5.vt`).

- **التحديد بالماوس:**
  - السطر اللي عليه تحديد كان بيرجع شمال، لأن أي خلفية ملوّنة كانت بتخلّي السطر يتحسب كود. **الخلفية لوحدها مابقتش بتعمل كده**، وسطور الـ diff بتتعرف من شكلها: رقم سطر وبعده `+` أو `-`.
  - **مكان الماوس بيتحوّل** من الشاشة لمكانه عند البرنامج (صيغة SGR)، فالتحديد بيمسك الكلام اللي انت شايفه.
- **الـ scroll:** الجدول اللي جزء منه برا الشاشة كان بيبطّل يتعكس، وبعدين يرجع يتعكس وهو بيتحرك. دلوقتي **أي جدول بيتعرف من أي سطر حدود فيه** (`┌┬┐` أو `├┼┤` أو `└┴┘`)، حتى لو أوله أو آخره برا الشاشة.
- **سطر الكتابة:** لو كتبت عربي، السطر بيتحاذى يمين، والعلامة `❯` بتروح معاه، والمؤشر بيبقى في آخر الكلام.
- **opencode:**
  - **الكود بيتعرف بالألوان اللي مختلفة عن لون الكلام العادي في الشاشة.** opencode بيلوّن كل الكلام بلون واحد، فكل سطر كان بيتحسب كود.
  - **المربعات السودا في الجدول اختفت:** الفراغ اللي مرآة بتحطه بقى بنفس خلفية السطر.

**English:** Fixes from the owner's test (four screenshots and `mirah-session-5.vt`). A selected row no longer jumps back to the left: a coloured background alone no longer marks a row as code, and diff rows are recognised by their shape (a line number then `+` or `-`). SGR mouse reports are mapped from the screen back to the program's columns, so a selection covers the text you see. A table partly scrolled off screen is now recognised from any of its border rows and stays mirrored while scrolling. The input row aligns right when you type Arabic, with `❯` moving along and the cursor at the end of the text. Code is now recognised by colours that differ from the screen's usual text colour, which fixes opencode where every row counted as code, and padding keeps the row's background so opencode tables no longer show black boxes.

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
