import http from 'k6/http';
import { sleep, check } from 'k6';

export const options = {
  vus: 50,
  duration: '2m',
  thresholds: {
    http_req_duration: ['p(95)<10'], // p95 < 10ms target
    http_req_failed: ['rate<0.01'],  // <1% errors
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
  const res = http.get(`${BASE_URL}/status`);
  check(res, {
    'status is 200': (r) => r.status === 200,
  });
  sleep(0.1);
}
