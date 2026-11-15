#ifndef ORCHESTRATA_H
#define ORCHESTRATA_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum orc_status {
    ORC_OK = 0,
    ORC_INVALID_ARGUMENT = 1,
    ORC_INVALID_PROBLEM = 2,
    ORC_EXECUTION_FAILED = 3,
    ORC_BUFFER_TOO_SMALL = 4,
    ORC_INTERNAL_ERROR = 5
} orc_status;

typedef enum orc_constraint_kind {
    ORC_NOT_EQUAL_OFFSET = 1,
    ORC_PREFER_VALUE = 2
} orc_constraint_kind;

typedef struct orc_variable {
    const int64_t *domain;
    size_t domain_len;
} orc_variable;

typedef struct orc_constraint {
    uint32_t kind;
    size_t left;
    size_t right;
    int64_t value;
    int64_t weight;
} orc_constraint;

typedef struct orc_score {
    int64_t hard;
    int64_t soft;
} orc_score;

uint32_t orc_abi_version(void);

orc_status orc_solve(
    const orc_variable *variables,
    size_t variable_count,
    const orc_constraint *constraints,
    size_t constraint_count,
    uint64_t max_evaluations,
    int64_t *output,
    size_t output_capacity,
    orc_score *score
);

#ifdef __cplusplus
}
#endif

#endif
