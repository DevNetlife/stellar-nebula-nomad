# Monitoring Runbook

Stack: `monitoring/docker-compose.yml` (issue #516) — Prometheus, Alertmanager, Grafana, node-exporter, the contract exporter, and Loki/Promtail for logs.

```sh
docker compose -f monitoring/docker-compose.yml up -d
```

Grafana: http://localhost:3000 (`GRAFANA_USER`/`GRAFANA_PASSWORD` env vars, defaults `admin`/`changeme` — change before any real deployment).

## Dashboards (`monitoring/grafana/dashboards/`)

- **System Health** — uptime, CPU/memory/disk, API latency/error rate, DB/cache connections.
- **Contract Metrics** (`nebula-overview.json`, pre-existing) — error rate, TX/min, gas usage.
- **Economic** — TVL, 24h volume, token price.
- **Game Metrics** — active players, scans/min, upgrades/hr.
- **Alerts** — firing alert counts by severity, and a live table.

## Alerting

`monitoring/prometheus/alert-rules.yml` defines the alert conditions (15+ rules across service-down, error-rate, latency, resource-exhaustion, and contract-specific conditions). `monitoring/alertmanager/alertmanager.yml` routes them: Slack for all severities, plus email for `critical`. Set `SLACK_WEBHOOK_URL`, `SMTP_SMARTHOST`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `ALERTS_FROM_EMAIL`, and `ONCALL_EMAIL` in the environment before starting the stack.

To test an alert fires end-to-end: temporarily lower a rule's threshold (or stop the service it watches), confirm it appears in the Alerts dashboard and the configured Slack channel/email within `group_wait` (30s), then revert.

## Logs

Promtail ships every container's stdout/stderr on the host into Loki; query them from Grafana's Explore view with the `Loki` datasource, e.g. `{container="nebula-contract-exporter"}`.

## Incident response

1. Check the **Alerts** dashboard for what's firing and its severity.
2. Check **System Health** for resource exhaustion, then the relevant metrics dashboard (Contract/Economic/Game) for the affected subsystem.
3. Cross-reference logs in Loki for the affected container around the alert's start time.
4. Escalate per the receiver that fired (`critical-alerts` → Slack `#nebula-critical` + on-call email; `warning-alerts` → Slack `#nebula-alerts`).

## Scope note

Distributed tracing (Jaeger/Tempo) and structured (JSON) application logging in the Rust/TS services are not included here — they're a larger change to each service's logging setup, out of scope for this pass. Loki still gives you the existing plain-text logs.
