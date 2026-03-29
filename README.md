# CryptoTax

## Overview

CryptoTax is a Rust backend workspace for crypto accounting and tax workflows.
The system is split into microservices with clear responsibilities and shared domain crates.

## Services

- `auth-svc`
  Handles registration, login, refresh-token rotation, logout, email verification, and tax-profile bootstrap calls.
  Uses Postgres and Redis.

- `ledger-svc`
  Handles exchange import ingestion (CSV/API), transaction normalization, import lifecycle, and outbox publishing.
  Uses Postgres, Redis, and RabbitMQ.

- `crates/*`
  Shared libraries (for example cache and external client adapters) reused across services.

## Testing

Both services follow the same testing model:

- **Unit tests**
  Pure domain/business logic tests.

- **Contract tests**
  HTTP router/handler behavior tests (`axum` routes, status codes, payload shape).

- **Integration tests** (`#[ignore]`, manual)
  Repository and transaction/UoW tests against real Postgres.

- **E2E smoke tests** (`#[ignore]`, manual)
  Service-level boot + real HTTP request smoke validation.

### Run `auth-svc` tests

1. Start test Postgres:

```bash
docker rm -f cryptotax-auth-test-pg >/dev/null 2>&1 || true
docker run -d \
  --name cryptotax-auth-test-pg \
  -e POSTGRES_USER=auth_user \
  -e POSTGRES_PASSWORD=auth_pass \
  -e POSTGRES_DB=auth \
  -p 55433:5432 \
  postgres:16
```

2. Apply migrations:

```bash
for f in auth-svc/migrations/*.sql; do
  psql "postgres://auth_user:auth_pass@127.0.0.1:55433/auth" -v ON_ERROR_STOP=1 -f "$f"
done
```

3. Run unit + contract tests:

```bash
DATABASE_URL=postgres://auth_user:auth_pass@127.0.0.1:55433/auth \
AUTH_TEST_DATABASE_URL=postgres://auth_user:auth_pass@127.0.0.1:55433/auth \
cargo test -p auth-svc
```

4. Run integration + e2e smoke tests (manual):

```bash
DATABASE_URL=postgres://auth_user:auth_pass@127.0.0.1:55433/auth \
AUTH_TEST_DATABASE_URL=postgres://auth_user:auth_pass@127.0.0.1:55433/auth \
cargo test -p auth-svc -- --ignored --nocapture
```

### Run `ledger-svc` tests

1. Start test Postgres:

```bash
docker rm -f cryptotax-ledger-test-pg >/dev/null 2>&1 || true
docker run -d \
  --name cryptotax-ledger-test-pg \
  -e POSTGRES_USER=ledger_user \
  -e POSTGRES_PASSWORD=ledger_pass \
  -e POSTGRES_DB=ledger \
  -p 55434:5432 \
  postgres:16
```

2. Apply migrations:

```bash
for f in ledger-svc/migrations/*up.sql; do
  psql "postgres://ledger_user:ledger_pass@127.0.0.1:55434/ledger" -v ON_ERROR_STOP=1 -f "$f"
done
```

3. Run unit + contract tests:

```bash
DATABASE_URL=postgres://ledger_user:ledger_pass@127.0.0.1:55434/ledger \
LEDGER_TEST_DATABASE_URL=postgres://ledger_user:ledger_pass@127.0.0.1:55434/ledger \
cargo test -p ledger-svc
```

4. Run integration + e2e smoke tests (manual):

```bash
DATABASE_URL=postgres://ledger_user:ledger_pass@127.0.0.1:55434/ledger \
LEDGER_TEST_DATABASE_URL=postgres://ledger_user:ledger_pass@127.0.0.1:55434/ledger \
cargo test -p ledger-svc -- --ignored --nocapture
```

## CI Workflows

- `.github/workflows/auth-unit-contract.yml`
  Auto-runs `auth-svc` unit + contract tests on `push`/`pull_request`.

- `.github/workflows/auth-manual-integration-e2e.yml`
  Manual `workflow_dispatch` for ignored integration + e2e tests.

- `.github/workflows/ledger-unit-contract.yml`
  Auto-runs `ledger-svc` unit + contract tests on `push`/`pull_request`.

- `.github/workflows/ledger-manual-integration-e2e.yml`
  Manual `workflow_dispatch` for ignored integration + e2e tests.
