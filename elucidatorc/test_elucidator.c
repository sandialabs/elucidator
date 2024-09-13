#include <stdio.h>
#include "elucidator.h"

int main() {
    int failures = 0;
    failures += add_designation_from_text("foo", "bar: i32");
    failures += add_designation_from_text("baz", "cat: u32[], dog: f32[]");
    if ( failures ) {
        fprintf(stderr, "Failed!\n");
    }
    else {
        printf("Succeeded!\n");
    }
}
