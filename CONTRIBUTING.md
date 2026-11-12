<div dir="rtl">

# المساهمة في مرآة

[English](#contributing-to-mirah)

شكراً إنك عايز تساعد. مرآة من [خيمة (Khaymah)](https://github.com/Khaymah)، وأي حاجة بتفرق، حتى لو مش كود.

**عايز تسأل أو تتناقش قبل ما تبدأ؟** [النقاشات على قبيلة](https://qabilah.com/communities/Khaymah) أو [جروب الواتساب](https://chat.whatsapp.com/GIixmbibIix8PyUzFBbNoq).

## من غير كود

- **سطر عربي بيظهر غلط؟** [افتح issue](https://github.com/Khaymah/mirah/issues/new/choose)، ومعاه screenshot وإسم البرنامج والترمنال.
- **جرّبت مرآة على ترمنال مش في الجدول؟** قولنا النتيجة في issue.
- **تسجيل `.vt` للمشكلة** بيوفّر وقت كتير. شغّل `mirah --record <البرنامج>`، ومرآة بتكتبلك الـ ID وهي خارجة، والتسجيل بيبقى في فولدر الجلسة. **راجعه قبل ما تبعته:** ممكن يبقى فيه مسارات من جهازك أو أي حاجة ظهرت على الشاشة.

## حالة للـ corpus

الـ corpus هو أسهل طريقة تثبّت بيها إن سطر بيظهر صح، وإنه هيفضل صح:

1. ضيف سطرين في `corpus/rows/plain.txt`:
   ```
   L|السطر زي ما البرنامج كتبه
   V|السطر زي ما المفروض يظهر على الشاشة، من الشمال لليمين
   ```
2. شغّل `cargo test -p mirah-bidi --test corpus`.
3. لو الاختبار فشل، هيطبع اللي مرآة رسمته (`got`). لو اللي رسمته صح، يبقى غلطت في سطر `V`. ولو غلط، يبقى لقيت bug، وابعت الـ PR كده.

> `plain.txt` من غير توصيل حروف، فتقدر تكتب `V` بحروف عادية. باقي الملفات بتستخدم أشكال الحروف الموصولة.

## الكود

محتاج Rust 1.97 أو أحدث. قبل أي PR:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- **مفيش comments في الكود.** السبب وراء أي قرار بيتكتب في `Plan/ADR/`. الاستثناء الوحيد: comment بيبدأ بـ `IMPORTANT:` لحاجة حرجة فعلاً.
- **`unwrap` و`expect` و`panic` ممنوعين** برا الاختبارات.
- **كل تصليح معاه اختبار** بيفشل من غير التصليح.
- **رسالة الـ commit بالإنجليزي.**
- **أي حاجة اليوزر بيشوفها** (أمر، إعداد، شكل) بتتوثّق في `README.md` و`README.en.md` في نفس الـ PR.
- **واجهة مرآة نفسها** ليها قواعد في [`.claude/skills/mirah-ui`](.claude/skills/mirah-ui/SKILL.md): الاتجاه، والنصوص بالفصحى المبسّطة، والرموز، والألوان.

### فين إيه

| المكان | فيه إيه |
|---|---|
| `crates/mirah-bidi` | الترتيب وتوصيل الحروف. من غير I/O |
| `crates/mirah-term` | المحاكي والرسم والمحاذاة. بيتختبر من غير pty |
| `crates/mirah` | البرنامج: التابات، والشريط، والإعدادات، والجلسات، والتحديث |
| `corpus/` | سطور وتسجيلات بالشكل المتوقع ليها |
| `Plan/ADR` | القرارات وأسبابها |

## الإصدارات

مفيش حد بيعمل tag بإيده. ارفع النسخة في `Cargo.toml`، وضيف قسم في `CHANGELOG.md`، والـ CI بيعمل الباقي. التفاصيل في [Plan/Releasing.md](Plan/Releasing.md).

## الأمان

ماتفتحش issue عام لثغرة أمنية. اتبع [SECURITY.md](SECURITY.md).

</div>

---

# Contributing to mirah

Thanks for wanting to help. mirah is made by [Khaymah (خيمة)](https://github.com/Khaymah), and everything counts, code or not.

**Want to ask or discuss before starting?** [Discussions on Qabilah](https://qabilah.com/communities/Khaymah) or the [WhatsApp group](https://chat.whatsapp.com/GIixmbibIix8PyUzFBbNoq).

## Without code

- **An Arabic line shows wrong?** [Open an issue](https://github.com/Khaymah/mirah/issues/new/choose) with a screenshot, the program and the terminal.
- **Tried mirah on a terminal missing from the table?** Tell us in an issue.
- **A `.vt` recording of the problem** saves a lot of time. Run `mirah --record <program>`; mirah prints the session ID on exit, and the recording is in the session folder. **Review it before sending:** it can contain paths from your machine or anything that appeared on screen.

## A corpus case

The corpus is the easiest way to prove a line shows right and keeps showing right:

1. Add two lines to `corpus/rows/plain.txt`:
   ```
   L|the line as the program wrote it
   V|the line as it should appear on screen, left to right
   ```
2. Run `cargo test -p mirah-bidi --test corpus`.
3. If it fails, it prints what mirah drew (`got`). If that is right, your `V` line is wrong; if not, you found a bug, so send the PR as it is.

> `plain.txt` has no letter joining, so `V` can use plain letters. The other files use joined letter forms.

## Code

Needs Rust 1.97 or newer. Before any PR:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- **No comments in code.** The reason behind a decision goes in `Plan/ADR/`. The only exception is a comment starting with `IMPORTANT:` for something truly critical.
- **No `unwrap`, `expect` or `panic`** outside tests.
- **Every fix comes with a test** that fails without it.
- **Commit messages in English.**
- **Anything a user sees** (a command, a setting, a look) is documented in `README.md` and `README.en.md` in the same PR.
- **mirah's own interface** has rules in [`.claude/skills/mirah-ui`](.claude/skills/mirah-ui/SKILL.md): direction, simplified Standard Arabic wording, glyphs and colours.

### Where things are

| Path | Contents |
|---|---|
| `crates/mirah-bidi` | Ordering and letter joining. No I/O |
| `crates/mirah-term` | The emulator, rendering and alignment. Tested without a pty |
| `crates/mirah` | The program: tabs, the bar, settings, sessions, updates |
| `corpus/` | Lines and recordings with their expected look |
| `Plan/ADR` | Decisions and their reasons |

## Releases

Nobody tags by hand. Bump the version in `Cargo.toml`, add a section to `CHANGELOG.md`, and CI does the rest. Details in [Plan/Releasing.md](Plan/Releasing.md).

## Security

Do not open a public issue for a security problem. Follow [SECURITY.md](SECURITY.md).
