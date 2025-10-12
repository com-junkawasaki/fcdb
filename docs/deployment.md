# Deployment Guide (Staging / Production)

## Prerequisites
- Kubernetes 1.25+
- NGINX Ingress Controller (or equivalent)
- Prometheus Operator (optional, for ServiceMonitor)
- Access to container registry (GHCR)

## Staging (Kubernetes Manifests)

```bash
# Apply Deployment/Service/Ingress
kubectl apply -f deploy/k8s/enishi-deployment.yaml

# Verify
kubectl get pods,svc,ingress | grep enishi
curl -H 'Host: enishi.local' http://<INGRESS_IP>/health
```

## Production (Helm)

```bash
# Add chart (local path)
helm upgrade --install enishi ./charts/enishi \
  --namespace enishi --create-namespace \
  --set image.repository=ghcr.io/<org>/<repo> \
  --set image.tag=0.1.0 \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=enishi.prod.example.com

# Check rollout
kubectl -n enishi rollout status deploy/enishi
```

## Observability

- Metrics: `GET /metrics` (Prometheus format)
- Health: `GET /health`
- Readiness: `GET /ready`

Prometheus scrape example (ServiceMonitor shown in docs/operations): see operations guide.

## SLO Targets (initial)
- p95: 3-hop ≤ 10ms、9-hop 35–80ms
- エラーレート: < 1%
- 可用性: ≥ 99.9%

## Load Testing (k6)

```bash
k6 run -e BASE_URL=http://enishi.prod.example.com loadtest/k6_3hop.js
```

## Rollback

```bash
# Helm rollback to previous revision
helm -n enishi rollback enishi 1
```

## Security
- Enable image signing (Cosign) and scanning (Trivy) in CI
- Store secrets via SealedSecrets/SOPS
- Restrict network access with NetworkPolicies
