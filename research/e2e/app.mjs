import React, { useState } from 'react';
import { render, Box, Text, useInput, useApp } from 'ink';
const h = React.createElement;
function App() {
  const [input, setInput] = useState('');
  const [cursor, setCursor] = useState(0);
  const [log, setLog] = useState([]);
  const { exit } = useApp();
  useInput((ch, key) => {
    if (key.escape) return exit();
    if (key.return) {
      setLog(l => [...l, '> ' + input, '● تمام، هعدّل الملف src/main.rs في السطر 42 وأشغّل npm test']);
      setCursor(0);
      return setInput('');
    }
    if (key.leftArrow) return setCursor(c => Math.max(0, c - 1));
    if (key.rightArrow) return setCursor(c => Math.min(input.length, c + 1));
    if (key.backspace || key.delete) {
      if (cursor === 0) return;
      setInput(s => s.slice(0, cursor - 1) + s.slice(cursor));
      return setCursor(c => c - 1);
    }
    setInput(s => s.slice(0, cursor) + ch + s.slice(cursor));
    setCursor(c => c + ch.length);
  });
  const chars = [...input];
  const before = chars.slice(0, cursor).join('');
  const at = chars[cursor] ?? ' ';
  const after = chars.slice(cursor + 1).join('');
  return h(Box, { flexDirection: 'column' },
    h(Box, { borderStyle: 'round', paddingX: 1, flexDirection: 'column', width: 60 },
      h(Text, null, 'مرحبا بك في Claude Code'),
      h(Text, { dimColor: true }, 'cwd: ~/mirah  ·  الحالة: جاهز')),
    ...log.map((l, i) => h(Text, { key: i }, l)),
    h(Box, { borderStyle: 'single', width: 60 },
      h(Text, null, '> ' + before, h(Text, { inverse: true }, at), after)));
}
render(h(App));
