dev:
    docker compose up postgres -d

sqlx-prepare:
    cargo sqlx prepare
