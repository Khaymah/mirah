import os
BIN = os.environ.get('MIRAH_BIN', os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', 'target', 'release', 'mirah'))
import os, pty, sys, time, select, fcntl, termios, struct
def run(argv):
    pid, fd = pty.fork()
    if pid == 0:
        os.execvp(argv[0], argv)
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack('HHHH', 40, 120, 0, 0))
    t0 = time.time(); n = 0
    while True:
        r, _, _ = select.select([fd], [], [], 5)
        if not r: break
        try: d = os.read(fd, 65536)
        except OSError: break
        if not d: break
        n += len(d)
        if b'\x1b[?2026l' not in d and b'[8h' in d: pass
        if d.startswith(b'\x1b[6n') or b'\x1b[c' in d: os.write(fd, b'\x1b[?62c')
    os.waitpid(pid, 0)
    return time.time() - t0, n
N = ''
d, n = run(['python3', 'frames.py']); print(f'direct : {d:.2f}s  {n} bytes')
d, n = run([BIN, 'python3', 'frames.py']); print(f'mirah  : {d:.2f}s  {n} bytes')
