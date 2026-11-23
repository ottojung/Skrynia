FROM docker.io/library/rust:1.93-bookworm AS build
WORKDIR /work
COPY Cargo.toml Cargo.lock ./
COPY rust-server/Cargo.toml rust-server/Cargo.toml
COPY rust-server/src/ rust-server/src/
RUN cargo build --locked --release -p skrynia-server

FROM docker.io/library/debian:bookworm-slim
RUN groupadd -g 1000 skrynia && useradd -u 1000 -g 1000 -M skrynia && mkdir -p /var/lib/skrynia/operator /var/lib/skrynia/user && chmod 0700 /var/lib/skrynia /var/lib/skrynia/operator /var/lib/skrynia/user && chown -R skrynia:skrynia /var/lib/skrynia
COPY --from=build /work/target/release/skrynia-server /usr/local/bin/skrynia-server
USER 1000:1000
ENV SKRYNIA_BIND=0.0.0.0:17380
ENV SKRYNIA_OPERATOR_DIR=/var/lib/skrynia/operator
ENV SKRYNIA_USER_DIR=/var/lib/skrynia/user
EXPOSE 17380
# Operator MUST mount a separately compiled root WASM and set SKRYNIA_ROOT_WASM.
# There is no default Program 0, admin CLI, or native /health endpoint.
ENTRYPOINT ["/usr/local/bin/skrynia-server"]
