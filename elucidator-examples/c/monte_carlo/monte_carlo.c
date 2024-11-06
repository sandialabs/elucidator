#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <time.h>
#include <math.h>
#include "elucidator.h"

#define N_STEPS 100000
#define SAMPLES_PER_STEP 50
#define DISPLAY_INTERVAL 5000

#define MADEIT() \
    do { \
        printf("Made it to line %d\n", __LINE__); \
    } while (0)


typedef struct {
    uint64_t hits;
    uint64_t misses;
} StepSummary;

ElucidatorStatus print_error_if_not_ok(ErrorHandle * eh, ElucidatorStatus status); 
ElucidatorStatus setup(SessionHandle * sh, ErrorHandle * eh);
ElucidatorStatus run_experiment(SessionHandle * sh, ErrorHandle * eh, uint64_t n_steps, uint64_t samples_per_step);
StepSummary run_experiment_step(uint64_t samples_per_step);
ElucidatorStatus run_analysis(SessionHandle * sh, ErrorHandle * eh, uint64_t n_steps, uint64_t display_interval);
StepSummary summarize_bufnodes(BufNode **bufnodes);

int main() {
    srand(time(NULL));

    SessionHandle * sh = ALLOCATE_HANDLE();
    ErrorHandle * eh = ALLOCATE_HANDLE();
    ElucidatorStatus status;

    status = setup(sh, eh);
    if (status != ELUCIDATOR_OK) {
        fprintf(stderr, "Setup failed. Exiting.");
        return 1;
    }
    run_experiment(sh, eh, N_STEPS, SAMPLES_PER_STEP);
    if (status != ELUCIDATOR_OK) {
        fprintf(stderr, "Experiment failed. Exiting.");
        return 1;
    }
    run_analysis(sh, eh, N_STEPS, DISPLAY_INTERVAL);
    if (status != ELUCIDATOR_OK) {
        fprintf(stderr, "Analysis failed. Exiting.");
        return 1;
    }
    free(sh);
    free(eh);
}


ElucidatorStatus print_error_if_not_ok(ErrorHandle * eh, ElucidatorStatus status) {
    if ( status != ELUCIDATOR_OK ) {
        char * msg = get_error_string(eh);
        fprintf(stderr, "%s\n", msg);
        free(msg);
    }
    return status;
}

ElucidatorStatus setup(SessionHandle * sh, ErrorHandle * eh) {
    ElucidatorStatus status;
    char * designation = "state";
    char * spec = "hits: u64, misses: u64";

    status = new_session(sh, ELUCIDATOR_RTREE);
    if (print_error_if_not_ok(eh, status) != ELUCIDATOR_OK) return status;
    status = add_spec_to_session(designation, spec, sh, eh);
    print_error_if_not_ok(eh, status);
    return status;
}

ElucidatorStatus run_experiment(SessionHandle * sh, ErrorHandle * eh, uint64_t n_steps, uint64_t samples_per_step) {
    ElucidatorStatus status;
    Point a, b;
    BoundingBox bb;

    for (uint64_t step = 0; step < n_steps; step++) {
        StepSummary summary = run_experiment_step(samples_per_step);
        // Upper and lower bounds for experiment region: x, y, z, t
        Point a = { -1.0, -1.0, -1.0, step};
        Point b = {1.0, 1.0, 1.0, step};
        BoundingBox bb = {a, b};
        uint8_t *blob = (uint8_t*)&summary;
        status = insert_metadata_in_session(sh, bb, "state", blob, sizeof(summary), eh);
        if (print_error_if_not_ok(eh, status) != ELUCIDATOR_OK) return status;
    }
    
    return status;
}

StepSummary run_experiment_step(uint64_t samples_per_step) {
    double x, y;
    uint64_t hits = 0;
    uint64_t misses = 0;
    for (uint64_t i = 0; i < samples_per_step; i++) {
        x = (double)rand() / RAND_MAX;
        y = (double)rand() / RAND_MAX;

        if (x * x + y * y <= 1) {
            ++hits;
        }
        else {
            ++misses;
        }
    }
    StepSummary summary = {hits, misses};
    return summary;
}

ElucidatorStatus run_analysis(SessionHandle * sh, ErrorHandle * eh, uint64_t n_steps, uint64_t display_interval) {
    const char * designation = "state";
    const double epsilon = 0.0;
    const double Z_SCORE_95_CI = 1.959963984540054;

    ElucidatorStatus status;
    double n;
    double pi, pi_upper_95_ci, pi_lower_95_ci, p, se;
    BufNode ** results = (BufNode **)malloc(sizeof(BufNode));

    for (uint64_t timestep = display_interval; timestep <= n_steps; timestep += display_interval) {
        // Upper and lower bounds for experiment region: x, y, z, t
        Point a = { -1.0, -1.0, -1.0, 0};
        Point b = {1.0, 1.0, 1.0, timestep};
        BoundingBox bb = {a, b};
        status = get_metadata_in_bb(sh, bb, designation, epsilon, results, eh);
        if (status != ELUCIDATOR_OK) return status;
        StepSummary summary = summarize_bufnodes(results);

        n = summary.hits + summary.misses;
        p = summary.hits / n;
        se = sqrt((p*(1 - p)/(summary.misses)));
        pi = 4.0 * summary.hits / n;
        pi_upper_95_ci = 4.0 * (p + Z_SCORE_95_CI * se);
        pi_lower_95_ci = 4.0 * (p - Z_SCORE_95_CI * se);

        printf("Step %llu: pi ~= %f, 95%% CI (%f, %f)\n", timestep, pi, pi_lower_95_ci, pi_upper_95_ci); 
        free_bufnodes(*results);
    }
    return status;
}

StepSummary summarize_bufnodes(BufNode **bufnodes) {
    StepSummary summary = {0, 0};
    BufNode *current = *bufnodes; // Start with the head of the linked list

    while (current != NULL) {
        // Check if the size of the data matches the size of StepSummary
        // If this isn't true an unrecoverable error has occured.
        if (current->n != sizeof(StepSummary)) {
            fprintf(stderr, "Error: BufNode data size does not match StepSummary size. Aborting.\n");
            exit(1);
        }

        StepSummary *stepSummary = (StepSummary *)current->p;
        summary.hits += stepSummary->hits;
        summary.misses += stepSummary->misses;
        current = current->next;
    }
    return summary;
}