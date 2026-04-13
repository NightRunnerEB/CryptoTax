SHELL := /bin/zsh

AUTH_ENV := .env
MIGRATIONS_DIR := migrations
SQLX := env DATABASE_URL="$$(sed -n 's/^DATABASE_URL=//p' $(AUTH_ENV) | head -n1)" sqlx

.PHONY: migrate-up migrate-down migrate-down-all migrate-revert migrate-info migrate-add

migrate-up:
	@$(SQLX) migrate run --source $(MIGRATIONS_DIR)

migrate-down:
	@$(SQLX) migrate revert --source $(MIGRATIONS_DIR)

migrate-down-all:
	@count=0; \
	while $(SQLX) migrate revert --source $(MIGRATIONS_DIR) >/dev/null 2>&1; do \
		count=$$((count + 1)); \
	done; \
	echo "Reverted $$count migration(s)."

migrate-revert:
	@$(SQLX) migrate revert --source $(MIGRATIONS_DIR)

migrate-info:
	@$(SQLX) migrate info --source $(MIGRATIONS_DIR)

migrate-add:
	@if [ -z "$(name)" ]; then \
		echo "Usage: make migrate-add name=create_users"; \
		exit 1; \
	fi
	@$(SQLX) migrate add "$(name)" --source $(MIGRATIONS_DIR)
