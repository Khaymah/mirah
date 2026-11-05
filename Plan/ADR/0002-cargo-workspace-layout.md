# ADR-0002: تقسيم مرآة لـ Cargo workspace من 3 crates

- **Status:** Accepted (2026-11-05)
- **Date:** 2026-11-05
- **Deciders:** المالك (Sabry Dawood)
- **Project:** mirah
- **Tags:** architecture, maintainability, rust, structure

## السياق

الـ spike كله crate واحد من نوع binary فيه 5 ملفات: `main` و`bridge` و`render` و`segment` و`shape`. اشتغل، بس تقسيمه مش هيستحمل لما المشروع يكبر:

- **قلب المشروع مايتاخدش لوحده.** الـ segmenter والـ bidi والـ shaper جوه binary. ده بيناقض أهم سبب اخترنا عشانه Rust في ADR-0001، إن المكتبة تتحط جوه ترمنالات أو تتعمل WASM.
- **الأنواع الأساسية مربوطة بمكتبة خارجية.** `Style` بيستخدم `vt100::Color`، فأي حد عايز المنطق لازم ياخد vt100 معاه.
- **حدود المسؤولية مش واضحة:**
  - `run()` في `src/main.rs:73-173` (101 سطر) فيها الـ pty والـ threads والـ signals والـ loop والرسم مع بعض.
  - `segment.rs` فيه الأنواع والتقسيم والـ bidi في ملف واحد.
- **أرقام سحرية:** `65536` و`4096` و`>= 2` والمدى `U+2500..U+259F` و`0x07`. ده مخالف لقاعدة "No magic numbers" في Senior-Engineer.
- **مفيش observability خالص.** ومينفعش نكتب logs على stdout أو stderr، لأن ده نفس الترمنال اللي بنرسم عليه.
- **مفيش integration tests** في `tests/` زي ما `Stacks/Rust.md` بيطلب، ومفيش CI، و`clippy` ماشتغلش بـ `-D warnings`.

**القيود:**
- لازم نفضل binary واحد للمستخدم.
- التقسيم مايزوّدش latency.
- الكود اللي اتكتب يتنقل زي ما هو من غير ما سلوكه يتغير.

## القرار

نحوّل الريبو لـ **Cargo workspace من 3 crates**، والاعتماد بينهم في **اتجاه واحد**:

```
mirah (cli) ──► mirah-term ──► mirah-bidi
```

| Crate | المسؤولية | يعتمد على | ممنوع يعتمد على |
|---|---|---|---|
| `mirah-bidi` | المنطق الصافي: الخلية، والتقسيم، والـ bidi، والتشكيل، والانعكاس، وخريطة المؤشر | `unicode-bidi` بس | أي I/O، وvt100، والـ pty |
| `mirah-term` | من bytes لـ frames: محاكي الشاشة، والرسم بالفرق، والرد على أسئلة الترمنال، وثوابت escape sequences | `mirah-bidi` و`vt100` و`thiserror` | الـ pty، والـ OS، والـ threads |
| `mirah` | الـ binary: الـ CLI، والـ pty، والـ threads، والـ signals، وraw mode، والـ logging، والـ config | `mirah-term` و`portable-pty` و`crossterm` و`anyhow` و`tracing` | |

## البدائل اللي اتقيّمت

### Option 1: crate واحد ونقسّمه لفولدرات
- **المميزات:** أبسط حاجة، ومفيش ملفات `Cargo.toml` زيادة.
- **العيوب:**
  - الحدود مش متفرضة من الـ compiler، فأي module يقدر يستورد أي module.
  - مينفعش ننشر المنطق لوحده.
  - vt100 وportable-pty بيتعملهم compile مع أي حد عايز المنطق بس.

### Option 2: package واحد فيه `lib.rs` و`main.rs`
- **المميزات:** المكتبة بقت متاحة.
- **العيوب:**
  - المكتبة لسه شايلة كل الـ dependencies، ومنها الـ pty اللي مابيتعملهاش compile لـ WASM.
  - لسه مفيش فصل بين المنطق الصافي ومحاكي الترمنال.

### Option 3: workspace من 3 crates ← المختار
- **المميزات:**
  - الـ compiler بيفرض الحدود.
  - `mirah-bidi` ينفع يتعمله compile لـ `wasm32` ويتنشر لوحده.
  - `mirah-term` يتختبر من غير pty: bytes تدخل وframes تطلع، وده مثالي لـ snapshot tests من الـ corpus.
  - الـ CLI بيبقى رفيع.
- **العيوب:** 3 ملفات `Cargo.toml` بدل واحد. بنخفّفها بـ `[workspace.dependencies]` و`[workspace.lints]`.

### Option 4: crates أكتر (pty لوحده، shape لوحده، bridge لوحده)
- **العيوب:** تقسيم بدري. لسه مفيش استخدام تاني يبرر ده، والقاعدة "3 instances before abstracting".

## Quality Gate

| المحور | حمّال؟ | ليه |
|---|---|---|
| 🔧 **Maintainability** | ✅ الأساسي | الاعتماد في اتجاه واحد، وكل ملف مسؤوليته واحدة، ومفيش ملف أكتر من 600 سطر، والثوابت ليها أسماء. مطور جديد يقدر يلاقي "التقسيم بيحصل فين" من اسم الـ crate |
| 🛡️ **Reliability** | ✅ | أخطاء typed بـ `thiserror` في المكتبات و`anyhow` في الـ CLI، زي `Stacks/Rust.md`. `unwrap` و`expect` و`panic` ممنوعين بـ lints. الترمنال لازم يرجع لحالته الأصلية حتى لو حصل panic، بـ guard وpanic hook. الـ child process دايماً بيتعمله `wait` |
| 👁️ **Observability** | ✅ | `tracing` بيكتب في ملف (`$XDG_STATE_HOME/mirah/mirah.log`) لما `MIRAH_LOG` يتحدد، ومش بيكتب على الترمنال أبداً. بيسجّل حجم الـ frame ووقته، والأسئلة اللي عدّت للترمنال، وحالة BDSM. ده اللي هيخلّي مستخدم يبعت bug report مفيد |
| 🔒 **Security** | ✅ جزئي | الـ output اللي جاي من البرنامج اللي جوه الأداة **untrusted**. كل حاجة بتعدّي منه للترمنال الحقيقي (الأسئلة، وOSC 52 للنسخ، والعنوان) تبقى في **allowlist واحدة** في `mirah-term/src/bridge/passthrough.rs`، وعليها اختبارات. مفيش أي passthrough عام |
| 🚀 Performance | محايد | التقسيم لـ crates مالوش تكلفة وقت التشغيل مع `lto = true`. الـ benchmark الحالي (14 µs للسطر) يتحوّل لـ `criterion` bench عشان نكشف أي تراجع |
| ⚡ Scalability | N/A | أداة محلية لمستخدم واحد. الجزء الخاص بعدد المساهمين داخل تحت Maintainability |
| 🟢 Availability | N/A | مش service |
| 💰 Cost | N/A | open source ومفيش infrastructure |

**الـ trade-off:** بنتنازل عن **سرعة التعديل دلوقتي** (ملفات أكتر، و`pub` API لازم نفكر فيه، وتعديلات في 3 manifests) مقابل **سرعة التغيير بعد سنة**، ومقابل إن المكتبة تتنشر لوحدها. ده مقبول لأن ADR-0001 كله قايم على إن المكتبة تتحط جوه ترمنالات تانية.

## النتايج

### الإيجابي
- ✅ `mirah-bidi` يتعمله compile لـ `wasm32-unknown-unknown` في الـ CI. ده إثبات عملي إن المكتبة تقدر تدخل xterm.js وVS Code.
- ✅ snapshot tests لـ `mirah-term` من غير pty، فالـ corpus يبقى باب المساهمة اللي في Roadmap T1.7.
- ✅ كل ملف مسؤوليته واحدة، وكل الملفات تحت 600 سطر.

### السلبي (trade-offs مقبولة)
- ⚠️ ملفات أكتر. بنخفّفها بـ workspace inheritance للنسخة والـ lints والـ dependencies.
- ⚠️ `pub` API لـ `mirah-bidi` لازم يبقى مستقر لو اتنشر. بنخلّيه `0.x` ومش هننشر على crates.io قبل v1 بتاع مرآة.

### المخاطر
- 🚨 **الـ refactor يغيّر السلوك من غير ما نحس.** الحل: قبل ما نبدأ نسجّل snapshots من الـ binary الحالي، وبعد كل batch لازم تطلع متطابقة. ونفضل نشغّل الـ 20 اختبار والـ e2e (Ink وvim) والـ corpus.

## خطة التحقق
- **Metric 1:** الـ CI أخضر: `fmt` و`clippy -D warnings` و`test` و`wasm32 build` لـ mirah-bidi.
- **Metric 2:** مخرجات الـ binary على الـ corpus والجلسات المسجّلة متطابقة 100% قبل التقسيم وبعده.
- **Metric 3:** مفيش ملف أكتر من 600 سطر، و`run()` يبقى أقل من 40 سطر.
- **Review date:** 2026-12-05، بعد أول feature كبيرة (المؤشر الوهمي أو copy mode). نشوف هل لمسنا أكتر من crate من غير لازمة.

## المراجع
- ADR-0001 (Rust)
- `Plan/Restructure.md` (المهام بالتفصيل)
- `Stacks/Rust.md` في sabry-kit
- Quality-Gate: Maintainability وReliability وObservability
