FROM docker.io/library/rust:1.93-bookworm AS build
RUN rustup target add wasm32-unknown-unknown
WORKDIR /work
COPY Cargo.toml Cargo.lock ./
COPY rust-server/Cargo.toml rust-server/Cargo.toml
COPY program0/Cargo.toml program0/Cargo.toml
COPY rust-server/src/ rust-server/src/
COPY program0/src/ program0/src/
RUN cargo build --locked --release -p skrynia-server
RUN cargo build --locked --release --target wasm32-unknown-unknown -p skrynia-program0

FROM docker.io/library/debian:bookworm-slim
RUN groupadd -g 1000 skrynia && useradd -u 1000 -g 1000 -M skrynia \
    && mkdir -p /var/lib/skrynia /usr/local/share/skrynia \
    && chown skrynia:skrynia /var/lib/skrynia
COPY --from=build /work/target/release/skrynia-server /usr/local/bin/skrynia-server
COPY --from=build /work/target/wasm32-unknown-unknown/release/skrynia_program0.wasm /usr/local/share/skrynia/root.wasm
USER 1000:1000
ENV SKRYNIA_ROOT_WASM=/usr/local/share/skrynia/root.wasm
ENV SKRYNIA_DATA_DIR=/var/lib/skrynia
ENV SKRYNIA_BIND=0.0.0.0:17380
EXPOSE 17380
ENTRYPOINT ["/usr/local/bin/skrynia-server"]
