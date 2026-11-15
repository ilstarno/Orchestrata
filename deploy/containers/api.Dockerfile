# syntax=docker/dockerfile:1
ARG RUST_IMAGE=rust:1-bookworm
ARG RUNTIME_IMAGE=debian:bookworm-slim
FROM ${RUST_IMAGE} AS build
WORKDIR /src
COPY . .
WORKDIR /src/backend
RUN cargo build --release -p orchestrata-api

FROM ${RUNTIME_IMAGE}
COPY --from=build /src/backend/target/release/orchestrata-api /usr/local/bin/orchestrata-api
ENV ORCHESTRATA_HTTP_BIND=0.0.0.0:3000
USER 65532:65532
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/orchestrata-api"]
