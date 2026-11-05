package main

import (
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/charmbracelet/x/vt"
)

func main() {
	data, _ := os.ReadFile(os.Args[1])
	em := vt.NewEmulator(80, 24)
	go func() { _, _ = io.Copy(io.Discard, em) }()
	_, _ = em.Write(data)
	for y := 0; y < 24; y++ {
		var b strings.Builder
		for x := 0; x < 80; x++ {
			c := em.CellAt(x, y)
			if c == nil {
				continue
			}
			if c.Width == 0 {
				continue
			}
			if c.Content == "" {
				b.WriteString(" ")
			} else {
				b.WriteString(c.Content)
			}
		}
		fmt.Println(strings.TrimRight(b.String(), " "))
	}
	p := em.CursorPosition()
	fmt.Printf("@cursor %d,%d\n", p.Y, p.X)
}
