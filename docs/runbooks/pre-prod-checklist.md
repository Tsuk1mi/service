# Pre-production checklist

- [ ] Все secrets в K8s Secrets / External Secrets (не в git)
- [ ] `APP_ENV=production`, `RETURN_SMS_CODE_IN_RESPONSE=false`
- [ ] `REDIS_URL` задан, backend fail-fast без Redis
- [ ] RabbitMQ worker running, DLQ `notifications.dlq` мониторится
- [ ] TLS на Ingress, HSTS headers
- [ ] Swagger отключён (`APP_ENV=production`)
- [ ] `/send_code` защищён `X-Internal-Token`
- [ ] `/metrics` защищён basic auth (если exposed)
- [ ] PostgreSQL backup CronJob настроен
- [ ] Grafana dashboards + Alertmanager работают
- [ ] `cargo test` и `npm run test` проходят в CI
- [ ] k6 load test: p95 < 500ms при 50+ RPS
