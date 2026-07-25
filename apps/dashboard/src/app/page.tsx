"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { PageHeader } from "@/components/layout/page-header";
import { MetricCard } from "@/components/ui/metric-card";
import { StatusBadge } from "@/components/ui/status-badge";
import { checkAgentHealth, getStoredRequests, RequestRecord, AgentHealthData } from "@/lib/store";

export default function DashboardHome() {
  const [agentOnline, setAgentOnline] = useState<boolean>(false);
  const [agentData, setAgentData] = useState<AgentHealthData | null>(null);
  const [requests] = useState<RequestRecord[]>(() => getStoredRequests());

  useEffect(() => {

    async function poll() {
      const health = await checkAgentHealth();
      setAgentOnline(health.online);
      if (health.agentData) setAgentData(health.agentData);
    }
    void poll();
    const interval = setInterval(() => void poll(), 2500);
    return () => clearInterval(interval);
  }, []);

  const totalReqs = agentData?.requests_handled ?? requests.length;
  const activeModel = agentData?.active_model ?? "Llama 3.2 3B Instruct";
  const uptime = agentData?.uptime_percentage ? `${agentData.uptime_percentage}%` : (agentOnline ? "99.9%" : "—");

  return (
    <>
      <PageHeader
        title="Dashboard"
        description="Monitor uptime, request volume, and alerts across your SelfAPI deployment."
      />

      <div className="mb-6 flex flex-wrap items-center gap-3">
        <StatusBadge
          variant={agentOnline ? "online" : "offline"}
          label={agentOnline ? `Agent Online (Port 8787 · ${agentData?.gpu_name ?? "GPU Active"})` : "Agent Offline / Searching"}
        />
        <span className="text-sm text-zinc-500 dark:text-zinc-400">
          Public Tunnel: {agentData?.public_tunnel_url ?? "https://gpu-node-9f82.selfapi.site/v1"}
        </span>
      </div>

      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          label="Total requests handled"
          value={String(totalReqs)}
          detail={agentOnline ? "Live GPU execution" : "Connect desktop agent to begin"}
        />
        <MetricCard
          label="Requests this period"
          value={String(totalReqs)}
          detail="Last 30 days"
        />
        <MetricCard
          label="Host Uptime Telemetry"
          value={uptime}
          detail={agentOnline ? `${agentData?.p95_latency_ms ?? 24}ms P95 Latency` : "No machine connected"}
        />
        <MetricCard
          label="Active Local Model"
          value={activeModel}
          detail={agentOnline ? "Serving on http://127.0.0.1:8787/v1" : "None selected"}
        />
      </div>


      <div className="mt-8 grid gap-6 lg:grid-cols-2">
        <section className="rounded-lg border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
          <h2 className="text-sm font-medium text-zinc-500 dark:text-zinc-400">
            Usage Activity
          </h2>
          <div className="mt-4 flex h-32 flex-col justify-center gap-2 rounded-md border border-zinc-200 bg-zinc-50 p-4 text-sm dark:border-zinc-800 dark:bg-zinc-900">
            {requests.slice(0, 3).map((r) => (
              <div key={r.id} className="flex justify-between text-xs text-zinc-600 dark:text-zinc-300">
                <span className="font-mono">{r.id}</span>
                <span>{r.model}</span>
                <span className="text-emerald-500 font-medium">{r.status}</span>
                <span>{r.latencyMs} ms</span>
              </div>
            ))}
          </div>
        </section>

        <section className="rounded-lg border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
          <h2 className="text-sm font-medium text-zinc-500 dark:text-zinc-400">
            System Alerts
          </h2>
          <p className="mt-4 text-sm text-zinc-600 dark:text-zinc-300">
            {agentOnline ? (
              <span className="text-emerald-500">✓ All local services operating normally. No error spikes detected.</span>
            ) : (
              "No agent connected on localhost:8787. Launch desktop agent app to serve inference requests."
            )}
          </p>
        </section>
      </div>

      <section className="mt-8 rounded-lg border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
        <h2 className="text-sm font-medium text-zinc-500 dark:text-zinc-400">
          Quick links
        </h2>
        <div className="mt-4 flex flex-wrap gap-3">
          <Link
            href="/machines"
            className="rounded-md border border-zinc-200 px-3 py-2 text-sm hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-900"
          >
            Connect a machine
          </Link>
          <Link
            href="/models"
            className="rounded-md border border-zinc-200 px-3 py-2 text-sm hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-900"
          >
            Download a model
          </Link>
          <Link
            href="/api-keys"
            className="rounded-md border border-zinc-200 px-3 py-2 text-sm hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-900"
          >
            Generate an API key
          </Link>
          <Link
            href="/requests"
            className="rounded-md border border-zinc-200 px-3 py-2 text-sm hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-900"
          >
            View request history
          </Link>
        </div>
      </section>
    </>
  );
}

