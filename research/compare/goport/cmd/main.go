package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
	"time"

	"goport"
)

func main() {
	f, _ := os.Open(os.Args[1])
	sc := bufio.NewScanner(f)
	var rows [][]goport.Cell
	for sc.Scan() {
		var r []goport.Cell
		for _, c := range sc.Text() {
			r = append(r, goport.Cell{Text: string(c)})
		}
		for len(r) < 80 {
			r = append(r, goport.Cell{Text: " "})
		}
		rows = append(rows, r)
	}
	for _, r := range rows {
		out, _ := goport.VisualRow(r)
		var b strings.Builder
		for _, c := range out {
			b.WriteString(c.Text)
		}
		fmt.Println(strings.TrimRight(b.String(), " "))
	}
	start := time.Now()
	n := 0
	for i := 0; i < 5000; i++ {
		for _, r := range rows {
			out, _ := goport.VisualRow(r)
			n += len(out)
		}
	}
	el := time.Since(start)
	fmt.Fprintf(os.Stderr, "go:   %d rows in %v (%.2f µs/row) %d\n", 5000*len(rows), el, el.Seconds()*1e6/float64(5000*len(rows)), n)
}
