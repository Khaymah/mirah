import os
BIN = os.environ.get('MIRAH_BIN', os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', 'target', 'release', 'mirah'))
import os, pty, sys, time, select, pyte, fcntl, termios, struct, signal
ROWS, COLS = 12, 60
pid, fd = pty.fork()
if pid == 0:
    os.environ['TERM'] = 'xterm-256color'
    os.execvp(BIN, ['mirah', 'vim', '-u', 'NONE', '-n', 'ar.txt'])
fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack('HHHH', ROWS, COLS, 0, 0)); os.kill(pid, signal.SIGWINCH)
class Host(pyte.Screen):
    def write_process_input(self, data): os.write(fd, data.encode())
screen = Host(COLS, ROWS); stream = pyte.ByteStream(screen)
def pump(t):
    end = time.time() + t
    while time.time() < end:
        r, _, _ = select.select([fd], [], [], 0.05)
        if r:
            try: stream.feed(os.read(fd, 65536))
            except OSError: return
def show(t):
    print(f'== {t} cursor=({screen.cursor.y},{screen.cursor.x})')
    for l in screen.display[:4]: print('|' + l.rstrip())
pump(1.5); show('vim open')
os.write(fd, b'jw'); pump(0.5); show('after j w (cursor on 2nd word of line 2)')
fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack('HHHH', 10, 40, 0, 0)); screen.resize(10, 40); os.kill(pid, signal.SIGWINCH)
pump(1.0); show('after resize to 40 cols')
os.write(fd, b':q!\r'); pump(1.0)
_, st = os.waitpid(pid, 0); print('exit', os.waitstatus_to_exitcode(st))
