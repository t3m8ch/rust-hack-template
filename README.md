# rust-hack-template

Backend-шаблон на Rust.

## Стек

- Rust
- Axum
- PostgreSQL
- SQLx
- Socketioxide

## Архитектура

Проект разделён по слоям:

- `src/main.rs` — точка входа, загрузка `.env`, создание `Config`, подключение к Postgres, запуск сервера
- `src/lib.rs` — сборка `AppState`, роутера и запуск приложения
- `src/rest/` — HTTP роуты и handlers
- `src/dto/` — request/response DTO
- `src/db/` — SQLx-запросы и row-структуры
- `src/auth/` — логика паролей и сессий
- `src/extractors/` — переиспользуемые Axum extractors
- `src/error.rs` — единый формат ошибок API
- `migrations/` — источник истины для схемы БД
- `tests/` — интеграционные и e2e тесты

Сейчас в проекте есть:
- `GET /hello/`
- `POST /auth/register`
- `POST /auth/login`
- `POST /auth/logout`
- `GET /auth/me`

## Запуск

### 1. Поднять Postgres

```bash
just dev
```

Это поднимает чистый Postgres. Схема будет создана самим приложением через SQLx migrations при старте.

По умолчанию Postgres будет доступен по адресу:

```text
postgres://postgres:postgres@localhost:1311/postgres
```

### 2. При необходимости создать `.env`

Проект умеет работать и без `.env`, используя значения по умолчанию.

Пример:

```env
HOST=0.0.0.0
PORT=8050
DATABASE_URL=postgres://postgres:postgres@localhost:1311/postgres
SESSION_COOKIE_NAME=session
SESSION_SECURE_COOKIE=false
SESSION_TTL_DAYS=7
```

### 3. Запустить приложение

```bash
cargo run
```

При старте приложение автоматически применит все миграции из `migrations/`, а затем поднимет HTTP-сервер. Если миграции не применятся, приложение завершится с ошибкой и не начнёт принимать запросы.

Сервер стартует на:

```text
http://127.0.0.1:8050
```

## Полезные команды

```bash
cargo fmt
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Если меняли SQL или схему БД:

```bash
just sqlx-prepare
```

`migrations/` — единственный источник истины для схемы БД в проекте.
