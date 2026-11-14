FROM docker.io/library/rust:1.93-bookworm AS build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /work
COPY Cargo.toml Cargo.lock ./
COPY rust-server/Cargo.toml rust-server/Cargo.toml
COPY rust-server/src/ rust-server/src/
RUN cargo build --locked --release -p skrynia-server

FROM docker.io/library/debian:bookworm-slim
ARG SKRYNIA_VERSION=development
ARG SKRYNIA_COMMIT=development
ENV SKRYNIA_VERSION=${SKRYNIA_VERSION}
ENV SKRYNIA_COMMIT=${SKRYNIA_COMMIT}
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
RUN groupadd -g 1000 skrynia && useradd -u 1000 -g 1000 -M skrynia \
    && mkdir -p /var/lib/skrynia && chown skrynia:skrynia /var/lib/skrynia
COPY --from=build /work/target/release/skrynia-server /usr/local/bin/skrynia-server
USER 1000:1000
ENV SKRYNIA_HOST=0.0.0.0
EXPOSE 17380
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 CMD skrynia-server healthcheck
ENTRYPOINT ["/usr/local/bin/skrynia-server"]
