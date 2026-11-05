<div dir="rtl">

# ConPTY من Microsoft

الملفات دي من NuGet package اسمها `Microsoft.Windows.Console.ConPTY`، نسخة `1.25.260930003`، بترخيص MIT (راجع `LICENSE`). مرآة بتحطها جوه `mirah.exe` وبتفكّها في `%LOCALAPPDATA%\mirah\conpty\` على ويندوز. السبب في [ADR-0004](../../../../Plan/ADR/0004-bundle-conpty-on-windows.md).

- SHA-256 بتاع الـ package: `02b07b349af66d801159bdf9e440d4a1ce78bb951f37fc8609731665afdae7ee`
- SHA-256 بتاع كل ملف: `SHA256SUMS`

## التحديث

1. نزّل النسخة الجديدة من `https://api.nuget.org/v3-flatcontainer/microsoft.windows.console.conpty/<version>/microsoft.windows.console.conpty.<version>.nupkg`.
2. خد منها `runtimes/win-{x64,arm64}/native/conpty.dll` و`build/native/runtimes/{x64,arm64}/OpenConsole.exe`.
3. حدّث `SHA256SUMS` (`sha256sum x64/* arm64/*`)، و`VERSION` في `crates/mirah/src/host/windows/conpty.rs`، والـ SHA-256 اللي فوق.

</div>
