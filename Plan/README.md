# مرآة (mirah): خطة المشروع

| الملف | فيه إيه |
|---|---|
| [Architecture.md](Architecture.md) | المعمارية والرسومات والمخاطر التقنية |
| [ADR/0001-rust-as-language.md](ADR/0001-rust-as-language.md) | ليه Rust |
| [ADR/0002-cargo-workspace-layout.md](ADR/0002-cargo-workspace-layout.md) | تقسيم المشروع لـ 3 crates (مقبول ومنفّذ) |
| [Restructure.md](Restructure.md) | خطة إعادة الهيكلة بالمهام |
| [Research/RustVsGo.md](Research/RustVsGo.md) | المقارنة العملية بين Rust وGo |
| [RightAlign.md](RightAlign.md) | المحادثة من اليمين للشمال، والجداول، ووضع Windows Terminal (منفّذ في alpha.2) |
| [Releasing.md](Releasing.md) | إزاي بنطلّع release (أوتوماتيك من الـ CI) |
| [FakeCursor.md](FakeCursor.md) | تصليح المؤشر الوهمي جوه الكلام العربي (منفّذ) |
| [ADR/0003-windows-first-releases-and-updates.md](ADR/0003-windows-first-releases-and-updates.md) | ويندوز أولوية، وreleases، وتحديث بإذن المستخدم (مقبول) |
| [ADR/0004-bundle-conpty-on-windows.md](ADR/0004-bundle-conpty-on-windows.md) | مرآة بتشيل نسخة ConPTY الجديدة جواها على ويندوز (مقبول) |
| [ADR/0005-top-bar.md](ADR/0005-top-bar.md) | شريط مرآة فوق البرنامج، وملف إعدادات (مقبول ومنفّذ) |
| [ADR/0006-workspace-tabs-sessions.md](ADR/0006-workspace-tabs-sessions.md) | تابات، وشاشة اختيار البرنامج، وجلسات بـ ID (مقبول ومنفّذ في 0.3.0-alpha.1) |
| [ADR/0007-markdown-viewer.md](ADR/0007-markdown-viewer.md) | عارض ملفات Markdown جوه مرآة (مقبول ومنفّذ) |
| [Workspace.md](Workspace.md) | خطة التابات والجلسات والشكل الجديد (منفّذة في 0.3.0-alpha.1) |
| [TopBar.md](TopBar.md) | خطة الشريط بالمراحل (منفّذة في 0.2.0-alpha.1) |
| [WindowsAndReleases.md](WindowsAndReleases.md) | خطة ويندوز والـ releases والتحديث بالمهام (منفّذة، ومكتوبة وقت ما الريبو كان private) |
| [Roadmap.md](Roadmap.md) | أول خارطة طريق، قبل أول سطر كود (للتاريخ) |
| [Progress/](Progress/README.md) | سجل الشغل اليومي |

**الحالة (2026-11-12):** أول إصدار مستقر هو 0.4.0. أول قرار كان Adjust ببوابة تحقق مدتها أسبوعين.

## الكود

| المسار | فيه إيه |
|---|---|
| `crates/mirah-bidi/` | المنطق الصافي: التقسيم، والـ bidi، والتشكيل. من غير I/O، وبيتعمله build لـ WASM |
| `crates/mirah-term/` | من bytes لـ frames: المحاكي، والرسم، والرد على أسئلة الترمنال، والـ escape sequences |
| `crates/mirah/` | البرنامج: التابات، والشريط، والإعدادات، والجلسات، والتحديث، والـ pty |
| `corpus/` | بيانات الاختبار: سطور (`rows/`) وجلسات مسجّلة (`sessions/`) |
| `research/e2e/` | اختبارات end-to-end: تطبيق Ink، وvim، والأداء |
