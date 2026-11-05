import sys, time
for f in range(200):
    out = ['\x1b[H\x1b[2J']
    for i in range(38):
        out.append(f"│ {f:4d}:{i:2d} مرحبا بك في السطر رقم {i} من الملف src/main.rs والحالة: جاهز │ status: ok\r\n")
    sys.stdout.write(''.join(out)); sys.stdout.flush()
    time.sleep(0.01)
