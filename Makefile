
RUST=rustc
CC=gcc
AR=ar
CFLAGS=-Wall -W -Wextra -ansi -pedantic

SOURCES=hash.rs transaction.rs util.rs sha-wrapper.c

all: signed unsigned

signed: coinjoin-merge-signed.rs merge_signed.rs $(SOURCES)
	$(CC) $(CFLAGS) -c -static sha-wrapper.c
	$(AR) rcs libsha-wrapper.a sha-wrapper.o
	$(RUST) coinjoin-merge-signed.rs

unsigned: coinjoin-merge-unsigned.rs merge_signed.rs $(SOURCES)
	$(CC) $(CFLAGS) -c sha-wrapper.c
	$(AR) rcs libsha-wrapper.a sha-wrapper.o
	$(RUST) coinjoin-merge-unsigned.rs

clean:
	rm coinjoin-merge-unsigned
	rm coinjoin-merge-signed
	rm sha-wrapper.o

