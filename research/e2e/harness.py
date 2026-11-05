import os, pty, sys, time, select, pyte
ROWS, COLS = 20, 80
binary, cmd = sys.argv[1], sys.argv[2:]
pid, fd = pty.fork()
if pid == 0:
    os.environ['TERM'] = 'xterm-256color'
    os.execvp(binary, [binary] + cmd)
import fcntl, termios, struct
fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack('HHHH', ROWS, COLS, 0, 0))
os.kill(pid, 28)
class Host(pyte.Screen):
    def write_process_input(self, data):
        os.write(fd, data.encode())
screen = Host(COLS, ROWS)
stream = pyte.ByteStream(screen)
raw = bytearray()
def pump(t):
    end = time.time() + t
    while time.time() < end:
        r, _, _ = select.select([fd], [], [], 0.05)
        if r:
            try: data = os.read(fd, 65536)
            except OSError: return
            raw.extend(data); stream.feed(data)
def show(title):
    print(f'===== {title} (cursor row={screen.cursor.y} col={screen.cursor.x})')
    for line in screen.display:
        if line.strip(): print('|' + line.rstrip())
pump(2.0); show('startup')
for ch in 'اكتب ملف main.rs':
    os.write(fd, ch.encode()); pump(0.05)
pump(0.5); show('typed')
os.write(fd, b'\r'); pump(0.8); show('after enter')
os.write(fd, b'\x1b'); pump(1.0)
_, status = os.waitpid(pid, 0)
print('exit status', os.waitstatus_to_exitcode(status))
print('BDSM off sent:', b'\x1b[8l' in raw, '| restored:', b'\x1b[8h' in raw, '| bytes:', len(raw))
