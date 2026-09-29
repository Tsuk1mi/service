# Pre-production checklist

Legend: **[manifest]** = closed by in-repo manifests/CI when applied correctly; **[ops]** = cluster/environment action.

## Secrets & config

- [ ] **[ops]** Все secrets в K8s Secrets / External Secrets (не в git). Template: [`infra/k8s/base/secret.yaml.example`](../../infra/k8s/base/secret.yaml.example)
- [x] **[manifest]** `APP_ENV=production`, `RETURN_SMS_CODE_IN_RESPONSE=false` — [`infra/k8s/base/configmap.yaml`](../../infra/k8s/base/configmap.yaml)
- [x] **[manifest]** `REDIS_URL` задан в ConfigMap; backend fail-fast без Redis при `APP_ENV=production` (`src/config.rs`)
- [ ] **[ops]** `RABBITMQ_URL` password в Secret совпадает с RabbitMQ; worker Deployment running
- [x] **[manifest]** `INTERNAL_API_TOKEN` в secret example; `/send_code` ожидает `X-Internal-Token`
- [ ] **[ops]** `/metrics` basic auth: задать `METRICS_AUTH_USER` / `METRICS_AUTH_PASSWORD` в Secret, если metrics exposed

## Ingress / TLS

- [x] **[manifest]** TLS + HSTS на Ingress — [`infra/k8s/base/ingress.yaml`](../../infra/k8s/base/ingress.yaml) (`ssl-redirect`, `Strict-Transport-Security`)
- [ ] **[ops]** Заменить `rimskiy.example.com` / cert-manager issuer на реальный хост и ClusterIssuer
- [x] **[manifest]** Swagger отключён при `APP_ENV=production` (код + ConfigMap)

## Messaging & observability

- [x] **[manifest]** DLQ `notifications.dlq` alert — `NotificationsDlqNotEmpty` в [`infra/monitoring/prometheus/alerts.yml`](../../infra/monitoring/prometheus/alerts.yml)
- [ ] **[ops]** Prometheus scrapes RabbitMQ exporter / queue metrics so DLQ alert fires in cluster
- [x] **[manifest]** PostgreSQL backup CronJob — [`infra/k8s/base/postgres-backup-cronjob.yaml`](../../infra/k8s/base/postgres-backup-cronjob.yaml) + PVC
- [ ] **[ops]** Backup PVC retention / off-cluster copy verified
- [x] **[manifest]** Grafana provisioning path — [`infra/monitoring/grafana/provisioning/`](../../infra/monitoring/grafana/provisioning/)
- [ ] **[ops]** Grafana + Alertmanager reachable; receivers configured for on-call

## CI / load

- [x] **[manifest]** `cargo test` и `npm run test` в CI (`.github/workflows/github.yaml`)
- [ ] **[ops]** k6 load test before go-live: `k6 run tests/k6/auth_flow.js` — target **p95 < 500ms** at **50+ RPS** (see runbook)

## Ops go-live reminders

1. Create Secret from `secret.yaml.example` (never commit filled secret).
2. `kubectl apply -k infra/k8s/overlays/staging` → verify migrate Job + `/health/ready`.
3. Confirm Ingress TLS cert ready; curl `https://<host>/health/ready`.
4. Confirm worker consumes `notifications` and DLQ stays empty.
5. Run k6 against staging BASE_URL.
