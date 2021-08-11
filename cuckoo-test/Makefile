CFLAGS=-Wall -Wextra
OPENSSL_DIR = /usr/local/opt/openssl@1.1
OPENSSL_SUPPORT = -I$(OPENSSL_DIR)/include -L$(OPENSSL_DIR)/lib

%.o: %.c
	cc -c $^ cuckoo.h $(OPENSSL_SUPPORT) -lcrypto

main: cuckoo.o
	cc -o main main.c cuckoo.o $(OPENSSL_SUPPORT) -lcrypto

clean:
	rm *.h.gch
	rm *.o
	rm main

.PHONY: main clean
