import React, { useState } from 'react';
import { render, Box, Text, useInput, useApp } from 'ink';
const h = React.createElement;
function App() {
  const [input, setInput] = useState('');
  const [log, setLog] = useState([]);
  const { exit } = useApp();
  useInput((ch, key) => {
    if (key.escape) return exit();
    if (key.return) {
      setLog(l => [...l, '> ' + input, '● تمام، هعدّل الملف src/main.rs في السطر 42 وأشغّل npm test']);
      return setInput('');
    }
    if (key.backspace || key.delete) return setInput(s => s.slice(0, -1));
    setInput(s => s + ch);
  });
  return h(Box, { flexDirection: 'column' },
    h(Box, { borderStyle: 'round', paddingX: 1, flexDirection: 'column', width: 60 },
      h(Text, null, 'مرحبا بك في Claude Code'),
      h(Text, { dimColor: true }, 'cwd: ~/mirah  ·  الحالة: جاهز')),
    ...log.map((l, i) => h(Text, { key: i }, l)),
    h(Box, { borderStyle: 'single', width: 60 }, h(Text, null, '> ' + input), h(Text, { inverse: true }, ' ')));
}
render(h(App));
