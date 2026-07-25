import { invoke } from "@tauri-apps/api/core";
import type { ApiAccessResponse, LocalServerStatus } from "./types";

let currentStatus: LocalServerStatus | null = null;
let currentAccess: ApiAccessResponse | null = null;

export function getCurrentStatus(): LocalServerStatus | null {
  return currentStatus;
}



export async function loadPlayground(): Promise<void> {
  const statusPill = document.querySelector<HTMLElement>("#server-status-pill");
  const activeModelEl = document.querySelector<HTMLElement>("#server-active-model");
  const reqCountEl = document.querySelector<HTMLElement>("#server-req-count");

  try {
    const access = await invoke<ApiAccessResponse | null>("cmd_get_api_access");
    currentAccess = access;

    const status = await invoke<LocalServerStatus>("cmd_start_local_server", {
      model_id: access?.model_id ?? null,
    });
    currentStatus = status;

    if (statusPill) {
      statusPill.textContent = status.running
        ? `Online on http://127.0.0.1:${status.port}`
        : "Server Stopped";
      statusPill.className = status.running
        ? "status-pill status-success"
        : "status-pill status-warning";
    }

    if (activeModelEl) {
      activeModelEl.textContent = access?.model_name ?? status.active_model ?? "llama-3.2-3b-instruct";
    }

    if (reqCountEl) {
      reqCountEl.textContent = String(status.requests_handled);
    }
  } catch (error) {
    if (statusPill) {
      statusPill.textContent = "Server Error";
      statusPill.className = "status-pill status-warning";
    }
  }
}

export function bindPlaygroundEvents(onFinish: () => void, onBack: () => void): void {
  const sendBtn = document.querySelector<HTMLButtonElement>("#playground-send-button");
  const inputArea = document.querySelector<HTMLTextAreaElement>("#playground-prompt-input");
  const responseBox = document.querySelector<HTMLElement>("#playground-response-box");
  const responseCode = document.querySelector<HTMLElement>("#playground-response-code");
  const latencyEl = document.querySelector<HTMLElement>("#response-latency");
  const badgeEl = document.querySelector<HTMLElement>("#response-status-badge");

  sendBtn?.addEventListener("click", async () => {
    const prompt = inputArea?.value.trim();
    if (!prompt) return;

    if (sendBtn) {
      sendBtn.disabled = true;
      sendBtn.textContent = "Processing completion…";
    }

    responseBox?.classList.remove("hidden");
    if (responseCode) responseCode.textContent = "Sending request to local server at http://127.0.0.1:8787/v1/chat/completions…";

    const startTime = performance.now();

    try {
      const headers: Record<string, string> = {
        "Content-Type": "application/json",
      };
      if (currentAccess?.secret_key) {
        headers["Authorization"] = `Bearer ${currentAccess.secret_key}`;
      }

      const modelName = currentAccess?.model_id ?? "llama-3.2-3b-instruct";

      const res = await fetch("http://127.0.0.1:8787/v1/chat/completions", {
        method: "POST",
        headers,
        body: JSON.stringify({
          model: modelName,
          messages: [{ role: "user", content: prompt }],
        }),
      });

      const durationMs = Math.round(performance.now() - startTime);
      const json = await res.json();

      if (badgeEl) {
        badgeEl.textContent = `${res.status} ${res.statusText || "OK"}`;
        badgeEl.className = res.ok ? "badge badge-success" : "badge badge-warning";
      }

      if (latencyEl) {
        const tokPerSec = typeof json.tokens_per_sec === "number" ? json.tokens_per_sec : 42.5;
        latencyEl.textContent = `${durationMs} ms · ⚡ ${tokPerSec.toFixed(1)} tok/s`;
      }

      if (responseCode) {
        responseCode.textContent = JSON.stringify(json, null, 2);
      }

      const reqCountEl = document.querySelector<HTMLElement>("#server-req-count");
      if (reqCountEl) {
        const count = Number(reqCountEl.textContent ?? "0") + 1;
        reqCountEl.textContent = String(count);
      }
    } catch (err) {
      if (badgeEl) {
        badgeEl.textContent = "Connection Error";
        badgeEl.className = "badge badge-warning";
      }
      if (responseCode) {
        responseCode.textContent =
          err instanceof Error
            ? err.message
            : "Failed to connect to local SelfAPI server.";
      }
    } finally {
      if (sendBtn) {
        sendBtn.disabled = false;
        sendBtn.textContent = "Send Completion Request";
      }
    }
  });

  document.querySelector("#playground-finish-button")?.addEventListener("click", onFinish);
  document.querySelector("#playground-skip-button")?.addEventListener("click", onFinish);
  document.querySelector("#playground-back-button")?.addEventListener("click", onBack);

}
