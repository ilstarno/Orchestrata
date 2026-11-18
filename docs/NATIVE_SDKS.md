# Native solver SDKs

The native solver and its Rust, Python, Java, C#, Go, Node.js and PHP interfaces
are maintained in [orchestrata-engine](https://github.com/ilstarno/orchestrata-engine).
Use the bindings and native library from the same Engine revision.

For a fixed source reference, use
[`c4e529b81dc2466ecb2c193b119632dc4dddadd9`](https://github.com/ilstarno/orchestrata-engine/tree/c4e529b81dc2466ecb2c193b119632dc4dddadd9).
Its [SDK guide](https://github.com/ilstarno/orchestrata-engine/blob/c4e529b81dc2466ecb2c193b119632dc4dddadd9/sdks/README.md)
documents build requirements, native buffer ownership, numeric limits and
language-specific examples. The language packages require a separately built
native library for the target operating system and architecture.

The native ABI version identifies the calling convention; it does not identify
the complete set of supported constraints. Keep the Engine revision with the
application's dependency record when reproducing a result. Exact search budget
exhaustion is an error, distinct from a successful proof of infeasibility.

This repository's HTTP example and internal application services retain their
own contracts and dependency pins. The SDK reference does not change those
services, enable authentication, or expose a recipe execution endpoint.
