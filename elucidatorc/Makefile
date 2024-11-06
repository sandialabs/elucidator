libdir = $(realpath ../target/release)
debugdir = $(realpath ../target/debug)
headerdir = $(realpath ../target/include)

header:
	cbindgen -c cbindgen.toml -o $(headerdir)/elucidator.h

lib: header
	cargo build
	cargo build -r

exec:	lib
	cc -o test_elucidator -I$(headerdir) -L/$(libdir) -lelucidatorc test_elucidator.c

all: exec
	./test_elucidator
 
debug: lib
	cc -o test_elucidator -I$(headerdir) -L/$(debugdir) -lelucidatorc test_elucidator.c

adb: debug
	./test_elucidator
