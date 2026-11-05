package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"

	"golang.org/x/text/unicode/bidi"
)

func reorder(levels []int) []int {
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
	lv := append([]int(nil), levels...)
	for cur := maxL; cur >= minOdd && cur > 0; cur-- {
		for i := 0; i < len(lv); {
			if lv[i] >= cur {
				j := i
				for j < len(lv) && lv[j] >= cur {
					j++
				}
				for a, b := i, j-1; a < b; a, b = a+1, b-1 {
					idx[a], idx[b] = idx[b], idx[a]
					lv[a], lv[b] = lv[b], lv[a]
				}
				i = j
			} else {
				i++
			}
		}
	}
	return idx
}

func firstStrongRTL(rs []rune) bool {
	for _, r := range rs {
		p, _ := bidi.LookupRune(r)
		switch p.Class() {
		case bidi.L:
			return false
		case bidi.R, bidi.AL:
			return true
		}
	}
	return false
}

func main() {
	f, _ := os.Open(os.Args[1])
	sc := bufio.NewScanner(f)
	sc.Buffer(make([]byte, 1<<20), 1<<20)
	total, ok, errs := 0, 0, 0
	start := time.Now()
	for sc.Scan() {
		line := sc.Text()
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		fl := strings.Split(line, ";")
		var rs []rune
		for _, h := range strings.Split(fl[0], " ") {
			v, _ := strconv.ParseUint(h, 16, 32)
			rs = append(rs, rune(v))
		}
		expLevels := strings.Split(fl[3], " ")
		var expOrder []int
		for _, s := range strings.Fields(fl[4]) {
			v, _ := strconv.Atoi(s)
			expOrder = append(expOrder, v)
		}
		total++
		var opts []bidi.Option
		paraRTL := false
		switch fl[1] {
		case "0":
			opts = append(opts, bidi.DefaultDirection(bidi.LeftToRight))
		case "1":
			opts = append(opts, bidi.DefaultDirection(bidi.RightToLeft))
			paraRTL = true
		default:
			paraRTL = firstStrongRTL(rs)
		}
		var p bidi.Paragraph
		p.SetString(string(rs), opts...)
		o, err := p.Order()
		if err != nil {
			errs++
			continue
		}
		lev := make([]int, len(rs))
		for i := 0; i < o.NumRuns(); i++ {
			r := o.Run(i)
			s, e := r.Pos()
			l := 0
			if r.Direction() == bidi.RightToLeft {
				l = 1
			} else if paraRTL {
				l = 2
			}
			for k := s; k <= e && k < len(lev); k++ {
				lev[k] = l
			}
		}
		var kept, kl []int
		for i := range rs {
			if i < len(expLevels) && expLevels[i] != "x" {
				kept = append(kept, i)
				kl = append(kl, lev[i])
			}
		}
		ord := reorder(kl)
		match := len(ord) == len(expOrder)
		for i := 0; match && i < len(ord); i++ {
			if kept[ord[i]] != expOrder[i] {
				match = false
			}
		}
		if match {
			ok++
		}
	}
	fmt.Printf("go x/text/unicode/bidi %s: total=%d order_ok=%d (%.2f%%) errors=%d time=%v\n",
		"v0.43.0", total, ok, float64(ok)*100/float64(total), errs, time.Since(start))
}
