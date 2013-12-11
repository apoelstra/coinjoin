
RUST=rustc
CC=gcc
CFLAGS=-Wall -W -Wextra -ansi -pedantic

SOURCES=hash.rs merge.rs transaction.rs util.rs sha-wrapper.c

all: signed unsigned

signed: coinjoin-merge-signed.rs $(SOURCES)
	$(CC) $(CFLAGS) -c sha-wrapper.c
	$(RUST) coinjoin-merge-signed.rs

unsigned: coinjoin-merge-unsigned.rs $(SOURCES)
	$(CC) $(CFLAGS) -c sha-wrapper.c
	$(RUST) coinjoin-merge-unsigned.rs

clean:
	rm coinjoin-merge-unsigned
	rm coinjoin-merge-signed
	rm sha-wrapper.o

