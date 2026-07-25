# SelfAPI Desktop Agent

Rust + Tauri desktop agent for local GPU inference.

## Onboarding flow

1. **Hardware detection** — GPU, VRAM, CPU, and RAM scan
2. **Model library** — hardware-filtered catalog, pre-selected default, one-click GGUF download
3. **API key generation** — planned
4. **Dashboard handoff** — planned

## Development

```bash
cd apps/agent
npm install
npm run tauri dev
```

From the repo root:

```bash
npm run dev:agent
```

## Model library (step 2)

- Curated GGUF models from Hugging Face (Llama, Phi, Qwen, Mistral)
- Recommendations filtered by detected VRAM (or RAM fallback for CPU)
- Quantization auto-selected (`Q4_K_M` by default when VRAM allows)
- **Best match** pre-selected with conservative headroom
- Downloads stored in `%APPDATA%/SelfAPI/models` (Windows) or platform equivalent
- Progress events streamed to the UI during download

## Tauri commands

| Command | Description |
|---|---|
| `get_hardware_profile` | GPU and system scan |
| `get_model_library` | Filtered catalog with recommendations |
| `get_installed_models` | Locally downloaded models |
| `start_model_download` | Download selected model + quantization |
| `get_download_status` | Poll in-flight download progress |
