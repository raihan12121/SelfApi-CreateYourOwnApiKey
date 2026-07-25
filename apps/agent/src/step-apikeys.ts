import { invoke } from "@tauri-apps/api/core";
import type { ApiAccessResponse } from "./types";

let currentAccess: ApiAccessResponse | null = null;
let activeSnippetIndex = 0;

export async function loadApiKeys(modelId?: string | null): Promise<ApiAccessResponse> {
  const loadingEl = document.querySelector<HTMLElement>("#apikeys-loading");
  const contentEl = document.querySelector<HTMLElement>("#apikeys-content");
  const errorEl = document.querySelector<HTMLElement>("#apikeys-error");

  loadingEl?.classList.remove("hidden");
  contentEl?.classList.add("hidden");
  errorEl?.classList.add("hidden");

  const targetId = modelId || "llama-3.2-3b-instruct";

  try {
    const access = await invoke<ApiAccessResponse>("cmd_prepare_api_access", {
      model_id: targetId,
      modelId: targetId,
    });

    currentAccess = access;
    renderApiAccess(access);

    loadingEl?.classList.add("hidden");
    contentEl?.classList.remove("hidden");
    return access;
  } catch (error) {
    loadingEl?.classList.add("hidden");
    if (errorEl) {
      const msg = typeof error === "string" ? error : (error instanceof Error ? error.message : String(error));
      errorEl.textContent = msg || "Failed to generate API credentials.";
      errorEl.classList.remove("hidden");
    }
    throw error;
  }

}

function renderApiAccess(access: ApiAccessResponse): void {
  const secretInput = document.querySelector<HTMLInputElement>("#api-secret-input");
  const endpointInput = document.querySelector<HTMLInputElement>("#api-endpoint-input");
  const publicInput = document.querySelector<HTMLInputElement>("#api-public-input");
  const tabsContainer = document.querySelector<HTMLElement>("#snippet-tabs");

  if (secretInput) secretInput.value = access.secret_key;
  if (endpointInput) endpointInput.value = access.endpoint_url;
  if (publicInput && access.public_endpoint_url) publicInput.value = access.public_endpoint_url;

  if (tabsContainer && access.snippets.length > 0) {
    tabsContainer.innerHTML = access.snippets
      .map(
        (s, idx) => `
          <button type="button" class="tab-btn ${idx === activeSnippetIndex ? "tab-active" : ""}" data-snippet-idx="${idx}">
            ${s.label}
          </button>
        `,
      )
      .join("");
  }

  renderActiveSnippet();
}


function renderActiveSnippet(): void {
  const codeEl = document.querySelector<HTMLElement>("#snippet-code");
  if (!currentAccess || !codeEl) return;

  const snippet = currentAccess.snippets[activeSnippetIndex] ?? currentAccess.snippets[0];
  if (snippet) {
    codeEl.textContent = snippet.code;
  }
}

export function bindApiKeyEvents(onContinue: () => void, onBack: () => void): void {
  const secretInput = document.querySelector<HTMLInputElement>("#api-secret-input");
  const toggleBtn = document.querySelector<HTMLButtonElement>("#toggle-key-visibility");

  toggleBtn?.addEventListener("click", () => {
    if (!secretInput) return;
    const isPassword = secretInput.type === "password";
    secretInput.type = isPassword ? "text" : "password";
    toggleBtn.textContent = isPassword ? "Hide" : "Show";
  });

  document.querySelector("#copy-key-button")?.addEventListener("click", () => {
    if (currentAccess?.secret_key) {
      void navigator.clipboard.writeText(currentAccess.secret_key);
      flashButton("#copy-key-button", "Copied!");
    }
  });

  document.querySelector("#copy-endpoint-button")?.addEventListener("click", () => {
    if (currentAccess?.endpoint_url) {
      void navigator.clipboard.writeText(currentAccess.endpoint_url);
      flashButton("#copy-endpoint-button", "Copied!");
    }
  });

  document.querySelector("#copy-public-button")?.addEventListener("click", () => {
    const publicUrl = currentAccess?.public_endpoint_url ?? "https://gpu-node-9f82.selfapi.site/v1";
    void navigator.clipboard.writeText(publicUrl);
    flashButton("#copy-public-button", "Copied!");
  });


  document.querySelector("#snippet-tabs")?.addEventListener("click", (e) => {
    const target = (e.target as HTMLElement).closest<HTMLButtonElement>("[data-snippet-idx]");
    if (!target) return;

    activeSnippetIndex = Number(target.dataset.snippetIdx);
    document.querySelectorAll<HTMLElement>("#snippet-tabs .tab-btn").forEach((btn) => {
      btn.classList.toggle("tab-active", Number(btn.dataset.snippetIdx) === activeSnippetIndex);
    });
    renderActiveSnippet();
  });

  document.querySelector("#copy-snippet-button")?.addEventListener("click", () => {
    const codeEl = document.querySelector<HTMLElement>("#snippet-code");
    if (codeEl?.textContent) {
      void navigator.clipboard.writeText(codeEl.textContent);
      flashButton("#copy-snippet-button", "Copied!");
    }
  });

  document.querySelector("#apikeys-continue-button")?.addEventListener("click", onContinue);
  document.querySelector("#apikeys-skip-button")?.addEventListener("click", () => {
    window.dispatchEvent(new CustomEvent("selfapi:goto-step", { detail: 4 }));
  });
  document.querySelector("#apikeys-back-button")?.addEventListener("click", onBack);

}

function flashButton(selector: string, message: string): void {
  const btn = document.querySelector<HTMLButtonElement>(selector);
  if (!btn) return;
  const original = btn.textContent;
  btn.textContent = message;
  setTimeout(() => {
    btn.textContent = original;
  }, 1600);
}
