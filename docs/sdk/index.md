# Use a native SDK

An **SDK** is a software development kit: a library your application calls directly. Orchestrata's native bindings load a compiled solver library into your process. They do not call an HTTP server.

## Choose a binding

Choose your language before installing anything. These are **source packages in the public engine repository**, not instructions to install a published package from PyPI, npm, Maven Central or NuGet. The binding and native library must come from a matching checkout.

| Language | Typed call | Where to start |
| --- | --- | --- |
| Python | `Solver.solve_problem` | Follow the complete Python walkthrough below. |
| Go | `SolveProblem` | [Go source and module](https://github.com/ilstarno/orchestrata-engine/tree/main/sdks/go); use the local module instructions in the [SDK guide](https://github.com/ilstarno/orchestrata-engine/tree/main/sdks). |
| Node.js | `solve` | [Node package guide](https://github.com/ilstarno/orchestrata-engine/tree/main/sdks/node). Requires a native addon build and lossless integer support. |
| C# | `Solver.Solve` | [C# project](https://github.com/ilstarno/orchestrata-engine/tree/main/sdks/csharp). Reference the source project or a locally built package. |
| Java | `Solver.solve` | [Java guide](https://github.com/ilstarno/orchestrata-engine/tree/main/sdks/java). Build its JAR and enable the documented native access. |
| PHP | `Solver->solve` | [PHP source](https://github.com/ilstarno/orchestrata-engine/tree/main/sdks/php) and [local Composer instructions](https://github.com/ilstarno/orchestrata-engine/tree/main/sdks). Requires the FFI extension. |

All six bindings expose typed problems and results. **Typed** means you construct language objects rather than assembling JSON strings. Raw JSON entrypoints remain available. See the matching repository guide for each language's runtime requirements and package-consumer commands.

If you already have an Orchestrata server and want to avoid building native code, use the [HTTP API](../solve-api/) from your language's HTTP client instead. It needs no native SDK or Rust installation on the client.

## Build and locate the library

The walkthrough below uses Linux and Python 3.10 or later. It installs the Python source package into an isolated environment and solves one small problem. A Python binding package does **not** include the native solver binary.

**Rust** is the language used to implement the solver. **Cargo** is Rust's build and dependency tool; here it compiles the shared library your Python program loads. Install stable Rust through the [official Rust guide](https://rust-lang.org/tools/install/) and the compiler/linker required for your platform. You also need Git and Python with `venv` and `pip` support.

Check the tools in a new terminal:

```sh
git --version
rustc --version
cargo --version
python3 --version
```

Download the **standalone engine**, which is distinct from the HTTP server repository. Keep all following commands at this checkout root:

```sh
git clone https://github.com/ilstarno/orchestrata-engine.git
cd orchestrata-engine
cargo build --locked --release -p orchestrata-ffi
```

`--release` builds the optimized library. On Linux, the resulting file is `target/release/liborchestrata_ffi.so`. On macOS it is `liborchestrata_ffi.dylib`; on Windows it is `orchestrata_ffi.dll`. Use the binary for your operating system and processor. If you configure a different Cargo target directory, use that output path instead.

## Python · install and solve

A **virtual environment** keeps this project's Python packages separate from your system installation. The [official Python venv guide](https://docs.python.org/3/library/venv.html) explains platform setup.

From the engine checkout root:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install ./sdks/python
.venv/bin/python - <<'PY'
from orchestrata import PreferValue, Problem, Solver, Variable

solver = Solver("target/release/liborchestrata_ffi.so")
problem = Problem(
    (Variable("job", (0, 1)),),
    (PreferValue(0, 1, 1),),
)
result = solver.solve_problem(problem, max_evaluations=2)
assert result is not None
print(result.values)
PY
```

Expected output:

```text
(1,)
```

The job can use resource `0` or `1` and prefers `1`. Both candidates fit the evaluation budget of `2`. The result selects resource `1`. Use an absolute native-library path if your application runs from another directory. This shell walkthrough is for Linux; adjust the library name and Python environment executable for other platforms.

## Problem and result format

A problem supplies variables, their allowed integer values and constraints. References to variables start at zero. Your application supplies the meaning of each resource identifier; the solver does not infer people, dates or units.

The current typed bindings support `AllDifferent`, `NotEqualOffset`, `PreferValue`, `CountValue`, `Precedence`, `NoOverlap` and `ResourceCapacity`. Use the [typed examples and requirements](https://github.com/ilstarno/orchestrata-engine/tree/main/sdks) from the same checkout. This native contract is separate from the fields supported by an HTTP server build.

| Outcome | Meaning |
| --- | --- |
| Feasible result | The completed exact search returns an optimum for the supplied supported finite-domain model. This does not describe the separate heuristic routing algorithms. |
| No feasible assignment | Python returns `None`; Go returns `nil, nil`; Node returns `kind: 'infeasible'`; C# returns `Infeasible`; Java returns an empty `Optional`; PHP returns `null`. |
| Evaluation budget exceeded | An error, not an infeasible result and not a partial plan. Reduce the model or choose an appropriate budget. |
| Invalid input, load failure or execution error | Fix the request or runtime problem before interpreting a planning result. |

Domains and scores use signed 64-bit integers. Preserve those values exactly in your language. For Node's typed API, use `bigint` for 64-bit quantities and the runtime requirements in its source guide; ordinary JSON parsing can round large numbers. Calls are synchronous, so keep long work off an application event-loop or request thread.

## Troubleshooting

| Symptom | What to check |
| --- | --- |
| `cargo` or `rustc` not found | Finish the Rust installation and reopen the terminal. Cargo is included with rustup. |
| Native build cannot find a linker | Install the compiler/linker described by Rust's platform guide. |
| Python cannot create the environment | Check your Python installation's `venv` and `pip` support using the official Python guide. |
| `ModuleNotFoundError: orchestrata` | Install `./sdks/python` and run the `.venv/bin/python` from that same environment. |
| The library cannot be opened | Check the full library path, filename, operating system and processor architecture. Installing the Python package alone does not build it. |
| A constraint is rejected | Use binding and native library from the same checkout. An older library can reject a newer constraint even when its ABI version matches. |
| Budget error | The product of the domain lengths exceeds the allowed evaluations. It does not mean the model is infeasible. |

## Other language quickstarts and native contract

Use the [engine SDK guide](https://github.com/ilstarno/orchestrata-engine/tree/main/sdks) for the current local package instructions and typed examples for all six languages. No registry package name in that source is a promise of a published release.

The [C interface](https://github.com/ilstarno/orchestrata-engine/blob/main/crates/ffi/include/orchestrata.h) defines the native ABI, memory ownership, input limits and error codes. **ABI** means the binary interface between your language and the compiled library. Most applications should use their language binding rather than call that interface themselves.
