#include <stdio.h>
#include <assert.h>
#include "elucidator.h"

#define MADEIT() \
    do { \
        printf("Made it to line %d\n", __LINE__); \
    } while (0)

int main() {
    struct DesignationHandle * dh;
    struct ErrorHandle * eh;
    int err = get_designation_from_text("bar: u32", dh, eh);
    if ( err ) {
        printf("%s\n", get_error_string(eh));
    }
    else {
        print_designation(dh);
    }
    err = get_designation_from_text("invalid", dh, eh);
    if ( err ) {
        printf("%s\n", get_error_string(eh));
    }
    else {
        print_designation(dh);
    }
}
