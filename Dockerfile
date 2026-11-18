FROM docker.io/library/rust:1.93-bookworm AS build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev
WORKDIR /work
COPY Cargo.toml Cargo.lock ./
COPY rust-server/Cargo.toml rust-server/Cargo.toml
COPY rust-server/src/ rust-server/src/
RUN cargo build --locked --release -p skrynia-server

FROM docker.io/library/debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3
RUN groupadd -g 1000 skrynia && useradd -u 1000 -g 1000 -M skrynia && mkdir -p /var/lib/skrynia && chown skrynia:skrynia /var/lib/skrynia
COPY --from=build /work/target/release/skrynia-server /usr/local/bin/skrynia-server
USER 1000:1000
ENV SKRYNIA_BIND=0.0.0.0:17380
ENV SKRYNIA_DATA_DIR=/var/lib/skrynia
EXPOSE 17380
# Operator MUST mount a separately compiled root WASM and set SKRYNIA_ROOT_WASM.
# There is no default Program 0, admin CLI, or native /health endpoint.
ENTRYPOINT ["/usr/local/bin/skrynia-server"]
