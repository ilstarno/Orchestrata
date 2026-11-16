# syntax=docker/dockerfile:1
ARG RUST_IMAGE=rust:1-trixie
ARG RUNTIME_IMAGE=debian:trixie-slim

FROM ${RUST_IMAGE} AS build
WORKDIR /src
COPY backend/ backend/
COPY backend_tests/ backend_tests/
COPY recipes/ recipes/
RUN cargo build --manifest-path backend/Cargo.toml --locked --release -p orchestrata-api

FROM ${RUNTIME_IMAGE}
COPY --from=build /src/backend/target/release/orchestrata-api /usr/local/bin/orchestrata-api
ENV ORCHESTRATA_HTTP_BIND=0.0.0.0:3000
EXPOSE 3000
USER 10001:10001
ENTRYPOINT ["/usr/local/bin/orchestrata-api"]
