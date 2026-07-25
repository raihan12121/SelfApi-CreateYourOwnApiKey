# Technical Requirements Document (TRD)

## 1. System architecture overview

Five major components:

1. **Desktop agent** — runs on the user's machine, manages the model runtime and the outbound tunnel connection.
2. **Relay/tunnel layer** — cloud-hosted, holds persistent outbound connections from every agent, performs NAT traversal.
3. **API gateway** — cloud-hosted, terminates all public API traffic, handles auth/rate-limiting/billing, and multiplexes requests down to the correct relay connection.
4. **Analytics & logging pipeline** — captures every request/response metadata event, aggregates it, and serves it to the dashboard.
5. **Dashboard/control-plane web app** — where the user manages keys, models, machines, billing, and views analytics/history.

```
[User's GPU + model runtime] <-> [Desktop agent] <-> (outbound tunnel) <-> [Relay layer] <-> [API gateway] <-> [Developer's app]
                                                                                   |
                                                                          [Analytics pipeline] <-> [Dashboard]
```

## 2. Desktop agent

- **Language/runtime**: native binary (Rust or Go preferred for low overhead and cross-platform builds) with a thin UI shell (Tauri recommended over Electron for lower resource usage, since the agent already competes with the GPU/CPU it's monitoring).
- **Responsibilities**:
  - Hardware detection (GPU vendor/model, VRAM, driver version) via vendor APIs (NVML for NVIDIA, ROCm-smi for AMD, Metal APIs for Apple Silicon).
  - Model runtime management: wraps llama.cpp / vLLM (where hardware supports it) / Ollama as pluggable backends.
  - Local health check server (loopback only) to confirm the model is responsive.
  - Tunnel client: maintains a persistent authenticated outbound connection (see Section 4) with automatic exponential-backoff reconnect.
  - Local metrics emitter: sends request-level events (timestamp, model, token counts, latency, status) to the analytics pipeline in near-real-time, batched to reduce overhead.
  - Kill switch: instantly stops accepting new requests and optionally drains in-flight ones on user command.

## 3. Model runtime layer

- Support llama.cpp as the baseline backend (broadest hardware compatibility, CPU fallback).
- Support vLLM where CUDA + sufficient VRAM is detected (better throughput for higher-end GPUs).
- Standardize on an **OpenAI-compatible API contract** (`/v1/chat/completions`, `/v1/completions`, `/v1/models`) at the local runtime level so the gateway can treat every backend identically.
- Quantization presets (Q4_K_M, Q5_K_M, Q8_0, etc.) selected automatically based on detected VRAM, with manual override available.

## 4. Relay / tunnel layer (NAT traversal)

- **Pattern**: reverse-connection tunneling, matching the proven approach used by ngrok/Cloudflare Tunnel/Localtonet.
  1. Agent opens a long-lived outbound connection (QUIC preferred, WebSocket/HTTP2 fallback) to a relay node.
  2. Relay authenticates the agent (mutual TLS or signed token) and registers a mapping: `account_id/machine_id -> open connection`.
  3. Inbound API requests arrive at the API gateway, which looks up the correct relay connection and multiplexes the request down it as a stream.
  4. Response streams back up the same connection to the gateway, then to the caller.
- **Multiplexing**: use HTTP/2 or QUIC streams so a single TCP/UDP connection per agent can carry many concurrent requests without needing one process per user.
- **Reconnection**: exponential backoff with jitter; on reconnect, the same public endpoint/key must remain valid (session resumption tied to account, not to the physical connection).
- **Geographic routing**: deploy relay nodes in multiple regions; route each agent to the nearest relay to minimize added latency, since total response time = network round trip + model inference time.
- **Encryption**: TLS between developer's app and gateway; separate encrypted tunnel between gateway/relay and agent, so plaintext prompts never traverse an unencrypted hop.

## 5. API gateway

- **Responsibilities**: API key issuance/validation, per-key rate limiting, request routing to the correct relay connection, request/response logging hook, billing meter, and fallback routing.
- **Auth model**: bearer-token API keys, scoped per key (model access, rate limit, spend cap), validated on every request before any bytes reach the user's machine — this is the mandatory compensating control given that most local LLM runtimes ship with no built-in authentication.
- **Rate limiting**: token-bucket per API key, with burst allowance configurable per tier.
- **Fallback logic**: if the relay reports the target agent connection is not currently open (or a health-check ping times out within a short threshold, e.g. 2–3 seconds), the gateway transparently reroutes the request to a configured low-cost cloud model and tags the resulting log entry as `served_via: fallback`.
- **Statelessness**: gateway instances should be horizontally scalable and stateless; connection-to-agent mapping lives in a shared fast store (e.g., Redis) so any gateway node can route to any relay.

## 6. Analytics & request-history pipeline

This directly implements PRD FR-7 through FR-14.

- **Event ingestion**: every completed request emits a structured event (not the full prompt/response by default — see privacy note below) containing:
  - `request_id`, `account_id`, `api_key_id`, `machine_id`, `model`, `timestamp_start`, `timestamp_end`
  - `input_tokens`, `output_tokens`, `ttft_ms`, `total_duration_ms`
  - `status` (success/error/rate_limited/timeout), `served_via` (local/fallback)
- **Pipeline**: agent and gateway both emit events to a streaming ingestion endpoint (e.g., Kafka or a managed equivalent) → written to a time-series/analytics store (e.g., ClickHouse or TimescaleDB) for fast aggregation queries → a separate hot-path row store (e.g., Postgres) holds recent detailed logs for the searchable history table.
- **Aggregation jobs**: pre-compute rollups (per-hour, per-day) for the dashboard charts (FR-9, FR-10) so the UI never runs expensive raw scans on demand.
- **Content storage (prompt/response text)**: stored separately from metadata, encrypted at rest, and governed by the user-configurable retention window (FR-13). Default should be metadata-only or short retention, opt-in for longer content retention, to respect the privacy value proposition.
- **Query API**: the dashboard backend exposes paginated, filterable endpoints (`GET /v1/requests?from=&to=&model=&status=&key=`) and aggregate endpoints (`GET /v1/analytics/usage?granularity=day`).
- **Export**: async job that generates a CSV/JSON file for a given date range and account (FR-12), delivered via signed download URL.
- **Alerting**: a rules engine evaluating rollup metrics (error rate, uptime, usage thresholds) against user-defined triggers, dispatching via email/webhook (FR-14).

## 7. Dashboard / control-plane web app

- **Stack**: standard web app (React/Next.js) talking to the gateway's control-plane API; no direct access to any user's local machine except through the relay.
- **Key screens** (detailed further in the design description): dashboard home (totals + alerts), request history table, usage/analytics charts, model management, machine management, API key management, billing/marketplace earnings, settings (including data retention controls).
- **Real-time updates**: dashboard home and "live" request count use a WebSocket or SSE feed from the gateway for near-real-time updates; historical views use the paginated query API.

## 8. Data model (core tables, simplified)

- `accounts (id, email, tier, created_at)`
- `machines (id, account_id, hardware_profile, status, last_seen_at)`
- `api_keys (id, account_id, scope, rate_limit, spend_cap, created_at, revoked_at)`
- `models (id, machine_id, name, quantization, status)`
- `requests (id, account_id, api_key_id, machine_id, model_id, input_tokens, output_tokens, ttft_ms, duration_ms, status, served_via, created_at)` — partitioned by time for scale.
- `usage_rollups (account_id, period_start, granularity, request_count, token_count, error_count, avg_latency_ms)`
- `marketplace_earnings (account_id, period, jobs_served, earnings_amount)`

## 9. Security requirements

- All public traffic terminates TLS at the gateway; internal relay-to-agent traffic is separately encrypted.
- API keys hashed at rest; never logged in plaintext.
- Mandatory gateway-level auth (Section 5) compensates for local runtimes' lack of native authentication.
- Abuse detection: anomaly detection on request rate/pattern per key, automatic temporary throttling.
- Content moderation hook available at the gateway for marketplace-shared capacity (PRD FR-29).
- Audit log table is append-only/immutable, separate from the mutable request-history table used for general analytics.

## 10. Non-functional requirements

- **Latency overhead**: relay + gateway hop should add no more than ~50–100ms round-trip beyond direct connection, achieved via regional relay placement.
- **Availability**: gateway and relay infrastructure targets 99.9% uptime independent of any individual user's machine uptime.
- **Scalability**: architecture must support horizontal scaling of gateway and relay nodes independently of the analytics pipeline's ingestion rate.
- **Data retention default**: request metadata retained 90 days minimum for analytics continuity; full content retention governed by user setting, default short/off.

## 11. Suggested tech stack summary

| Layer | Suggested technology |
|---|---|
| Desktop agent | Rust or Go core, Tauri UI shell |
| Model runtime | llama.cpp, vLLM, Ollama (pluggable) |
| Tunnel transport | QUIC (preferred) / HTTP2 WebSocket fallback |
| Gateway | Stateless service (Go/Node), Redis for connection routing state |
| Analytics store | ClickHouse or TimescaleDB (rollups), Postgres (hot request log) |
| Streaming ingestion | Kafka or managed equivalent |
| Dashboard | React/Next.js |
| Auth | Signed JWT/API keys, mutual TLS for agent-relay handshake |
