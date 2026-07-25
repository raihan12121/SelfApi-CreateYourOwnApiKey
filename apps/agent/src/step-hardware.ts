import { invoke } from "@tauri-apps/api/core";
import { escapeHtml } from "./dom-safe";
import type { HardwareProfile } from "./types";

export function formatRam(gb: number): string {
  return `${gb.toFixed(0)} GB`;
}

export function formatVram(gpu: HardwareProfile["primary_gpu"]): string {
  if (!gpu?.vram_gb) return "VRAM unknown";
  return gpu.vram_gb >= 10 ? `${gpu.vram_gb.toFixed(0)} GB` : `${gpu.vram_gb.toFixed(1)} GB`;
}

export function statusLabel(gpu: HardwareProfile["gpus"][number]): string {
  if (gpu.recommended_for_inference) return "Recommended";
  if (gpu.vram_gb && gpu.vram_gb > 0) return "Limited VRAM";
  return "Detected";
}

export function renderGpuCard(gpu: HardwareProfile["gpus"][number]): string {
  const statusClass = gpu.recommended_for_inference
    ? "badge badge-success"
    : gpu.vram_gb && gpu.vram_gb > 0
      ? "badge badge-warning"
      : "badge badge-muted";

  return `
    <article class="gpu-card">
      <div class="gpu-card-header">
        <div>
          <p class="gpu-vendor">${gpu.vendor.toUpperCase()}</p>
          <h3>${escapeHtml(gpu.name)}</h3>
        </div>
        <span class="${statusClass}">${statusLabel(gpu)}</span>
      </div>
      <dl class="gpu-meta">
        <div>
          <dt>VRAM</dt>
          <dd>${gpu.vram_gb ? formatVram(gpu) : "Unknown"}</dd>
        </div>
        <div>
          <dt>Driver</dt>
          <dd>${escapeHtml(gpu.driver_version ?? "Unknown")}</dd>
        </div>
        <div>
          <dt>CUDA</dt>
          <dd>${escapeHtml(gpu.cuda_version ?? "N/A")}</dd>
        </div>
      </dl>
    </article>
  `;
}

export function renderHardwareProfile(profile: HardwareProfile): void {
  const summaryEl = document.querySelector<HTMLElement>("#capability-summary");
  const metaEl = document.querySelector<HTMLElement>("#system-meta");
  const gpuListEl = document.querySelector<HTMLElement>("#gpu-list");
  const statusEl = document.querySelector<HTMLElement>("#scan-status");

  if (summaryEl) {
    summaryEl.textContent = profile.capability_summary;
  }

  if (metaEl) {
    metaEl.innerHTML = `
      <span>OS: ${escapeHtml(profile.os)}</span>
      <span>CPU: ${escapeHtml(profile.cpu_model ?? "Unknown")}</span>
      <span>RAM: ${formatRam(profile.total_ram_gb)}</span>
      <span>Scanned: ${new Date(profile.detected_at).toLocaleString()}</span>
    `;
  }

  if (gpuListEl) {
    gpuListEl.innerHTML =
      profile.gpus.length > 0
        ? profile.gpus.map(renderGpuCard).join("")
        : `<div class="empty-state">No GPUs detected. SelfAPI can still fall back to CPU inference for smaller models.</div>`;
  }

  if (statusEl) {
    statusEl.textContent = profile.primary_gpu?.recommended_for_inference
      ? "Hardware ready for model recommendations"
      : "Hardware detected — review recommendations below";
    statusEl.className = profile.primary_gpu?.recommended_for_inference
      ? "status-pill status-success"
      : "status-pill status-warning";
  }
}

export async function loadHardwareProfile(): Promise<HardwareProfile> {
  const loadingEl = document.querySelector<HTMLElement>("#hardware-loading");
  const contentEl = document.querySelector<HTMLElement>("#hardware-content");
  const errorEl = document.querySelector<HTMLElement>("#hardware-error");

  loadingEl?.classList.remove("hidden");
  contentEl?.classList.add("hidden");
  errorEl?.classList.add("hidden");

  try {
    const profile = await invoke<HardwareProfile>("get_hardware_profile");
    renderHardwareProfile(profile);
    loadingEl?.classList.add("hidden");
    contentEl?.classList.remove("hidden");
    return profile;
  } catch (error) {
    loadingEl?.classList.add("hidden");
    if (errorEl) {
      errorEl.textContent =
        error instanceof Error
          ? error.message
          : "Failed to scan hardware. Try again.";
      errorEl.classList.remove("hidden");
    }
    throw error;
  }
}
