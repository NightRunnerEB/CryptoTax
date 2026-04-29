## syntax=docker/dockerfile:1.7

FROM rust:1-bookworm AS build
WORKDIR /workspace/auth-svc

COPY crates /workspace/crates
COPY auth-svc /workspace/auth-svc

ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/workspace/auth-svc/target,sharing=locked \
    cargo build --release && \
    install -D /workspace/auth-svc/target/release/auth-svc /out/auth-svc

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10001 --create-home --shell /usr/sbin/nologin app

WORKDIR /app
COPY --from=build --chown=10001:10001 /out/auth-svc /app/auth-svc
COPY --chown=10001:10001 auth-svc/secrets /app/secrets
RUN chmod -R a+r /app/secrets

EXPOSE 8085
USER 10001
ENTRYPOINT ["/app/auth-svc"]
