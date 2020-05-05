package main

import "C"

//export add
func add(a int, b int) int {
	return a + b
}

//export sub
func sub(a int, b int) int {
	return a - b
}

//export mul
func mul(a int, b int) int {
	return a * b
}

func main() {

}
