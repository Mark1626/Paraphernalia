package main

import (
	"log"
	"net/http"
	"os"
)

func main() {
	if len(os.Args) < 2 {
		log.Fatal("Please enter a path to serve")
	}
	folder := os.Args[1]

	fs := http.FileServer(http.Dir(folder))
	http.Handle("/", fs)

	log.Println("Starting static server on ")
	err := http.ListenAndServe(":3000", nil)
	if err != nil {
		log.Fatal(err)
	}
}
