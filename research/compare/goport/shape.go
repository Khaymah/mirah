package goport

import ob "github.com/go-opentype/bidi"

var persian = map[rune][4]rune{
	0x067E: {0xFB56, 0xFB58, 0xFB59, 0xFB57},
	0x0686: {0xFB7A, 0xFB7C, 0xFB7D, 0xFB7B},
	0x0698: {0xFB8A, 0, 0, 0xFB8B},
	0x06A9: {0xFB8E, 0xFB90, 0xFB91, 0xFB8F},
	0x06AF: {0xFB92, 0xFB94, 0xFB95, 0xFB93},
	0x06CC: {0xFBFC, 0xFBFE, 0xFBFF, 0xFBFD},
}

var lamAlef = map[rune][2]rune{
	0x0622: {0xFEF5, 0xFEF6},
	0x0623: {0xFEF7, 0xFEF8},
	0x0625: {0xFEF9, 0xFEFA},
	0x0627: {0xFEFB, 0xFEFC},
}

var unshapeTable = func() map[rune]rune {
	m := map[rune]rune{}
	for r := rune(0x0621); r <= 0x06FF; r++ {
		for f := ob.Isolated; f <= ob.Final; f++ {
			if g := present(r, f); g != r {
				m[g] = r
			}
		}
	}
	return m
}()

func present(r rune, f ob.JoinForm) rune {
	if p, ok := persian[r]; ok {
		if g := p[f]; g != 0 {
			return g
		}
		return p[0]
	}
	return ob.PresentationForm(r, f)
}

func unshape(r rune) rune {
	if b, ok := unshapeTable[r]; ok {
		return b
	}
	return r
}

type Glyph struct {
	Text     string
	Absorbed bool
}

func Shape(cells []string) []Glyph {
	base := make([]rune, len(cells))
	for i, c := range cells {
		rs := []rune(c)
		if len(rs) == 0 {
			base[i] = ' '
			continue
		}
		base[i] = unshape(rs[0])
	}
	forms := ob.JoinForms(base)
	out := make([]Glyph, 0, len(cells))
	for i := 0; i < len(cells); i++ {
		marks := string([]rune(cells[i])[min(1, len([]rune(cells[i]))):])
		if base[i] == 0x0644 && i+1 < len(cells) {
			if la, ok := lamAlef[base[i+1]]; ok {
				g := la[0]
				if forms[i] == ob.Final || forms[i] == ob.Medial {
					g = la[1]
				}
				alefMarks := string([]rune(cells[i+1])[1:])
				out = append(out, Glyph{Text: string(g) + marks + alefMarks}, Glyph{Absorbed: true})
				i++
				continue
			}
		}
		g := present(base[i], forms[i])
		if g == base[i] && len(cells[i]) > 0 {
			out = append(out, Glyph{Text: cells[i]})
			continue
		}
		out = append(out, Glyph{Text: string(g) + marks})
	}
	return out
}
