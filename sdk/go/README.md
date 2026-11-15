# Orchestrata Go SDK

The Go SDK uses the standalone native engine through the C ABI.

Build the native library first:

```sh
cd backend
cargo build -p orchestrata-ffi --release
```

Then run the Go tests from `sdk/go` with the library directory on the linker and runtime search paths:

```sh
CGO_LDFLAGS="-L$(pwd)/../../backend/target/release" LD_LIBRARY_PATH="$(pwd)/../../backend/target/release" go test ./...
```

The current ABI supports finite integer domains, offset inequality constraints, and value preferences. The native ABI must be expanded before the SDK can express all planning models.
