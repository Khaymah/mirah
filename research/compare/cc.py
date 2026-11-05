import os, pty, sys, time, select, fcntl, termios, struct, signal, pyte
ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..'))
E2E = os.path.join(ROOT, 'research', 'e2e')
BIN = os.environ.get('MIRAH_BIN', os.path.join(ROOT, 'target', 'release', 'mirah'))
class S(pyte.Screen):
    def select_graphic_rendition(self, *a, **k): k.pop('private', None); return super().select_graphic_rendition(*a, **k)
def run(argv):
    pid, fd = pty.fork()
    if pid == 0:
        os.environ['TERM'] = 'xterm-256color'; os.chdir(E2E); os.execvp(argv[0], argv)
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack('HHHH', 24, 80, 0, 0)); os.kill(pid, signal.SIGWINCH)
    s = S(80, 24); s.write_process_input = lambda d: os.write(fd, d.encode()); st = pyte.ByteStream(s)
    t0 = time.time()
    while time.time() - t0 < 6:
        r, _, _ = select.select([fd], [], [], 0.05)
        if r:
            try: st.feed(os.read(fd, 65536))
            except OSError: break
    os.write(fd, b'\x1b[B'); t1 = time.time()
    while time.time() - t1 < 1.5:
        r, _, _ = select.select([fd], [], [], 0.05)
        if r:
            try: st.feed(os.read(fd, 65536))
            except OSError: break
    os.kill(pid, 9)
    return [l.rstrip() for l in s.display]
a = run(['claude']); b = run([BIN, 'claude'])
same = sum(1 for x, y in zip(a, b) if x == y)
print(f'rows identical: {same}/24')
for x, y in zip(a, b):
    if x != y: print('direct:', repr(x)); print('mirah :', repr(y))
print('\n'.join(l for l in b if l)[:900])
