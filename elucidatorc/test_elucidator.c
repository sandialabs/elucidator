#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include "elucidator.h"

#define MADEIT() \
    do { \
        printf("Made it to line %d\n", __LINE__); \
    } while (0)

void wrap_insertion(SessionHandle * sh, const char * designation, const char * spec) {
    ErrorHandle * eh = ALLOCATE_HANDLE();
    ElucidatorStatus status = add_spec_to_session(designation, spec, sh, eh);
    if ( status != ELUCIDATOR_OK ) {
        char * msg = get_error_string(eh);
        fprintf(stderr, "Encountered error while inserting %s\n", designation);
        fprintf(stderr, "%s\n", msg);
        free(msg);
    }
    else {
        printf("Successfully inserted %s\n", designation);
    }
    free(eh);
}

void print_hdl(SessionHandle * hdl) {
    printf("Pointer points to address %p, value %u\n", hdl, hdl->hdl);
}

void print_buf(BufNode * b) {
    while (b != NULL) {
        for (int i = 0; i < b->n; ++i) {
            printf("%d, ", *(b->p + i));
        }
        printf("\n");
        b = b->next;
    }
}

int main() {
    SessionHandle * sh = ALLOCATE_HANDLE();
    ElucidatorStatus status;
    status = new_session(sh, ELUCIDATOR_RTREE);
    if ( status != ELUCIDATOR_OK ) {
        fprintf(stderr, "Whoops\n");
        exit(1);
    }
    /* This should succeed */
    wrap_insertion(sh, "foo", "bar: u32");
    /* This should fail */
    wrap_insertion(sh, "baz", "invalid");
    print_session(sh);
    BufNode * b = fetch_sample_blob();
    print_buf(b);
}
