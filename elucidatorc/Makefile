libdir = $(realpath ../target/release)

header:
	cbindgen --lang c --cpp-compat --style tag -o elucidator_auto.h
	cat elucidator_auto.h > elucidator.h
	cat elucidator_manual.h >> elucidator.h

lib: header
	cargo build -r

exec:	lib
	cc -o test_elucidator -L/$(libdir) -lelucidatorc test_elucidator.c

all: exec
	./test_elucidator
