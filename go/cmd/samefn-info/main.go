// Command samefn-info prints the linked samefn library's build description. CI builds it
// with and without cgo, so linking a whole program is checked both ways.
package main

import (
	"fmt"

	"github.com/mjonss/samefn-poc/go/samefn"
)

func main() {
	fmt.Println(samefn.BuildInfo())
	fmt.Println("sin max version:", samefn.MaxVersion("sin"))
}
