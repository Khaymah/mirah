package goport

import ob "github.com/go-opentype/bidi"

type Style struct {
	FG, BG  uint32
	Inverse bool
}

type Cell struct {
	Text      string
	Style     Style
	Wide, Cnt bool
}

func baseOf(c Cell) rune {
	for _, r := range c.Text {
		return r
	}
	return ' '
}

func isRTL(r rune) bool {
	c := ob.ClassOf(r)
	return c == ob.R || c == ob.AL
}

func isStrong(r rune) bool {
	c := ob.ClassOf(r)
	return c == ob.L || c == ob.R || c == ob.AL
}

func isFrame(r rune) bool { return r >= 0x2500 && r <= 0x259F }

func hasRTL(cells []Cell) bool {
	for _, c := range cells {
		for _, r := range c.Text {
			if isRTL(r) {
				return true
			}
		}
	}
	return false
}

func fixedColumns(row []Cell) []bool {
	fixed := make([]bool, len(row))
	for i, c := range row {
		fixed[i] = c.Wide || c.Cnt || isFrame(baseOf(c))
	}
	for i := 0; i < len(row); {
		if baseOf(row[i]) != ' ' {
			i++
			continue
		}
		s := i
		for i < len(row) && baseOf(row[i]) == ' ' {
			i++
		}
		if i-s >= 2 {
			for k := s; k < i; k++ {
				fixed[k] = true
			}
		}
	}
	return fixed
}

func segments(row []Cell) [][2]int {
	fixed := fixedColumns(row)
	var out [][2]int
	for i := 0; i < len(row); {
		if fixed[i] {
			i++
			continue
		}
		s := i
		key := row[i].Style
		for i < len(row) && !fixed[i] && row[i].Style.BG == key.BG && row[i].Style.Inverse == key.Inverse {
			i++
		}
		out = append(out, [2]int{s, i})
	}
	return out
}

func trim(row []Cell, a, b int) (int, int) {
	for a < b && !isStrong(baseOf(row[a])) {
		a++
	}
	for b > a && baseOf(row[b-1]) == ' ' {
		b--
	}
	return a, b
}

func reorderSegment(row []Cell, a, b int, out []Cell, l2v []int) {
	texts := make([]string, b-a)
	for i := range texts {
		texts[i] = row[a+i].Text
	}
	type item struct {
		col  int
		text string
	}
	var items, pads []item
	for off, g := range Shape(texts) {
		if g.Absorbed {
			pads = append(pads, item{a + off, " "})
		} else {
			items = append(items, item{a + off, g.Text})
		}
	}
	items = append(items, pads...)
	runes := make([]rune, len(items))
	for i, it := range items {
		if it.text == " " {
			runes[i] = ' '
		} else {
			runes[i] = baseOf(row[it.col])
		}
	}
	levels := ob.ResolveLevels(runes, ob.Auto)
	mirrored := ob.MirrorRunes([]rune(func() string {
		s := make([]rune, len(items))
		for i, it := range items {
			s[i] = []rune(it.text)[0]
		}
		return string(s)
	}()), levels)
	for slot, idx := range ob.Reorder(runes, levels) {
		it := items[idx]
		text := it.text
		if levels[idx]%2 == 1 {
			rs := []rune(text)
			rs[0] = mirrored[idx]
			text = string(rs)
		}
		out[a+slot] = Cell{Text: text, Style: row[it.col].Style}
		l2v[it.col] = a + slot
	}
}

func VisualRow(row []Cell) ([]Cell, []int) {
	out := append([]Cell(nil), row...)
	l2v := make([]int, len(row))
	for i := range l2v {
		l2v[i] = i
	}
	if !hasRTL(row) {
		return out, l2v
	}
	for _, seg := range segments(row) {
		a, b := trim(row, seg[0], seg[1])
		if a < b && hasRTL(row[a:b]) {
			reorderSegment(row, a, b, out, l2v)
		}
	}
	return out, l2v
}
