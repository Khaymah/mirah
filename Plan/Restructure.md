# خطة إعادة الهيكلة: من crate واحد لـ workspace

> القرار وأسبابه في `Plan/ADR/0002-cargo-workspace-layout.md`. الملف ده فيه **التنفيذ** بس.
> **الحالة: اتنفّذت (2026-11-05).** التغييرات اللي حصلت عن الخطة مكتوبة في آخر الملف.
>
> القاعدة: **مفيش تغيير في السلوك.** أي feature جديدة (المؤشر الوهمي، copy mode، `doctor`) مش جوه الخطة دي.

## الشكل المستهدف

```
mirah/
├── Cargo.toml                       workspace: members، [workspace.package]، [workspace.dependencies]، [workspace.lints]
├── rust-toolchain.toml              تثبيت نسخة الـ toolchain (1.97)
├── crates/
│   ├── mirah-bidi/                  المنطق الصافي، من غير I/O
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs               الـ API العام بس: re-exports + visual_row()
│   │   │   ├── cell.rs              Cell، Style، Color (نوع خاص بينا بدل vt100::Color)
│   │   │   ├── classify.rs          is_rtl، is_strong، is_blank، is_frame + الثوابت
│   │   │   ├── segment.rs           fixed_columns، segments، trim
│   │   │   ├── reorder.rs           reorder_segment + logical_to_visual
│   │   │   ├── mirror.rs            mirror()
│   │   │   └── shape/
│   │   │       ├── mod.rs           shape()، Shaped
│   │   │       ├── forms.rs         جداول Presentation Forms (عربي + فارسي)
│   │   │       ├── joining.rs       Joining، joins_next/prev، is_transparent، unshape
│   │   │       └── ligature.rs      lam_alef
│   │   ├── tests/
│   │   │   ├── corpus.rs            اختبارات بتقرا corpus/rows/
│   │   │   └── conformance.rs       BidiCharacterTest (#[ignore]، وبيشتغل في job خاص في الـ CI)
│   │   └── benches/visual_row.rs    criterion
│   ├── mirah-term/                  من bytes لـ frames، من غير pty
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── error.rs             TermError (thiserror)
│   │   │   ├── sequences.rs         كل الـ escape sequences كثوابت ليها أسماء
│   │   │   ├── emulator.rs          Emulator: feed(bytes) → Output { to_host, to_child }، resize، frame
│   │   │   ├── screen.rs            vt100::Screen → صفوف mirah_bidi::Cell (الملف الوحيد اللي بيلمس vt100 cells)
│   │   │   ├── render/
│   │   │   │   ├── mod.rs           Renderer (الفرق بين الـ frames)
│   │   │   │   ├── sgr.rs           Style → SGR
│   │   │   │   └── cursor.rs        مكان المؤشر المرئي
│   │   │   └── bridge/
│   │   │       ├── mod.rs           Bridge: impl vt100::Callbacks
│   │   │       ├── local.rs         الأسئلة اللي بنرد عليها بنفسنا (DSR 5n/6n)
│   │   │       └── passthrough.rs   الـ allowlist بتاعة اللي بيعدّي للترمنال الحقيقي + إعادة بناء CSI/OSC
│   │   └── tests/sessions.rs        snapshot: corpus/sessions/*.vt → frame متوقع
│   └── mirah/                       الـ binary
│       ├── Cargo.toml               [[bin]] name = "mirah"
│       ├── src/
│       │   ├── main.rs              رفيع: logging → run → exit code
│       │   ├── command.rs           بناء CommandBuilder من الـ args
│       │   ├── logging.rs           tracing لملف بس (MIRAH_LOG)
│       │   ├── host.rs              TerminalGuard: raw mode، الدخول والخروج، panic hook
│       │   └── session/
│       │       ├── mod.rs           Session::run (الـ loop بس)
│       │       ├── event.rs         enum Event
│       │       ├── pty.rs           فتح الـ pty، الـ spawn، الـ resize
│       │       └── io.rs            threads: قراية الـ pty، stdin، SIGWINCH
│       └── tests/run.rs             integration: `mirah printf ...` جوه pty وبنتأكد من الـ exit code والـ restore
├── corpus/
│   ├── rows/*.txt                   سطر منطقي + المرئي المتوقع
│   └── sessions/*.vt + *.expected   جلسات مسجّلة (Ink وvim)
├── research/                        زي ما هو، بس corpus.rs يعتمد على mirah-bidi بدل #[path]
├── Plan/
└── .github/workflows/ci.yml
```

## الملفات القديمة رايحة فين

| القديم | الجديد |
|---|---|
| `src/shape.rs` | `crates/mirah-bidi/src/shape/{mod,forms,joining,ligature}.rs` |
| `src/segment.rs` (الأنواع) | `crates/mirah-bidi/src/cell.rs` |
| `src/segment.rs` (المنطق) | `classify.rs` + `segment.rs` + `reorder.rs` + `mirror.rs` |
| `src/render.rs` | `crates/mirah-term/src/{screen.rs, render/*}` |
| `src/bridge.rs` | `crates/mirah-term/src/bridge/*` |
| `src/main.rs` (ENTER وLEAVE والثوابت) | `crates/mirah-term/src/sequences.rs` |
| `src/main.rs` (الباقي) | `crates/mirah/src/*` |
| `Plan/ADR/0001-rust-as-language.md` | `Plan/ADR/0001-rust-as-language.md`، عشان التسمية تمشي على `Decisions/README.md` |

## الثوابت اللي هتاخد أسماء

| القيمة | الاسم | المكان |
|---|---|---|
| `65536` | `PTY_READ_BUFFER` | `mirah/src/session/io.rs` |
| `4096` | `STDIN_READ_BUFFER` | `mirah/src/session/io.rs` |
| `2` (عدد المسافات اللي بتفصل) | `MIN_GAP_COLUMNS` | `mirah-bidi/src/classify.rs` |
| `'\u{2500}'..='\u{259F}'` | `FRAME_CHARS` | `mirah-bidi/src/classify.rs` |
| `0x07` | `BEL` | `mirah-term/src/sequences.rs` |
| `"\x1b[8l"` و`"\x1b[8h"` | `BDSM_EXPLICIT` و`BDSM_IMPLICIT` | `sequences.rs` |
| `"\x1b[?2026h/l"` | `SYNC_BEGIN` و`SYNC_END` | `sequences.rs` |
| `"\x1b[?1049h/l"` | `ALT_SCREEN_ENTER` و`ALT_SCREEN_LEAVE` | `sequences.rs` |
| `2026` (الـ mode اللي بنبلعه) | `MODE_SYNCHRONIZED_OUTPUT` | `bridge/passthrough.rs` |

## Workspace lints

```toml
[workspace.lints.rust]
unsafe_code = "forbid"
[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
dbg_macro = "deny"
print_stdout = "deny"
print_stderr = "warn"
```

جوه `#[cfg(test)]` مسموح بـ `unwrap`، عن طريق `allow-unwrap-in-tests = true` في `clippy.toml`.

---

## المهام (4 batches، وبعد كل batch تحقق قبل اللي بعده)

### Batch 0: صورة السلوك الحالي (قبل أي تغيير)

**R0.1 تسجيل الـ baseline**
- نشغّل الـ binary الحالي على:
  - `research/compare/corpus.txt` عن طريق `research/compare/rs`.
  - جلستين مسجّلتين: Ink وvim. نسجّلهم بـ `research/compare/record.py`.
- نحفظ المخرجات في `corpus/rows/` و`corpus/sessions/`، ودي اللي هنقارن بيها.
- **حالة حرجة:** جلسة Claude Code **مابتتحطش** في الريبو، لأنها output بتاع برنامج تاني. Ink وvim بس.
- **التحقق:** الملفات موجودة، والـ 20 اختبار ناجحين على الكود القديم.

### Batch 1: الـ workspace و`mirah-bidi`

**R1.1 هيكل الـ workspace**
- `Cargo.toml` الرئيسي يبقى workspace فيه:
  - `members = ["crates/*"]`
  - `resolver = "3"`
  - `[workspace.package]` (edition 2024، الرخصة MIT، الـ repository)
  - `[workspace.dependencies]`
  - `[workspace.lints]`
  - `[profile.release]`
- نضيف `rust-toolchain.toml`.
- **حالة حرجة:** `Cargo.lock` يفضل في الـ root. الـ binary لازم يطلع في `target/release/mirah` عشان سكريبتات الـ research.

**R1.2 `mirah-bidi`**
- ننقل المنطق زي الجدول اللي فوق.
- `Style` يبقى `{ fg: Color, bg: Color, bold, dim, italic, underline, inverse }`، و`Color` بتاعنا: `Default`، `Indexed(u8)`، `Rgb(u8, u8, u8)`.
- الـ API العام بس: `Cell`، `Style`، `Color`، `VisualRow`، `visual_row()`، `shape()`، `Shaped`. أي حاجة تانية `pub(crate)`.
- الـ 20 اختبار يتنقلوا مع الكود بتاعهم، و`tests/corpus.rs` يقرا `corpus/rows/`.
- **حالات حرجة:** الحركات في خلية لوحدها، "لا" في آخر سطر، صف كله إطارات، صف فاضي، خلايا عريضة.
- **التحقق:**
  - `cargo test -p mirah-bidi`
  - `cargo clippy -p mirah-bidi -- -D warnings`
  - `cargo build -p mirah-bidi --target wasm32-unknown-unknown`

### Batch 2: `mirah-term`

**R2.1 `sequences.rs` و`error.rs`**
- كل escape sequence متكتبة في الكود دلوقتي تتنقل كثابت.
- `TermError` فيه الحالات اللي ممكن تحصل فعلاً بس (مثلاً حجم صفر). مفيش variants احتياطية.

**R2.2 `screen.rs` و`render/*`**
- `screen.rs` بيحوّل `vt100::Color` لـ `mirah_bidi::Color`، وده المكان الوحيد اللي بيحصل فيه التحويل.
- `Renderer::frame()` يتقسّم:
  - `render/sgr.rs`: `sgr(style)`.
  - `render/cursor.rs`: `visual_cursor(row, col)`.
  - `render/mod.rs`: المقارنة بالـ frame اللي فات والرسم.

**R2.3 `bridge/*`**
- `local.rs`: الرد على DSR.
- `passthrough.rs`: allowlist على شكل `match` واحد، ومعاها `rebuild_csi` و`rebuild_osc`.
- **حالة أمان:** OSC 52 (النسخ) يعدّي زي ما هو، لأن ده اللي أي ترمنال بيعمله أصلاً. بس يتسجّل في الـ log على مستوى debug، والـ ADR بيذكره.

**R2.4 `emulator.rs`**
- `Emulator::new(rows, cols)`، و`feed(&[u8]) -> Output`، و`resize(rows, cols)`، و`frame() -> String`.
- ده الـ API الوحيد اللي الـ CLI بيستخدمه.

**R2.5 `tests/sessions.rs`**
- لكل ملف `.vt` في `corpus/sessions/`: نعدّيه على `Emulator`، والـ frame النهائي (النص + مكان المؤشر) لازم يطابق `.expected`.
- **التحقق:** `cargo test -p mirah-term`، والمخرجات مطابقة للـ baseline بتاع R0.1.

### Batch 3: `mirah` (الـ CLI)

**R3.1 `host.rs`**
- `TerminalGuard` بيتنقل، ونضيف `std::panic::set_hook` بيرجّع الترمنال لحالته (raw mode، والـ alt screen، وBDSM) **قبل** ما الـ hook القديم يطبع رسالة الـ panic.
- **حالات حرجة:**
  - panic في thread جانبي.
  - البرنامج اللي جوه يموت بـ signal.
  - stdin مقفول.

**R3.2 `session/*`**
- `run()` بيتقسّم:
  - `pty.rs`: `PtySession::spawn(cmd, size)` و`resize`.
  - `io.rs`: `spawn_pty_reader` و`spawn_stdin_forwarder` و`spawn_resize_watcher`، وكلهم بيبعتوا `Event`.
  - `mod.rs`: `Session::run` فيها الـ loop بس، وتحت 40 سطر.
- **حالة حرجة:** الـ exit code بتاع البرنامج اللي جوه يرجع زي ما هو، ولو مات بـ signal يرجع 128+n.

**R3.3 `logging.rs`**
- `tracing` و`tracing-subscriber` و`tracing-appender`.
- مفيش أي log إلا لو `MIRAH_LOG` متحدد، ووقتها يكتب في `$XDG_STATE_HOME/mirah/mirah.log` (أو `~/.local/state/...`).
- **ممنوع** stdout وstderr وقت التشغيل.
- بيسجّل:
  - عند البداية: الترمنال (`TERM` و`TERM_PROGRAM`)، وإن BDSM اتبعت، والمقاس.
  - لكل frame: الوقت وعدد الـ bytes والصفوف اللي اتغيرت (debug).
  - كل passthrough (debug).
  - الأخطاء.

**R3.4 `tests/run.rs`**
- integration: نشغّل `mirah sh -c 'printf "مرحبا\n"; exit 3'` جوه `portable-pty`.
- بنتأكد من 3 حاجات: الـ exit code = 3، والـ output فيه `BDSM_EXPLICIT` ثم `BDSM_IMPLICIT`، وآخر الـ output فيه `ALT_SCREEN_LEAVE`.

### Batch 4: الـ CI والتوثيق

**R4.1 `.github/workflows/ci.yml`**
- Jobs على `ubuntu-latest` و`macos-latest`:
  - `cargo fmt --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `cargo build -p mirah-bidi --target wasm32-unknown-unknown`
  - `conformance`: يحمّل `BidiCharacterTest.txt` ويشغّل `cargo test -p mirah-bidi --test conformance -- --ignored`.
- الـ actions تتثبت على SHA.

**R4.2 التوثيق**
- `Plan/Architecture.md`: جدول المكونات يتربط بالـ crates.
- `research/compare/rs`: يعتمد على `mirah-bidi` بـ path dependency بدل `#[path]`.
- `Plan/Progress/` يتحدّث.

## التحقق النهائي (Definition of Done)
1. `cargo fmt --check`، و`cargo clippy --workspace --all-targets -- -D warnings`، و`cargo test --workspace`: كله أخضر.
2. مخرجات الـ corpus والجلسات **مطابقة 100%** للـ baseline بتاع R0.1.
3. الـ e2e (`research/e2e/harness.py` و`vimtest.py`) شغال بـ `target/release/mirah`.
4. `mirah-bidi` بيتعمله build لـ `wasm32-unknown-unknown`.
5. مفيش ملف أكتر من 600 سطر، ومفيش comments في الكود، ومفيش `unwrap` برا الاختبارات.

## برا الخطة دي (ليها مهام لوحدها بعدين)
- تصليح المؤشر الوهمي بتاع Ink.
- `mirah doctor` وflags زي `--no-bdsm`، وأماكنها `command.rs` و`doctor.rs` لما ييجي وقتها.
- copy mode وOSC 52 من عندنا.
- Windows ConPTY.

## التغييرات اللي حصلت وقت التنفيذ

| البند | الخطة | اللي حصل | السبب |
|---|---|---|---|
| R3.2: exit code لما البرنامج يموت بـ signal | 128+n | فضل 1 زي القديم، واسم الـ signal بيتسجّل في الـ log | `portable-pty` بيرجّع اسم الـ signal كنص وصفي (`strsignal`)، مش رقم |
| آخر frame | مفيش تغيير | **اتصلح bug:** الـ frame الأخير بيترسم قبل الخروج | لو البرنامج طبع حاجة وخرج على طول، كان آخر output بيضيع. اختبار `tests/run.rs` اكتشفه |
| `research/` | يعتمد على `mirah-bidi` | اتعمل كده، واتضاف `exclude = ["research"]` للـ workspace | crates الـ research ليها `Cargo.lock` لوحدها |
| `research/compare/record.py` | | **اتصلح bug** في ترميز الحروف العربية اللي بتتكتب كـ keystrokes | `unicode_escape` كان بيبوّظ UTF-8 |
| حجم الـ binary | | زاد من 0.93 لـ 2.0 MB | `tracing` و`tracing-subscriber` |
