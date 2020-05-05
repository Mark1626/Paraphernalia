ifeq ($(origin .RECIPEPREFIX), undefined)
  $(error This Make does not support .RECIPEPREFIX. Please use GNU Make 4.0 or later)
endif
.RECIPEPREFIX = >
.ONESHELL:
.DELETE_ON_ERROR:

dynamic:
> rm -f libarith.dylib
> gcc -c -fPIC -Iinclude src/arith.c
> gcc -dynamiclib -o libarith.dylib arith.o
> rm -f arith.o

shared:
> rm -f libarith.so
> gcc -c -fPIC -Iinclude src/arith.c
> gcc -shared -W1 -o libarith.so arith.o
> rm -f arith.o

clean:
> rm -f libarith.so
> rm -f libarith.dylib
> rm -f go-dynamic

build: dynamic
> go build
