# Единый Docker-стек Rimskiy Service

Используйте корневой `docker-compose.yml` из репозитория:

```bash
cp .env.docker.example .env
# отредактируйте секреты

# App stack (db, redis, rabbitmq, backend, web, worker)
docker compose up --build -d

# + Telegram bot
docker compose --profile bot up -d

# + Observability (Prometheus, Grafana, Loki, Promtail, Alertmanager)
docker compose --profile obs up -d

# Всё вместе
docker compose --profile bot --profile obs up -d
```

Профили:
- **default** — приложение
- **bot** — telegram-bot
- **obs** — мониторинг и логи
- **full** — alias для bot+obs (используйте оба профиля)
