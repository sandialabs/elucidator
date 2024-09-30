libdir = $(realpath ../target/release)
debugdir = $(realpath ../target/debug)

header:
	cbindgen -c cbindgen.toml -o elucidator.h

lib: header
	cargo build
	cargo build -r

exec:	lib
	cc -o test_elucidator -L/$(libdir) -lelucidatorc test_elucidator.c

all: exec
	./test_elucidator
 
debug: lib
	cc -o test_elucidator -L/$(debugdir) -lelucidatorc test_elucidator.c

adb: debug
	./test_elucidator
