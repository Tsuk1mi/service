import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 10,
  duration: '30s',
  thresholds: {
    http_req_duration: ['p(95)<500'],
    http_req_failed: ['rate<0.05'],
  },
};

const BASE = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
  const start = http.post(`${BASE}/api/auth/start`, JSON.stringify({
    phone: '+79001234567',
  }), { headers: { 'Content-Type': 'application/json' } });

  check(start, {
    'auth start status 200': (r) => r.status === 200,
  });

  sleep(1);
}
