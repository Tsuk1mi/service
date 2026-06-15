# Rimskiy Service

Backend и веб-клиент для сервиса управления перекрытыми автомобилями на парковке.

## Структура проекта

| Каталог | Описание |
|---------|----------|
| `src/` | Rust backend (Axum REST API) |
| `src/bin/telegram_bot.rs` | Telegram-бот (авторизация, проверка блокировок) |
| `web/` | React SPA (Vite + TypeScript + MUI) |
| `deploy/` | Примеры конфигурации nginx |

## Быстрый старт

### Docker (HTTP, всё в контейнерах)

```bash
cp .env.docker.example .env
docker compose up --build -d
```

Откройте **http://localhost** — веб-приложение. API: **http://localhost:8080**.

С Telegram-ботом (нужен `TELEGRAM_BOT_TOKEN` в `.env`):

```bash
docker compose --profile bot up --build -d
```

Остановка: `docker compose down` (данные PostgreSQL в volume `pgdata`).

### 1. Backend (локально)

```bash
cp .env.example .env
# Отредактируйте .env (DATABASE_URL, JWT_SECRET, ENCRYPTION_KEY)
cargo run
```

### 2. Telegram-бот (опционально)

```bash
cargo run --bin telegram_bot
```

Требуется `TELEGRAM_BOT_TOKEN` в `.env`.

### 3. Веб-клиент (разработка)

```bash
cd web
npm install
npm run dev
```

Откройте http://localhost:5173 — API проксируется на `http://localhost:8080`.

### 4. Production (nginx)

1. Соберите frontend: `cd web && npm run build`
2. Скопируйте `web/dist/` на сервер (например `/var/www/rimskiy/web`)
3. Настройте nginx по примеру [`deploy/nginx.conf.example`](deploy/nginx.conf.example)
4. Укажите `WEB_APP_URL=https://your-domain.com` в `.env` backend

## Переменные окружения

### Обязательные

- `DATABASE_URL` — PostgreSQL
- `JWT_SECRET` — секрет JWT (минимум 32 символа)
- `ENCRYPTION_KEY` — 64 hex символа (32 байта)

### Опциональные

- `SERVER_HOST` / `SERVER_PORT` — адрес API (по умолчанию `0.0.0.0:8080`)
- `WEB_APP_URL` — URL веб-приложения (для `/server-info` и Telegram `/apk`)
- `TELEGRAM_BOT_TOKEN` / `TELEGRAM_BOT_USERNAME` — Telegram-бот
- `SMS_API_URL` / `SMS_API_KEY` — SMS-провайдер
- `TELEPHONY_API_URL` / `TELEPHONY_API_KEY` — звонки владельцам

Полный список: [`.env.example`](.env.example)

## Генерация ключа шифрования

```bash
# Linux/Mac
openssl rand -hex 32

# Windows PowerShell
-join ((48..57) + (97..102) | Get-Random -Count 64 | ForEach-Object {[char]$_})
```

## Архитектура сервисов (Docker / K8s)

| Сервис | Назначение |
|--------|------------|
| `backend` | REST API (Axum), auth, blocks, OCR |
| `web` | React SPA (nginx) |
| `telegram-bot` | Telegram + HTTP `/send_code` |
| `notification-worker` | RabbitMQ → Telegram/FCM/SMS |
| `db` | PostgreSQL |
| `redis` | OTP, rate limit, JWT blacklist |
| `rabbitmq` | Async notifications + DLQ |

Observability (profile `obs`): Prometheus, Grafana, Loki, Promtail, Alertmanager.

## Kubernetes

Манифесты: [`infra/k8s/`](infra/k8s/)

```bash
# Staging
kubectl apply -k infra/k8s/overlays/staging

# Production (manual approval в CI)
kubectl apply -k infra/k8s/overlays/prod
```

Перед деплоем создайте Secret из [`infra/k8s/base/secret.yaml.example`](infra/k8s/base/secret.yaml.example).

CI: push в `main` → build images → GHCR → deploy staging (если `K8S_STAGING_ENABLED=true`).

Runbooks: [`docs/runbooks/`](docs/runbooks/)

## CI/CD (GitHub Actions)

### Variables

- `DEPLOY_HOST`, `DEPLOY_USER`, `BACKEND_DEPLOY_PATH`, `BACKEND_SERVICE_NAME`, `TELEGRAM_BOT_SERVICE_NAME`
- `WEB_DEPLOY_HOST`, `WEB_DEPLOY_PATH` — деплой статики frontend (опционально)

### Secrets

- `DEPLOY_KEY` — SSH ключ для backend
- `WEB_DEPLOY_KEY` — SSH ключ для frontend (опционально)

## Telegram-бот

Команды:

- `/help` — справка
- `/code <телефон>` — код авторизации (SMS + Telegram)
- `/block <номер>` — проверка блокировки
- `/apk` — ссылка на веб-приложение

## API

- Swagger UI: http://localhost:8080/swagger-ui/
- OpenAPI: http://localhost:8080/api-doc/openapi.json
- Health: http://localhost:8080/health
- Server info: http://localhost:8080/server-info

## Функции веб-клиента

- Авторизация по SMS / Telegram
- Профиль и управление автомобилями
- Создание блокировок (ручной ввод + OCR фото)
- Список «кто меня перекрыл» с контактами
- In-app уведомления (polling)
- Предупреждение владельца (звонок через backend)
