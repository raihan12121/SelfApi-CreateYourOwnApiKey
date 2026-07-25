import { loadHardwareProfile } from "./step-hardware";

import { bindModelStepEvents, getModelStepState, loadModelLibrary } from "./step-models";
import { bindApiKeyEvents, loadApiKeys } from "./step-apikeys";
import { bindPlaygroundEvents, loadPlayground } from "./step-playground";
import { initNativeDashboard, refreshDashboardData } from "./step-dashboard";

const TOTAL_STEPS = 4;

function setStep(step: number): void {
  document.querySelectorAll<HTMLElement>("[data-step-panel]").forEach((panel) => {
    panel.classList.toggle(
      "hidden",
      Number(panel.dataset.stepPanel) !== step,
    );
  });

  const label = document.querySelector<HTMLElement>("#step-label");
  if (label) {
    if (step === 5) {
      label.classList.add("hidden");
    } else {
      label.classList.remove("hidden");
      label.textContent = `Onboarding · Step ${step} of ${TOTAL_STEPS}`;
    }
  }
}

async function gotoStep(step: number): Promise<void> {
  setStep(step);

  if (step === 1) {
    await loadHardwareProfile();
  }

  if (step === 2) {
    await loadModelLibrary();
  }

  if (step === 3) {
    let state = getModelStepState();
    if (!state.library && !state.selectedModelId) {
      try {
        await loadModelLibrary();
        state = getModelStepState();
      } catch {
        // ignore offline fallback
      }
    }
    const modelId = state.selectedModelId ?? state.library?.default_model_id ?? "llama-3.2-3b-instruct";
    await loadApiKeys(modelId);
  }

  if (step === 4) {
    await loadPlayground();
  }

  if (step === 5) {
    await refreshDashboardData();
  }
}



window.addEventListener("DOMContentLoaded", () => {
  document
    .querySelector("#rescan-button")
    ?.addEventListener("click", () => void loadHardwareProfile());

  document
    .querySelector("#hardware-continue-button")
    ?.addEventListener("click", () => void gotoStep(2));

  bindModelStepEvents(() => {
    void gotoStep(3);
  });

  bindApiKeyEvents(
    () => void gotoStep(4),
    () => void gotoStep(2),
  );

  bindPlaygroundEvents(
    () => {
      void gotoStep(5);
    },
    () => void gotoStep(3),
  );

  document.querySelector("#dashboard-back-onboarding")?.addEventListener("click", () => void gotoStep(1));
  document.querySelector("#dashboard-reload-btn")?.addEventListener("click", () => void refreshDashboardData());
  initNativeDashboard();



  window.addEventListener("selfapi:goto-step", (event) => {
    const custom = event as CustomEvent<number>;
    void gotoStep(custom.detail);
  });

  void gotoStep(1);
});

export { gotoStep, setStep };

