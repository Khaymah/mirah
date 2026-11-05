package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"

	ob "github.com/go-opentype/bidi"
	sk "github.com/SCKelemen/unicode/uax9"
)

type tc struct {
	rs        []rune
	dir       string
	expLevels []string
	expOrder  []int
}

func load(path string) []tc {
	f, _ := os.Open(path)
	sc := bufio.NewScanner(f)
	sc.Buffer(make([]byte, 1<<20), 1<<20)
	var out []tc
	for sc.Scan() {
		line := sc.Text()
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		fl := strings.Split(line, ";")
		var t tc
		for _, h := range strings.Split(fl[0], " ") {
			v, _ := strconv.ParseUint(h, 16, 32)
			t.rs = append(t.rs, rune(v))
		}
		t.dir = fl[1]
		t.expLevels = strings.Split(fl[3], " ")
		for _, s := range strings.Fields(fl[4]) {
			v, _ := strconv.Atoi(s)
			t.expOrder = append(t.expOrder, v)
		}
		out = append(out, t)
	}
	return out
}

func l2(levels []int) []int {
	idx := make([]int, len(levels))
	for i := range idx {
		idx[i] = i
	}
	maxL, minOdd := 0, 1<<30
	for _, l := range levels {
		if l > maxL {
			maxL = l
		}
		if l%2 == 1 && l < minOdd {
			minOdd = l
		}
	}
	for cur := maxL; cur >= minOdd && cur > 0; cur-- {
		for i := 0; i < len(idx); {
			if levels[idx[i]] < cur {
				i++
				continue
			}
			j := i
			for j < len(idx) && levels[idx[j]] >= cur {
				j++
			}
			for a, b := i, j-1; a < b; a, b = a+1, b-1 {
				idx[a], idx[b] = idx[b], idx[a]
			}
			i = j
		}
	}
	return idx
}

func score(t tc, levels []int) (bool, bool) {
	var kept, kl []int
	levelsOK := true
	for i := range t.rs {
		if t.expLevels[i] == "x" {
			continue
		}
		kept = append(kept, i)
		kl = append(kl, levels[i])
		if strconv.Itoa(levels[i]) != t.expLevels[i] {
			levelsOK = false
		}
	}
	ord := l2(kl)
	if len(ord) != len(t.expOrder) {
		return levelsOK, false
	}
	for i := range ord {
		if kept[ord[i]] != t.expOrder[i] {
			return levelsOK, false
		}
	}
	return levelsOK, true
}

func main() {
	cases := load(os.Args[1])
	run := func(name string, f func(t tc) []int) {
		start := time.Now()
		lv, or := 0, 0
		for _, t := range cases {
			a, b := score(t, f(t))
			if a {
				lv++
			}
			if b {
				or++
			}
		}
		n := float64(len(cases))
		fmt.Printf("%-34s total=%d levels_ok=%.2f%% order_ok=%.2f%% time=%v\n", name, len(cases), float64(lv)*100/n, float64(or)*100/n, time.Since(start))
	}
	run("go-opentype/bidi v0.2.0", func(t tc) []int {
		d := ob.Auto
		if t.dir == "0" {
			d = ob.LeftToRight
		} else if t.dir == "1" {
			d = ob.RightToLeft
		}
		ls := ob.ResolveLevels(t.rs, d)
		out := make([]int, len(ls))
		for i, l := range ls {
			out[i] = int(l)
		}
		return out
	})
	run("SCKelemen/unicode uax9 v1.1.1", func(t tc) []int {
		p := 0
		switch t.dir {
		case "1":
			p = 1
		case "2":
			if sk.GetParagraphDirection(string(t.rs)) == sk.DirectionRTL {
				p = 1
			}
		}
		cls := make([]sk.BidiClass, len(t.rs))
		for i, r := range t.rs {
			cls[i] = sk.GetBidiClass(r)
		}
		return sk.ComputeLevels(cls, p)
	})
}
