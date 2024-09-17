#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include "elucidator.h"

#define MADEIT() \
    do { \
        printf("Made it to line %d\n", __LINE__); \
    } while (0)

int main() {
    DesignationHandle * dh = ALLOCATE_HANDLE();
    ErrorHandle * eh = ALLOCATE_HANDLE();
    SessionHandle * sh = ALLOCATE_HANDLE();
    char * err_string = NULL;
    int err = get_designation_from_text("bar: u32", dh, eh);
    if ( err ) {
        printf("%s\n", get_error_string(eh));
    }
    else {
        print_designation(dh);
    }
    err = get_designation_from_text("invalid", dh, eh);
    if ( err ) {
        err_string = get_error_string(eh);
        printf("%s\n", err_string);
    }
    else {
        print_designation(dh);
    }
    new_session(sh);
    err = add_spec_to_session("animal", "name: string", sh, eh);
    if ( err ) {
        err_string = get_error_string(eh);
        printf("%s\n", err_string);
    }
    else {
        print_session(sh);
    }
    free(dh);
    free(eh);
    free(sh);
    free(err_string);
}
