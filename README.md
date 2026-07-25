# SelfAPI

Turn your GPU into your own AI API.

SelfAPI is a desktop application and control-plane dashboard that turns your PC and GPU into a private, hosted AI inference API. Install the agent, download a model sized to your hardware, and receive an OpenAI-compatible API key and endpoint — with tunneling, authentication, monitoring, and analytics handled for you.

## Project structure

```
SelfAPI/
├── apps/
│   ├── dashboard/     # Next.js control-plane web app
│   └── agent/         # Tauri desktop agent (GPU detection)
├── docs/
│   ├── PRD.md
│   ├── TRD.md
│   └── design-description.md
└── packages/          # Shared packages (future)
```

## Getting started

```bash
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) for the dashboard.

### Desktop agent

```bash
npm run dev:agent
```

This launches the Tauri agent with GPU detection onboarding.

## Documentation

- [Product Requirements (PRD)](docs/PRD.md)
- [Technical Requirements (TRD)](docs/TRD.md)
- [Design Description](docs/design-description.md)

## Release phasing

- **v1 (MVP)**: Desktop agent, model library, API keys, persistent tunnel, gateway auth, analytics dashboard
- **v2**: Cloud fallback, multi-model hot-swap, premium tiers
- **v3**: Marketplace, enterprise compliance
