# Native SDK reference

Orchestrata selects a feasible assignment from the options you provide. The quickstarts below use the default branch of the [standalone engine repository](https://github.com/ilstarno/orchestrata-engine). Read the worked input and expected result without installing anything. To run it locally, build the library and binding from the same checkout. These are source builds, not packaged SDK releases.

## Build and locate the library

Open a terminal at the root of the standalone engine repository. With stable Rust and a linker for your platform, run:

```sh
if [ -f Cargo.toml ] && [ -f crates/ffi/include/orchestrata.h ]; then
    engine_manifest=Cargo.toml
    engine_target=target
    engine_include=crates/ffi/include
elif [ -f backend/Cargo.toml ] && [ -f backend/crates/ffi/include/orchestrata.h ]; then
    engine_manifest=backend/Cargo.toml
    engine_target=backend/target
    engine_include=backend/crates/ffi/include
else
    printf '%s\n' 'Run these commands from the standalone engine repository.' >&2
    exit 1
fi

cargo build --manifest-path "$engine_manifest" --release --locked -p orchestrata-ffi
```

If `Cargo.toml` is at the top of your checkout, look in `target/release/`. If it is inside `backend/`, look in `backend/target/release/`. The dynamic library is named `liborchestrata_ffi.so` on Linux, `liborchestrata_ffi.dylib` on macOS, or `orchestrata_ffi.dll` on Windows. Use the library built for your operating system and processor. In the same Linux terminal, set these paths for the examples below. If you use a custom Cargo target directory, set the library directory to match:

```sh
export ORCHESTRATA_LIB_DIR="$PWD/$engine_target/release"
export ORCHESTRATA_LIBRARY="$ORCHESTRATA_LIB_DIR/liborchestrata_ffi.so"
export LD_LIBRARY_PATH="$ORCHESTRATA_LIB_DIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export ORCHESTRATA_INCLUDE="$PWD/$engine_include"
```

C#, Java, Python and PHP take a library path. Go and Node.js link to the library and need the loader to find it at runtime. Do not assume a library from an unrelated checkout or installation is compatible. These bindings require ABI version 1.

## Choose a binding

| Language | Source requirements | Typed fields and limits |
| --- | --- | --- |
| C# | .NET 8; reference `sdks/csharp/Orchestrata.csproj`. | `SolveJson` accepts UTF-8 JSON as a string or byte span and an unsigned 64-bit budget. Returns the complete JSON text. |
| Java | A 64-bit JDK 22 or newer, with native access enabled. | Accepts a UTF-8 `byte[]` or a string and a `long` limit. Returns raw JSON text. No checked-in problem or solution model. |
| Python | Python 3.10 or newer; uses the standard-library native loader. | `solve_problem` takes a typed `Problem` and returns `Solution \| None`. Use the raw method for native rules not modeled by the typed binding. |
| Node.js | A Node release with Node-API BigInt support, a C toolchain, and `node-gyp`. | `solveJson` accepts a UTF-8 string or `Buffer` and a `bigint` budget. Returns the complete JSON text. |
| PHP | PHP 8.1 or newer and the FFI extension; enable FFI only in an appropriate local environment. | Accepts a string and a nonnegative PHP integer limit; returns raw JSON text. No checked-in problem or solution model. |
| Go | Go 1.20 or newer and cgo with the platform C toolchain. | `SolveProblem` accepts a typed `Problem`. `Solve` accepts JSON and returns a typed result; `SolveJSON` accepts and returns JSON bytes. |

The default-branch C#, Java, Node.js, and PHP quickstarts use raw JSON. The Python and Go quickstarts use their source checkout’s typed models. Where a wrapper returns raw JSON, use a lossless decoder and validate the fields you consume.

## Problem and result format

Here is the baseline problem used by most SDK examples:

```json
{"variables":[{"name":"job","domain":[0,1]}],"constraints":[{"kind":"prefer_value","variable":0,"value":1,"soft_weight":1}]}
```

Set the maximum number of evaluations to `2`. Both candidates fit the budget. The solver selects `1`, satisfying the preference, and returns:

```json
{"solution":{"values":[1],"score":{"hard":0,"soft":0},"evaluated":2}}
```

- Each variable requires a nonempty integer domain. Values follow input variable order. Constraints identify variables by zero-based position. Include `constraints: []` if you have none. Use unique, nonblank names and distinct domain values if you intend to submit the same problem to the local HTTP API, which applies additional input checks.
- `all_different` uses `variables` and positive `hard_weight` to penalize equal pairs. `not_equal_offset` uses `left`, `right`, `offset` and positive `hard_weight` to forbid the indicated left-minus-right difference. `prefer_value` uses `variable`, `value` and positive `soft_weight`. The model has no date, currency, time or speed units.
- A success contains a verified optimal assignment, or `{"solution":null}` when the exact search proved no feasible assignment. Python’s typed and decoded methods return `None`, and Go’s typed method returns `nil, nil` for that no-plan case. Exceeding the budget returns an error. There is no null response or partial result. This native contract is separate from that of a running HTTP build.
- Domain values, offsets, weights and scores use signed 64-bit integers. An evaluation count is unsigned 64-bit. The budget counts candidate combinations. Pass it as a separate SDK argument.
- For full-range values, do not round through a floating-point JSON decoder. In C# use `GetInt64()` for values/scores and `GetUInt64()` for evaluation counts. In Java choose a JSON codec that preserves exact integer values, including unsigned counts. In Node.js, keep the raw JSON text or use a lossless decoder; ordinary `JSON.parse` can round numbers. In PHP, keep JSON text or choose integer-safe decoding appropriate to your PHP integer width.

## C# · .NET 8

In a .NET 8 console project, reference `sdks/csharp/Orchestrata.csproj` and use this as `Program.cs`. Pass the full library path as the first argument, for example `dotnet run --project /path/to/solver-quickstart -- "$ORCHESTRATA_LIBRARY"`.

```csharp
using System;
using Orchestrata;

using var solver = new Solver(args[0]);
const string problem = """
    {"variables":[{"name":"job","domain":[0,1]}],"constraints":[{"kind":"prefer_value","variable":0,"value":1,"soft_weight":1}]}
    """;
Console.WriteLine(solver.SolveJson(problem, 2UL));
```

On the default branch, `SolveJson` returns the complete response as text. A native failure throws `SolveException`; its `Status` gives the native error code. A load or disposal error is not a solver outcome.

## Java · JDK 22+

Save this as `SolverQuickstart.java`. It uses the host library path supplied as an argument. Keep the `Solver` in a try-with-resources block.

```java
import java.nio.file.Path;
import io.orchestrata.Solver;

public class SolverQuickstart {
    public static void main(String[] args) {
        try (var solver = new Solver(Path.of(args[0]))) {
            String problem = """
                {"variables":[{"name":"job","domain":[0,1]}],"constraints":[{"kind":"prefer_value","variable":0,"value":1,"soft_weight":1}]}
                """;
            System.out.println(solver.solveJson(problem, 2L));
        }
    }
}
```

From the engine checkout root (use the actual path to your `SolverQuickstart.java`):

```sh
classes="$(mktemp -d)"
javac -d "$classes" $(find sdks/java/src/main/java -name '*.java') SolverQuickstart.java
java --enable-native-access=ALL-UNNAMED -cp "$classes" SolverQuickstart "$ORCHESTRATA_LIBRARY"
```

The classpath example enables native access for unnamed modules. A modular application should enable it for the module making the native call. A `SolveException` supplies `status()`. The limit is a Java `long` passed as an unsigned 64-bit bit pattern; ordinary examples should use a small nonnegative limit. Do not assume a negative Java value means a negative native budget.

## Python · 3.10+

Run from the engine repository root. The binding is imported directly from this checkout:

```sh
PYTHONPATH="$PWD/sdks/python" python3 - <<'PY'
import os
from orchestrata import PreferValue, Problem, Solver, Variable

solver = Solver(os.environ["ORCHESTRATA_LIBRARY"])
problem = Problem(
    variables=(Variable("job", (0, 1)),),
    constraints=(PreferValue(variable=0, value=1, soft_weight=1),),
)
solution = solver.solve_problem(problem, max_evaluations=2)
print(solution.values if solution is not None else "No feasible assignment")
PY
```

The default-branch typed constraints are `AllDifferent`, `NotEqualOffset`, `PreferValue`, `CountValue`, and `ResourceCapacity`. Use `solve_json` for `precedence` and `no_overlap`. Use tuples for collection fields. A solution exposes `values`, `score.hard`, `score.soft`, and `evaluated`. `None` is proven infeasibility. A malformed typed response raises `ResultFormatError`; invalid local values raise `TypeError` or `ValueError`.

The mapping method is `solve(mapping, max_evaluations)`. Use `solve_json(problem_bytes, max_evaluations)` for the complete UTF-8 response. The budget is an integer in `0..2^64-1`. Catch `SolveError` and inspect its numeric `status` for a native failure. A library load failure is separate.

## Node.js · Node-API native addon

Run from the matching engine checkout after building and locating its native library.

```sh
(cd sdks/node && npm run build)
node - <<'JS'
const { solveJson } = require("./sdks/node");
const problem = '{"variables":[{"name":"job","domain":[0,9007199254740993]}],"constraints":[{"kind":"prefer_value","variable":0,"value":9007199254740993,"soft_weight":1}]}';
console.log(solveJson(problem, 2n));
JS
```

The raw JSON response preserves `9007199254740993` exactly. Keep the problem and response as JSON text when values may exceed JavaScript’s safe integer range. The evaluation budget is a separate `bigint`.

## PHP · 8.1+ with FFI

Use the Composer path package or load the two source classes in a local script. From the engine repository root:

```sh
php -d ffi.enable=1 <<'PHP'
<?php
require 'sdks/php/SolveException.php';
require 'sdks/php/Solver.php';

$solver = new Orchestrata\Solver(getenv('ORCHESTRATA_LIBRARY'));
$problem = '{"variables":[{"name":"job","domain":[0,1]}],"constraints":[{"kind":"prefer_value","variable":0,"value":1,"soft_weight":1}]}';
echo $solver->solveJson($problem, 2), PHP_EOL;
PHP
```

The budget cannot exceed `PHP_INT_MAX` on the host. A native failure throws `Orchestrata\SolveException` with a public integer `$status`. Path, unavailable FFI and local argument errors do not imply a solver outcome. Do not enable FFI broadly to make a deployment pass a local smoke check.

## Go · 1.20+ with cgo

In a new Go module, use a local module replacement pointing at this checkout’s `sdks/go` directory. Use the module name shown below. From your new project directory:

```sh
go mod init orchestrata-quickstart
go mod edit -require=github.com/ilstarno/Orchestrata/sdks/go@v0.0.0
go mod edit -replace=github.com/ilstarno/Orchestrata/sdks/go=/absolute/path/to/engine/sdks/go
```

Save this as `main.go`:

```go
package main

import (
    "fmt"
    "log"

    orchestrata "github.com/ilstarno/Orchestrata/sdks/go"
)

func main() {
    problem := orchestrata.Problem{
        Variables: []orchestrata.Variable{
            {Name: "job", Domain: []int64{0, 1}},
        },
        Constraints: []orchestrata.Constraint{
            orchestrata.PreferValue{Variable: 0, Value: 1, SoftWeight: 1},
        },
    }
    solution, err := orchestrata.SolveProblem(problem, 2)
    if err != nil {
        log.Fatal(err)
    }
    if solution == nil {
        fmt.Println("No feasible assignment")
        return
    }
    fmt.Println(solution.Values)
}
```

Run with the header include path and the native library available to the linker and loader:

```sh
CGO_ENABLED=1 CGO_CFLAGS="-I$ORCHESTRATA_INCLUDE" \
  CGO_LDFLAGS="-L$ORCHESTRATA_LIB_DIR" go run .
```

On the default branch, `SolveProblem` accepts `AllDifferent`, `NotEqualOffset`, and `PreferValue`. Use a JSON method for other native rules. Variable indices are `uint32`; domains, offsets, preference values, and weights are `int64`. Weights must be positive. The typed decoder checks each returned assignment value belongs to the request domain. Use `Solve(problemJSON, limit)` when you need a checked `*Solution` from a JSON request. Use `SolveJSON(problemJSON, limit)` when you want the complete JSON response instead. Test native errors using `errors.As` with `*orchestrata.SolveError`, whose `Status` is a typed numeric value. A loader, ABI or decoding failure is not a native search result.

## Scheduling and capacity constraints

These rules are supported by the native JSON interface. The table gives the wire names and fields. Check your HTTP interface separately before sending the same request.

<div class="table-region" role="region" aria-label="Reference table" tabindex="0">

| `kind` | Required fields | Meaning |
| --- | --- | --- |
| `count_value` | `variables`: zero-based indices; `value`: signed 64-bit integer; `min` and `max`: inclusive nonnegative counts; `weight`: positive signed 64-bit integer; `tier`: `"hard"` or `"soft"`. | Counts how many referenced variables choose `value`. Each count below `min` or above `max` contributes one deduction of `weight` to the selected score. |
| `resource_capacity` | `items`: array of `{"variable": index, "load": quantity}`; `limits`: array of `{"resource": id, "capacity": quantity}`. Loads and capacities are nonnegative unsigned 64-bit integers; resource IDs are signed 64-bit integers. | Sums the load of the items assigned to each resource and checks its inclusive capacity. Provide a limit for each resource in the listed item domains and use the same unit throughout a constraint. The rule covers total assignment load only. It does not model changing capacity or routes. |
| `precedence` | `before`, `after`: distinct zero-based variable indices; `min_gap`: unsigned 64-bit duration. | The selected value of `after` must be at least the selected value of `before` plus the gap. Choose one time unit for both variables and the gap. |
| `no_overlap` | `intervals`: at least two objects with distinct `start` variable indices and an unsigned 64-bit `duration`. | On one exclusive resource, intervals occupy their selected start through, but not including, the end. Back-to-back intervals are allowed. A zero-duration interval occupies no time. |

</div>

For example, the raw request below requires exactly one item on resource `1` and caps the two resource loads:

```json
{
  "variables": [
    {"name": "item_a", "domain": [0, 1]},
    {"name": "item_b", "domain": [0, 1]}
  ],
  "constraints": [
    {"kind": "count_value", "variables": [0, 1], "value": 1, "min": 1, "max": 1, "weight": 1, "tier": "hard"},
    {"kind": "resource_capacity", "items": [{"variable": 0, "load": 5}, {"variable": 1, "load": 3}], "limits": [{"resource": 0, "capacity": 3}, {"resource": 1, "capacity": 5}]}
  ]
}
```

With a budget of `4`, the only feasible result is `[1, 0]`, with both scores zero. This request is usable from Go with `Solve([]byte(problemJSON), 4)`, and from the other bindings via their JSON methods.

## Error and buffer reference

<div class="table-region" role="region" aria-label="Reference table" tabindex="0">

| Status | Meaning | What to do |
| --- | --- | --- |
| `0` | The output document was returned. A null solution means proved infeasibility. | Distinguish a solution object from a null before accessing fields. |
| `1` | Invalid UTF-8 JSON or invalid problem. | Check JSON fields and types, nonempty domains, valid indices and positive weights. |
| `2` | The combinations exceed the caller’s evaluation cap. | Reduce the example legitimately or choose a different budget. No result or proof is returned. |
| `3` | Execution failure, such as an unrepresentable winning score. | Review weights and score bounds. Keep a reproducible sample input when reporting the failure. |
| `4` | Invalid native argument, such as an empty or oversized request. | Confirm argument types and UTF-8 byte count. |

</div>

The native entry point reads the exact supplied length and does not retain the input. Requests may not exceed 1,048,576 bytes (1 MiB). A native response allocation is at most 2,097,152 bytes (2 MiB), including the terminating NUL. The SDK wrappers copy a successful result into language-owned storage and release the native buffer before returning it. If calling the C ABI yourself, release each successful output exactly once with `orchestrata_free_json_v1`; do not modify the buffer or free it with the host allocator. Native calls are synchronous. No network connection, queue, partial progress, or timeout setting is provided by these methods.

If a library cannot be loaded, verify its absolute path, process architecture, and loader path before debugging the JSON. If it loads but reports an ABI mismatch, use a matching source build. If the documented source smoke tests pass only on one runtime and platform, report that scope; it is not evidence of packaged or general platform support.
