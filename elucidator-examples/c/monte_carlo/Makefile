# Define the paths to the Rust library and header file
LIBDIR = $(realpath ../../../target/release)
DEBUGDIR = $(realpath ../../../target/debug)
HEADER = $(realpath ../../../target/include)

# Define the name of the Rust library (without the lib prefix and file extension)
LIBNAME = elucidatorc

# Define the C source file and the output executable
SRC = monte_carlo.c
EXEC = bin/monte_carlo

# Default target: build and run the release version
all: release run

# Build the release version of the C example
release: $(SRC) $(HEADER)
	cc -O2 -o $(EXEC) -I$(HEADER) -L$(LIBDIR) -l$(LIBNAME) $(SRC) 

# Build the debug version of the C example
debug: $(SRC) $(HEADER)
	cc -o $(EXEC) -I$(HEADER) -L$(DEBUGDIR) -l$(LIBNAME) $(SRC)

# Run the release version of the C example
run: release
	./$(EXEC)

# Run the debug version of the C example
run-debug: debug
	./$(EXEC)

# Clean up the generated files
clean:
	rm -f $(EXEC)

.PHONY: all release debug run run-debug clean
