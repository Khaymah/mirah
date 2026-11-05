package goport

import (
	"sort"
	"strings"
	"testing"
)

func row(s string) []Cell {
	var out []Cell
	for _, r := range s {
		out = append(out, Cell{Text: string(r)})
	}
	return out
}

func render(cells []Cell) string {
	var b strings.Builder
	for _, c := range cells {
		b.WriteString(c.Text)
	}
	return b.String()
}

func glyphs(gs []Glyph) string {
	var b strings.Builder
	for _, g := range gs {
		if g.Absorbed {
			b.WriteString("_")
		} else {
			b.WriteString(g.Text)
		}
	}
	return b.String()
}

func cells(s string) []string {
	var out []string
	for _, r := range s {
		out = append(out, string(r))
	}
	return out
}

func eq(t *testing.T, got, want string) {
	t.Helper()
	if got != want {
		t.Fatalf("got %q (%U) want %q (%U)", got, []rune(got), want, []rune(want))
	}
}

func TestShapesWordWithInitialMedialFinal(t *testing.T) {
	eq(t, glyphs(Shape(cells("بيت"))), "ﺑﻴﺖ")
}
func TestRightJoiningLetterBreaksTheChain(t *testing.T) {
	eq(t, glyphs(Shape(cells("دار"))), "ﺩﺍﺭ")
}
func TestLamAlefBecomesOneGlyph(t *testing.T) {
	eq(t, glyphs(Shape(cells("لا"))), "ﻻ_")
	eq(t, glyphs(Shape(cells("سلام"))), "ﺳﻼ_ﻡ")
}
func TestKeepsHarakatAttached(t *testing.T) {
	eq(t, glyphs(Shape([]string{"بَ", "ت"})), "ﺑَﺖ")
}
func TestShapesPersianLetters(t *testing.T) {
	eq(t, glyphs(Shape(cells("پی"))), "ﭘﯽ")
}
func TestReshapesPreShapedText(t *testing.T) {
	eq(t, glyphs(Shape(cells("ﻡﺮﺤﺑﺍ"))), "ﻣﺮﺣﺒﺎ")
}
func TestLeavesLatinUntouched(t *testing.T) { eq(t, glyphs(Shape(cells("ab"))), "ab") }

func TestLeavesLTRRowsUntouched(t *testing.T) {
	out, _ := VisualRow(row("hello │ world"))
	eq(t, render(out), "hello │ world")
}
func TestKeepsFrameBordersInPlace(t *testing.T) {
	out, _ := VisualRow(row("│ مرحبا بك  │  status: جاهز"))
	s := []rune(render(out))
	if s[0] != '│' || s[12] != '│' || !strings.Contains(string(s), "status: ") {
		t.Fatalf("%q", string(s))
	}
}
func TestReversesArabicKeepsLatin(t *testing.T) {
	out, _ := VisualRow(row("افتح src/main.rs الآن"))
	s := render(out)
	if !strings.Contains(s, "src/main.rs") || strings.Index(s, "ﺢ") < strings.Index(s, "src") {
		t.Fatalf("%q", s)
	}
}
func TestKeepsPromptPrefix(t *testing.T) {
	out, _ := VisualRow(row("> مرحبا"))
	if !strings.HasPrefix(render(out), "> ") {
		t.Fatal(render(out))
	}
}
func TestKeepsWidthAfterLamAlef(t *testing.T) {
	r := row("│ السلام عليكم │")
	out, _ := VisualRow(r)
	if len(out) != len(r) || out[len(out)-1].Text != "│" {
		t.Fatal(render(out))
	}
}
func TestUniqueMapping(t *testing.T) {
	r := row("> اكتب هنا 42 مرة")
	_, m := VisualRow(r)
	s := append([]int(nil), m...)
	sort.Ints(s)
	for i := range s {
		if s[i] != i {
			t.Fatal(m)
		}
	}
}
func TestMirrorsBrackets(t *testing.T) {
	out, _ := VisualRow(row("مرحبا (عالم)"))
	s := render(out)
	if strings.Index(s, "(") > strings.Index(s, ")") {
		t.Fatal(s)
	}
}
