package orchestrata

/*
#cgo CFLAGS: -I../../../backend/crates/ffi/include
#cgo LDFLAGS: -lorchestrata_ffi
#include <stdlib.h>
#include "orchestrata.h"
*/
import "C"

import (
    "errors"
    "fmt"
    "unsafe"
)

type ConstraintKind uint32

const (
    NotEqualOffset ConstraintKind = 1
    PreferValue ConstraintKind = 2
)

type Constraint struct {
    Kind ConstraintKind
    Left int
    Right int
    Value int64
    Weight int64
}

type Problem struct {
    Domains [][]int64
    Constraints []Constraint
}

type Score struct {
    Hard int64
    Soft int64
}

type Solution struct {
    Values []int64
    Score Score
}

func Solve(problem Problem, maxEvaluations uint64) (Solution, error) {
    if C.orc_abi_version() != 1 {
        return Solution{}, errors.New("unsupported native ABI version")
    }

    count := len(problem.Domains)
    var variables *C.orc_variable
    if count > 0 {
        size := C.size_t(count) * C.size_t(unsafe.Sizeof(C.orc_variable{}))
        variables = (*C.orc_variable)(C.calloc(1, size))
        if variables == nil {
            return Solution{}, errors.New("native variable allocation failed")
        }
        defer C.free(unsafe.Pointer(variables))
    }

    rawVariables := unsafe.Slice(variables, count)
    for index, domain := range problem.Domains {
        if len(domain) == 0 {
            return Solution{}, fmt.Errorf("variable %d has an empty domain", index)
        }
        bytes := C.size_t(len(domain)) * C.size_t(unsafe.Sizeof(C.int64_t(0)))
        memory := C.malloc(bytes)
        if memory == nil {
            return Solution{}, errors.New("native domain allocation failed")
        }
        defer C.free(memory)
        rawDomain := unsafe.Slice((*C.int64_t)(memory), len(domain))
        for i, value := range domain {
            rawDomain[i] = C.int64_t(value)
        }
        rawVariables[index] = C.orc_variable{
            domain: (*C.int64_t)(memory),
            domain_len: C.size_t(len(domain)),
        }
    }

    var constraints *C.orc_constraint
    if len(problem.Constraints) > 0 {
        size := C.size_t(len(problem.Constraints)) * C.size_t(unsafe.Sizeof(C.orc_constraint{}))
        constraints = (*C.orc_constraint)(C.calloc(1, size))
        if constraints == nil {
            return Solution{}, errors.New("native constraint allocation failed")
        }
        defer C.free(unsafe.Pointer(constraints))
    }

    rawConstraints := unsafe.Slice(constraints, len(problem.Constraints))
    for index, constraint := range problem.Constraints {
        if constraint.Left < 0 || constraint.Right < 0 {
            return Solution{}, fmt.Errorf("constraint %d has a negative index", index)
        }
        if constraint.Kind != NotEqualOffset && constraint.Kind != PreferValue {
            return Solution{}, fmt.Errorf("constraint %d has an unsupported kind", index)
        }
        rawConstraints[index] = C.orc_constraint{
            kind: C.uint32_t(constraint.Kind),
            left: C.size_t(constraint.Left),
            right: C.size_t(constraint.Right),
            value: C.int64_t(constraint.Value),
            weight: C.int64_t(constraint.Weight),
        }
    }

    output := make([]C.int64_t, count)
    var outputPointer *C.int64_t
    if count > 0 {
        outputPointer = &output[0]
    }
    var nativeScore C.orc_score
    status := C.orc_solve(
        variables,
        C.size_t(count),
        constraints,
        C.size_t(len(problem.Constraints)),
        C.uint64_t(maxEvaluations),
        outputPointer,
        C.size_t(count),
        &nativeScore,
    )
    if status != C.ORC_OK {
        return Solution{}, fmt.Errorf("native solve failed with status %d", int(status))
    }

    values := make([]int64, count)
    for index, value := range output {
        values[index] = int64(value)
    }
    return Solution{
        Values: values,
        Score: Score{Hard: int64(nativeScore.hard), Soft: int64(nativeScore.soft)},
    }, nil
}
