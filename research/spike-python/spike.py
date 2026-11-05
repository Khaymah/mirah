import os, pty, sys, re, select
import pyte
from bidi.algorithm import get_display
import arabic_reshaper

R = arabic_reshaper.ArabicReshaper({'delete_harakat': False, 'support_ligatures': True})
ARAB = re.compile(r'[֐-ࣿ]')
SEP = re.compile(r'([│┃║╭╮╰╯─━┌┐└┘├┤┬┴┼]+|\s{2,})')

def vis(s):
    if not ARAB.search(s):
        return s
    return get_display(R.reshape(s), base_dir='R' if ARAB.search(s[:1]) else None)

def naive(row):
    return vis(row.rstrip())

def segmented(row):
    out = []
    for part in SEP.split(row.rstrip()):
        if not part or SEP.fullmatch(part):
            out.append(part)
        else:
            lead = len(part) - len(part.lstrip())
            t = part.strip()
            v = vis(t) if ARAB.search(t) else t
            out.append(part[:lead] + v + part[len(part.rstrip()):])
    return ''.join(out)

def run():
    pid, fd = pty.fork()
    if pid == 0:
        os.execvp(sys.executable, [sys.executable, os.path.join(os.path.dirname(os.path.abspath(__file__)), 'child.py')])
    scr = pyte.Screen(60, 10)
    st = pyte.ByteStream(scr)
    while True:
        try:
            r, _, _ = select.select([fd], [], [], 2)
            if not r: break
            d = os.read(fd, 4096)
        except OSError:
            break
        if not d: break
        st.feed(d)
    return scr

scr = run()
rows = [l.rstrip() for l in scr.display if l.strip()]
print("== logical =="); [print(l) for l in rows]
print("== naive whole-row =="); [print(naive(l)) for l in rows]
print("== segmented =="); [print(segmented(l)) for l in rows]
print("== widths (len vs len after reshape, row 1) ==")
l = rows[0]; print(len(l), len(R.reshape(l)))
lam = "لا"; print("lam-alef cells:", len(lam), "->", len(R.reshape(lam)))
