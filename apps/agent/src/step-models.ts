import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  DownloadProgress,
  InstalledModel,
  ModelFit,
  ModelLibraryResponse,
  ModelRecommendation,
} from "./types";

export type ModelStepState = {
  library: ModelLibraryResponse | null;
  selectedModelId: string | null;
  selectedQuantizationId: string | null;
  download: DownloadProgress | null;
};

const state: ModelStepState = {
  library: null,
  selectedModelId: null,
  selectedQuantizationId: null,
  download: null,
};

let progressListenerReady = false;

function formatBytes(bytes: number): string {
  if (bytes >= 1_000_000_000) {
    return `${(bytes / 1_000_000_000).toFixed(1)} GB`;
  }
  return `${(bytes / 1_000_000).toFixed(0)} MB`;
}

function fitLabel(fit: ModelFit): string {
  switch (fit) {
    case "recommended":
      return "Recommended";
    case "caution":
      return "Caution";
    case "too_large":
      return "Too large";
  }
}

function fitClass(fit: ModelFit): string {
  switch (fit) {
    case "recommended":
      return "badge badge-success";
    case "caution":
      return "badge badge-warning";
    case "too_large":
      return "badge badge-muted";
  }
}

function selectedModel(): ModelRecommendation | null {
  if (!state.library || !state.selectedModelId) return null;
  return (
    state.library.models.find((model) => model.model.id === state.selectedModelId) ??
    null
  );
}

function renderQuantOptions(model: ModelRecommendation): string {
  const options = model.available_quantizations.length
    ? model.available_quantizations
    : model.model.quantizations;

  return options
    .map((quant) => {
      const checked =
        (state.selectedQuantizationId ?? model.recommended_quantization.id) ===
        quant.id;
      return `
        <label class="quant-option ${checked ? "quant-option-selected" : ""}">
          <input
            type="radio"
            name="quantization"
            value="${quant.id}"
            ${checked ? "checked" : ""}
          />
          <span>
            <strong>${quant.label}</strong>
            <small>${quant.min_vram_gb.toFixed(1)} GB VRAM · ${formatBytes(quant.file_size_bytes)}</small>
          </span>
        </label>
      `;
    })
    .join("");
}

function renderModelCard(model: ModelRecommendation): string {
  const selected = state.selectedModelId === model.model.id;
  const disabled = model.fit === "too_large";

  return `
    <div
      class="model-card ${selected ? "model-card-selected" : ""} ${disabled ? "model-card-disabled" : ""}"
      data-model-id="${model.model.id}"
    >
      <div class="model-card-header">
        <div>
          <p class="model-family">${model.model.family}</p>
          <h3>${model.model.name}</h3>
        </div>
        <div class="model-card-badges">
          ${model.is_default ? '<span class="badge badge-accent">Best match</span>' : ""}
          <span class="${fitClass(model.fit)}">${fitLabel(model.fit)}</span>
          ${model.installed ? '<span class="badge badge-success">Installed</span>' : ""}
        </div>
      </div>
      <p class="model-description">${model.model.description}</p>
      <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 10px;">
        <p class="model-meta">${model.model.parameter_count_b.toFixed(1)}B params · ${model.recommended_quantization.label}</p>
        ${
          model.installed
            ? `<button type="button" class="button-small hotswap-btn" data-hotswap-id="${model.model.id}">Make Active (Hot Swap)</button>`
            : ""
        }
      </div>
    </div>
  `;
}


function updateDownloadUi(): void {
  const progressEl = document.querySelector<HTMLElement>("#download-progress");
  const errorEl = document.querySelector<HTMLElement>("#download-error");
  const downloadButton = document.querySelector<HTMLButtonElement>("#download-button");
  const continueButton = document.querySelector<HTMLButtonElement>("#models-continue-button");

  if (!progressEl || !downloadButton || !continueButton) return;

  const model = selectedModel();
  const download = state.download;

  if (!model) {
    progressEl.classList.add("hidden");
    downloadButton.disabled = true;
    continueButton.disabled = true;
    return;
  }

  downloadButton.disabled =
    model.fit === "too_large" ||
    download?.status === "downloading" ||
    model.installed ||
    download?.status === "completed";

  if (model.installed || download?.status === "completed") {
    progressEl.classList.remove("hidden");
    progressEl.innerHTML = `
      <div class="progress-copy">Model ready at <code>${download?.file_path ?? "local storage"}</code></div>
      <div class="progress-bar"><div class="progress-bar-fill" style="width: 100%"></div></div>
    `;
    continueButton.disabled = false;
    if (errorEl) errorEl.classList.add("hidden");
    return;
  }

  if (download?.status === "downloading") {
    progressEl.classList.remove("hidden");
    const percent = download.progress_percent ?? 0;
    progressEl.innerHTML = `
      <div class="progress-copy">Downloading ${model.model.name} · ${formatBytes(download.bytes_downloaded)}${
        download.total_bytes ? ` / ${formatBytes(download.total_bytes)}` : ""
      }</div>
      <div class="progress-bar"><div class="progress-bar-fill" style="width: ${percent}%"></div></div>
    `;
    continueButton.disabled = true;
    return;
  }

  if (download?.status === "failed") {
    progressEl.classList.add("hidden");
    if (errorEl) {
      errorEl.textContent = download.error ?? "Download failed.";
      errorEl.classList.remove("hidden");
    }
    continueButton.disabled = true;
    return;
  }

  progressEl.classList.add("hidden");
  continueButton.disabled = true;
  if (errorEl) errorEl.classList.add("hidden");
}

function renderModelLibrary(): void {
  const summaryEl = document.querySelector<HTMLElement>("#model-library-summary");
  const listEl = document.querySelector<HTMLElement>("#model-list");
  const quantEl = document.querySelector<HTMLElement>("#quantization-options");

  if (!state.library || !summaryEl || !listEl || !quantEl) return;

  summaryEl.textContent = `Based on ${state.library.memory_source} (${state.library.available_vram_gb.toFixed(1)} GB available), SelfAPI pre-selected a model that fits your hardware.`;

  listEl.innerHTML = state.library.models.map(renderModelCard).join("");

  const model = selectedModel();
  quantEl.innerHTML = model
    ? renderQuantOptions(model)
    : `<div class="empty-state">Select a model to choose quantization.</div>`;

  updateDownloadUi();
}

async function ensureProgressListener(): Promise<void> {
  if (progressListenerReady) return;

  await listen<DownloadProgress>("download-progress", (event) => {
    if (event.payload.model_id !== state.selectedModelId) return;
    state.download = event.payload;
    renderModelLibrary();
  });

  progressListenerReady = true;
}

export async function loadModelLibrary(): Promise<ModelLibraryResponse> {
  const loadingEl = document.querySelector<HTMLElement>("#models-loading");
  const contentEl = document.querySelector<HTMLElement>("#models-content");
  const errorEl = document.querySelector<HTMLElement>("#models-error");

  loadingEl?.classList.remove("hidden");
  contentEl?.classList.add("hidden");
  errorEl?.classList.add("hidden");

  await ensureProgressListener();

  try {
    const [library, installed] = await Promise.all([
      invoke<ModelLibraryResponse>("get_model_library"),
      invoke<InstalledModel[]>("get_installed_models"),
    ]);

    const installedById = new Map(
      installed.map((entry) => [entry.model_id, entry]),
    );
    library.models = library.models.map((entry) => ({
      ...entry,
      installed: installedById.has(entry.model.id),
    }));

    state.library = library;
    state.selectedModelId = library.default_model_id;
    const selected = selectedModel();
    state.selectedQuantizationId =
      selected?.recommended_quantization.id ?? null;

    const installedRecord =
      state.selectedModelId != null
        ? installedById.get(state.selectedModelId)
        : undefined;

    if (installedRecord) {
      state.download = {
        model_id: installedRecord.model_id,
        quantization_id: installedRecord.quantization_id,
        status: "completed",
        bytes_downloaded: installedRecord.file_size_bytes,
        total_bytes: installedRecord.file_size_bytes,
        progress_percent: 100,
        file_path: installedRecord.file_path,
        error: null,
      };
    } else {
      state.download = null;
    }

    renderModelLibrary();
    loadingEl?.classList.add("hidden");
    contentEl?.classList.remove("hidden");
    return library;
  } catch (error) {
    loadingEl?.classList.add("hidden");
    if (errorEl) {
      errorEl.textContent =
        error instanceof Error
          ? error.message
          : "Failed to load model library.";
      errorEl.classList.remove("hidden");
    }
    throw error;
  }
}

export function bindModelStepEvents(onContinue: () => void): void {
  document.querySelector("#model-list")?.addEventListener("click", async (event) => {
    const hotswapBtn = (event.target as HTMLElement).closest<HTMLButtonElement>(".hotswap-btn");
    if (hotswapBtn) {
      const hotswapId = hotswapBtn.dataset.hotswapId;
      if (hotswapId) {
        hotswapBtn.disabled = true;
        hotswapBtn.textContent = "Swapping...";
        try {
          await invoke("cmd_hot_swap_model", { model_id: hotswapId });
          hotswapBtn.textContent = "Active in VRAM ✓";
          setTimeout(() => renderModelLibrary(), 1500);
        } catch (err) {
          window.alert(err instanceof Error ? err.message : "Hot swap failed.");
          renderModelLibrary();
        }
      }
      return;
    }

    const target = (event.target as HTMLElement).closest<HTMLElement>(
      "[data-model-id]",
    );
    if (!target) return;

    state.selectedModelId = target.dataset.modelId ?? null;
    const model = selectedModel();
    state.selectedQuantizationId =
      model?.recommended_quantization.id ?? null;
    state.download = null;
    renderModelLibrary();
  });


  document
    .querySelector("#quantization-options")
    ?.addEventListener("change", (event) => {
      const target = event.target as HTMLInputElement;
      if (target.name !== "quantization") return;
      state.selectedQuantizationId = target.value;
      state.download = null;
      renderModelLibrary();
    });

  document
    .querySelector("#download-button")
    ?.addEventListener("click", () => void startSelectedDownload());

  document
    .querySelector("#models-back-button")
    ?.addEventListener("click", () => {
      window.dispatchEvent(new CustomEvent("selfapi:goto-step", { detail: 1 }));
    });

  document
    .querySelector("#models-skip-button")
    ?.addEventListener("click", () => {
      window.dispatchEvent(new CustomEvent("selfapi:goto-step", { detail: 3 }));
    });


  document.querySelector("#scan-system-models-btn")?.addEventListener("click", async () => {
    const scanBtn = document.querySelector<HTMLButtonElement>("#scan-system-models-btn");
    const container = document.querySelector<HTMLElement>("#scanned-models-container");
    const listEl = document.querySelector<HTMLElement>("#scanned-models-list");

    if (scanBtn) {
      scanBtn.disabled = true;
      scanBtn.textContent = "Scanning Ollama & System...";
    }

    try {
      const result = await invoke<{
        scanned_models: Array<{ id: string; name: string; source: string; file_path: string; size_gb: number }>;
        ollama_available: boolean;
      }>("cmd_scan_system_models");

      if (scanBtn) {
        scanBtn.disabled = false;
        scanBtn.textContent = result.ollama_available
          ? "✓ Ollama Connected"
          : "🔍 Re-scan System Models";
      }

      if (container && listEl && result.scanned_models.length > 0) {
        container.classList.remove("hidden");
        listEl.innerHTML = result.scanned_models
          .map(
            (m) => `
            <div class="model-card model-card-selected" style="border-color: #3b82f6;" data-model-id="${m.id}">
              <div class="model-card-header">
                <div>
                  <p class="model-family" style="color: #60a5fa;">${m.source}</p>
                  <h3>${m.name}</h3>
                </div>
                <span class="badge badge-success">Installed & Ready</span>
              </div>
              <p class="model-description">${m.size_gb.toFixed(1)} GB · Instant API access via SelfAPI</p>
            </div>
          `,
          )
          .join("");

        // Auto-select first scanned model so user can proceed immediately
        const first = result.scanned_models[0];
        state.selectedModelId = first.id;
        state.download = {
          model_id: first.id,
          quantization_id: "Q4_K_M",
          status: "completed",
          bytes_downloaded: Math.round(first.size_gb * 1024 * 1024 * 1024),
          total_bytes: Math.round(first.size_gb * 1024 * 1024 * 1024),
          progress_percent: 100,
          file_path: first.file_path,
          error: null,
        };

        const continueButton = document.querySelector<HTMLButtonElement>("#models-continue-button");
        if (continueButton) continueButton.disabled = false;
      } else if (container) {
        container.classList.remove("hidden");
        container.innerHTML = `<p style="margin:0; color:#a1a1aa; font-size:0.85rem;">No pre-installed Ollama or GGUF models found in default system paths.</p>`;
      }
    } catch (err) {
      if (scanBtn) {
        scanBtn.disabled = false;
        scanBtn.textContent = "Scan Failed - Retry";
      }
    }
  });

  document.querySelector("#import-custom-gguf-btn")?.addEventListener("click", async () => {
    const importBtn = document.querySelector<HTMLButtonElement>("#import-custom-gguf-btn");
    const customPath = window.prompt("Enter absolute path to local .gguf file:");
    if (!customPath) return;

    if (importBtn) {
      importBtn.disabled = true;
      importBtn.textContent = "Registering GGUF...";
    }

    try {
      const installed = await invoke<InstalledModel>("cmd_add_custom_gguf_file", { path: customPath });
      state.selectedModelId = installed.model_id;
      state.download = {
        model_id: installed.model_id,
        quantization_id: installed.quantization_id,
        status: "completed",
        bytes_downloaded: installed.file_size_bytes,
        total_bytes: installed.file_size_bytes,
        progress_percent: 100,
        file_path: installed.file_path,
        error: null,
      };

      if (importBtn) {
        importBtn.disabled = false;
        importBtn.textContent = "✓ Custom GGUF Added";
      }

      await loadModelLibrary();
      const continueButton = document.querySelector<HTMLButtonElement>("#models-continue-button");
      if (continueButton) continueButton.disabled = false;
    } catch (err) {
      if (importBtn) {
        importBtn.disabled = false;
        importBtn.textContent = "📁 Import Custom GGUF File";
      }
      window.alert(err instanceof Error ? err.message : String(err));
    }
  });

  document
    .querySelector("#models-continue-button")
    ?.addEventListener("click", onContinue);

}

async function startSelectedDownload(): Promise<void> {
  const model = selectedModel();
  if (!model || !state.selectedQuantizationId) return;

  const errorEl = document.querySelector<HTMLElement>("#download-error");
  errorEl?.classList.add("hidden");

  try {
    const progress = await invoke<DownloadProgress>("start_model_download", {
      model_id: model.model.id,
      modelId: model.model.id,
      quantization_id: state.selectedQuantizationId,
      quantizationId: state.selectedQuantizationId,
    });

    state.download = progress;
    if (progress.status === "completed") {
      model.installed = true;
      state.download = progress;
    }
    renderModelLibrary();
  } catch (error) {
    if (errorEl) {
      errorEl.textContent =
        error instanceof Error ? error.message : "Download failed.";
      errorEl.classList.remove("hidden");
    }
  }
}

export function getModelStepState(): ModelStepState {
  return state;
}
