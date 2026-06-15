# Runbook: Deploy

## Docker Compose (staging/VPS)

1. `cp .env.docker.example .env` и заполните секреты
2. `docker compose up --build -d`
3. `docker compose --profile bot up -d` (если нужен бот)
4. Проверка: `curl http://localhost/health/ready`

## Kubernetes

1. Создайте secret из `infra/k8s/base/secret.yaml.example`
2. `kubectl apply -k infra/k8s/overlays/staging`
3. Дождитесь Job `db-migrate`
4. Проверка Ingress: `curl https://staging.example.com/health/ready`

## Rollback

- Docker: `docker compose pull && docker compose up -d` предыдущий тег
- K8s: `kubectl rollout undo deployment/backend -n rimskiy`

## Incident: Redis down

- Симптом: OTP/auth failures, 503 на `/health/ready`
- Действие: проверить pod/redis, восстановить PVC, перезапустить backend

## Incident: DLQ messages

- Проверить очередь `notifications.dlq` в RabbitMQ management
- Исправить причину (Telegram token, FCM key) и requeue вручную
