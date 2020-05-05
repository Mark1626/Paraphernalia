package main

/*
#cgo CFLAGS: -Iinclude
#cgo LDFLAGS: -L. -larith
#include "arith.h"
*/
import "C"
import (
	"fmt"
)

func main() {
	fmt.Println("Sum", C.add(5, 4))
	fmt.Println("Diff", C.sub(5, 4))
	fmt.Println("Product", C.mul(5, 4))
}
