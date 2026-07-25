# Design description

## 1. Design principles

- **Invisible infrastructure**: the user should feel like they clicked one button and got an API — tunneling, NAT traversal, and gateway routing should never surface as concepts the user has to understand.
- **Trust through visibility**: since the product's core promise is "your data stays on your hardware," the dashboard must make usage, history, and data-retention settings extremely transparent — this is the product's main credibility lever.
- **Calm, technical, uncluttered**: the audience is developers and technical GPU owners. Favor dense, scannable data tables and clear charts over marketing-style visuals.
- **Status-first**: at every level (machine, model, API key), the user should be able to tell at a glance whether it's online, serving requests, or in fallback mode.

## 2. Information architecture

```
Dashboard (home)
├── Machines
│    └── [Machine detail: hardware, models installed, status]
├── Models
│    └── [Model detail: quantization, hot-swap, download management]
├── API keys
│    └── [Key detail: scope, rate limit, spend cap]
├── Request history & analytics   <-- core new feature
│    ├── History table (searchable/filterable log)
│    ├── Usage charts (requests, tokens, errors over time)
│    ├── Performance view (latency, throughput, uptime)
│    └── Export
├── Billing & marketplace earnings
├── Alerts & notifications
└── Settings
     ├── Data retention controls
     ├── Fallback configuration
     └── Team/multi-key permissions
```

## 3. Key screens

### 3.1 Onboarding flow
- Step 1: download/install agent, auto-detects GPU and shows a friendly hardware summary ("You have an RTX 4070, 12GB VRAM — here's what you can run").
- Step 2: recommended model pre-selected, one click to download.
- Step 3: API key generated automatically; a copy-paste code snippet shown immediately (curl + Python + JS variants) so the user can test before leaving the screen.
- Step 4: light-touch prompt to open the dashboard and see "your first request appear in real time."

### 3.2 Dashboard home
- Top row: total requests (all-time and current period), current uptime %, machine status indicator (online/offline/fallback-active).
- Middle: compact usage sparkline (last 7 days) and an alerts panel if anything needs attention (error spike, machine offline).
- Bottom: quick links into machines, models, and the request history/analytics section.

### 3.3 Request history & analytics (the feature just added)

This section has four sub-views, accessible via tabs so it doesn't overwhelm a single screen:

**a. History table**
- Columns: timestamp, model, input/output tokens, latency (TTFT + total), status (success/error/rate-limited), served-via (local/fallback), API key used.
- Filters: date range, model, status, API key, served-via.
- Search by request ID.
- Row expansion reveals a truncated prompt/response preview (only if content retention is enabled in settings) plus full metadata.
- Pagination with adjustable page size; sticky header for scanning long logs.

**b. Usage charts**
- Line/bar chart: requests per hour/day/week/month, with a toggle to overlay token volume.
- Stacked view option: local-served vs. fallback-served, so the user can see how often their own GPU actually handled the load vs. cloud fallback.
- Adjustable date range picker (last 24h, 7d, 30d, custom).

**c. Performance view**
- Average and p95 latency over time, per model.
- Throughput (tokens/sec) trend.
- Uptime percentage, visualized as a simple status timeline (green/red bar per hour) so outages are immediately visible.

**d. Export**
- Button to export the currently filtered view as CSV or JSON.
- Background job with a toast notification and download link when ready for large ranges.

### 3.4 Model management
- Card-based list of installed models, each showing quantization level, VRAM footprint, and a "make active" toggle for hot-swapping.
- A hardware-aware recommendation badge next to models that fit well vs. a caution badge for models that are too large for detected VRAM.

### 3.5 API key management
- Table of keys with scope, rate limit, spend cap, and a per-key mini-analytics sparkline (reusing the same components as the main analytics view, scoped to that key).
- Create/revoke flows are simple modal forms.

### 3.6 Machines (multi-machine pooling)
- List of connected machines with live status, hardware summary, and which models are active on each.
- A simple toggle to include/exclude a machine from load balancing.

### 3.7 Billing & marketplace earnings
- For paying users: current plan, usage-to-cost breakdown (reusing the analytics data, mapped to price).
- For marketplace participants: earnings history, jobs served, payout schedule — visually consistent with the request-history table so it feels like the same product, not a bolted-on feature.

### 3.8 Settings — data retention & privacy
- Clear radio/toggle control: "Store full prompt/response content" with retention window options (off / 24 hours / 30 days), directly tied to TRD Section 6's content storage policy.
- Fallback mode toggle and explanation of when cloud fallback is triggered.
- Alert thresholds configuration (error rate %, uptime drop, usage cap).

## 4. Visual style guidance

- **Layout**: dense, table-first for technical screens (history, keys, machines); card-based only for onboarding and model selection where recognition matters more than density.
- **Status color coding**: consistent green/amber/red used only for machine/request status — not decorative. Green = online/success, amber = fallback/degraded, red = offline/error.
- **Charts**: flat, minimal line/bar charts; avoid decorative gradients; prioritize legibility of trend over aesthetic flourish.
- **Typography**: monospace for request IDs, API keys, and log values; standard sans-serif for labels and navigation, to visually separate "data" from "chrome."
- **Empty states**: onboarding-style empty states (e.g., "No requests yet — here's a snippet to make your first call") rather than blank tables, to keep new users oriented.

## 5. Accessibility & trust cues

- Every screen showing usage or history includes a persistent, small indicator of the current data-retention setting, reinforcing the privacy promise at the point of use.
- Uptime/status indicators use both color and text/icon (not color alone) for colorblind accessibility.
- All monetary and usage figures are shown with clear units and time windows to avoid ambiguity (e.g., "1,204 requests — last 30 days" rather than a bare number).
