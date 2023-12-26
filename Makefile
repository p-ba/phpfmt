src := $(shell find . -type f -name '*.c')

default: all

all: $(src)
	gcc -I$(shell pwd) -o phpfmt -v $(src)

clean:
	rm phpfmt
