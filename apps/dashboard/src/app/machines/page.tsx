"use client";

import { useEffect, useState } from "react";
import { PageHeader } from "@/components/layout/page-header";
import { checkAgentHealth } from "@/lib/store";

type ConnectedMachine = {
  id: string;
  name: string;
  gpu: string;
  vram: string;
  os: string;
  publicTunnelUrl: string;
  relayRegion: string;
  pingMs: number;
  status: "online" | "offline";
  pooled: boolean;
};

const initialMachines: ConnectedMachine[] = [
  {
    id: "mach_rtx4070_9f82",
    name: "Desktop Workstation (RTX 4070)",
    gpu: "NVIDIA GeForce RTX 4070",
    vram: "12 GB GDDR6X",
    os: "Windows 11 Pro",
    publicTunnelUrl: "https://gpu-node-9f82.selfapi.site/v1",
    relayRegion: "US-East (Virginia)",
    pingMs: 24,
    status: "online",
    pooled: true,
  },
  {
    id: "mach_macbook_m3",
    name: "MacBook Pro M3 Max",
    gpu: "Apple M3 Max (38-core GPU)",
    vram: "36 GB Unified Memory",
    os: "macOS Sequoia",
    publicTunnelUrl: "https://macbook-m3-a12.selfapi.site/v1",
    relayRegion: "US-West (Oregon)",
    pingMs: 42,
    status: "offline",
    pooled: false,
  },
];

export default function MachinesPage() {
  const [machines, setMachines] = useState<ConnectedMachine[]>(initialMachines);

  useEffect(() => {
    async function check() {
      const health = await checkAgentHealth();
      if (health.agentData) {
        const d = health.agentData;
        setMachines((prev) =>
          prev.map((m, idx) =>
            idx === 0
              ? {
                  ...m,
                  name: `Host Workstation (${d.gpu_name ?? "Local Processor"})`,
                  gpu: d.gpu_name ?? "Local Processor",
                  vram: `${d.vram_gb ?? 8} GB Memory`,
                  status: health.online ? "online" : "offline",
                  pingMs: d.p95_latency_ms ?? 24,
                }
              : m,
          ),
        );
      } else {
        setMachines((prev) =>
          prev.map((m, idx) => (idx === 0 ? { ...m, status: health.online ? "online" : "offline" } : m)),
        );
      }
    }
    void check();
  }, []);


  const togglePooling = async (id: string) => {
    try {
      await fetch("http://127.0.0.1:8787/v1/tunnel/toggle", { method: "POST" });
    } catch {
      // offline fallback
    }
    setMachines((prev) =>
      prev.map((m) => (m.id === id ? { ...m, pooled: !m.pooled } : m)),
    );
  };

  return (
    <>
      <PageHeader
        title="Machines"
        description="Connected hardware, live NAT status, public tunnel URLs, and load-balancing controls."
        action={
          <button
            type="button"
            onClick={() => window.open("http://localhost:3000", "_blank")}
            className="rounded-md bg-zinc-900 px-3.5 py-2 text-sm font-medium text-white hover:bg-zinc-800 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-zinc-200"
          >
            Connect Machine (Desktop Agent)
          </button>
        }
      />

      <div className="grid gap-4">
        {machines.map((machine) => (
          <div
            key={machine.id}
            className="rounded-lg border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950"
          >
            <div className="flex flex-wrap items-start justify-between gap-3 border-b border-zinc-100 pb-4 dark:border-zinc-800/80">
              <div>
                <div className="flex items-center gap-2">
                  <span
                    className={`h-2.5 w-2.5 rounded-full ${
                      machine.status === "online" ? "bg-emerald-500 animate-pulse" : "bg-zinc-500"
                    }`}
                  />
                  <h3 className="text-base font-semibold text-zinc-900 dark:text-zinc-100">
                    {machine.name}
                  </h3>
                </div>
                <p className="mt-1 text-xs text-zinc-500">{machine.os} · {machine.gpu}</p>
              </div>
              <div className="flex items-center gap-3">
                <span
                  className={`inline-flex rounded-full px-2.5 py-1 text-xs font-semibold ${
                    machine.status === "online"
                      ? "bg-emerald-500/15 text-emerald-500"
                      : "bg-zinc-500/15 text-zinc-400"
                  }`}
                >
                  {machine.status === "online" ? `Connected (${machine.pingMs}ms NAT)` : "Disconnected"}
                </span>
                <button
                  type="button"
                  onClick={() => togglePooling(machine.id)}
                  className={`rounded-md border px-3 py-1 text-xs font-medium transition-colors ${
                    machine.pooled
                      ? "border-blue-500/40 bg-blue-500/10 text-blue-400"
                      : "border-zinc-200 bg-zinc-50 text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                  }`}
                >
                  {machine.pooled ? "Pooled in Load Balancer ✓" : "Enable Pooling"}
                </button>
              </div>
            </div>

            <div className="mt-4 grid gap-3 sm:grid-cols-4">
              <div>
                <span className="text-[11px] uppercase tracking-wider text-zinc-500 font-mono">
                  VRAM Memory
                </span>
                <p className="mt-0.5 text-sm font-mono font-medium text-zinc-900 dark:text-zinc-100">
                  {machine.vram}
                </p>
              </div>
              <div>
                <span className="text-[11px] uppercase tracking-wider text-zinc-500 font-mono">
                  Relay Node Region
                </span>
                <p className="mt-0.5 text-sm font-medium text-zinc-900 dark:text-zinc-100">
                  {machine.relayRegion}
                </p>
              </div>
              <div>
                <span className="text-[11px] uppercase tracking-wider text-zinc-500 font-mono">
                  Host Reliability (FR-26)
                </span>
                <p className="mt-0.5 text-xs font-mono text-amber-400 font-medium">
                  🥇 Gold Host (99.6% · {machine.pingMs}ms P95)
                </p>
              </div>
              <div>
                <span className="text-[11px] uppercase tracking-wider text-zinc-500 font-mono">
                  Compute Rate (FR-25)
                </span>
                <p className="mt-0.5 font-mono text-xs text-emerald-400 font-medium">
                  $0.20 / 1M Tokens
                </p>
              </div>
            </div>

          </div>
        ))}
      </div>
    </>
  );
}

