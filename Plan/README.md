# مرآة (mirah): خطة المشروع

| الملف | فيه إيه |
|---|---|
| [MarketResearch.md](MarketResearch.md) | بحث السوق والطلب والمنافسين وجدول الترمنالات والقرار |
| [Architecture.md](Architecture.md) | المعمارية والرسومات والمخاطر التقنية |
| [ADR/0001-Stack.md](ADR/0001-Stack.md) | ليه Rust |
| [Roadmap.md](Roadmap.md) | المراحل والبوابات والمهام بالتفصيل |
| [Progress/](Progress/README.md) | سجل الشغل اليومي |

**القرار:** Adjust. نبنيها بشرط إنها تعدّي بوابة تحقق مدتها أسبوعين الأول.

## الكود

| المسار | فيه إيه |
|---|---|
| `src/` | الأداة نفسها (Rust): `main` · `bridge` · `render` · `segment` · `shape` |
| `research/e2e/` | اختبارات end-to-end: تطبيق Ink، وvim، والأداء |
| `research/compare/` | مقارنة Rust وGo: الـ conformance، ومحاكيات الترمنال، والـ port بـ Go |
| `research/spike-python/` | أول spike بـ Python |
