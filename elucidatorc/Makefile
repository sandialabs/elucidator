libdir = $(realpath ../target/release)

header:
	cbindgen -c cbindgen.toml -o elucidator.h

lib: header
	cargo build -r

exec:	lib
	cc -o test_elucidator -L/$(libdir) -lelucidatorc test_elucidator.c

all: exec
	./test_elucidator
