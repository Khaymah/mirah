import base64, os, pty, sys, time, select, fcntl, termios, struct, json
def capture(argv, out, cols=100, rows=30, keys=(), total=8.0, env=None):
    pid, fd = pty.fork()
    if pid == 0:
        os.execvpe(argv[0], argv, env or os.environ)
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
    start = time.time(); chunks = []; pending = sorted(keys)
    while time.time() - start < total:
        now = time.time() - start
        while pending and pending[0][0] <= now:
            os.write(fd, pending.pop(0)[1].encode() if isinstance(pending[0][1], str) else pending.pop(0)[1])
        r, _, _ = select.select([fd], [], [], 0.02)
        if r:
            try: d = os.read(fd, 65536)
            except OSError: break
            if not d: break
            if b"\x1b[6n" in d: os.write(fd, b"\x1b[1;1R")
            chunks.append([round(time.time() - start, 3), base64.b64encode(d).decode()])
    try: os.kill(pid, 9)
    except Exception: pass
    json.dump({"cols": cols, "rows": rows, "chunks": chunks}, open(out, "w"), ensure_ascii=False)
if __name__ == "__main__":
    spec = json.load(open(sys.argv[1]))
    env = dict(os.environ, **spec.get("env", {}))
    capture(spec["argv"], spec["out"], spec.get("cols", 100), spec.get("rows", 30),
            [(k[0], k[1]) for k in spec.get("keys", [])], spec.get("total", 8.0), env)
