const { chromium } = require('playwright');
(async () => {
  const files = require('fs').readFileSync(0, 'utf8').trim().split('\n');
  const b = await chromium.launch();
  const p = await b.newPage({ deviceScaleFactor: 2, viewport: { width: 1000, height: 700 } });
  for (const f of files) {
    await p.goto('file://' + f);
    await p.evaluate(() => document.fonts.ready);
    await (await p.$('#w')).screenshot({ path: f.replace(/\.html$/, '.png') });
  }
  await b.close();
})();
