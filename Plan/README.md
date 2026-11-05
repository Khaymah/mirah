# مرآة (mirah): خطة المشروع

| الملف | فيه إيه |
|---|---|
| [MarketResearch.md](MarketResearch.md) | بحث السوق والطلب والمنافسين وجدول الترمنالات والقرار |
| [Architecture.md](Architecture.md) | المعمارية والرسومات والمخاطر التقنية |
| [ADR/0001-rust-as-language.md](ADR/0001-rust-as-language.md) | ليه Rust |
| [ADR/0002-cargo-workspace-layout.md](ADR/0002-cargo-workspace-layout.md) | تقسيم المشروع لـ 3 crates (مقبول ومنفّذ) |
| [Restructure.md](Restructure.md) | خطة إعادة الهيكلة بالمهام |
| [Research/RustVsGo.md](Research/RustVsGo.md) | المقارنة العملية بين Rust وGo |
| [FakeCursor.md](FakeCursor.md) | تصليح المؤشر الوهمي جوه الكلام العربي (منفّذ) |
| [Roadmap.md](Roadmap.md) | المراحل والبوابات والمهام بالتفصيل |
| [Progress/](Progress/README.md) | سجل الشغل اليومي |

**القرار:** Adjust. نبنيها بشرط إنها تعدّي بوابة تحقق مدتها أسبوعين الأول.

## الكود

| المسار | فيه إيه |
|---|---|
| `crates/mirah-bidi/` | المنطق الصافي: التقسيم، والـ bidi، والتشكيل. من غير I/O، وبيتعمله build لـ WASM |
| `crates/mirah-term/` | من bytes لـ frames: المحاكي، والرسم، والرد على أسئلة الترمنال، والـ escape sequences |
| `crates/mirah/` | الـ binary: الـ pty، والـ threads، والـ logging، وترجيع الترمنال لحالته |
| `corpus/` | بيانات الاختبار: سطور (`rows/`) وجلسات مسجّلة (`sessions/`) |
| `research/e2e/` | اختبارات end-to-end: تطبيق Ink، وvim، والأداء |
| `research/compare/` | مقارنة Rust وGo: الـ conformance، ومحاكيات الترمنال، والـ port بـ Go |
| `research/spike-python/` | أول spike بـ Python |
