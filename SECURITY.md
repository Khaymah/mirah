<div dir="rtl">

# الأمان

[English](#security-policy)

## النسخ المدعومة

بنصلّح المشاكل الأمنية في آخر نسخة مستقرة بس. حدّث بـ `mirah update`.

## إزاي تبلّغ عن ثغرة

**ماتفتحش issue عام.** بلّغ بشكل خاص من [Report a vulnerability](https://github.com/Khaymah/mirah/security/advisories/new) في تبويب Security في الريبو.

ابعت:
- إيه المشكلة، وإزاي توصلها خطوة بخطوة.
- النسخة (`mirah --version`)، والنظام، والترمنال.
- لو فيه تسجيل أو log، **شيل منه أي أسرار أو مسارات** قبل ما تبعته.

هنرد عليك في خلال أسبوع، وهنقولك هنصلّحها إمتى. ولما التصليح ينزل، هنكتب اسمك في الـ CHANGELOG لو حابب.

## إيه اللي يعتبر مشكلة أمنية

- برنامج شغال جوه مرآة يقدر يخلّي مرآة تنفّذ أوامر، أو تكتب ملفات برا فولدر الجلسات، أو تبعت حاجة للترمنال من غير ما البرنامج يقصد.
- `mirah update` يسطّب ملف مش مطابق للـ release.
- تسجيلات أو logs بتتحفظ في مكان الناس التانية تقدر تقراه.

</div>

---

# Security policy

## Supported versions

Security fixes go into the latest stable release only. Update with `mirah update`.

## Reporting a vulnerability

**Do not open a public issue.** Report it privately through [Report a vulnerability](https://github.com/Khaymah/mirah/security/advisories/new) in the repository's Security tab.

Include:
- What the problem is, and how to reproduce it step by step.
- The version (`mirah --version`), the OS and the terminal.
- Any recording or log, **with secrets and paths removed**.

We reply within a week and tell you when a fix will ship. Once it ships, we credit you in the CHANGELOG if you want.

## What counts

- A program running inside mirah making mirah run commands, write files outside the sessions folder, or send something to the terminal the program did not intend.
- `mirah update` installing a file that does not match the release.
- Recordings or logs being saved where other users can read them.
