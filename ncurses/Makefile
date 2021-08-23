CFLAGS=$(shell cat ~/flags)

print:
	echo $(CFLAGS)

hello: hello.c
	$(CC) -o hello hello.c -lcurses

window: window.c
	$(CC) -o window window.c -lcurses

.PHONY: print
