.ONESHELL:
.DELETE_ON_ERROR:

build:
	rm -f libarith.a libarith.h
	go build -buildmode=c-archive -o libarith.a

clean:
	rm -f *.a libarith.h
