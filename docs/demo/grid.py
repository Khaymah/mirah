import base64, json, sys, html, pyte
FG, BG = "#d4d4d8", "#0c0c0c"
NAMED = {"black":"#1e1f23","red":"#e06c75","green":"#98c379","brown":"#e5c07b","blue":"#61afef","magenta":"#c678dd","cyan":"#56b6c2","white":"#d4d4d8"}
def color(c, default):
    if c == "default": return default
    if len(c) == 6 and all(x in "0123456789abcdefABCDEF" for x in c): return "#" + c
    return NAMED.get(c, default)
def frame(screen, cw, ch, cursor=True):
    rows = []
    for y in range(screen.lines):
        cells = []
        line = screen.buffer[y]
        x = 0
        while x < screen.columns:
            c = line[x]; fg = color(c.fg, FG); bg = color(c.bg, BG)
            if c.reverse: fg, bg = bg, fg
            if cursor and screen.cursor.y == y and screen.cursor.x == x and not screen.cursor.hidden:
                bg = "#d4d4d8"; fg = "#0c0c0c"
            w = "font-weight:700;" if c.bold else ""
            cells.append(f'<span style="width:{cw}px;color:{fg};background:{bg};{w}">{html.escape(c.data or " ")}</span>')
            x += 1
        rows.append('<div class="r">' + "".join(cells) + "</div>")
    return "".join(rows)
def page(body, cols, cw, ch, font):
    return f'''<!doctype html><meta charset="utf-8"><style>
@font-face{{font-family:K;src:url("file://{font}")}}
body{{margin:0;background:#1a1b1f}}
#w{{display:inline-block;padding:16px;background:{BG};border-radius:10px}}
.r{{height:{ch}px;white-space:pre;display:flex}}
.r span{{display:inline-block;text-align:center;font:15px/{ch}px K,"DejaVu Sans Mono",monospace;unicode-bidi:isolate-override;direction:ltr;overflow:visible}}
</style><div id="w">{body}</div>'''
if __name__ == "__main__":
    cap = json.load(open(sys.argv[1])); out = sys.argv[2]; marks = [float(t) for t in sys.argv[3].split(",")]
    font = sys.argv[4]; cursor = len(sys.argv) < 6
    sc = pyte.Screen(cap["cols"], cap["rows"]); st = pyte.ByteStream(sc)
    i = 0; pages = []
    for n, t in enumerate(marks):
        while i < len(cap["chunks"]) and cap["chunks"][i][0] <= t:
            st.feed(base64.b64decode(cap["chunks"][i][1])); i += 1
        p = f"{out}-{n:03d}.html"
        open(p, "w").write(page(frame(sc, 9, 20, cursor), cap["cols"], 9, 20, font)); pages.append(p)
    print("\n".join(pages))
