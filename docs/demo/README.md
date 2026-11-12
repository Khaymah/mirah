# صور الـ README · README images

[`docs/images`](../images) اتعملت من تشغيل حقيقي لمرآة، بالسكربتات اللي هنا:

The images in [`docs/images`](../images) come from a real run of mirah, made with these scripts:

| الملف · File | بيعمل إيه · What it does |
|---|---|
| `demo.sh` | رد عربي مكتوب، شبه رد agent · A scripted Arabic reply, shaped like an agent's |
| `capture.py` | بيشغّل أمر في pty ويسجّل الـ output بوقته · Runs a command in a pty and records timed output |
| `grid.py` | بيحوّل الـ output لصفحة HTML خلية خلية، زي ترمنال من غير bidi · Turns the output into an HTML grid, cell by cell, like a terminal without bidi |
| `shoot.js` | بيصوّر الصفحات بـ Playwright · Screenshots the pages with Playwright |

```bash
python3 capture.py spec.json
python3 grid.py out.cap.json /abs/path/frame 4.4 /abs/path/KawkabMono-Regular.ttf nocursor | node shoot.js
ffmpeg -framerate 10 -i frame-%03d.png demo.gif
```

`spec.json` = `{"argv": ["mirah", "demo"], "out": "out.cap.json", "total": 5, "env": {"MIRAH_SHAPING": "full"}}`، ومحتاج `pyte` و`playwright` وخط [Kawkab Mono](http://makkuk.com/kawkab-mono).
