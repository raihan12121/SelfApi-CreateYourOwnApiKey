"use client";

import { useEffect, useState } from "react";
import { PageHeader } from "@/components/layout/page-header";
import { checkAgentHealth, getApiEndpoint } from "@/lib/store";

type ModelItem = {
  id: string;
  name: string;
  family: string;
  params: string;
  quantization: string;
  vramRequired: string;
  status: "active" | "installed" | "available";
};

const initialModels: ModelItem[] = [
  {
    id: "llama-3.2-3b-instruct",
    name: "Llama 3.2 3B Instruct",
    family: "Meta Llama",
    params: "3.2B",
    quantization: "Q4_K_M",
    vramRequired: "3.2 GB",
    status: "installed",
  },
  {
    id: "qwen2.5-7b-instruct",
    name: "Qwen 2.5 7B Instruct",
    family: "Alibaba Qwen",
    params: "7.6B",
    quantization: "Q4_K_M",
    vramRequired: "5.8 GB",
    status: "installed",
  },
  {
    id: "mistral-7b-instruct-v0.3",
    name: "Mistral 7B Instruct v0.3",
    family: "Mistral AI",
    params: "7.2B",
    quantization: "Q4_K_M",
    vramRequired: "5.5 GB",
    status: "installed",
  },
  {
    id: "deepseek-r1-distill-qwen-14b",
    name: "DeepSeek R1 Distill Qwen 14B",
    family: "DeepSeek AI",
    params: "14.8B",
    quantization: "Q4_K_M",
    vramRequired: "10.2 GB",
    status: "available",
  },
];

export default function ModelsPage() {
  const [models, setModels] = useState<ModelItem[]>(initialModels);
  const [swappingId, setSwappingId] = useState<string | null>(null);

  useEffect(() => {
    async function check() {
      const health = await checkAgentHealth();
      if (health.agentData?.active_model) {
        const activeName = health.agentData.active_model;
        setModels((prev) => {
          const exists = prev.some((m) => m.id === activeName || m.name === activeName);
          if (!exists) {
            return [
              {
                id: activeName,
                name: activeName.includes("ollama") ? `Ollama (${activeName.split('/').pop()})` : activeName,
                family: "Active Local Model",
                params: "12B",
                quantization: "Q4_K_M",
                vramRequired: "4.8 GB",
                status: "active",
              },
              ...prev.map((m) => ({ ...m, status: m.status === "active" ? ("installed" as const) : m.status })),
            ];
          }
          return prev.map((m) => ({
            ...m,
            status: m.id === activeName ? ("active" as const) : m.status === "active" ? ("installed" as const) : m.status,
          }));
        });
      }
    }
    void check();
  }, []);


  const handleHotSwap = async (targetId: string) => {
    setSwappingId(targetId);
    try {
      const res = await fetch(`${getApiEndpoint()}/v1/models/swap`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ model_id: targetId }),
      });
      if (res.ok) {
        setModels((prev) =>
          prev.map((m) => ({
            ...m,
            status: m.id === targetId ? "active" : m.status === "active" ? "installed" : m.status,
          })),
        );
      }
    } catch {
      // server offline
    } finally {
      setSwappingId(null);
    }
  };

  return (
    <>
      <PageHeader
        title="Models"
        description="Installed models, quantization levels, VRAM footprint, and hot-swap controls."
      />

      <div className="grid gap-4 md:grid-cols-2">
        {models.map((model) => (
          <div
            key={model.id}
            className={`rounded-lg border p-5 transition-all ${
              model.status === "active"
                ? "border-blue-500/80 bg-blue-950/20 dark:border-blue-500/60"
                : "border-zinc-200 bg-white dark:border-zinc-800 dark:bg-zinc-950"
            }`}
          >
            <div className="flex items-start justify-between gap-3">
              <div>
                <span className="text-[11px] font-mono uppercase tracking-wider text-zinc-500">
                  {model.family}
                </span>
                <h3 className="text-base font-semibold text-zinc-900 dark:text-zinc-100">
                  {model.name}
                </h3>
              </div>
              <div>
                {model.status === "active" ? (
                  <span className="inline-flex rounded-full bg-emerald-500/15 px-2.5 py-1 text-xs font-semibold text-emerald-500">
                    Active in VRAM
                  </span>
                ) : model.status === "installed" ? (
                  <span className="inline-flex rounded-full bg-zinc-500/15 px-2.5 py-1 text-xs font-semibold text-zinc-400">
                    Installed
                  </span>
                ) : (
                  <span className="inline-flex rounded-full bg-blue-500/15 px-2.5 py-1 text-xs font-semibold text-blue-400">
                    Catalog
                  </span>
                )}
              </div>
            </div>

            <div className="mt-4 grid grid-cols-3 gap-2 text-xs">
              <div className="rounded border border-zinc-200 bg-zinc-50 p-2 dark:border-zinc-800 dark:bg-zinc-900">
                <span className="text-zinc-500 block">Parameters</span>
                <span className="font-mono font-medium">{model.params}</span>
              </div>
              <div className="rounded border border-zinc-200 bg-zinc-50 p-2 dark:border-zinc-800 dark:bg-zinc-900">
                <span className="text-zinc-500 block">Quantization</span>
                <span className="font-mono font-medium">{model.quantization}</span>
              </div>
              <div className="rounded border border-zinc-200 bg-zinc-50 p-2 dark:border-zinc-800 dark:bg-zinc-900">
                <span className="text-zinc-500 block">VRAM Required</span>
                <span className="font-mono font-medium">{model.vramRequired}</span>
              </div>
            </div>

            <div className="mt-4 flex justify-end">
              {model.status === "active" ? (
                <button
                  type="button"
                  disabled
                  className="rounded-md border border-emerald-500/30 bg-emerald-500/10 px-3 py-1.5 text-xs font-medium text-emerald-400 opacity-80"
                >
                  Loaded & Serving
                </button>
              ) : model.status === "installed" ? (
                <button
                  type="button"
                  onClick={() => handleHotSwap(model.id)}
                  disabled={swappingId !== null}
                  className="rounded-md border border-zinc-200 bg-zinc-900 px-3 py-1.5 text-xs font-medium text-white hover:bg-zinc-800 dark:border-zinc-700 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-zinc-200"
                >
                  {swappingId === model.id ? "Hot Swapping VRAM..." : "Make Active (Hot Swap)"}
                </button>
              ) : (
                <button
                  type="button"
                  disabled
                  className="rounded-md border border-zinc-200 px-3 py-1.5 text-xs font-medium text-zinc-400 dark:border-zinc-800"
                >
                  Download Model First
                </button>
              )}
            </div>
          </div>
        ))}
      </div>
    </>
  );
}

