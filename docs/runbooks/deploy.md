# Runbook: Deploy

## Docker Compose (staging/VPS)

1. `cp .env.docker.example .env` и заполните секреты
2. `docker compose up --build -d`
3. `docker compose --profile bot up -d` (если нужен бот)
4. Observability: `docker compose --profile obs up -d` (Prometheus/Grafana/Loki/Alertmanager)
5. Проверка: `curl http://localhost/health/ready`

## Kubernetes

1. Создайте secret из [`infra/k8s/base/secret.yaml.example`](../../infra/k8s/base/secret.yaml.example) (JWT ≥32, ENCRYPTION_KEY 64 hex, `INTERNAL_API_TOKEN`, passwords; optional FCM/OCR/SMS/metrics auth).
2. Проверьте ConfigMap: `APP_ENV=production`, `RETURN_SMS_CODE_IN_RESPONSE=false`, `REDIS_URL` — [`infra/k8s/base/configmap.yaml`](../../infra/k8s/base/configmap.yaml).
3. Подставьте реальный Ingress host/TLS issuer — [`infra/k8s/base/ingress.yaml`](../../infra/k8s/base/ingress.yaml) (уже: ssl-redirect + HSTS).
4. `kubectl apply -k infra/k8s/overlays/staging`
5. Дождитесь Job `db-migrate`
6. Проверка Ingress: `curl https://staging.example.com/health/ready`
7. Убедитесь, что backup CronJob активен: [`infra/k8s/base/postgres-backup-cronjob.yaml`](../../infra/k8s/base/postgres-backup-cronjob.yaml)
8. Grafana dashboards: provisioning из [`infra/monitoring/grafana/provisioning/`](../../infra/monitoring/grafana/provisioning/) (Compose profile `obs` или cluster stack).

## Rollback

- Docker: `docker compose pull && docker compose up -d` предыдущий тег
- K8s: `kubectl rollout undo deployment/backend -n rimskiy`

## Incident: Redis down

- Симптом: OTP/auth failures, 503 на `/health/ready`
- Действие: проверить pod/redis, восстановить PVC, перезапустить backend

## Incident: DLQ messages

- Alert: `NotificationsDlqNotEmpty` (`infra/monitoring/prometheus/alerts.yml`) when `notifications.dlq` depth > 0 for 5m
- Проверить очередь `notifications.dlq` в RabbitMQ management
- Исправить причину (Telegram token, FCM key, SMS provider) и requeue вручную

## Load test (k6, not in CI)

Target before prod: **p95 < 500ms** at **≥50 RPS** on auth start (or representative flow).

```bash
# Default: 10 VUs / 30s, p95 threshold 500ms (see tests/k6/auth_flow.js)
k6 run -e BASE_URL=https://staging.example.com tests/k6/auth_flow.js

# Stronger soak toward 50+ RPS
k6 run -e BASE_URL=https://staging.example.com \
  --vus 50 --duration 2m \
  tests/k6/auth_flow.js
```

Script thresholds already assert `http_req_duration p(95)<500` and error rate &lt; 5%. Fail the go-live checklist if thresholds are missed.
