src := $(shell find . -type f -name '*.c')

default: all

all: $(src)
	gcc -Wall -I$(shell pwd) -o phpfmt -v $(src)

clean:
	rm phpfmt
