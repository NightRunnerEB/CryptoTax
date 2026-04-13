## syntax=docker/dockerfile:1.7

FROM rust:1-bookworm AS build
WORKDIR /workspace/ledger-svc

COPY crates /workspace/crates
COPY ledger-svc /workspace/ledger-svc

ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/workspace/ledger-svc/target,sharing=locked \
    cargo build --release && \
    install -D /workspace/ledger-svc/target/release/ledger-svc /out/ledger-svc

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10001 --create-home --shell /usr/sbin/nologin app

WORKDIR /app
COPY --from=build --chown=10001:10001 /out/ledger-svc /app/ledger-svc
COPY --chown=10001:10001 ledger-svc/config.yaml /app/config.yaml
COPY --chown=10001:10001 ledger-svc/worker_config.yaml /app/worker_config.yaml
COPY --chown=10001:10001 ledger-svc/src/application/exchanges/configs /app/src/application/exchanges/configs

EXPOSE 8086
USER 10001
ENTRYPOINT ["/app/ledger-svc"]
