# Rimskiy Web Client

React SPA для сервиса Rimskiy.

## Разработка

```bash
cd web
npm install
npm run dev
```

Vite проксирует `/api`, `/server-info`, `/health` на `http://localhost:8080`.

## Сборка

```bash
npm run build
```

Артефакт в `dist/` — раздавать через nginx (см. `deploy/nginx.conf.example`).

## Переменные окружения

| Переменная | Описание |
|------------|----------|
| `VITE_API_URL` | Базовый URL API. Пусто = same-origin (nginx proxy). |

Пример `.env.local`:

```
VITE_API_URL=
```
