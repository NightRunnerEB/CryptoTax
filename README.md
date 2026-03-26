# CryptoTax

## Testing

На текущем этапе полноценное покрытие тестами добавлено для `auth-svc`:
- `unit` и `contract` тесты запускаются обычной командой `cargo test -p auth-svc`
- `integration` и `e2e smoke` помечены как `#[ignore]` и запускаются вручную

Аналогичная схема добавлена для `ledger-svc`:
- `unit` и `contract`: `cargo test -p ledger-svc`
- `integration` и `e2e smoke`: `cargo test -p ledger-svc -- --ignored --nocapture`

### 1) Поднять тестовый Postgres

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

### 2) Применить миграции

```bash
for f in auth-svc/migrations/*.sql; do
  psql "postgres://auth_user:auth_pass@127.0.0.1:55433/auth" -v ON_ERROR_STOP=1 -f "$f"
done
```

### 3) Запустить unit + contract

```bash
DATABASE_URL=postgres://auth_user:auth_pass@127.0.0.1:55433/auth \
AUTH_TEST_DATABASE_URL=postgres://auth_user:auth_pass@127.0.0.1:55433/auth \
cargo test -p auth-svc
```

### 4) Запустить integration + e2e smoke (manual)

```bash
DATABASE_URL=postgres://auth_user:auth_pass@127.0.0.1:55433/auth \
AUTH_TEST_DATABASE_URL=postgres://auth_user:auth_pass@127.0.0.1:55433/auth \
cargo test -p auth-svc -- --ignored --nocapture
```

### 5) CI workflows

- `.github/workflows/auth-unit-contract.yml` — автоматический запуск только `unit+contract`
- `.github/workflows/auth-manual-integration-e2e.yml` — ручной запуск `integration+e2e` (`workflow_dispatch`)
- `.github/workflows/ledger-unit-contract.yml` — автоматический запуск только `unit+contract` для `ledger-svc`
- `.github/workflows/ledger-manual-integration-e2e.yml` — ручной запуск `integration+e2e` для `ledger-svc`

### 6) Быстрый запуск тестов ledger-svc

```bash
docker rm -f cryptotax-ledger-test-pg >/dev/null 2>&1 || true
docker run -d \
  --name cryptotax-ledger-test-pg \
  -e POSTGRES_USER=ledger_user \
  -e POSTGRES_PASSWORD=ledger_pass \
  -e POSTGRES_DB=ledger \
  -p 55434:5432 \
  postgres:16

for f in ledger-svc/migrations/*up.sql; do
  psql "postgres://ledger_user:ledger_pass@127.0.0.1:55434/ledger" -v ON_ERROR_STOP=1 -f "$f"
done

DATABASE_URL=postgres://ledger_user:ledger_pass@127.0.0.1:55434/ledger \
LEDGER_TEST_DATABASE_URL=postgres://ledger_user:ledger_pass@127.0.0.1:55434/ledger \
cargo test -p ledger-svc

DATABASE_URL=postgres://ledger_user:ledger_pass@127.0.0.1:55434/ledger \
LEDGER_TEST_DATABASE_URL=postgres://ledger_user:ledger_pass@127.0.0.1:55434/ledger \
cargo test -p ledger-svc -- --ignored --nocapture
```
