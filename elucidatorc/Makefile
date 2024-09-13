libdir = $(realpath ../target/release)
lib:
	cargo build -r
	cbindgen --lang c --cpp-compat -o elucidator.h

exec:	lib
	cc -o test_elucidator -L/$(libdir) -lelucidatorc test_elucidator.c

all: exec
	./test_elucidator
