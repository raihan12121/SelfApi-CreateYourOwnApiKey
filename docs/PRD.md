# Product Requirements Document (PRD)

## Product name
SelfAPI — "Turn your GPU into your own AI API"

## 1. Summary

SelfAPI is a desktop application that turns a user's own PC and GPU into a private, hosted AI inference API. The user installs an agent, downloads a model sized to their hardware, and instantly receives an OpenAI-compatible API key and endpoint. Inference runs entirely on their local machine; SelfAPI provides the tunneling, authentication, billing, monitoring, and dashboard layer so the user never touches AWS, never provisions a server, and never hires an engineer to deploy a model.

## 2. Problem statement

Running open-source AI models locally (Ollama, LM Studio, llama.cpp) is now easy. Exposing that local model as a stable, secure, production-usable API is not — it currently requires stitching together a tunneling tool (ngrok/Cloudflare Tunnel), manually configuring authentication (which most local LLM tools lack by default), and accepting that the "API" dies the moment the laptop sleeps or the tunnel session expires. There is no polished, single product that takes a user from "I have a GPU" to "I have a stable API key" in one flow.

Separately, cloud inference is not always the right answer: it costs money per token, sends data off-premises (a blocker for privacy-sensitive industries), and requires infrastructure knowledge to deploy custom or fine-tuned models.

## 3. Target users

- **Indie developers / hobbyists** who want a free or near-free API for side projects and prototypes.
- **Privacy-conscious teams** (healthcare, legal, finance, internal tools) who need inference to never leave their own hardware.
- **Small businesses / startups** who want to avoid cloud GPU bills for moderate-traffic AI features.
- **GPU owners with idle hardware** (gamers, miners, workstation owners) who want to monetize spare capacity.
- **AI hobbyist communities** running fine-tunes/LoRAs who want an easy way to serve their custom models.

## 4. Goals and success metrics

| Goal | Metric |
|---|---|
| Fast time-to-first-API-call | Under 10 minutes from install to first successful API request |
| Reliability | 99%+ tunnel uptime while the host machine is online |
| Adoption | Number of active API keys generated, weekly active agents |
| Engagement with monitoring | % of users who view the dashboard/analytics at least weekly |
| Monetization | Conversion rate from free to paid tier; marketplace transaction volume |

## 5. Non-goals (v1)

- SelfAPI is not a training platform (no fine-tuning compute provided in-app in v1).
- SelfAPI does not guarantee frontier-model-level quality — it is bounded by consumer GPU capability (up to ~30B parameter models at usable quantization).
- SelfAPI is not responsible for the user's electricity costs or hardware wear.

## 6. Feature requirements

### 6.1 Core (MVP)

- **FR-1 — Desktop agent**: installable app for Windows/macOS/Linux that auto-detects GPU model, VRAM, and available drivers.
- **FR-2 — Model library**: curated list of open-weight models, filtered to what the user's hardware can realistically run; one-click download and quantization selection.
- **FR-3 — Instant API key generation**: a single action produces an OpenAI-compatible endpoint URL and secret key.
- **FR-4 — Persistent tunnel**: the connection auto-reconnects after sleep, network drop, or restart without the API key/URL changing.
- **FR-5 — Gateway-enforced authentication**: every request is authenticated at SelfAPI's cloud gateway before it ever reaches the user's machine; the local model itself never needs to implement auth.
- **FR-6 — Start / stop / pause controls**: the user can instantly halt serving to reclaim their GPU, with requests queuing or falling back per their configuration.

### 6.2 Request analytics & history dashboard (new — required)

This is a first-class dashboard section, not an afterthought.

- **FR-7 — Total request counter**: prominent running total of all-time and current-billing-period requests, visible at a glance on the dashboard home.
- **FR-8 — Full request history log**: a searchable, filterable, paginated table of every request, including:
  - Timestamp
  - Model used
  - Input token count / output token count
  - Latency (time-to-first-token and total duration)
  - HTTP status (success / error / rate-limited)
  - Source (which API key / which app, if labeled)
  - Truncated prompt/response preview (with an option to disable storing content entirely, for privacy)
- **FR-9 — Time-series usage charts**: requests per hour/day/week/month, tokens processed over time, and error rate over time, with adjustable date ranges.
- **FR-10 — Performance analytics**: average and p95 latency, throughput (tokens/sec), uptime percentage, broken down by model.
- **FR-11 — Cost/usage breakdown**: for paid tiers, a view of how usage maps to cost (or, for marketplace hosts, how usage maps to earnings).
- **FR-12 — Exportable data**: CSV/JSON export of request history and aggregated analytics for the user's own records or billing reconciliation.
- **FR-13 — Data retention controls**: user-configurable retention window for storing full request/response content (e.g., 24 hours, 30 days, or metadata-only/never), to balance debuggability against privacy.
- **FR-14 — Alerting**: optional notifications (email/webhook) when error rate spikes, GPU goes offline, or usage crosses a threshold.

### 6.3 Differentiating features

- **FR-15 — Cloud fallback mode**: when the local machine is offline, requests automatically reroute to a low-cost cloud-hosted model so the developer's downstream app never sees downtime; clearly flagged in the request history which requests were served locally vs. via fallback.
- **FR-16 — Multi-model hot-swap**: switch the active model behind an existing API key without regenerating the key or breaking client integrations.
- **FR-17 — Hardware-aware quantization**: automatic recommendation of quantization level based on detected VRAM.
- **FR-18 — Multi-machine pooling**: link more than one machine (e.g., a gaming PC and a home server) under one account, with load balancing and combined analytics across both.
- **FR-19 — Framework SDKs**: drop-in packages/snippets for LangChain, LlamaIndex, Vercel AI SDK, and raw OpenAI-client compatibility.

### 6.4 Premium tier features

- **FR-20 — Team & multi-key management**: multiple API keys per account, each with its own spend cap, rate limit, and permission scope; per-key analytics (an extension of FR-8/FR-9 filtered by key).
- **FR-21 — Priority routing / higher rate limits**.
- **FR-22 — Custom domains** for the API endpoint.
- **FR-23 — SLA-backed uptime** (requires cloud fallback, FR-15).
- **FR-24 — Fine-tune/LoRA hosting**: upload a custom fine-tune and serve it through the same pipeline.

### 6.5 Marketplace layer (phase 2)

- **FR-25 — Idle capacity sharing**: opt spare GPU cycles into a shared pool for other users' inference jobs, earning credits or cash.
- **FR-26 — Host reliability scoring**: uptime- and latency-based reputation shown to renters.
- **FR-27 — Marketplace transaction analytics**: earnings history, jobs served, and payout tracking — same analytics infrastructure as FR-8/FR-9, reused for the earner's perspective.

### 6.6 Trust, safety, and compliance

- **FR-28 — Abuse detection**: automatic rate-limiting and anomaly detection to prevent a host's machine from being overwhelmed.
- **FR-29 — Content moderation hooks** at the gateway for marketplace-shared capacity.
- **FR-30 — Audit logs** for enterprise/compliance customers, distinct from general request history (immutable, exportable, tamper-evident).
- **FR-31 — Data residency guarantee** messaging and enforcement: no prompt/response content leaves the user's own hardware unless fallback mode is explicitly enabled.

## 7. Key user flows

1. **Onboarding**: Install agent → hardware auto-detected → recommended model downloaded → API key generated → copy-paste snippet shown for immediate testing.
2. **Daily monitoring**: User opens dashboard → sees total requests, uptime, and any alerts → drills into history log to debug a specific failed request.
3. **Scaling up**: User adds a second machine → pools capacity → sets a custom domain and higher rate limit on a paid tier.
4. **Monetizing idle time**: User opts into the marketplace → sets availability hours → tracks earnings via the same analytics dashboard.

## 8. Assumptions and risks

- Assumes a meaningful share of users have a GPU with at least 8–12GB VRAM.
- Reliability of consumer internet/hardware is a real risk to uptime promises; cloud fallback mitigates but adds cost.
- Local LLM tools' lack of native authentication means all security must be enforced at the gateway — a single point of failure that must be hardened.
- Competitive pressure from very cheap managed inference APIs means the strongest differentiators are privacy/data residency and zero recurring cloud cost, not raw price-per-token.

## 9. Release phasing

- **v1 (MVP)**: FR-1 through FR-14 (core + analytics dashboard).
- **v2**: FR-15 through FR-19 (differentiators) + FR-20–24 (premium tiers).
- **v3**: FR-25–27 (marketplace) + FR-28–31 (trust/safety/compliance hardening for enterprise).
