import os, pty, sys, time, select, fcntl, termios, struct, re
out, keys, secs, argv = sys.argv[1], sys.argv[2].replace('\\r', '\r').replace('\\e', '\x1b').encode('utf-8'), float(sys.argv[3]), sys.argv[4:]
pid, fd = pty.fork()
if pid == 0:
    os.environ['TERM'] = 'xterm-256color'; os.environ['COLUMNS']='80'; os.environ['LINES']='24'
    os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'e2e')); os.execvp(argv[0], argv)
fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack('HHHH', 24, 80, 0, 0))
raw = bytearray(); t0 = time.time(); sent = False
while time.time() - t0 < secs:
    r, _, _ = select.select([fd], [], [], 0.05)
    if r:
        try: d = os.read(fd, 65536)
        except OSError: break
        raw += d
        if b'\x1b[6n' in d: os.write(fd, b'\x1b[1;1R')
        if re.search(rb'\x1b\[0?c', d): os.write(fd, b'\x1b[?62;22c')
    if not sent and time.time() - t0 > secs / 2:
        for token in re.findall(r'\x1b\[[0-9;]*[A-Za-z~]|.', keys.decode('utf-8'), re.S):
            os.write(fd, token.encode()); time.sleep(0.05)
        sent = True
open(out, 'wb').write(raw); print(out, len(raw), 'bytes')
try: os.kill(pid, 9)
except Exception: pass
